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

use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use node_runtime::constants::{currency::*, time::*};
use node_runtime::Block;
use node_runtime::{
    wasm_binary_unwrap, AssetsConfig, BabeConfig, BalancesConfig,
    CouncilConfig, DemocracyConfig, ElectionsConfig, ImOnlineConfig, IndicesConfig,
    NominationPoolsConfig, PhalaRegistryConfig, SessionConfig, SessionKeys, SocietyConfig,
    StakerStatus, StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::{ChainSpecExtension, Properties};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill,
};

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::RuntimeGenesisConfig;

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
pub type ChainSpec = sc_service::GenericChainSpec<GenesisExt, Extensions>;

/// Extension for the Phala devnet genesis config to support a custom changes to the genesis state.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GenesisExt {
    /// The runtime genesis config.
    runtime_genesis_config: RuntimeGenesisConfig,
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
    SessionKeys {
        grandpa,
        babe,
        im_online,
        authority_discovery,
    }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
    seed: &str,
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{seed}//stash")),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

fn development_config_genesis() -> RuntimeGenesisConfig {
    testnet_genesis(
        vec![authority_keys_from_seed("Alice")],
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
            block_milliseconds: Some(MILLISECS_PER_BLOCK),
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
            block_milliseconds: Some(bd),
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
            block_milliseconds: Some(MILLISECS_PER_BLOCK),
        },
        vec![],
        None,
        None,
        None,
        Some(properties),
        Default::default(),
    )
}

fn local_genesis() -> RuntimeGenesisConfig {
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
        "Phala PoC-6",
        "phala_poc_6",
        ChainType::Local,
        move || GenesisExt {
            runtime_genesis_config: testnet_local_config_genesis(),
            block_milliseconds: Some(MILLISECS_PER_BLOCK),
        },
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Staging telemetry url is valid; qed"),
        ),
        Some(protocol_id),
        None,
        Some(properties),
        Default::default(),
    )
}

fn testnet_local_config_genesis() -> RuntimeGenesisConfig {
    // stash, controller, session-key
    // generated with secret:
    // for i in 1 2 3 ; do for j in stash controller session; do subkey inspect "$secret"//phat6//$j//$i; done; done
    // and
    // for i in 1 2 3 ; do for j in session; do subkey inspect --scheme ed25519 "$secret"//phat6//$j//$i; done; done

    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![
        (
            // Stash
            // 5FEZrqVZiH4dZcF6MgbTSqQfecVZXHEHX27xL7Tii3qs4eLm
            hex!["8c3a3bd7c0a574eb77c4d5d69878bfb8d9030dfe7e73642538e71481c08afb79"].into(),
            // Controller
            // 5H3bejBtg8rmbTvGxNyKLBqMdhodieURefKiAaHMHu2Wymdc
            hex!["dc5507f1d9e93f73e61fa97350461e8778cc6fe1232b256550caf68ef27df00d"].into(),
            // Session key ed25519
            // 5F2ofyKzrgEAoCxpnrxPJGoFkDNyp6Zn8AoY33iAADoRTLdk
            hex!["8341c958f79898e2ddfb2567cfa5d11c95875626ff8c8917040f16add4f96623"]
                .unchecked_into(),
            // Session key sr25519
            // 5CBDezCwzjk6ikLT5WEVRkx1BYw91veEeu9mxWUhvMWkxBFu
            hex!["04f7c88eee3129cec90e54c8c445f36e7eba8fd215bae69fe5e32683058b655e"]
                .unchecked_into(),
            hex!["04f7c88eee3129cec90e54c8c445f36e7eba8fd215bae69fe5e32683058b655e"]
                .unchecked_into(),
            hex!["04f7c88eee3129cec90e54c8c445f36e7eba8fd215bae69fe5e32683058b655e"]
                .unchecked_into(),
        ),
        (
            // Stash
            // 5G4VNJSdTXNJrTZmwUk7ibDjhGH8VWtuXoJLhQfAcqraAVwg
            hex!["b0c6e44c3310a01ad14fa4542bb7d2e1a3ea68b57de11102288e660811608a64"].into(),
            // Controller
            // 5FHDmycrysT2YXE9KcyCBmYt1Ds78BCNtC1kb1Ah48b4B7JM
            hex!["8e405ca5d7957600050b3f839fce17d77b791ce9226da07838e2fc9e8542da7d"].into(),
            // Session key ed25519
            // 5CN1vAk5WBxaVNhrKKQuaxJNL7vN2hu6W9WDNz9zB9jRCqXR
            hex!["0d33f7ea29a03a070256f3a100930e59bbc2b14f4bf19fe934a4daeba6f289d4"]
                .unchecked_into(),
            // Session key sr25519
            // 5Dqz6DKKyJPpdzUD3PRrYPuydeiGfrX55cz7QV4LRdsd83cw
            hex!["4ec4ec6ea5905534f8db3b023be896fc9bdf9fe11b263c1ce443821fc987cd28"]
                .unchecked_into(),
            hex!["4ec4ec6ea5905534f8db3b023be896fc9bdf9fe11b263c1ce443821fc987cd28"]
                .unchecked_into(),
            hex!["4ec4ec6ea5905534f8db3b023be896fc9bdf9fe11b263c1ce443821fc987cd28"]
                .unchecked_into(),
        ),
        (
            // Stash
            // 5F9wyR7msu4UWteR8bTcBYYHyk8ZZyU5nyUus5xEj3Eqo5ud
            hex!["88b4727d666a88486eedaa26a5ccd87c9e2304c6960e65f482573e1604b5d348"].into(),
            // Controller
            // 5CLhsfvHByq8w2gTs7vdXbC9FMByciGmMko4CjAyzE64SGjj
            hex!["0c33fc14823f4acae2dc148bed3cad02ba545482a76964dd92477241491a9b13"].into(),
            // Session key ed25519
            // 5HHq7P4mBsfdiFe95LnxsRtwouPav4iczaFkEym8HuYPkrRm
            hex!["e72fc68925a9818142591465aa6de444ad4fb7b4d59c3195646c7d5b6eb5128f"]
                .unchecked_into(),
            // Session key sr25519
            // 5GTk2Y5Fs4KFEX7HXqDtKQVJ79RwhStUmBc2LDjYDuwpv2ap
            hex!["c282e228eb0df70088874ed091786faa19204068a94986c1fa7947bdebaed241"]
                .unchecked_into(),
            hex!["c282e228eb0df70088874ed091786faa19204068a94986c1fa7947bdebaed241"]
                .unchecked_into(),
            hex!["c282e228eb0df70088874ed091786faa19204068a94986c1fa7947bdebaed241"]
                .unchecked_into(),
        ),
    ];

    // generated with secret: subkey inspect -n phala --scheme Sr25519 "$secret"//phat6
    // 41rXoa2REMA1c7zVyta3s4CSG35VTUkJCFA8KE4GwuFzp57n
    let root_key: AccountId =
        hex!["36b75666c4f259ed112b01601a301a1e8add7749ab0dd97045c5407862ff0c2a"].into();

    let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

    testnet_genesis(initial_authorities, root_key, Some(endowed_accounts), false)
}

/// Helper function to create RuntimeGenesisConfig for testing
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
) -> RuntimeGenesisConfig {
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
    initial_authorities.iter().for_each(|x| {
        if !endowed_accounts.contains(&x.0) {
            endowed_accounts.push(x.0.clone())
        }
    });
    let num_endowed_accounts = endowed_accounts.len();

    const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
    const STASH: Balance = ENDOWMENT / 1000;
    // The pubkey of "0x1"
    let raw_dev_sr25519_pubkey: [u8; 32] =
        hex!["3a3d45dc55b57bf542f4c6ff41af080ec675317f4ed50ae1d2713bf9f892692d"];
    let dev_sr25519_pubkey = sp_core::sr25519::Public::from_raw(raw_dev_sr25519_pubkey);
    let dev_ecdh_pubkey =
        hex!["3a3d45dc55b57bf542f4c6ff41af080ec675317f4ed50ae1d2713bf9f892692d"].to_vec();

    let phala_registry = match dev {
        true => PhalaRegistryConfig {
            workers: vec![(
                dev_sr25519_pubkey,
                dev_ecdh_pubkey,
                Some(endowed_accounts[0].clone()),
            )],
            gatekeepers: Vec::new(),
            benchmark_duration: 1,
        },
        false => PhalaRegistryConfig {
            workers: Vec::new(),
            gatekeepers: Vec::new(),
            benchmark_duration: 50,
        },
    };

	RuntimeGenesisConfig {
        system: SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
            ..Default::default()
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| (x, ENDOWMENT))
                .collect(),
        },
        indices: IndicesConfig { indices: vec![] },
        session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        },
        staking: StakingConfig {
            validator_count: initial_authorities.len() as u32,
            minimum_validator_count: initial_authorities.len() as u32,
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
                .collect(),
            ..Default::default()
        },
        assets: AssetsConfig::default(),
        democracy: DemocracyConfig::default(),
        elections: ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, STASH))
                .collect(),
        },
        council: CouncilConfig::default(),
        technical_committee: TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
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
            ..Default::default()
        },
        im_online: ImOnlineConfig { keys: vec![] },
        authority_discovery: Default::default(),
        grandpa: Default::default(),
        treasury: Default::default(),
		society: SocietyConfig { pot: 0 },
        vesting: Default::default(),
        phala_registry,
        phala_computation: Default::default(),
        transaction_payment: Default::default(),
        nomination_pools: NominationPoolsConfig {
            min_create_bond: 10 * DOLLARS,
            #[allow(clippy::identity_op)]
            min_join_bond: DOLLARS,
            ..Default::default()
        },
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::service::{new_full_base, NewFullBase};
    use sc_service_test;
    use sp_runtime::BuildStorage;

    fn local_testnet_genesis_instant_single() -> RuntimeGenesisConfig {
        testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
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
                block_milliseconds: Some(MILLISECS_PER_BLOCK),
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
                runtime_genesis_config: testnet_local_config_genesis(),
                block_milliseconds: Some(MILLISECS_PER_BLOCK),
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
            let NewFullBase {
                task_manager,
                client,
                network,
                sync,
                transaction_pool,
                ..
            } = new_full_base(config, false, |_, _| ())?;
            Ok(sc_service_test::TestNetComponents::new(
                task_manager,
                client,
                network,
                sync,
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
        testnet_local_config().build_storage().unwrap();
    }
}
