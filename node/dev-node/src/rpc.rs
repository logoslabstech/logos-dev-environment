#![warn(missing_docs)]

//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

/// IMPORT AND TYPE DEFINITION
/////////////////////////////////////////////////////////////////////////////////////

// A thread-safe reference counter that allows multiple owners of a value.
use std::sync::Arc;
// Enables the creation of modular RPC APIs.
use jsonrpsee::RpcModule;
// Enables access to the transaction pool of the node.
use sc_transaction_pool_api::TransactionPool;
// A trait provided by the client implementation to gain access to the specific API functions of the runtime.
use sp_api::ProvideRuntimeApi;
// Facilitates block construction.
use sp_block_builder::BlockBuilder;
// Core functionalities and types for blockchain management
use sp_blockchain::{
	Error as BlockChainError,
	HeaderBackend,
	HeaderMetadata,
};
use dev_runtime::{
	opaque::Block,
	AccountId,
	Balance,
	Nonce,
	BlockNumber,
	Hash,
	EventRecord,
};

/// Controls access to potentially unsafe RPC methods.
// use frame_system::EventRecord;
pub use sc_rpc_api::DenyUnsafe;

/// STRUCTURE AND TYPES
/////////////////////////////////////////////////////////////////////////////////////

/// Defines structures and types that are necessary for the configuration and functioning of the RPC extensions.
/// This is a central part of the RPC configuration.
/// It brings together the components required to make the RPC interface fully functional
/// Full client dependencies.
pub struct FullDeps<C, P> {
	// The client instance to use.
	pub client: Arc<C>,
	// Transaction pool instance.
	pub pool: Arc<P>,
	// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// RPC EXTENSION FUNCTION
/////////////////////////////////////////////////////////////////////////////////////

/// Is responsible for Instantiate a full RPC extensions structure for a Substrate Node.
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	C::Api: pallet_contracts::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash, EventRecord>,
	P: TransactionPool + 'static,	
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;
// ADDITIONAL RPC METHODS ...

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client).into_rpc())?;

// CUSTOM APIs 
	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	// `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`

// CHAIN SPECIFICATION AND METADATA 
	// You probably want to enable the `rpc v2 chainSpec` API as well
	//
	// let chain_name = chain_spec.name().to_string();
	// let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	// let properties = chain_spec.properties();
	// module.merge(ChainSpec::new(chain_name, genesis_hash, properties).into_rpc())?;

	Ok(module)
}
