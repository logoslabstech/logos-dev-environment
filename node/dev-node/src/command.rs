#![warn(missing_docs)]

/// IMPORT AND TYPE DEFINITION
/////////////////////////////////////////////////////////////////////////////////////

// Trait for Substrate CLI commands.
use sc_cli::SubstrateCli;
// Components for node setup.
use sc_service::PartialComponents;
// Predefined SR25519 keys utility.
use sp_keyring::Sr25519Keyring;

use dev_runtime::{
	Block,
	EXISTENTIAL_DEPOSIT,
};

use crate::{
	chain_spec,
	service,
	cli::{
		Cli,
		Subcommand,
	},
	benchmarking::{
		inherent_benchmark_data,
		RemarkBuilder,
		TransferKeepAliveBuilder,
	},
};

use frame_benchmarking_cli::{
	// Command for executing benchmarks on runtime pallets.
	BenchmarkCmd,
	// Interface for creating extrinsics during benchmarks.
	ExtrinsicFactory,
	// Defined metrics for standard hardware performance.
	SUBSTRATE_REFERENCE_HARDWARE,
};

/// Imports the timestamp_with_aura_info function from the try_runtime_cli crate.
/// This function provides information related to block building,
/// specifically for using the Aura consensus mechanism, and integrates it with timestamp details.
#[cfg(feature = "try-runtime")]
use try_runtime_cli::block_building_info::timestamp_with_aura_info;

/// SUBSTRATE CLI IMPLEMENTATION
/////////////////////////////////////////////////////////////////////////////////////

/// This struct customizes the command-line interface (CLI)
/// It provides specific information about the implementation and configures how blockchain specifications are loaded based on command-line arguments.

impl SubstrateCli for Cli {
	// Returns the name of the implementation, useful for identification purposes.
	fn impl_name() -> String {
		"Logos dev-node".into()
	}
	// Fetches the implementation version from the environment variables, set during compile time.
	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}
	// Retrieves the package description from Cargo's package metadata.
	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}
	// Obtains the author information from Cargo's package metadata.
	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}
	// Provides a URL where users can find support or report issues.
	fn support_url() -> String {
		"https://github.com/logoslabstech/logos-dev-environment/issues/new".into()
	}
	// Sets the copyright start year for the CLI application.
	fn copyright_start_year() -> i32 {
		2024
	}
	// Crucial function that loads the blockchains specification based on the provided identifier.
	// It supports loading predefined specs like "dev" or "local" configurations, or loading from a JSON file.
	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"dev" => Box::new(chain_spec::development_config()?),
			"" | "local" => Box::new(chain_spec::local_devnet_config()?),
			path =>
				Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?),
		})
	}
}

/// MAIN FUNCTION run
/////////////////////////////////////////////////////////////////////////////////////

/// The main function run is the central entry point
/// and orchestrates the execution of the program based on the CLI arguments
/// and sub-commands specified by the user.

// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	// SUB COMMAND HANDLER
	match &cli.subcommand {
		// Manages crypto keys.
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		// Creates a configuration for the blockchain.
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		// Checks the validity of a block without importing it.
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		// Exports blockchain blocks to a file.
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		// Exports the state of the blockchain at a specific block.
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		// Imports blocks from a file into the blockchain.
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		// Deletes all blockchain data.
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},
		// Resets the blockchain to a specific block.
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, backend, .. } =
					service::new_partial(&config)?;
				let aux_revert = Box::new(|client, _, blocks| {
					sc_consensus_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},

		// BENCHMARKING
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				// This switch needs to be in the client, since the client decides
				// which sub-commands it wants to support.
				match cmd {
					BenchmarkCmd::Pallet(cmd) => {
						if !cfg!(feature = "runtime-benchmarks") {
							return Err(
								"Runtime benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
									.into(),
							)
						}

						cmd.run::<Block, ()>(config)
					},
					BenchmarkCmd::Block(cmd) => {
						let PartialComponents { client, .. } = service::new_partial(&config)?;
						cmd.run(client)
					},
					#[cfg(not(feature = "runtime-benchmarks"))]
					BenchmarkCmd::Storage(_) => Err(
						"Storage benchmarking can be enabled with `--features runtime-benchmarks`."
							.into(),
					),
					#[cfg(feature = "runtime-benchmarks")]
					BenchmarkCmd::Storage(cmd) => {
						let PartialComponents { client, backend, .. } =
							service::new_partial(&config)?;
						let db = backend.expose_db();
						let storage = backend.expose_storage();

						cmd.run(config, client, db, storage)
					},
					BenchmarkCmd::Overhead(cmd) => {
						let PartialComponents { client, .. } = service::new_partial(&config)?;
						let ext_builder = RemarkBuilder::new(client.clone());

						cmd.run(
							config,
							client,
							inherent_benchmark_data()?,
							Vec::new(),
							&ext_builder,
						)
					},
					BenchmarkCmd::Extrinsic(cmd) => {
						let PartialComponents { client, .. } = service::new_partial(&config)?;
						// Register the *Remark* and *TKA* builders.
						let ext_factory = ExtrinsicFactory(vec![
							Box::new(RemarkBuilder::new(client.clone())),
							Box::new(TransferKeepAliveBuilder::new(
								client.clone(),
								Sr25519Keyring::Alice.to_account_id(),
								EXISTENTIAL_DEPOSIT,
							)),
						]);

						cmd.run(client, inherent_benchmark_data()?, Vec::new(), &ext_factory)
					},
					BenchmarkCmd::Machine(cmd) =>
						cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()),
				}
			})
		},
		// TRY RUNTIME
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			use crate::service::ExecutorDispatch;
			use sc_executor::{sp_wasm_interface::ExtendedHostFunctions, NativeExecutionDispatch};
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				// we don't need any of the components of new_partial, just a runtime, or a task
				// manager to do `async_run`.
				let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager =
					sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
						.map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;
				let info_provider = timestamp_with_aura_info(6000);

				Ok((
					cmd.run::<Block, ExtendedHostFunctions<
						sp_io::SubstrateHostFunctions,
						<ExecutorDispatch as NativeExecutionDispatch>::ExtendHostFunctions,
					>, _>(Some(info_provider)),
					task_manager,
				))
			})
		},
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
			.into()),
		Some(Subcommand::ChainInfo(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run::<Block>(&config))
		},		

		// FALLBACK AND NODE START
		// Fallback scenario when no subcommand was specified when the node was started.
		// In this case, the standard action is executed, which is the start of the complete node.
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				service::new_full(config).map_err(sc_cli::Error::Service)
			})
		},
	}
}
