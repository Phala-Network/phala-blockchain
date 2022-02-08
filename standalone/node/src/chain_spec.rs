// This file is part of Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Substrate chain configurations.

use sc_chain_spec::{ChainSpecExtension, Properties};
use sp_core::{Pair, Public, crypto::UncheckedInto, sr25519};
use serde::{Serialize, Deserialize};
use node_runtime::{
	AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, CouncilConfig,
	DemocracyConfig,GrandpaConfig, ImOnlineConfig, SessionConfig, SessionKeys, StakerStatus,
	StakingConfig, ElectionsConfig, IndicesConfig, SocietyConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, wasm_binary_unwrap,
	PhalaRegistryConfig,
};
use node_runtime::Block;
use node_runtime::constants::{currency::*, time::*};
use sc_service::ChainType;
use hex_literal::hex;
use sc_telemetry::TelemetryEndpoints;
use grandpa_primitives::{AuthorityId as GrandpaId};
use sp_consensus_babe::{AuthorityId as BabeId};
use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::GenesisConfig;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<
	GenesisExt,
	Extensions,
>;

/// Extension for the Phala devnet genesis config to support a custom changes to the genesis state.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GenesisExt {
	/// The runtime genesis config.
	runtime_genesis_config: GenesisConfig,
	/// The block duration in milliseconds.
	///
	/// If `None` is supplied, the default value is used.
	block_milliseconds: Option<u64>,
}

impl sp_runtime::BuildStorage for GenesisExt {
	fn assimilate_storage(&self, storage: &mut sp_core::storage::Storage) -> Result<(), String> {
		sp_state_machine::BasicExternalities::execute_with_storage(storage, || {
			if let Some(bm) = self.block_milliseconds.as_ref() {
				MillisecsPerBlock::set(bm);
				let bm_f = *bm as f64;
				let secs_per_block: f64 = bm_f / 1000.0;
				SecsPerBlock::set(&(secs_per_block as u64));

				let minutes = (60.0 / secs_per_block) as u32;
				let hours = minutes * 60;
				let days = hours * 24;

				Minutes::set(&minutes);
				Hours::set(&hours);
				Days::set(&days);

				SlotDuration::set(bm);
				EpochDurationInBlocks::set(&hours);

				EpochDurationInSlots::set(&(hours as u64));
			}
		});
		self.runtime_genesis_config.assimilate_storage(storage)
	}
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (
	AccountId,
	AccountId,
	GrandpaId,
	BabeId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		true,
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Phala Development",
		"phala_dev",
		ChainType::Development,
		move || GenesisExt {
			runtime_genesis_config: development_config_genesis(),
			block_milliseconds: Some(MILLISECS_PER_BLOCK)
		},
		vec![],
		None,
		None,
		None,
		None,
		Default::default(),
	)
}

/// Development config (single validator Alice, custom block duration)
pub fn development_config_custom_block_duration(bd: u64) -> ChainSpec {
	ChainSpec::from_genesis(
		"Phala Development",
		"phala_dev",
		ChainType::Development,
		move || GenesisExt {
			runtime_genesis_config: development_config_genesis(),
			block_milliseconds: Some(bd)
		},
		vec![],
		None,
		None,
		None,
		None,
		Default::default(),
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_config() -> ChainSpec {
	let properties = {
		let mut p = Properties::new();
		p.insert("tokenSymbol".into(), "PHA".into());
		p.insert("tokenDecimals".into(), 12u32.into());
		p.insert("ss58Format".into(), 30u32.into());
		p
	};

	ChainSpec::from_genesis(
		"Phala Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || GenesisExt {
			runtime_genesis_config: local_genesis(),
			block_milliseconds: Some(MILLISECS_PER_BLOCK)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Default::default(),
	)
}

fn local_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		false,
	)
}

pub fn testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/phala_testnet.json")[..])
}

pub fn testnet_local_config() -> ChainSpec {
	let boot_nodes = vec![];
	let protocol_id: &str = "phat";
	let properties = {
		let mut p = Properties::new();
		p.insert("tokenSymbol".into(), "PHA".into());
		p.insert("tokenDecimals".into(), 12u32.into());
		p.insert("ss58Format".into(), 30u32.into());
		p
	};

	ChainSpec::from_genesis(
		"Phala PoC-5",
		"phala_poc_5",
		ChainType::Local,
		move || GenesisExt {
			runtime_genesis_config: testnet_local_config_genesis(),
			block_milliseconds: Some(MILLISECS_PER_BLOCK)
		},
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Staging telemetry url is valid; qed")),
		Some(protocol_id),
		None,
		Some(properties),
		Default::default(),
	)
}

fn testnet_local_config_genesis() -> GenesisConfig {
	// stash, controller, session-key
	// generated with secret:
	// for i in 1 2 3 4 ; do for j in stash controller session; do ./phala-node key inspect "$secret"/phat/$j/$i; done; done
	// and
	// for i in 1 2 3 4 ; do for j in session; do ./phala-node key inspect --scheme ed25519 "$secret"//phat//$j//$i; done; done

	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
		(
			// Stash
			// 5En7VH7Tu4gY44oZ5jwga6gFyAqkQfNGt3nQomyuj9fQH4Kb
			hex!["780d14058a7aa9c1ee50bd5a5061847e873d602f47e3b9aa744cf26c00a67247"].into(),
			// Controller
			// 5Fpix6cfFQQcbtg2s1Ff37r5HWF5Z9QgvNvKisg2tHaKnnPu
			hex!["a6472df662dd55418f00cca35ba4b07feb3965414075caf56e563b8440ff6865"].into(),
			// Session key ed25519
			// 5HfQhACBgihgDAKzdpTTJHuheRtGMjgBDf5MxUQ26VwTHDik
			hex!["f7a4fa3658cc703ae9049b524dc3925e48d45e2e9fa941d500a28f2d247763b3"].unchecked_into(),
			// Session key sr25519
			// 5GQivZvuDiyrqYf11h55RXXqhtgft5SRt5sUHAvu5iqTEEni
			hex!["c0356f3e352b2f144fdd19204afdc31316aad2b6d851c7f76b5b9a610a87546d"].unchecked_into(),
			hex!["c0356f3e352b2f144fdd19204afdc31316aad2b6d851c7f76b5b9a610a87546d"].unchecked_into(),
			hex!["c0356f3e352b2f144fdd19204afdc31316aad2b6d851c7f76b5b9a610a87546d"].unchecked_into()
		),(
			// Stash
			// 5CmJoRR71uDKFPsBocRQTSftgynBN3o3R22mAdUjkXHQiL9F
			hex!["1ef772bec998a0b906f46e0e74e3cfe2b91115cf1cda701c77ebbb8350e0447e"].into(),
			// Controller
			// 5FR9zGMtvT2gKPdyYbazf3P6Mqpsm4Szx5DWY1oA58zd2Ue6
			hex!["944d92bb3b3a8b776972b86d1a9074e3c18ea508b967f8b7cb0479ce2c059266"].into(),
			// Session key ed25519
			// 5CpyfUqT6DcBvVok1vFNhNUvgnbvhJdmTsNtqAo8NnNsfTLz
			hex!["21c4025dd9d433e3792d245b0b5f92509badbe22b0d0d8e188557262f1182c56"].unchecked_into(),
			// Session key sr25519
			// 5E9wf8T3gfwmFywzBTQHcbexxTUKpFKX9aJ13iCx5LjKui1f
			hex!["5c77270260f15fbb819117b664ff370b6404bba6a51ff8fd4f89e20a407c3908"].unchecked_into(),
			hex!["5c77270260f15fbb819117b664ff370b6404bba6a51ff8fd4f89e20a407c3908"].unchecked_into(),
			hex!["5c77270260f15fbb819117b664ff370b6404bba6a51ff8fd4f89e20a407c3908"].unchecked_into()
		),(
			// Stash
			// 5DvGCXQ6FMpKwgmTbG9oEDS1MjKYAcYTxaB3r6yrs5KZLv1b
			hex!["520821df5b84c9c5db1fd7196713bab83b9cd958b01ce0abb1760396e3ab627b"].into(),
			// Controller
			// 5CG4bYYR7RqK6nLstPxn4S6FsSjFcfu1moGYadgqXhN5udMZ
			hex!["08a983924e693bd94877619f5f72763f0408be82069df4b4bb4836286c58f15d"].into(),
			// Session key ed25519
			// 5CsefRq3LDSThqRjWzVAPoLy9E3Q9vtRHDdYUXC7EXkyM7eE
			hex!["23cdc8621cfad1645ad1323ee25c8e4efb7f8baafdc57de78041eb8426b77396"].unchecked_into(),
			// Session key sr25519
			// 5DtTuTDML8SASebfJ1JGrUbPyDGcUwpAY6qHdSxtRVn5UMG3
			hex!["50a90bd6ac4c56f47ac761d6cc17f32e156e0ae4fa2fa6d464a01dd272d07040"].unchecked_into(),
			hex!["50a90bd6ac4c56f47ac761d6cc17f32e156e0ae4fa2fa6d464a01dd272d07040"].unchecked_into(),
			hex!["50a90bd6ac4c56f47ac761d6cc17f32e156e0ae4fa2fa6d464a01dd272d07040"].unchecked_into()
		),(
			// Stash
			// 5HmS6UaSkvs4W4QCZ83DSisw8jchD26tm3X3tjPHQZyzAjPS
			hex!["fc3d2b5f8885202bce2534726e9e33b8e47aa7cafa8c0e6331314537671b5c0c"].into(),
			// Controller
			// 5FZozFZa2Qo4J3RxeJZLsiSsZbDGNgMfrdMQfvy8mgQPgV3x
			hex!["9ae7747b65f53647d52c97e08436402b18bdeee96fb0f0c94078e03d6fe6c575"].into(),
			// Session key ed25519
			// 5ETyA4Kz4cGXWAi72aRn2NLEzBwQAaDgsqBfYon8TNV6gArL
			hex!["6a369d6f98d4cbda264eb2fa4506d381a28c545e2065413b9119767d8e6a779a"].unchecked_into(),
			// Session key sr25519
			// 5GT7V3s35JXA1ep1F3CESXrQszq814G3auczywSSF5XCm8tb
			hex!["c207de823da6c566f7be24c9d0c126cd508c231f26f3f0961e9421f875374326"].unchecked_into(),
			hex!["c207de823da6c566f7be24c9d0c126cd508c231f26f3f0961e9421f875374326"].unchecked_into(),
			hex!["c207de823da6c566f7be24c9d0c126cd508c231f26f3f0961e9421f875374326"].unchecked_into()
		),];

	// generated with secret: ./phala-node key inspect -n phala --scheme Sr25519 "$secret"/phat
	// 44ccaL1cQ8zVenUa5YHaxNbHY5orvrW3Hs2aCDFNLGKxDiDQ
	let root_key: AccountId = hex![
        "b0ceaa483e57eec37a475788ea99c063c6fc765cbdaf1822f1385761cffb972d"
    ].into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(
		initial_authorities,
		root_key,
		Some(endowed_accounts),
		false,
	)
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	dev: bool,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});
	initial_authorities.iter().for_each(|x|
		if !endowed_accounts.contains(&x.0) {
			endowed_accounts.push(x.0.clone())
		}
	);
	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;
	// The pubkey of "0x1"
	let raw_dev_sr25519_pubkey: [u8; 32] = hex!["3a3d45dc55b57bf542f4c6ff41af080ec675317f4ed50ae1d2713bf9f892692d"];
	let dev_sr25519_pubkey = sp_core::sr25519::Public::from_raw(raw_dev_sr25519_pubkey);
	let dev_ecdh_pubkey = hex!["3a3d45dc55b57bf542f4c6ff41af080ec675317f4ed50ae1d2713bf9f892692d"].to_vec();

	let phala_registry = match dev {
		true => PhalaRegistryConfig {
			workers: vec![
				(dev_sr25519_pubkey.clone(), dev_ecdh_pubkey, Some(endowed_accounts[0].clone()))
			],
			gatekeepers: Vec::new(),
			benchmark_duration: 1,
		},
		false => PhalaRegistryConfig {
			workers: Vec::new(),
			gatekeepers: Vec::new(),
			benchmark_duration: 50,
		},
	};

	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned()
				.map(|x| (x, ENDOWMENT))
				.collect()
		},
		indices: IndicesConfig {
			indices: vec![],
		},
		session: SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.0.clone(), session_keys(
					x.2.clone(),
					x.3.clone(),
					x.4.clone(),
					x.5.clone(),
				))
			}).collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
			}).collect(),
			..Default::default()
		},
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: endowed_accounts.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		technical_membership: Default::default(),
		sudo: SudoConfig {
			key: Some(root_key),
		},
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(node_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig {
			keys: vec![],
		},
		authority_discovery: AuthorityDiscoveryConfig {
			keys: vec![],
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		treasury: Default::default(),
		society: SocietyConfig {
			members: endowed_accounts.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			pot: 0,
			max_members: 999,
		},
		vesting: Default::default(),
		phala_registry,
		phala_mining: Default::default(),
		transaction_payment: Default::default(),
	}
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		testnet_genesis(
			vec![
				authority_keys_from_seed("Alice"),
			],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
			false,
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			move || GenesisExt {
				runtime_genesis_config: local_testnet_genesis_instant_single(),
				block_milliseconds: Some(MILLISECS_PER_BLOCK)
			},
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			move || GenesisExt {
				runtime_genesis_config: local_testnet_genesis(),
				block_milliseconds: Some(MILLISECS_PER_BLOCK)
			},
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sp_tracing::try_init_simple();

		sc_service_test::connectivity(integration_test_config_with_two_authorities(), |config| {
			let NewFullBase { task_manager, client, network, transaction_pool, .. } =
				new_full_base(config, |_, _| ())?;
			Ok(sc_service_test::TestNetComponents::new(
				task_manager,
				client,
				network,
				transaction_pool,
			))
		});
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		staging_testnet_config().build_storage().unwrap();
	}
}
