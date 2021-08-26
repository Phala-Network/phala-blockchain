#![warn(unused_imports)]
#![warn(unused_extern_crates)]
#![feature(bench_black_box)]
#![feature(panic_unwind)]
#![feature(c_variadic)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate runtime as chain;
extern crate phactory_pal as pal;

use std;

use rand::*;

use crate::light_validation::LightValidation;
use crate::secret_channel::PeelingReceiver;
use std::collections::BTreeMap;
use std::str;

use anyhow::Result;
use core::convert::TryInto;
use parity_scale_codec::{Decode, Encode};
use ring::rand::SecureRandom;
use serde_json::{Value, json};
use sp_core::{crypto::Pair, sr25519, H256};

// use pink::InkModule;

use phactory_api::prpc::InitRuntimeResponse;
use phactory_api::storage_sync::{
    ParachainSynchronizer, SolochainSynchronizer, StorageSynchronizer,
};
use phactory_api::{
    blocks::{self, SyncCombinedHeadersReq, SyncParachainHeaderReq},
};

use phala_crypto::{
    aead,
    ecdh::EcdhKey,
    sr25519::{Persistence, Sr25519SecretKey, KDF, SEED_BYTES},
};
use phala_mq::{BindTopic, ContractId, MessageDispatcher, MessageOrigin, MessageSendQueue};
use phala_pallets::pallet_mq;
use phala_types::WorkerRegistrationInfo;

pub mod benchmark;

mod contracts;
mod cryptography;
mod light_validation;
mod prpc_service;
mod rpc_types;
mod secret_channel;
mod storage;
mod system;
mod types;

use crate::light_validation::utils::storage_map_prefix_twox_64_concat;
use contracts::{ExecuteEnv, SYSTEM};
use storage::{Storage, StorageExt};
use types::BlockInfo;
use types::Error;

// TODO: Completely remove the reference to Phala/Khala runtime. Instead we can create a minimal
// runtime definition locally.
type RuntimeHasher = <chain::Runtime as frame_system::Config>::Hashing;

struct RuntimeState {
    contracts: BTreeMap<ContractId, Box<dyn contracts::Contract + Send>>,
    send_mq: MessageSendQueue,
    recv_mq: MessageDispatcher,

    // chain storage synchonizing
    storage_synchronizer: Box<dyn StorageSynchronizer + Send>,
    chain_storage: Storage,
    genesis_block_hash: H256,
    identity_key: sr25519::Pair,
    ecdh_key: EcdhKey,
}

impl RuntimeState {
    fn purge_mq(&mut self) {
        self.send_mq.purge(|sender| {
            use pallet_mq::StorageMapTrait as _;
            type OffchainIngress = pallet_mq::OffchainIngress<chain::Runtime>;

            let module_prefix = OffchainIngress::module_prefix();
            let storage_prefix = OffchainIngress::storage_prefix();
            let key = storage_map_prefix_twox_64_concat(module_prefix, storage_prefix, sender);
            let sequence: u64 = self.chain_storage.get_decoded(&key).unwrap_or(0);
            debug!("purging, sequence = {}", sequence);
            sequence
        })
    }
}

pub struct Phactory<Platform> {
    platform: Platform,
    sealing_path: String,
    skip_ra: bool,
    dev_mode: bool,
    machine_id: Vec<u8>,
    runtime_info: Option<InitRuntimeResponse>,
    runtime_state: Option<RuntimeState>,
    system: Option<system::System<Platform>>,
}

impl<Platform: pal::Platform> Phactory<Platform> {
    pub fn new(platform: Platform) -> Self {
        let machine_id = platform.machine_id();
        Phactory {
            platform,
            sealing_path: Default::default(),
            skip_ra: false,
            dev_mode: false,
            machine_id,
            runtime_info: None,
            runtime_state: None,
            system: None,
        }
    }


    fn init_secret_keys(
        &self,
        genesis_block_hash: H256,
        predefined_identity_key: Option<sr25519::Pair>,
    ) -> Result<PersistentRuntimeData> {
        let data = if let Some(sr25519_sk) = predefined_identity_key {
            save_secret_keys(genesis_block_hash, sr25519_sk, true)?
        } else {
            match load_secret_keys() {
                Ok(data) => data,
                Err(e)
                    if e.is::<Error>()
                        && matches!(
                            e.downcast_ref::<Error>().unwrap(),
                            Error::PersistentRuntimeNotFound
                        ) =>
                {
                    warn!("Persistent data not found.");
                    let sr25519_sk = new_sr25519_key();
                    save_secret_keys(genesis_block_hash, sr25519_sk, false)?
                }
                other_err => return other_err,
            }
        };

        // check genesis block hash
        let saved_genesis_block_hash: [u8; 32] = hex::decode(&data.genesis_block_hash)
            .expect("Unable to decode genesis block hash hex")
            .as_slice()
            .try_into()
            .expect("slice with incorrect length");
        let saved_genesis_block_hash = H256::from(saved_genesis_block_hash);
        if genesis_block_hash != saved_genesis_block_hash {
            panic!(
                "Genesis block hash mismatches with saved keys, expected {}",
                saved_genesis_block_hash
            );
        }

        // load identity
        let sr25519_raw_key: Sr25519SecretKey = hex::decode(&data.sk)
            .expect("Unable to decode identity key hex")
            .as_slice()
            .try_into()
            .expect("slice with incorrect length");

        let sr25519_sk = sr25519::Pair::restore_from_secret_key(&sr25519_raw_key);
        info!("Identity pubkey: {:?}", hex::encode(&sr25519_sk.public()));

        // derive ecdh key
        let ecdh_key = sr25519_sk
            .derive_ecdh_key()
            .expect("Unable to derive ecdh key");
        let ecdh_hex_pk = hex::encode(ecdh_key.public().as_ref());
        info!("ECDH pubkey: {:?}", ecdh_hex_pk);

        info!("Machine id: {:?}", hex::encode(&self.machine_id));
        info!("Init done.");
        Ok(data)
    }
}

// For bin_api
impl<Platform: pal::Platform> Phactory<Platform> {
    pub fn bin_sync_header(&mut self, input: blocks::SyncHeaderReq) -> Result<Value, Value> {
        let resp =
            self.sync_header(input.headers, input.authority_set_change).map_err(display)?;
        Ok(json!({ "synced_to": resp.synced_to }))
    }

    pub fn bin_sync_para_header(&mut self, input: SyncParachainHeaderReq) -> Result<Value, Value> {
        let resp = self.sync_para_header(input.headers, input.proof).map_err(display)?;
        Ok(json!({ "synced_to": resp.synced_to }))
    }

    pub fn bin_sync_combined_headers(&mut self, input: SyncCombinedHeadersReq) -> Result<Value, Value> {
        let resp = self.sync_combined_headers(
            input.relaychain_headers,
            input.authority_set_change,
            input.parachain_headers,
            input.proof,
        )
        .map_err(display)?;
        Ok(json!({
            "relaychain_synced_to": resp.relaychain_synced_to,
            "parachain_synced_to": resp.parachain_synced_to,
        }))
    }

    pub fn bin_dispatch_block(&mut self, input: blocks::DispatchBlockReq) -> Result<Value, Value> {
        let resp = self.dispatch_block(input.blocks).map_err(display)?;
        Ok(json!({ "dispatched_to": resp.synced_to }))
    }
}

struct PersistentRuntimeData {
    genesis_block_hash: String,
    sk: String,
    dev_mode: bool,
}

impl PersistentRuntimeData {
    pub fn identity_key(&self) -> sr25519::Pair {
        todo!("")
    }
    pub fn ecdh_key(&self) -> EcdhKey {
        todo!("")
    }
}

fn save_secret_keys(
    _genesis_block_hash: H256,
    _sr25519_sk: sr25519::Pair,
    _dev_mode: bool,
) -> Result<PersistentRuntimeData> {
    // TODO.kevin: save secret key to disk
    todo!()
}

fn load_secret_keys() -> Result<PersistentRuntimeData> {
    // TODO.kevin: load secret key from disk
    todo!()
}

fn new_sr25519_key() -> sr25519::Pair {
    let mut rng = rand::thread_rng();
    let mut seed = [0_u8; SEED_BYTES];
    rng.fill_bytes(&mut seed);
    sr25519::Pair::from_seed(&seed)
}

// TODO.kevin: Move to phactory-api when the std ready.
fn generate_random_iv() -> aead::IV {
    let mut nonce_vec = [0u8; aead::IV_BYTES];
    let rand = ring::rand::SystemRandom::new();
    rand.fill(&mut nonce_vec).unwrap();
    nonce_vec
}

fn generate_random_info() -> [u8; 32] {
    let mut nonce_vec = [0u8; 32];
    let rand = ring::rand::SystemRandom::new();
    rand.fill(&mut nonce_vec).unwrap();
    nonce_vec
}


// --------------------------------

fn display(e: impl core::fmt::Display) -> Value {
    error_msg(&e.to_string())
}

fn error_msg(msg: &str) -> Value {
    json!({ "message": msg })
}
