#![warn(missing_docs)]

//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

/// IMPORT
/////////////////////////////////////////////////////////////////////////////////////

// SR25519 key pair used by Aura consensus algorithm for block authoring.
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
// Grandpa consensus used to manage and share the state of a voter.
use sc_consensus_grandpa::SharedVoterState;
// Adding additional methods to the Future trait.
use futures::FutureExt;
// Interface for creating offchain transaction pools, that allows transactions to be submitted offchain.
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use dev_runtime::{
	opaque::Block,
	RuntimeApi,
};
// Provides interfaces for interacting with the blockchain database.
use sc_client_api::{
	// Generic interface for blockchain database system.
	Backend,
	// Provides specific methods for working with blocks.
	BlockBackend,
};
// Configuration types for setting up the Aura consensus mechanism,
use sc_consensus_aura::{
	ImportQueueParams,
	SlotProportion,
	StartAuraParams,
};
// Types for service configuration, error handling, task management, and warp sync parameters.
use sc_service::{
	error::Error as ServiceError,
	Configuration,
	TaskManager,
	WarpSyncParams,
};
// Components for configuring and managing telemetry,
// useful for gathering and reporting runtime metrics and blockchain operation data.
use sc_telemetry::{
	Telemetry,
	TelemetryWorker,
};
// Standard Rust imports for atomic reference counting (Arc) and time durations (Duration).
use std::{
	// A thread-safe, reference-counting pointer, used for shared ownership.
	sync::Arc,
	// A struct for measuring time durations, used for configuring time-related parameters.
	time::Duration,
};

/// SERVICE TYPE DEFINITION
/////////////////////////////////////////////////////////////////////////////////////

// Service type defined as a collection of components that are required to build and operate the node service.

// Its a specialized version of a Substrate client,
// configured for full node operation with specific components tailored to the blockchains requirements.
pub(crate) type FullClient = sc_service::TFullClient<
	Block,
	RuntimeApi,
	sc_executor::WasmExecutor<sp_io::SubstrateHostFunctions>,
>;

// Specifies the storage backend for the full node.
// This backend handles the physical storage of blockchain data, including blocks and state.
type FullBackend = sc_service::TFullBackend<Block>;

/// Determines the chain selection strategy for the node using the LongestChain policy.
/// This policy selects the chain with the most accumulated work or the longest chain in cases of forks.
/// It is parameterized with the FullBackend to interact with stored block data and the block type.
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

// Substrate service configuration for the partial components needed to assemble a blockchain node
pub type Service = sc_service::PartialComponents<
	FullClient,
	FullBackend,
	FullSelectChain,
	// Manages the processing and verification of incoming blocks before they are imported into the blockchain state.
	sc_consensus::DefaultImportQueue<Block>,
	// The transaction pool for the full node, handling transaction queuing, storage, and propagation.
	sc_transaction_pool::FullPool<Block, FullClient>,
	// Tuple of components
	(
		// A component for importing blocks with GRANDPA consensus data, integrating GRANDPA finality proofs into the blockchain.
		sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
		// Connects the block import logic with the GRANDPA consensus mechanism, enabling communication and data exchange between them.
		sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
		// Optional component for telemetry
		Option<Telemetry>,
	),
>;

/// CONSTANTS
/////////////////////////////////////////////////////////////////////////////////////

// The minimum period of blocks on which justifications will be imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// new_partial FUNCTION
/////////////////////////////////////////////////////////////////////////////////////

/// The new_partial function is crucial for initializing the node service by configuring and initializing basic components
/// such as the client, the backend, the transaction pool and the import queue.
/// This function is specifically designed to create the parts of the service that are necessary
/// for the operation of the node before the full service with all its functions is started

pub fn new_partial(config: &Configuration) -> Result<Service, ServiceError> {

	// Telemetry (creates a TelemetryWorker)
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	// WASM Executor (enable the execution of the runtime in WASM format)
	let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(config);

    // Initialization of the core components for the service
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;

	// Client Arc (provides core access to the blockchain data and APIs)
	let client = Arc::new(client);

	// Telemetry setup (starting a new asynchronous task for each telemetry worker, which processes and sends the telemetry data)
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	// Initializes a LongestChain instance,
	// which is a chain selection strategy used by the node to determine the canonical chain in the event of forks.
	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	// transaction_pool (manage transactions that are waiting to be included in blocks)
	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	// grandpa_block_import, grandpa_link (initialization of components for the GRANDPA consensus mechanism)
	let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
		client.clone(),
		GRANDPA_JUSTIFICATION_PERIOD,
		&client,
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;

	// Slot duration for the Aura consensus mechanism is calculated here
	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

	// Is crucial for the processing and validation of incoming blocks before they are added to the blockchain.
	let import_queue =
		sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(ImportQueueParams {
			block_import: grandpa_block_import.clone(),
			justification_import: Some(Box::new(grandpa_block_import.clone())),
			client: client.clone(),
			create_inherent_data_providers: move |_, ()| async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

				let slot =
					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
						*timestamp,
						slot_duration,
					);

				Ok((slot, timestamp))
			},
			spawner: &task_manager.spawn_essential_handle(),
			registry: config.prometheus_registry(),
			check_for_equivocation: Default::default(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			compatibility_mode: Default::default(),
		})?;

	// Return (all initialized components are combined in a PartialComponents object and returned).
	// PartialComponents provide a basis for the further initialization of the complete service.
	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (grandpa_block_import, grandpa_link, telemetry),
	})
}

/// new_full FUNCTION
/////////////////////////////////////////////////////////////////////////////////////

/// The new_full function is defined to performs the full node configuration and initialization for a blockchain node.
/// This function complements the subcomponents created by the new_partial function to create a fully functional node.

// Builds a new service for a full client.
pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {

	// Basic configuration and partial components
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (block_import, grandpa_link, mut telemetry),
	} = new_partial(&config)?;

	// General network configuration.
	// Requirements for communication in the Substrate network.
	let mut net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

	// General GRANDPA consensus configuration necessary for network initialization
	let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
		&client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
		&config.chain_spec,
	);
	// The protocol for GRANDPA is added to the network to allow nodes to communicate about finalization information
	let (grandpa_protocol_config, grandpa_notification_service) =
		sc_consensus_grandpa::grandpa_peers_set_config(grandpa_protocol_name.clone());
	net_config.add_notification_protocol(grandpa_protocol_config);

	// Warp Sync configuration necessary for network initialization.
	// Warp Sync allows new nodes to quickly synchronize the current state of the blockchain.
	let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		grandpa_link.shared_authority_set().clone(),
		Vec::default(),
	));
    
	// Network initialization with the previously defined configurations.
	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: Some(WarpSyncParams::WithProvider(warp_sync)),
			block_relay: None,
		})?;

	// General node configuration
	//  Defines the node's role in the network (full, light, authority).
	let role = config.role.clone();
	// Forces block production even if the node is not an authority.
	let force_authoring = config.force_authoring;
	// Disables automatic block production temporarily
	let backoff_authoring_blocks: Option<()> = None;
	// The node's name as identified in the network
	let name = config.network.node_name.clone();
	// Flags whether to enable the GRANDPA consensus protocol.
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();		

	// Offchain worker (configures and activates offchain workers for the node)
	if config.offchain_worker.enabled {
		// Initiates the offchain workers as background tasks.
		task_manager.spawn_handle().spawn(
			// Names for logging and task identification.
			"offchain-workers-runner",
			"offchain-worker",
			// Creates a new instance of offchain workers with specified options.
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				// Access to the runtime API
				runtime_api_provider: client.clone(),
				// Indicates if the node acts as a validator, affecting offchain worker behavior.
				is_validator: config.role.is_authority(),
				// Access to cryptographic keys for signing transactions or messages.
				keystore: Some(keystore_container.keystore()),
				// Storage for offchain data, separate from the blockchain state.
				offchain_db: backend.offchain_storage(),
				// Allows offchain workers to submit transactions to the network.
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				// Network service for offchain communication.
				network_provider: network.clone(),
				// Enables offchain workers to make HTTP requests.
				enable_http_requests: true,
				// Placeholder for additional offchain worker functionalities.
				custom_extensions: |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	// RPC extension (allows external clients and applications to interact with the node).
	// Defines a set of RPC dependencies and create the full RPC extension.
	let rpc_extensions_builder = {
		// Clones of the blockchain client and transaction pool are captured by the closure and will be used for creating RPC dependencies.
		let client = client.clone();
		let pool = transaction_pool.clone();
		// Creates a boxed closure, which is a function that can be stored and passed around.
		Box::new(move |deny_unsafe, _| {
			let deps =
				// Constructs the dependencies needed by the RPC extensions.
				crate::rpc::FullDeps { client: client.clone(), pool: pool.clone(), deny_unsafe };
			// Calls a function to create the full set of RPC extensions using the specified dependencies.
			crate::rpc::create_full(deps).map_err(Into::into)
		})
	};
	// This function takes the RPC extensions and integrates them into the service so that they are accessible via the network.
	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_builder: rpc_extensions_builder,
		backend,
		system_rpc_tx,
		tx_handler_controller,
		sync_service: sync_service.clone(),
		config,
		telemetry: telemetry.as_mut(),
	})?;

	// Start AURA authoring task (if Validator)
	if role.is_authority() {
		// Initializes a ProposerFactory for block authoring.
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			// Handles background task spawning for block proposals.
			task_manager.spawn_handle(),
			// Interacts with blockchain data for block construction.
			client.clone(),
			// Sources transactions for new blocks.
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);
		
		// Is important to determine how often blocks should be created.
		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

		// AURA consensus configuration
		// '_' are placeholders for other generic parameters that the function requires.
		let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _>(
			StartAuraParams {
				slot_duration,
				// Blockchain client interface for accessing chain data and state.
				client,
				// Mechanism to choose the canonical chain, (longest chain).
				select_chain,
				// Logic for importing blocks into the chain, integrating consensus information.
				block_import,
				// Factory for creating block proposers, entities that propose new blocks.
				proposer_factory,
				// Function to generate inherent data providers and provide data necessary for block construction like timestamps and slots.
				create_inherent_data_providers: move |_, ()| async move {
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
					let slot =
						sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);
					Ok((slot, timestamp))
				},
				// Forces block production, even if not an elected validator (testing)
				force_authoring,
				// Optionally pauses block production after an unsuccessful slot, to manage load and resources.
				backoff_authoring_blocks,
				// Secure storage for cryptographic keys used in consensus operations.
				keystore: keystore_container.keystore(),
				// Service for querying network sync status, ensuring nodes don't produce blocks while syncing.
				sync_oracle: sync_service.clone(),
				// Mechanism to manage block justifications, critical for finality in consensus.
				justification_sync_link: sync_service.clone(),
				// Fraction of a slot during which a block proposal is valid (aiming for a balance between network latency and block propagation).
				block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
				// Optional maximum fraction of a slot for block proposal, allowing dynamic adjustments.
				max_block_proposal_slot_portion: None,
				telemetry: telemetry.as_ref().map(|x| x.handle()),
				// Enables compatibility with different consensus versions or configurations.
				compatibility_mode: Default::default(),
			},
		)?;
		// The AURA authoring task is considered essential,
		// i.e. if it fails we take down the service with it.
		task_manager
			.spawn_essential_handle()
			.spawn_blocking("aura", Some("block-authoring"), aura);
	}

	// Start GRANDPA Voter (if Validator or FullNode or ArchiveNode or special case)
	if enable_grandpa {
		// if the node isn't actively participating in consensus then it doesn't
		// need a keystore, regardless of which protocol we use below.
		let keystore = if role.is_authority() { Some(keystore_container.keystore()) } else { None };

		// Full GRANDPA consensus and Warp-Sync configuration
		let grandpa_config = sc_consensus_grandpa::Config {
			// FIXME make this available through chainspec
			// Time interval for gossiping GRANDPA messages.
			gossip_duration: Duration::from_millis(333),
			// Frequency for generating justifications.
			justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
			// Node identifier for telemetry and logging.
			name: Some(name),
			// Flag for GRANDPA observer mode.
			observer_enabled: false,
			// Secure storage for cryptographic keys.
			keystore,
			// Node's role in consensus (authority or observer).
			local_role: role,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			// Identifier for the GRANDPA protocol.
			protocol_name: grandpa_protocol_name,
		};

		// NOTE: non-authorities could run the GRANDPA observer protocol, but at
		// this point the full voter should provide better guarantees of block
		// and vote data availability than the observer. The observer has not
		// been tested extensively yet and having most nodes in a network run it
		// could lead to finality stalls.
		
		// Start the full GRANDPA voter
		let grandpa_config = sc_consensus_grandpa::GrandpaParams {
			// The GRANDPA consensus configuration.
			config: grandpa_config,
			// Connection between the block import process and the consensus mechanism.
			link: grandpa_link,
			// Network service for communication between nodes.
			network,
			// Synchronization service wrapped in an Arc for shared access.
			sync: Arc::new(sync_service),
			// Manages notifications for GRANDPA.
			notification_service: grandpa_notification_service,
			// Defines the voting rules for the consensus process.
			voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			// Manages state shared among voters in the consensus.
			shared_voter_state: SharedVoterState::empty(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			// Factory for creating offchain transaction pools.
			offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool),
		};

		// The GRANDPA voter task is considered infallible,
		// i.e. if it fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_consensus_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	// TASK MANAGER AND SERVICE START
	network_starter.start_network();
	Ok(task_manager)
}
