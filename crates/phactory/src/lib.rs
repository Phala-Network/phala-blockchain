#![warn(unused_imports)]
#![warn(unused_extern_crates)]
#![feature(bench_black_box)]
#![feature(panic_unwind)]
#![feature(c_variadic)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate phactory_pal as pal;
extern crate runtime as chain;

use rand::*;
use side_task::SideTaskManager;

use crate::light_validation::LightValidation;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str;

use anyhow::{anyhow, Context as _, Result};
use core::convert::TryInto;
use parity_scale_codec::{Decode, Encode};
use ring::rand::SecureRandom;
use serde_json::{json, Value};
use sp_core::{crypto::Pair, sr25519, H256};

// use pink::InkModule;

use phactory_api::ecall_args::{InitArgs, git_revision};
use phactory_api::blocks::{self, SyncCombinedHeadersReq, SyncParachainHeaderReq};
use phactory_api::prpc::InitRuntimeResponse;
use phactory_api::storage_sync::{
    ParachainSynchronizer, SolochainSynchronizer, StorageSynchronizer,
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

mod bin_api_service;
mod contracts;
mod cryptography;
mod light_validation;
mod prpc_service;
mod rpc_types;
mod secret_channel;
mod storage;
mod system;
mod types;
mod side_task;

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

/// Master key filepath
pub const RUNTIME_SEALED_DATA_FILE: &str = "runtime-data.seal";

#[derive(Encode, Decode, Clone, Debug)]
struct PersistentRuntimeData {
    genesis_block_hash: H256,
    sk: Sr25519SecretKey,
    dev_mode: bool,
}

impl PersistentRuntimeData {
    pub fn decode_keys(&self) -> (sr25519::Pair, EcdhKey) {
        // load identity
        let identity_sk = sr25519::Pair::restore_from_secret_key(&self.sk);
        info!("Identity pubkey: {:?}", hex::encode(&identity_sk.public()));

        // derive ecdh key
        let ecdh_key = identity_sk
            .derive_ecdh_key()
            .expect("Unable to derive ecdh key");
        info!("ECDH pubkey: {:?}", hex::encode(&ecdh_key.public()));
        (identity_sk, ecdh_key)
    }
}

#[derive(Encode, Decode, Clone, Debug)]
enum RuntimeDataSeal {
    V1(PersistentRuntimeData),
}

pub struct Phactory<Platform> {
    platform: Platform,
    args: InitArgs,
    skip_ra: bool,
    dev_mode: bool,
    machine_id: Vec<u8>,
    runtime_info: Option<InitRuntimeResponse>,
    runtime_state: Option<RuntimeState>,
    system: Option<system::System<Platform>>,
    side_task_man: SideTaskManager,
}

impl<Platform: pal::Platform> Phactory<Platform> {
    pub fn new(platform: Platform) -> Self {
        let machine_id = platform.machine_id();
        Phactory {
            platform,
            args: Default::default(),
            skip_ra: false,
            dev_mode: false,
            machine_id,
            runtime_info: None,
            runtime_state: None,
            system: None,
            side_task_man: Default::default(),
        }
    }

    pub fn init(&mut self, args: InitArgs) {
        if args.git_revision != git_revision() {
            panic!("git revision mismatch: {}(app) vs {}(enclave)", args.git_revision, git_revision());
        }

        if args.init_bench {
            benchmark::resume();
        }

        self.args = args;
    }

    fn init_runtime_data(
        &self,
        genesis_block_hash: H256,
        predefined_identity_key: Option<sr25519::Pair>,
    ) -> Result<PersistentRuntimeData> {
        let data = if let Some(identity_sk) = predefined_identity_key {
            self.save_runtime_data(genesis_block_hash, identity_sk, true)?
        } else {
            match self.load_runtime_data() {
                Ok(data) => data,
                Err(Error::PersistentRuntimeNotFound) => {
                    warn!("Persistent data not found.");
                    let identity_sk = new_sr25519_key();
                    self.save_runtime_data(genesis_block_hash, identity_sk, false)?
                }
                Err(err) => return Err(anyhow!("Failed to load persistent data: {}", err)),
            }
        };

        // check genesis block hash
        if genesis_block_hash != data.genesis_block_hash {
            panic!(
                "Genesis block hash mismatches with saved keys, expected {}",
                data.genesis_block_hash
            );
        }
        info!("Machine id: {:?}", hex::encode(&self.machine_id));
        info!("Init done.");
        Ok(data)
    }

    fn save_runtime_data(
        &self,
        genesis_block_hash: H256,
        sr25519_sk: sr25519::Pair,
        dev_mode: bool,
    ) -> Result<PersistentRuntimeData> {
        // Put in PresistentRuntimeData
        let sk = sr25519_sk.dump_secret_key();

        let data = PersistentRuntimeData {
            genesis_block_hash,
            sk,
            dev_mode,
        };
        {
            let data = RuntimeDataSeal::V1(data.clone());
            let encoded_vec = data.encode();
            info!("Length of encoded slice: {}", encoded_vec.len());
            let filepath = PathBuf::from(&self.args.sealing_path).join(RUNTIME_SEALED_DATA_FILE);
            self.platform
                .seal_data(filepath, &encoded_vec)
                .map_err(Into::into)
                .context("Seal runtime data")?;
            info!("Persistent Runtime Data saved");
        }
        Ok(data)
    }

    fn load_runtime_data(&self) -> Result<PersistentRuntimeData, Error> {
        let filepath = PathBuf::from(&self.args.sealing_path).join(RUNTIME_SEALED_DATA_FILE);
        let data = self
            .platform
            .unseal_data(filepath)
            .map_err(Into::into)?
            .ok_or(Error::PersistentRuntimeNotFound)?;
        let data: RuntimeDataSeal = Decode::decode(&mut &data[..]).map_err(Error::DecodeError)?;
        match data {
            RuntimeDataSeal::V1(data) => Ok(data),
        }
    }
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
