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

use pal::Platform;
use rand::*;
use serde::{
    de::{self, DeserializeOwned, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::light_validation::LightValidation;
use std::str;
use std::{collections::BTreeMap, marker::PhantomData};
use std::{fs::File, path::PathBuf};

use anyhow::{anyhow, Context as _, Result};
use core::convert::TryInto;
use parity_scale_codec::{Decode, Encode};
use ring::rand::SecureRandom;
use serde_json::{json, Value};
use sp_core::{crypto::Pair, sr25519, H256};

// use pink::InkModule;

use phactory_api::blocks::{self, SyncCombinedHeadersReq, SyncParachainHeaderReq};
use phactory_api::ecall_args::{git_revision, InitArgs};
use phactory_api::prpc::InitRuntimeResponse;
use phactory_api::storage_sync::{
    ParachainSynchronizer, SolochainSynchronizer, StorageSynchronizer, Synchronizer,
};

use crate::light_validation::utils::storage_map_prefix_twox_64_concat;
use phala_crypto::{
    aead,
    ecdh::EcdhKey,
    sr25519::{Persistence, Sr25519SecretKey, KDF, SEED_BYTES},
};
use phala_mq::{BindTopic, ContractId, MessageDispatcher, MessageSendQueue};
use phala_pallets::pallet_mq;
use phala_serde_more as more;
use phala_types::WorkerRegistrationInfo;
use std::time::Instant;
use types::Error;

pub use contracts::pink;
pub use side_task::SideTaskManager;
pub use storage::{Storage, StorageExt};
pub use system::gk;
pub use types::BlockInfo;

pub mod benchmark;

mod bin_api_service;
mod contracts;
mod cryptography;
mod light_validation;
mod prpc_service;
mod rpc_types;
mod secret_channel;
mod side_task;
mod storage;
mod system;
mod types;

// TODO: Completely remove the reference to Phala/Khala runtime. Instead we can create a minimal
// runtime definition locally.
type RuntimeHasher = <chain::Runtime as frame_system::Config>::Hashing;

#[derive(Serialize, Deserialize)]
struct RuntimeState {
    send_mq: MessageSendQueue,

    #[serde(skip)]
    recv_mq: MessageDispatcher,

    // chain storage synchonizing
    storage_synchronizer: Synchronizer<LightValidation<chain::Runtime>>,

    // TODO.kevin: use a better serialization approach
    chain_storage: Storage,

    #[serde(with = "more::scale_bytes")]
    genesis_block_hash: H256,
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

#[derive(Serialize, Deserialize)]
enum StateVersion {
    V0,
}

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "Platform: Deserialize<'de>"))]
pub struct Phactory<Platform> {
    version: StateVersion,
    platform: Platform,
    args: InitArgs,
    skip_ra: bool,
    dev_mode: bool,
    machine_id: Vec<u8>,
    runtime_info: Option<InitRuntimeResponse>,
    runtime_state: Option<RuntimeState>,
    side_task_man: SideTaskManager,
    // The deserialzation of system requires the mq, which inside the runtime_state, to be ready.
    #[serde(skip)]
    system: Option<system::System<Platform>>,

    #[serde(skip)]
    #[serde(default = "Instant::now")]
    last_checkpoint: Instant,
}

impl<Platform: pal::Platform> Phactory<Platform> {
    pub fn new(platform: Platform) -> Self {
        let machine_id = platform.machine_id();
        Phactory {
            version: StateVersion::V0,
            platform,
            args: Default::default(),
            skip_ra: false,
            dev_mode: false,
            machine_id,
            runtime_info: None,
            runtime_state: None,
            system: None,
            side_task_man: Default::default(),
            last_checkpoint: Instant::now(),
        }
    }

    pub fn init(&mut self, args: InitArgs) {
        if args.git_revision != git_revision() {
            panic!(
                "git revision mismatch: {}(app) vs {}(enclave)",
                args.git_revision,
                git_revision()
            );
        }

        if args.init_bench {
            benchmark::resume();
        }

        self.args = args;
    }

    pub fn set_checkpoint(&mut self, file: String, interval: u64) {
        self.args.checkpoint_file = file;
        self.args.checkpoint_interval = interval;
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

impl<Platform: pal::Platform + Serialize + DeserializeOwned> Phactory<Platform> {
    pub fn take_checkpoint(&mut self) -> anyhow::Result<()> {
        use serde_cbor::ser::IoWrite;
        use serde_cbor::Serializer;

        let tmpfile = format!("{}.tmp", self.args.checkpoint_file);
        let file = File::create(&tmpfile)?;
        let mut serializer = Serializer::new(IoWrite::new(file));
        self.dump_state(&mut serializer)?;
        drop(serializer);
        std::fs::rename(&tmpfile, &self.args.checkpoint_file)?;
        info!("Checkpoint saved to {}", self.args.checkpoint_file);
        self.last_checkpoint = Instant::now();
        Ok(())
    }

    pub fn restore_from_checkpoint(checkpoint_file: &str) -> anyhow::Result<Option<Self>> {
        use serde_cbor::de::IoRead;
        use serde_cbor::Deserializer;

        let file = match File::open(checkpoint_file) {
            Ok(file) => file,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(None);
                }
                return Err(anyhow!("Failed to open checkpoint file: {}", err));
            }
        };
        let mut de = Deserializer::new(IoRead::new(file));
        let factory = Self::load_state(&mut de)?;
        Ok(Some(factory))
    }

    fn dump_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_seq(None)?;
        state.serialize_element(&self)?;
        state.serialize_element(&benchmark::dump_state())?;
        state.serialize_element(&self.system)?;
        state.end()
    }

    fn load_state<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PhactoryVisitor<Platform>(PhantomData<Platform>);

        impl<'de, Platform: pal::Platform + Serialize + DeserializeOwned> Visitor<'de>
            for PhactoryVisitor<Platform>
        {
            type Value = Phactory<Platform>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("Phactory")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut factory: Self::Value = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("Missing Phactory"))?;

                let state = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("Missing benchmark::State"))?;
                benchmark::restore_state(state);

                factory.system = {
                    let runtime_state = factory
                        .runtime_state
                        .as_mut()
                        .ok_or(de::Error::custom("Missing runtime_state"))?;

                    let recv_mq = &mut runtime_state.recv_mq;
                    let send_mq = &mut runtime_state.send_mq;
                    let seq = &mut seq;
                    phala_mq::checkpoint_helper::using_dispatcher(recv_mq, move || {
                        phala_mq::checkpoint_helper::using_send_mq(send_mq, || {
                            seq.next_element()?
                                .ok_or_else(|| de::Error::custom("Missing System"))
                        })
                    })?
                };
                Ok(factory)
            }
        }

        deserializer.deserialize_seq(PhactoryVisitor(PhantomData))
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
