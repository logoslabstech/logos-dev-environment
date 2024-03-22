 /// IMPORT AND SETUP
/////////////////////////////////////////////////////////////////////////////////////

/// Utility for working with predefined SR25519 key pairs (testing).
use sp_keyring::Sr25519Keyring;
/// Represent a fully operational blockchain client
use crate::service::FullClient;
/// Result type for CLI operations, aliasing a standard Result with an error type relevant to CLI commands.
use sc_cli::Result;
/// Trait from sc_client_api for querying blockchain data by blocks.
use sc_client_api::BlockBackend;
use dev_runtime as runtime;
use runtime::{
	AccountId,                   // Account identification.
	Balance,                     // Balance representation.
	BalancesCall,                // Interacting with the balances.
	SystemCall,                  // Interacting with system pallets
};
/// Used for encoding data and managing cryptographic key pairs.
use sp_core::{
	Encode,
	Pair,
};
/// Tools for managing and providing data for block inclusions that do not come from external transactions, like timestamps.
use sp_inherents::{
	InherentData,
	InherentDataProvider,
};
use sp_runtime::{
	OpaqueExtrinsic,      // A generic extrinsic type that the blockchain doesn't interpret directly, making it flexible for various runtime definitions.
	SaturatedConversion,  // A utility trait for converting between numerical types without overflow errors.
};
use std::{
	sync::Arc,            // A thread-safe reference-counting pointer from Rusts standard library, used for managing shared access to data.
	time::Duration,       // A time measurement unit from Rust's standard library, for specifying time durations in operations.
};

/// BENCHMARK STRUCTURE FOR RemarkBuilder
/////////////////////////////////////////////////////////////////////////////////////

/// Generates extrinsics for the `benchmark overhead` command.
/// Note: Should only be used for benchmarking.

pub struct RemarkBuilder {
	client: Arc<FullClient>,
}

/// SPECIFIC BENCHMARK IMPLEMENTATION
impl RemarkBuilder {
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Arc<FullClient>) -> Self {
		Self { client }
	}
}

/// EXTERNAL BENCHMARK IMPLEMENTATION
impl frame_benchmarking_cli::ExtrinsicBuilder for RemarkBuilder {
	fn pallet(&self) -> &str {
		"system"
	}

	fn extrinsic(&self) -> &str {
		"remark"
	}

	fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			self.client.as_ref(),
			acc,
			SystemCall::remark { remark: vec![] }.into(),
			nonce,
		)
		.into();

		Ok(extrinsic)
	}
}

/// BENCHMARK STRUCTURE FOR TransferKeepAliveBuilder
/////////////////////////////////////////////////////////////////////////////////////

/// Generates `Balances::TransferKeepAlive` extrinsics for the benchmarks.
/// Note: Should only be used for benchmarking.

pub struct TransferKeepAliveBuilder {
	client: Arc<FullClient>,
	dest: AccountId,
	value: Balance,
}

/// SPECIFIC BENCHMARK IMPLEMENTATION
impl TransferKeepAliveBuilder {
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Arc<FullClient>, dest: AccountId, value: Balance) -> Self {
		Self { client, dest, value }
	}
}

/// EXTERNAL BENCHMARK IMPLEMENTATION
impl frame_benchmarking_cli::ExtrinsicBuilder for TransferKeepAliveBuilder {
	fn pallet(&self) -> &str {
		"balances"
	}

	fn extrinsic(&self) -> &str {
		"transfer_keep_alive"
	}

	fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			self.client.as_ref(),
			acc,
			BalancesCall::transfer_keep_alive { dest: self.dest.clone().into(), value: self.value }
				.into(),
			nonce,
		)
		.into();

		Ok(extrinsic)
	}
}

/// SUPPORT FUNCTIONS FOR BENCHMARKING
/////////////////////////////////////////////////////////////////////////////////////

/// Create a transaction using the given `call`.
/// Note: Should only be used for benchmarking.

pub fn create_benchmark_extrinsic(
	client: &FullClient,
	sender: sp_core::sr25519::Pair,
	call: runtime::RuntimeCall,
	nonce: u32,
) -> runtime::UncheckedExtrinsic {
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let best_hash = client.chain_info().best_hash;
	let best_block = client.chain_info().best_number;

	let period = runtime::BlockHashCount::get()
		.checked_next_power_of_two()
		.map(|c| c / 2)
		.unwrap_or(2) as u64;
	let extra: runtime::SignedExtra = (
		frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
		frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
		frame_system::CheckTxVersion::<runtime::Runtime>::new(),
		frame_system::CheckGenesis::<runtime::Runtime>::new(),
		frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
			period,
			best_block.saturated_into(),
		)),
		frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
		frame_system::CheckWeight::<runtime::Runtime>::new(),
		pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
	);

	let raw_payload = runtime::SignedPayload::from_raw(
		call.clone(),
		extra.clone(),
		(
			(),
			runtime::VERSION.spec_version,
			runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| sender.sign(e));

	runtime::UncheckedExtrinsic::new_signed(
		call,
		sp_runtime::AccountId32::from(sender.public()).into(),
		runtime::Signature::Sr25519(signature),
		extra,
	)
}

/// INHERENT DATA FOR BENCHMARKING
/////////////////////////////////////////////////////////////////////////////////////

/// Generates inherent data for the `benchmark overhead` command.
/// Note: Should only be used for benchmarking.

pub fn inherent_benchmark_data() -> Result<InherentData> {
	let mut inherent_data = InherentData::new();
	let d = Duration::from_millis(0);
	let timestamp = sp_timestamp::InherentDataProvider::new(d.into());

	futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data))
		.map_err(|e| format!("creating inherent data: {:?}", e))?;
	Ok(inherent_data)
}
