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

use glob::PatternError;
use rand::*;
use serde::{
    de::{self, DeserializeOwned, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::light_validation::LightValidation;
use std::collections::HashMap;
use std::{fs::File, io::ErrorKind, path::PathBuf};
use std::{io::Write, marker::PhantomData};
use std::{path::Path, str};

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
use phactory_api::storage_sync::{StorageSynchronizer, Synchronizer};

use crate::light_validation::utils::storage_map_prefix_twox_64_concat;
use phala_crypto::{
    aead,
    ecdh::EcdhKey,
    sr25519::{Persistence, Sr25519SecretKey, KDF, SEED_BYTES},
};
use phala_fair_queue::FairQueue;
use phala_mq::{BindTopic, ContractId, MessageDispatcher, MessageSendQueue};
use phala_pallets::pallet_mq;
use phala_serde_more as more;
use phala_types::{EndpointType, WorkerEndpointPayload, WorkerRegistrationInfo};
use std::time::Instant;
use types::Error;

pub use chain::BlockNumber;
pub use contracts::pink;
pub use prpc_service::RpcService;
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

const RUNTIME_SEALED_DATA_FILE: &str = "runtime-data.seal";
const CHECKPOINT_FILE: &str = "checkpoint.seal";
const CHECKPOINT_VERSION: u32 = 2;

fn checkpoint_filename_for(block_number: chain::BlockNumber, basedir: &str) -> String {
    format!("{}/{}-{:0>9}", basedir, CHECKPOINT_FILE, block_number)
}

fn checkpoint_filename_pattern(basedir: &str) -> String {
    format!("{}/{}-*", basedir, CHECKPOINT_FILE)
}

fn glob_checkpoint_files(basedir: &str) -> Result<impl Iterator<Item = PathBuf>, PatternError> {
    let pattern = checkpoint_filename_pattern(basedir);
    Ok(glob::glob(&pattern)?.filter_map(|path| path.ok()))
}

fn glob_checkpoint_files_sorted(
    basedir: &str,
) -> Result<Vec<(chain::BlockNumber, PathBuf)>, PatternError> {
    fn parse_block(filename: &Path) -> Option<chain::BlockNumber> {
        let filename = filename.to_str()?;
        let block_number = filename.rsplit('-').next()?.parse().ok()?;
        Some(block_number)
    }
    let mut files = Vec::new();

    for filename in glob_checkpoint_files(basedir)? {
        match parse_block(&filename) {
            Some(block_number) => {
                files.push((block_number, filename));
            }
            _ => {}
        }
    }
    files.sort_by_key(|(block_number, _)| std::cmp::Reverse(*block_number));
    Ok(files)
}

fn maybe_remove_checkpoints(basedir: &str) {
    match glob_checkpoint_files(basedir) {
        Err(err) => error!("Error globbing checkpoints: {:?}", err),
        Ok(iter) => {
            for filename in iter {
                if let Err(e) = std::fs::remove_file(&filename) {
                    error!("failed to remove {}: {}", filename.display(), e);
                }
            }
        }
    }
}

fn remove_outdated_checkpoints(
    basedir: &str,
    max_kept: u32,
    current_block: chain::BlockNumber,
) -> Result<()> {
    let mut kept = 0_u32;
    for (block, filename) in glob_checkpoint_files_sorted(basedir)? {
        if block > current_block {
            continue;
        }
        kept += 1;
        if kept > max_kept {
            match std::fs::remove_file(&filename) {
                Err(e) => error!("Failed to remove {}: {}", filename.display(), e),
                Ok(_) => {
                    info!("Removed {}", filename.display());
                }
            }
        }
    }
    Ok(())
}

#[derive(Encode, Decode, Clone, Debug)]
struct PersistentRuntimeData {
    genesis_block_hash: H256,
    sk: Sr25519SecretKey,
    dev_mode: bool,
}

#[derive(Encode, Decode, Clone, Debug)]
struct PersistentRuntimeDataV2 {
    genesis_block_hash: H256,
    sk: Sr25519SecretKey,
    trusted_sk: bool,
    dev_mode: bool,
}

impl PersistentRuntimeDataV2 {
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
    V2(PersistentRuntimeDataV2),
}

#[derive(Clone)]
struct SignedEndpointCache {
    endpoints: HashMap<EndpointType, Vec<u8>>,
    endpoint_payload: WorkerEndpointPayload,
    signature: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "Platform: Deserialize<'de>"))]
pub struct Phactory<Platform> {
    platform: Platform,
    pub args: InitArgs,
    skip_ra: bool,
    dev_mode: bool,
    machine_id: Vec<u8>,
    runtime_info: Option<InitRuntimeResponse>,
    runtime_state: Option<RuntimeState>,
    side_task_man: SideTaskManager,
    #[serde(skip)]
    endpoint_cache: Option<SignedEndpointCache>,
    // The deserialzation of system requires the mq, which inside the runtime_state, to be ready.
    #[serde(skip)]
    system: Option<system::System<Platform>>,

    // tmp key for WorkerKey handover encryption
    #[serde(skip)]
    pub(crate) handover_ecdh_key: Option<EcdhKey>,

    #[serde(skip)]
    #[serde(default = "Instant::now")]
    last_checkpoint: Instant,
    #[serde(skip)]
    #[serde(default)]
    last_storage_purge_at: chain::BlockNumber,
    #[serde(skip)]
    #[serde(default = "default_fair_queue")]
    query_dispatch_queue: FairQueue<ContractId>,
}

fn default_fair_queue() -> FairQueue<ContractId> {
    const FAIR_QUEUE_BACKLOG: usize = 32;
    const FAIR_QUEUE_THREADS: u32 = 8;

    FairQueue::new(FAIR_QUEUE_BACKLOG, FAIR_QUEUE_THREADS)
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
            endpoint_cache: None,
            side_task_man: Default::default(),
            handover_ecdh_key: None,
            last_checkpoint: Instant::now(),
            last_storage_purge_at: 0,
            query_dispatch_queue: default_fair_queue(),
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

    pub fn set_args(&mut self, args: InitArgs) {
        self.args = args;
        if let Some(system) = &mut self.system {
            system.sealing_path = self.args.sealing_path.clone();
            system.storage_path = self.args.storage_path.clone();
            system.geoip_city_db = self.args.geoip_city_db.clone();
        }
    }

    fn init_runtime_data(
        &self,
        genesis_block_hash: H256,
        predefined_identity_key: Option<sr25519::Pair>,
    ) -> Result<PersistentRuntimeDataV2> {
        let data = if let Some(identity_sk) = predefined_identity_key {
            self.save_runtime_data(genesis_block_hash, identity_sk, false, true)?
        } else {
            match Self::load_runtime_data(&self.platform, &self.args.sealing_path) {
                Ok(data) => data,
                Err(Error::PersistentRuntimeNotFound) => {
                    warn!("Persistent data not found.");
                    let identity_sk = new_sr25519_key();
                    self.save_runtime_data(genesis_block_hash, identity_sk, true, false)?
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
        trusted_sk: bool,
        dev_mode: bool,
    ) -> Result<PersistentRuntimeDataV2> {
        // Put in PresistentRuntimeData
        let sk = sr25519_sk.dump_secret_key();

        let data = PersistentRuntimeDataV2 {
            genesis_block_hash,
            sk,
            trusted_sk,
            dev_mode,
        };
        {
            let data = RuntimeDataSeal::V2(data.clone());
            let encoded_vec = data.encode();
            info!("Length of encoded slice: {}", encoded_vec.len());
            let filepath = PathBuf::from(&self.args.sealing_path).join(RUNTIME_SEALED_DATA_FILE);
            self.platform
                .seal_data(filepath, &encoded_vec)
                .map_err(Into::into)
                .context("Failed to seal runtime data")?;
            info!("Persistent Runtime Data V2 saved");
        }
        Ok(data)
    }

    fn load_runtime_data(
        platform: &Platform,
        sealing_path: &str,
    ) -> Result<PersistentRuntimeDataV2, Error> {
        let filepath = PathBuf::from(sealing_path).join(RUNTIME_SEALED_DATA_FILE);
        let data = platform
            .unseal_data(filepath)
            .map_err(Into::into)?
            .ok_or(Error::PersistentRuntimeNotFound)?;
        let data: RuntimeDataSeal = Decode::decode(&mut &data[..]).map_err(Error::DecodeError)?;
        match data {
            RuntimeDataSeal::V1(data) => {
                Ok(PersistentRuntimeDataV2 {
                    genesis_block_hash: data.genesis_block_hash,
                    sk: data.sk,
                    // key injection is impossible for legacy persistent data
                    trusted_sk: true,
                    dev_mode: data.dev_mode,
                })
            }
            RuntimeDataSeal::V2(data) => Ok(data),
        }
    }
}

impl<P: pal::Platform> Phactory<P> {
    // Restored from checkpoint
    pub fn on_restored(&mut self) -> Result<()> {
        if let Some(system) = &mut self.system {
            system.on_restored()?;
        }
        Ok(())
    }
}

impl<Platform: pal::Platform + Serialize + DeserializeOwned> Phactory<Platform> {
    pub fn take_checkpoint(&mut self, current_block: chain::BlockNumber) -> anyhow::Result<()> {
        let key = self
            .system
            .as_ref()
            .context("Take checkpoint failed, runtime is not ready")?
            .identity_key
            .dump_secret_key();
        info!("Taking checkpoint...");
        let checkpoint_file = checkpoint_filename_for(current_block, &self.args.storage_path);
        let file = File::create(&checkpoint_file).context("Failed to create checkpoint file")?;
        self.take_checkpoint_to_writer(&key, file)
            .context("Take checkpoint to writer failed")?;
        info!("Checkpoint saved to {}", checkpoint_file);
        self.last_checkpoint = Instant::now();
        remove_outdated_checkpoints(
            &self.args.storage_path,
            self.args.max_checkpoint_files,
            current_block,
        )?;
        Ok(())
    }

    pub fn take_checkpoint_to_writer<W: std::io::Write>(
        &mut self,
        key: &[u8],
        writer: W,
    ) -> anyhow::Result<()> {
        let key128 = derive_key_for_checkpoint(&key);
        let nonce = rand::thread_rng().gen();
        let mut enc_writer = aead::stream::new_aes128gcm_writer(key128, nonce, writer);
        serde_cbor::ser::to_writer(&mut enc_writer, &PhactoryDumper(self))
            .context("Failed to write checkpoint")?;
        enc_writer
            .flush()
            .context("Failed to flush encrypted writer")?;
        Ok(())
    }

    pub fn restore_from_checkpoint(
        platform: &Platform,
        sealing_path: &str,
        storage_path: &str,
        remove_corrupted_checkpoint: bool,
        n_workers: usize,
    ) -> anyhow::Result<Option<Self>> {
        let runtime_data = match Self::load_runtime_data(platform, sealing_path) {
            Err(Error::PersistentRuntimeNotFound) => return Ok(None),
            other => other.context("Failed to load persistent data")?,
        };
        let files =
            glob_checkpoint_files_sorted(storage_path).context("Glob checkpoint files failed")?;
        if files.is_empty() {
            return Ok(None);
        }
        let (_block, ckpt_filename) = &files[0];

        let file = match File::open(&ckpt_filename) {
            Ok(file) => file,
            Err(err) if matches!(err.kind(), ErrorKind::NotFound) => {
                // This should never happen unless it was removed just after the glob.
                anyhow::bail!("Checkpoint file {:?} is not found", ckpt_filename);
            }
            Err(err) => {
                error!(
                    "Failed to open checkpoint file {:?}: {:?}",
                    ckpt_filename, err
                );
                if remove_corrupted_checkpoint {
                    error!("Removing {:?}", ckpt_filename);
                    std::fs::remove_file(&ckpt_filename)
                        .context("Failed to remove corrupted checkpoint file")?;
                }
                anyhow::bail!(
                    "Failed to open checkpoint file {:?}: {:?}",
                    ckpt_filename,
                    err
                );
            }
        };

        match Self::restore_from_checkpoint_reader(&runtime_data.sk, file, n_workers) {
            Ok(state) => {
                info!("Succeeded to load checkpoint file {:?}", ckpt_filename);
                Ok(Some(state))
            }
            Err(_err /*Don't leak it into the log*/) => {
                error!("Failed to load checkpoint file {:?}", ckpt_filename);
                if remove_corrupted_checkpoint {
                    error!("Removing {:?}", ckpt_filename);
                    std::fs::remove_file(&ckpt_filename)
                        .context("Failed to remove corrupted checkpoint file")?;
                }
                anyhow::bail!("Failed to load checkpoint file {:?}", ckpt_filename);
            }
        }
    }

    pub fn restore_from_checkpoint_reader<R: std::io::Read>(
        key: &[u8],
        reader: R,
        n_workers: usize,
    ) -> anyhow::Result<Self> {
        let key128 = derive_key_for_checkpoint(key);
        let dec_reader = aead::stream::new_aes128gcm_reader(key128, reader);
        system::sidevm_config(n_workers);
        let loader: PhactoryLoader<_> =
            serde_cbor::de::from_reader(dec_reader).context("Failed to decode state")?;
        Ok(loader.0)
    }
}

impl<Platform: Serialize + DeserializeOwned> Phactory<Platform> {
    fn dump_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_seq(None)?;
        state.serialize_element(&CHECKPOINT_VERSION)?;
        state.serialize_element(&benchmark::dump_state())?;
        state.serialize_element(&self)?;
        state.serialize_element(&self.system)?;
        state.end()
    }

    fn load_state<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PhactoryVisitor<Platform>(PhantomData<Platform>);

        impl<'de, Platform: Serialize + DeserializeOwned> Visitor<'de> for PhactoryVisitor<Platform> {
            type Value = Phactory<Platform>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("Phactory")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let version: u32 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("Checkpoint version missing"))?;
                if version > CHECKPOINT_VERSION {
                    return Err(de::Error::custom(format!(
                        "Checkpoint version {} is not supported",
                        version
                    )));
                }

                let state = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("Missing benchmark::State"))?;

                let mut factory: Self::Value = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("Missing Phactory"))?;

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
                benchmark::restore_state(state);
                Ok(factory)
            }
        }

        deserializer.deserialize_seq(PhactoryVisitor(PhantomData))
    }
}

struct PhactoryDumper<'a, Platform>(&'a Phactory<Platform>);
impl<Platform: Serialize + DeserializeOwned> Serialize for PhactoryDumper<'_, Platform> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.dump_state(serializer)
    }
}
struct PhactoryLoader<Platform>(Phactory<Platform>);
impl<'de, Platform: Serialize + DeserializeOwned + pal::Platform> Deserialize<'de>
    for PhactoryLoader<Platform>
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut factory = Phactory::load_state(deserializer)?;
        factory
            .on_restored()
            .map_err(|err| de::Error::custom(format!("Could not restore Phactory: {:?}", err)))?;
        Ok(Self(factory))
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

fn derive_key_for_checkpoint(identity_key: &[u8]) -> [u8; 16] {
    sp_core::blake2_128(&(identity_key, b"/checkpoint").encode())
}
