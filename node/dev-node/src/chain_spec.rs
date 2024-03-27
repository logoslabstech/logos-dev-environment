#![warn(missing_docs)]

/// IMPORT AND TYPE DEFINITION
/////////////////////////////////////////////////////////////////////////////////////

// Identifies the type of the blockchain (e.g. development, test, local, or live).
use sc_service::ChainType;
// The identifier used for validators in the Aura consensus mechanism, based on the SR25519 cryptography.
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
// The identifier for authorities participating in the GRANDPA consensus protocol, based on cryptographic principles.
use sp_consensus_grandpa::AuthorityId as GrandpaId;

use dev_runtime::{
	AccountId,
	RuntimeGenesisConfig,
	Signature,
	WASM_BINARY,
	LOST,
};
use sp_core::{
	// SR25519 cryptographic signature scheme
	sr25519,
	// Cryptographic key pair (private and public keys).
	Pair,
	// Denotes the public key component of a cryptographic key pair.
	Public,
};
use sp_runtime::traits::{
	// A trait for extracting an account ID from a signature verifier, such as a public key.
	IdentifyAccount,
	// Provides functionality to verify signatures made by an account, using a specific signer's public key or identifier.
	Verify,
};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
/// This setup is crucial for defining network-specific parameters, consensus settings, initial balances,
/// and other foundational data necessary for blockchain initialization and operation.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// HELPER FUNCTIONS
/////////////////////////////////////////////////////////////////////////////////////

// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

// This setup links account identities directly to their cryptographic verification mechanism.
type AccountPublic = <Signature as Verify>::Signer;

// Generate account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

// Generate an Aura and Grandpa authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// SINGLE NODE DEV-CHAIN SPECIFICATION
/////////////////////////////////////////////////////////////////////////////////////

pub fn development_config() -> Result<ChainSpec, String> {

	let mut properties = serde_json::map::Map::new();
	properties.insert("tokenSymbol".into(), "LOST".into());
	properties.insert("tokenDecimals".into(), 18.into());
	
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Logos Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(dev_genesis(
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice")],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
		],
		true,
	))
	.with_properties(properties)
	.build())
}

/// LOCAL DEV-NETWORK CONFIGURATION
/////////////////////////////////////////////////////////////////////////////////////

pub fn local_devnet_config() -> Result<ChainSpec, String> {
	
	let mut properties = serde_json::map::Map::new();
	properties.insert("tokenSymbol".into(), "LOST".into());
	properties.insert("tokenDecimals".into(), 18.into());
	
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Logos Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(dev_genesis(
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		],
		true,
	))
	.with_properties(properties)
	.build())
}

/// DEV GENESIS CONFIGURATION
/////////////////////////////////////////////////////////////////////////////////////

/// Generates the genesis configuration for a Substrate-based blockchain using JSON format.
fn dev_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> serde_json::Value {
	serde_json::json!({
		"balances": {
			// Sets up initial balances for specified accounts
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1_000 * LOST)).collect::<Vec<_>>(),
		},
		"aura": {
			// Configures the initial set of Aura authorities for block production
			"authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
		},
		"grandpa": {
			// Sets the initial Grandpa authorities for block finalization, with each authority given a weight of 1
			"authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
		},
		"sudo": {
			// Assign network admin rights.
			"key": Some(root_key),
		},
	})
}
