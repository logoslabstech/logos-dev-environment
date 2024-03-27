#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
// Integrates the automatically generated WASM binary of the Runtime into the Rust code
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs")); // std

/// RUNTIME IMPORTS
/////////////////////////////////////////////////////////////////////////////////////

/// (private imports)

// Block production.
use sp_consensus_aura::sr25519::AuthorityId as AuraId;

// Block finality.
use pallet_grandpa::AuthorityId as GrandpaId;

// Tools for defining and interacting with APIs.
use sp_api::impl_runtime_apis;

// no_std environment support.
use sp_std::prelude::*;

// Core definitions for cryptography and opaque metadata.
use sp_core::{
	crypto::KeyTypeId,
	OpaqueMetadata,
};

use sp_runtime::{
	// Generate a unique runtime string identifier (for runtime versioning and compatibility checks).
	create_runtime_str,
	// Providing foundational, reusable types and functions across the runtime.
	generic,
	impl_opaque_keys,
	codec,
	MultiSignature,
	// Processing and feedback of extrinsic operations (ok|Err).
	ApplyExtrinsicResult,
	transaction_validity::{
		TransactionSource,
		TransactionValidity,
	},
	traits::{
		BlakeTwo256,
		Block as BlockT,
		IdentifyAccount,
		// Numeric type for blocks and states.
		NumberFor,
		// Generates unit or identity values for numerical types.
		One,
		// Signature verification.
		Verify,
		// Mechanism for account identifiers.
		AccountIdLookup,
	},
};

// Runtime version information
#[cfg(feature = "std")]
use sp_version::NativeVersion;  // std
use sp_version::RuntimeVersion; // wasm

// Manages transaction fees
use pallet_transaction_payment::{
	// Static fee rate multiplier.
	ConstFeeMultiplier,
	// Dynamic fee rate adjustment.
	Multiplier,
	// Interface for currency manipulation.
	CurrencyAdapter,
};

// Import of the core blockchain functionalities
use frame_system::{
	EnsureSigned,
	// Configures blockchain limits
	limits::{
		BlockWeights,
		BlockLength,
	},
};

// Import of support functions for genesis configuration
use frame_support::genesis_builder_helper::{
	build_config,
	create_default_config,
};

/// (public imports)

pub use frame_support::{
	// Constructs the runtime with specified pallets
	construct_runtime,
	// Macro for deriving implementations (context-specific usage)
	derive_impl,
	// Macro for defining constant parameter types in runtime
	parameter_types,
	// Abstraction for defining a single storage value
	StorageValue,
	// Categorizes extrinsics for dispatch prioritization
	dispatch::DispatchClass,
	traits::{
		ConstBool,
		ConstU128,
		ConstU64,
		ConstU32,
		ConstU8,
		KeyOwnerProofSystem,
		Randomness,
		StorageInfo,
		Nothing,
		AsEnsureOriginWithArg, // Extends the functionality of origin checks.
		Everything, // Used to specify that any type of origin is accepted.
	},
	weights::{
		IdentityFee, // Fee type that do not alter the fee amount
		Weight,      // Represents computational and storage cost in the runtime
		constants::{
			BlockExecutionWeight,
			ExtrinsicBaseWeight,
			RocksDbWeight,
			WEIGHT_REF_TIME_PER_SECOND, // Reference for weight to time conversion (1_000_000_000_000)
		},
	},
};

// Imports core blockchain operations.
pub use frame_system::Call as SystemCall;

// Manages token balances.
pub use pallet_balances::Call as BalancesCall;

// Manages blockchain time.
pub use pallet_timestamp::Call as TimestampCall;

#[cfg(any(feature = "std", test))]
// Setup and initialization of the memory structure for development and runtime testing.
pub use sp_runtime::BuildStorage; // std
// Types for financial and proportional calculations within the Runtime
pub use sp_runtime::{Perbill, Permill}; // wasm

/// RUNTIME TYPE DEFINITION
/////////////////////////////////////////////////////////////////////////////////////

// Block height (4.290.000.000 max blocks - 595,83 days at 12 ms)
pub type BlockNumber = u32;
// Defines chain transactions signature
pub type Signature = MultiSignature;
// Identifying an account on the chain by extracting the account identifier from the signatures signer.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
// Balance of an account (represents token or asset quantity).
pub type Balance = u128;
// Index of a transaction in the chain. (one account can send 2^32−1 transactions)
pub type Nonce = u32;
// A 256-bit cryptographic hash used within the blockchain to uniquely identify and verify
// the integrity of various types of data (transactions, blocks, or other on-chain information).
pub type Hash = sp_core::H256;

/// RUNTIME OPAQUE TYPES (INTEROPERABILITY AND ABSTRACTION)
/////////////////////////////////////////////////////////////////////////////////////

/// These "opaque" types are used by the CLI to instantiate machinery that don't need to know the specifics of the runtime.
/// They can then be made to be agnostic over specific formats of data like extrinsics,
/// allowing for them to continue syncing the network through upgrades to even the core data structures.
pub mod opaque {
	use super::*;
	// Opaque representation of an extrinsic.
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	// A set of keys used for session management within the blockchain
	// This is a opaque session key definition that includes aura and grandpa keys,
	// which is crucial for block production and finalizing blocks.
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

/// RUNTIME VERSION CONFIGURATION
/////////////////////////////////////////////////////////////////////////////////////

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("dev-runtime"),
	impl_name: create_runtime_str!("dev-runtime"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	// runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	// `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App to use the compatible custom types.
	spec_version: 100,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() } // std
}

/// RUNTIME CONSTANTS
/////////////////////////////////////////////////////////////////////////////////////

/// This determines the average expected block time (ms) that are targeted.
pub const MILLISECS_PER_BLOCK: u64 = 12000;
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn slot_duration().
// NOTE: Currently it is not possible to change the slot duration after the chain has started.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

/// Time in this blockchain is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// ~5000 blocks per min
// ~300.000 blocks per hour
// ~7.200.000 blocks per day

/// Currency denomination system
pub const MICROLOST: Balance = 1_000_000_000_000;
pub const MILILOST: Balance = 1_000 * MICROLOST;
pub const LOST: Balance = 1_000 * MILILOST;

// LOST      (10^18) 1.000.000.000.000.000.000 | 1 LOST
// MILILOST  (10^15) 1.000.000.000.000.000     | 0.001 LOST
// MICROLOST (10^12) 1.000.000.000 000         | 0.000001 LOST

/// Method for calculating deposit fees based on storage usage
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	(items as Balance * LOST + (bytes as Balance) * (5 * MILILOST / 100)) / 10
}

/// Block processing and resource utilization definition
// On average, 10% of the maximum block weight is reserved for the initialization phase (critical transactions)
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
// 75% of the maximum block weight can be used for processing not-critical transactions
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
// This is a critical parameter that limits the total number and types of transactions
// and operations that can be contained in a single block.
const MAXIMUM_BLOCK_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), u64::MAX); // (2_000_000_000_000)

// PALLETS CONFIGURATION
/////////////////////////////////////////////////////////////////////////////////////

parameter_types! {
	// Defines for how many blocks a hash remains stored in the system
	pub const BlockHashCount: BlockNumber = 2400;
	// Specifies the current version of Runtime
	pub const Version: RuntimeVersion = VERSION;
	// Determines the maximum length of a block in bytes (how many transactions can contain a block).
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO); // (5 MB)
	// Determines the prefix for addresses in the blockchain.
	pub const SS58Prefix: u8 = 42;
	// Defines weight limits and the configurations for blocks
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		// Sets base weight for all blocks.
		.base_block(BlockExecutionWeight::get())
		// Applies to all transaction types
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		// Targets normal transactions
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		// Configuration for critical operations
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		// Sets an average value for the share of block processing time
		// or block weight used for the initialization phase of each block
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}
/// The frame_system is one of the core pallets in Substrate that provides the basic functionality for the blockchain runtime.
impl frame_system::Config for Runtime {
	// The basic call filter to use in dispatchable.
	type BaseCallFilter = Everything;
	// Aggregated dispatch type available for extrinsics.
	type RuntimeCall = RuntimeCall;
	// Defines the blockchain's block structure
	type Block = Block;
	// Sets base values and limits for block and extrinsics weights.
	type BlockWeights = RuntimeBlockWeights;
	// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	// Tracks the number of transactions made by an account.
	type Nonce = Nonce;
	// Type for block and data hashing.
	type Hash = Hash;
	// The hashing algorithm.
	type Hashing = BlakeTwo256;
	// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	// Represents the source of runtime calls.
	type RuntimeOrigin = RuntimeOrigin;
	// Links to the runtime's central event type
	type RuntimeEvent = RuntimeEvent;
	// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	// Version of the runtime.
	type Version = Version;
	// The data format to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	// Maximum number of reference counters.
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	// Information about runtime pallets.
	type PalletInfo = PalletInfo;
	// Actions on account creation.
	type OnNewAccount = ();
	// Actions on account removal.
	type OnKilledAccount = ();
	/// Enables the calculation of transaction costs (for resource consumption) for system operations based on the SubstrateWeight system.
	type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
	// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	// The aggregated Task type, injected by construct_runtime!.
	type RuntimeTask = ();
}

/// AURA consensus mechanism specifics
impl pallet_aura::Config for Runtime {
	// Identifies validators in Aura consensus.
	type AuthorityId = AuraId;
	// Specifies validators that are currently disabled
	type DisabledValidators = ();
	// Limits for the amount of validators
	type MaxAuthorities = ConstU32<32>;
	// Enable or disable multiple blocks per slot.
	type AllowMultipleBlocksPerSlot = ConstBool<false>;

	#[cfg(feature = "experimental")]
	// Defines the slot duration, potentially doubling the minimum period.
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

/// GRANDPA consensus protocol specifics
impl pallet_grandpa::Config for Runtime {
	// Links to the runtime's central event type
	type RuntimeEvent = RuntimeEvent;
	// Enables the calculation of transaction costs (for resource consumption) for GRANDPA operations.
	type WeightInfo = ();
	// Limits the amount of GRANDPA authorities.
	type MaxAuthorities = ConstU32<32>;
	// Sets the maximum number of nominators.
	type MaxNominators = ConstU32<0>;
	// State management
	type MaxSetIdSessionEntries = ConstU64<0>;
	// Defines the proof type for key ownership (no proof needed or used).
	type KeyOwnerProof = sp_core::Void;
	// Specifies the system for reporting equivocations
	type EquivocationReportSystem = ();
}

/// Configures the timestamp functionality
impl pallet_timestamp::Config for Runtime {
	// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	// Sets AURA to act upon timestamp updates.
	type OnTimestampSet = Aura;
	//  Defines half the slot duration as the minimum period between blocks.
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	// Enables the calculation of transaction costs (for resource consumption) for timestamp operations.
	type WeightInfo = ();
}

pub const EXISTENTIAL_DEPOSIT: Balance = MILILOST; // 0.001 LOST
/// Basic functionality for handling tokenized values within the blockchain
impl pallet_balances::Config for Runtime {
	// Limits the number of locks on an account's balance to 50
	type MaxLocks = ConstU32<50>;
	// Specifies no limit for balance reserves
	type MaxReserves = ();
	// Uses an 8-byte array to identify reserves
	type ReserveIdentifier = [u8; 8];
	// The type for recording an account's balance.
	type Balance = Balance;
	// Links to the runtime's central event type
	type RuntimeEvent = RuntimeEvent;
	// Mechanism for removing negligible balances
	type DustRemoval = ();
	// Sets the minimum balance that must exist for an account to be active.
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	// Utilizes the System pallet for account storage.
	type AccountStore = System;
	// Enables the calculation of transaction costs (for resource consumption) for balances operations based on the SubstrateWeight system.
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	// Identifier for balance freezes
	type FreezeIdentifier = ();
	// Limit for how many times a balance can be frozen.
	type MaxFreezes = ();
	// Specifies reasons for balance holds within the runtime.
	type RuntimeHoldReason = RuntimeHoldReason;
	// Unspecified reasons for freezing an account's balance.
	type RuntimeFreezeReason = ();
}

// By default, the system does not dynamically adjust transaction fees based on network congestion
// or other factors that might influence fee calculation.
parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}
/// Handling of transaction fees within a blockchain
impl pallet_transaction_payment::Config for Runtime {
	// Links to the runtime's central event type
	type RuntimeEvent = RuntimeEvent;
	// Utilizes CurrencyAdapter to handle transaction fee deduction,
	// interfacing with the Balances pallet without a specific refund policy.
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	// Sets a multiplier of 5 for fees on operational transactions,
	// making them more costly compared to standard transactions to prioritize their inclusion.
	type OperationalFeeMultiplier = ConstU8<5>;
	// Adopts IdentityFee policy for converting transaction weight into a fee, applying a 1:1 conversion rate.
	type WeightToFee = IdentityFee<Balance>;
	// Uses IdentityFee to convert the byte length of a transaction into a fee, similarly applying a straightforward 1:1 rate.
	type LengthToFee = IdentityFee<Balance>;
	// Determines how the transaction fee multiplier is adjusted over time, using a constant value defined by FeeMultiplier.
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

/// Establishes sudo functionalities, allowing a privileged account to execute administrative operations.
impl pallet_sudo::Config for Runtime {
	// Links to the runtime's central event type
	type RuntimeEvent = RuntimeEvent;
	// Links the sudo pallet to the runtimes call system, allowing the sudo user to invoke any callable function within the runtime
	type RuntimeCall = RuntimeCall;
	// Enables the calculation of transaction costs (for resource consumption) for sudo operations based on the SubstrateWeight system.
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

// Schedule for executing smart contracts
fn schedule<T: pallet_contracts::Config>() -> pallet_contracts::Schedule<T> {
	pallet_contracts::Schedule {
		limits: pallet_contracts::Limits {
			runtime_memory: 1024 * 1024 * 1024,
			..Default::default()
		},
		..Default::default()
	}
}
parameter_types! {
	// Sets the deposit amount required per item in contract storage.
	// This is calculated using the deposit function with 1 item and 0 bytes as parameters, emphasizing the cost per item regardless of its size.
	pub const DepositPerItem: Balance = deposit(1, 0);
	// Determines the deposit amount required per byte of data stored in the contract.
	// This is calculated using the deposit function with 0 items and 1 byte, focusing on the cost associated with the data size.
	pub const DepositPerByte: Balance = deposit(0, 1);
	// Establishes a default limit for the deposit amount required for creating a contract.
	// This limit is set based on the deposit function with parameters
	// that consider both the number of items and the size in bytes (1024 items and 1MB of data).
	pub const DefaultDepositLimit: Balance = deposit(1024, 1024 * 1024);
	// Configures the execution schedule for contracts, which includes parameters like gas costs and limits.
	pub Schedule: pallet_contracts::Schedule<Runtime> = schedule::<Runtime>();
	// Specifies the percentage of the deposit that is locked up for the uniqueness of a contract's code hash.
	// Setting this to 10% means that a portion of the deposit is reserved to discourage uploading duplicate
	// contracts and to ensure that resources are used efficiently.
	pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(10);
}
/// Implements and configures the contracts palette, which includes smart contract functionalities.
impl pallet_contracts::Config for Runtime {
	// Connects contract execution timing to the blockchain's timestamp mechanism.
	type Time = Timestamp;
	// Integrates the blockchain's collective flip randomness (temp) as a source for contracts.
	type Randomness = RandomnessCollectiveFlip;
	// Specifies the Balances pallet as the currency mechanism for contract transactions.
	type Currency = Balances;
	// Links to the runtime's central event type.
	type RuntimeEvent = RuntimeEvent;
	// Allows contracts to call other runtime functions, governed by the CallFilter.
	type RuntimeCall = RuntimeCall;
	/// The safest default is to allow no calls at all.
	/// Runtimes should whitelist dispatchables that are allowed to be called from contracts
	/// and make sure they are stable. Dispatchables exposed to contracts are not allowed to
	/// change because that would break already deployed contracts. The `Call` structure itself
	/// is not allowed to change the indices of existing pallets, too.
	/// Restricts contracts from calling certain runtime functions, with Nothing no calls are allowed by default.
	type CallFilter = Nothing;
	// Configures storage deposit costs.
	type DepositPerItem = DepositPerItem;
	// Configures storage deposit costs.
	type DepositPerByte = DepositPerByte;
	// Sets a default limit for contract deposits.
	type DefaultDepositLimit = DefaultDepositLimit;
	// Defines the call stack size for contract execution. (5 at the same time)
	type CallStack = [pallet_contracts::Frame<Self>; 5];
	// Integrates contract execution pricing with the transaction payment system.
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	// Enables the calculation of transaction costs (for resource consumption) for contracts operations based on the SubstrateWeight system.
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	// Allows for custom chain-specific functionalities in contracts.
	type ChainExtension = ();
	// Configures the execution schedule for contracts
	type Schedule = Schedule;
	// Specifies the mechanism for generating contract addresses.
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	// Limits the maximum code size for a contract. (125952 Bytes)
	type MaxCodeLen = ConstU32<{ 123 * 1024 }>;
	// Sets a maximum length for storage keys within contracts
	type MaxStorageKeyLen = ConstU32<128>;
	// Controls the exposure of potentially unstable interfaces to contracts.
	type UnsafeUnstableInterface = ConstBool<false>;
	// Specifies the maximum length for the debug buffer. (2097152 Bytes)
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	// Configures reasons for holding contract executions.
	type RuntimeHoldReason = RuntimeHoldReason;
	// Handles contract migrations, with distinctions between benchmarking scenarios.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Migrations = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Migrations = pallet_contracts::migration::codegen::BenchMigrations;
	// Limits the number of dependencies for delegate calls.
	type MaxDelegateDependencies = ConstU32<32>;
	// Sets a percentage of the deposit locked up for code hash uniqueness.
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Debug = ();
	type Environment = ();
	type Xcm = ();
}

 parameter_types! {
	// The amount of LOST that must be deposited for the creation of a new asset.
	pub const AssetDeposit: Balance = 100 * LOST;
	// The amount of "LOST" that must be deposited for the approval of an asset
	pub const ApprovalDeposit: Balance = 1 * LOST;
	// Maximum length of character strings, e.g. for names or descriptions of assets.
	pub const StringLimit: u32 = 50;
	// Basic deposit for the storage of asset metadata
	pub const MetadataDepositBase: Balance = 10 * LOST;
	// Additional insert per byte for the storage of asset metadata
	pub const MetadataDepositPerByte: Balance = 1 * LOST;
}
/// Configuration for digital assets with specific rules for the creation, management and storage.
impl pallet_assets::Config for Runtime {
	// Links to the runtime's central event type
	type RuntimeEvent = RuntimeEvent;
	// Type used for the balance of accounts.
	type Balance = u128;
	// Unique identifier for assets.
	type AssetId = u32;
	// Type for the compact representation of AssetId
	type AssetIdParameter = codec::Compact<u32>;
	// Origin required to create a new asset.
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	// The balance type that is used in this pallet,
	type Currency = Balances;
	// Origin required to perform certain forced actions, e.g. deleting an asset.
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	// A deposit that is required to register a new asset in the system.
	type AssetDeposit = AssetDeposit;
	// The deposit required to create a specific account to manage an asset.
	type AssetAccountDeposit = ConstU128<LOST>;
	// A basic charge for depositing metadata about an asset.
	type MetadataDepositBase = MetadataDepositBase;
	// An additional fee that is calculated based on the size of the metadata in bytes.
	type MetadataDepositPerByte = MetadataDepositPerByte;
	// The deposit required to register an approval or consent in the system.
	type ApprovalDeposit = ApprovalDeposit;
	// Limits the length of character strings within the pallet.
	type StringLimit = StringLimit;
	// Type used for freezing assets.
	type Freezer = ();
	type Extra = ();
	// Enables the calculation of transaction costs (for resource consumption) for asset operations based on the SubstrateWeight system.
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	// Limit for the number of items that can be removed in an operation
	type RemoveItemsLimit = ConstU32<1000>;
	type CallbackHandle = ();
}

/// (temp) This pallet provides a simple, albeit insecure, randomness beacon based on collective coin flipping.
impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

// COMPOSITION OF THE RUNTIME
/////////////////////////////////////////////////////////////////////////////////////

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Aura: pallet_aura,
		Grandpa: pallet_grandpa,
		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,
		Sudo: pallet_sudo,
		RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
		Contracts: pallet_contracts,
		Assets: pallet_assets,
	}
);

// Defines the format used for blockchain account addresses.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
// Specifies the structure of block headers in the blockchain.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
// Outlines the block structure for the blockchain. Encapsulating all data and transactions within a block.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
// Extends the basic transaction logic with additional checks.
pub type SignedExtra = (
	// Ensures the transaction sender is not zero.
	frame_system::CheckNonZeroSender<Runtime>,
	// Verifies if the runtime version match the current runtime specifications.
	frame_system::CheckSpecVersion<Runtime>,
	// Verifies if the transactions version match the current runtime specifications.
	frame_system::CheckTxVersion<Runtime>,
	// Confirms the transaction's genesis hash matches the chain's genesis hash.
	frame_system::CheckGenesis<Runtime>,
	// Validates the transaction's era against the current block's era.
	frame_system::CheckEra<Runtime>,
	// Ensures the transaction nonce is correct, preventing replay attacks.
	frame_system::CheckNonce<Runtime>,
	// Confirms the transaction does not exceed the block's weight limit.
	frame_system::CheckWeight<Runtime>,
	// Deducts the transaction fee from the sender's account.
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

// All migrations of the runtime, aside from the ones declared in the pallets.
#[allow(unused_parens)]
type Migrations = ();

// Represents the raw form of an extrinsic before it undergoes any validation.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
// Specifies the data structure that is signed to authenticate transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
// Orchestrates the processing of blocks and extrinsics, facilitating the interaction between the runtime and its modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime, // Runtime (frame_system)
	AllPalletsWithSystem,
	Migrations,	// Handles migrations, that is necessary during runtime upgrades
>;

// Store information about events that occur during the execution of blocks
pub type EventRecord = frame_system::EventRecord<
	<Runtime as frame_system::Config>::RuntimeEvent,
	<Runtime as frame_system::Config>::Hash,
>;

/// ADDITIONAL APIs AND BENCHMARKS
/////////////////////////////////////////////////////////////////////////////////////
/// Additional functions, APIs and user-defined logics that extend the functionality of the runtime.

// Benchmark modules for performance tests (optional)
#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_timestamp, Timestamp]
		[pallet_sudo, Sudo]
		[pallet_contracts, Contracts]
		[pallet_assets, Assets]
	);
}

// API implementations for the runtime
// Enable deeper interaction with the runtime.
impl_runtime_apis! {
	/// Enables basic functions such as retrieving the runtime version, executing and initializing blocks.
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	/// Provides access to runtimes metadata
	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	/// Used to add transactions to a block, finalize the block and generate inherent extrinsics.
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	/// Allows transactions to be validated before being added to the pool.
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	/// Defines how offchain workers act outside the blockchain
	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	/// Consensus-specific API (AURA)
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	/// Consensus-specific API (GRANDPA)
	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			_authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type with no values).
			None
		}
	}

	/// Enables the generation and decoding of session keys for validator nodes
	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	/// Enables requests for the current nonce of an account to prevent double-spending and replay attacks.
	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	/// Provides functions for requesting transaction fees and for calculating fees based on transaction size and weight.
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	/// Similar to the TransactionPaymentApi,
	/// but specifically for requesting fees and costs related to runtime calls, not extrinsic transactions.
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	/// Contracts API
	impl pallet_contracts::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash, EventRecord> for Runtime
	{
		fn call(
			origin: AccountId,
			dest: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			input_data: Vec<u8>,
		) -> pallet_contracts::ContractExecResult<Balance, EventRecord> {
			let gas_limit = gas_limit.unwrap_or(RuntimeBlockWeights::get().max_block);
			Contracts::bare_call(
				origin,
				dest,
				value,
				gas_limit,
				storage_deposit_limit,
				input_data,
				pallet_contracts::DebugInfo::UnsafeDebug,
				pallet_contracts::CollectEvents::UnsafeCollect,
				pallet_contracts::Determinism::Enforced,
			)
		}

		fn instantiate(
			origin: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			code: pallet_contracts::Code<Hash>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> pallet_contracts::ContractInstantiateResult<AccountId, Balance, EventRecord>
		{
			let gas_limit = gas_limit.unwrap_or(RuntimeBlockWeights::get().max_block);
			Contracts::bare_instantiate(
				origin,
				value,
				gas_limit,
				storage_deposit_limit,
				code,
				data,
				salt,
				pallet_contracts::DebugInfo::UnsafeDebug,
				pallet_contracts::CollectEvents::UnsafeCollect,
			)
		}

		fn upload_code(
			origin: AccountId,
			code: Vec<u8>,
			storage_deposit_limit: Option<Balance>,
			determinism: pallet_contracts::Determinism,
		) -> pallet_contracts::CodeUploadResult<Hash, Balance>
		{
			Contracts::bare_upload_code(
				origin,
				code,
				storage_deposit_limit,
				determinism,
			)
		}

		fn get_storage(
			address: AccountId,
			key: Vec<u8>,
		) -> pallet_contracts::GetStorageResult {
			Contracts::get_storage(
				address,
				key
			)
		}
	}

	/// Provides the necessary infrastructure and functions to perform the benchmarks
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use sp_storage::TrackedStorageKey;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}
	
	/// Allows runtime upgrades to be tested in a secure environment before they are performed on the live network
	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	/// Defined and customize a standard configuration for genesis block creation.
	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_config::<RuntimeGenesisConfig>(config)
		}
	}
}
