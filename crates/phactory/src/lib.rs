#![warn(unused_imports)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate log;
extern crate phactory_pal as pal;
extern crate runtime as chain;

use ::pink::types::AccountId;
use contracts::pink::http_counters;
use glob::PatternError;
use rand::*;
use serde::{
    de::{self, DeserializeOwned, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::light_validation::LightValidation;
use std::{collections::BTreeMap, str::FromStr};
use std::{fs::File, io::ErrorKind, path::PathBuf};
use std::{io::Write, marker::PhantomData};
use std::{path::Path, str};

use anyhow::{anyhow, Context as _, Result};
use core::convert::TryInto;
use parity_scale_codec::{Decode, Encode};
use phala_types::AttestationProvider;
use ring::rand::SecureRandom;
use serde_json::{json, Value};
use sp_core::{crypto::Pair, sr25519, H256};

// use pink::InkModule;

use phactory_api::{
    blocks::{self, SyncCombinedHeadersReq, SyncParachainHeaderReq},
    ecall_args::InitArgs,
    endpoints::EndpointType,
    prpc::{self as pb, GetEndpointResponse, InitRuntimeResponse, NetworkConfig},
    storage_sync::{StorageSynchronizer, Synchronizer},
};

use phala_crypto::{
    aead,
    ecdh::EcdhKey,
    sr25519::{Persistence, Sr25519SecretKey, KDF, SEED_BYTES},
};
use phala_mq::{BindTopic, MessageDispatcher, MessageSendQueue};
use phala_scheduler::RequestScheduler;
use phala_serde_more as more;
use std::time::Instant;
use types::Error;

pub use chain::BlockNumber;
pub use contracts::pink;
pub use prpc_service::RpcService;
pub use storage::ChainStorage;
pub use system::gk;
pub use types::BlockInfo;
pub type PRuntimeLightValidation = LightValidation<chain::Runtime>;

pub mod benchmark;

mod bin_api_service;
mod contracts;
mod cryptography;
mod light_validation;
mod prpc_service;
mod secret_channel;
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
    chain_storage: ChainStorage,

    #[serde(with = "more::scale_bytes")]
    genesis_block_hash: H256,

    para_id: u32,
}

impl RuntimeState {
    fn purge_mq(&mut self) {
        self.send_mq
            .purge(|sender| self.chain_storage.mq_sequence(sender))
    }
}

const RUNTIME_SEALED_DATA_FILE: &str = "runtime-data.seal";
const CHECKPOINT_FILE: &str = "checkpoint.seal";
const CHECKPOINT_VERSION: u32 = 2;

fn checkpoint_filename_for(block_number: chain::BlockNumber, basedir: &str) -> String {
    format!("{basedir}/{CHECKPOINT_FILE}-{block_number:0>9}")
}

fn checkpoint_info_filename_for(filename: &str) -> String {
    format!("{filename}.info.json")
}

fn checkpoint_filename_pattern(basedir: &str) -> String {
    format!("{basedir}/{CHECKPOINT_FILE}-*")
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
        if let Some(block_number) = parse_block(&filename) {
            files.push((block_number, filename));
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
            match remove_checkpoint(&filename) {
                Err(e) => error!("Failed to remove checkpoint {}: {e}", filename.display()),
                Ok(_) => {
                    info!("Removed {}", filename.display());
                }
            }
        }
    }
    Ok(())
}

fn remove_checkpoint(filename: &Path) -> Result<()> {
    std::fs::remove_file(filename).context("Failed to remove checkpoint file")?;
    let info_filename = checkpoint_info_filename_for(&filename.display().to_string());
    std::fs::remove_file(info_filename).context("Failed to remove checkpoint info file")?;
    Ok(())
}

#[derive(Encode, Decode, Clone, Debug)]
struct PersistentRuntimeData {
    genesis_block_hash: H256,
    para_id: u32,
    sk: Sr25519SecretKey,
    trusted_sk: bool,
    dev_mode: bool,
}

impl PersistentRuntimeData {
    pub fn decode_keys(&self) -> (sr25519::Pair, EcdhKey) {
        // load identity
        let identity_sk = sr25519::Pair::restore_from_secret_key(&self.sk);
        info!("Identity pubkey: {:?}", hex::encode(identity_sk.public()));

        // derive ecdh key
        let ecdh_key = identity_sk
            .derive_ecdh_key()
            .expect("Unable to derive ecdh key");
        info!("ECDH pubkey: {:?}", hex::encode(ecdh_key.public()));
        (identity_sk, ecdh_key)
    }
}

#[derive(Encode, Decode, Clone, Debug)]
enum RuntimeDataSeal {
    V1(PersistentRuntimeData),
}

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "Platform: Deserialize<'de>"))]
pub struct Phactory<Platform> {
    platform: Platform,
    #[serde(skip)]
    pub args: InitArgs,
    dev_mode: bool,
    attestation_provider: Option<AttestationProvider>,
    machine_id: Vec<u8>,
    runtime_info: Option<InitRuntimeResponse>,
    runtime_state: Option<RuntimeState>,
    endpoints: BTreeMap<EndpointType, String>,
    #[serde(skip)]
    signed_endpoints: Option<GetEndpointResponse>,
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
    #[serde(default = "default_query_scheduler")]
    query_scheduler: RequestScheduler<AccountId>,

    #[serde(default)]
    netconfig: Option<NetworkConfig>,

    #[serde(skip)]
    can_load_chain_state: bool,

    #[serde(skip)]
    trusted_sk: bool,

    #[serde(skip)]
    #[serde(default = "Instant::now")]
    started_at: Instant,
}

fn default_query_scheduler() -> RequestScheduler<AccountId> {
    const FAIR_QUEUE_BACKLOG: usize = 32;
    const FAIR_QUEUE_THREADS: u32 = 8;

    RequestScheduler::new(FAIR_QUEUE_BACKLOG, FAIR_QUEUE_THREADS)
}

impl<Platform: pal::Platform> Phactory<Platform> {
    pub fn new(platform: Platform) -> Self {
        let machine_id = platform.machine_id();
        Phactory {
            platform,
            args: Default::default(),
            dev_mode: false,
            attestation_provider: None,
            machine_id,
            runtime_info: None,
            runtime_state: None,
            system: None,
            endpoints: Default::default(),
            signed_endpoints: None,
            handover_ecdh_key: None,
            last_checkpoint: Instant::now(),
            query_scheduler: default_query_scheduler(),
            netconfig: Default::default(),
            can_load_chain_state: false,
            trusted_sk: false,
            started_at: Instant::now(),
        }
    }

    pub fn init(&mut self, args: InitArgs) {
        if args.init_bench {
            benchmark::resume();
        }

        self.can_load_chain_state = !system::gk_master_key_exists(&args.sealing_path);
        self.args = args;
    }

    pub fn set_args(&mut self, args: InitArgs) {
        self.args = args;
        if let Some(system) = &mut self.system {
            system.sealing_path = self.args.sealing_path.clone();
            system.storage_path = self.args.storage_path.clone();
        }
    }

    fn init_runtime_data(
        &self,
        genesis_block_hash: H256,
        para_id: u32,
        predefined_identity_key: Option<sr25519::Pair>,
    ) -> Result<PersistentRuntimeData> {
        let data = if let Some(identity_sk) = predefined_identity_key {
            self.save_runtime_data(genesis_block_hash, para_id, identity_sk, false, true)?
        } else {
            match Self::load_runtime_data(&self.platform, &self.args.sealing_path) {
                Ok(data) => data,
                Err(Error::PersistentRuntimeNotFound) => {
                    warn!("Persistent data not found.");
                    let identity_sk = new_sr25519_key();
                    self.save_runtime_data(genesis_block_hash, para_id, identity_sk, true, false)?
                }
                Err(err) => return Err(anyhow!("Failed to load persistent data: {}", err)),
            }
        };

        // check genesis block hash
        if genesis_block_hash != data.genesis_block_hash {
            anyhow::bail!(
                "Genesis block hash mismatches with saved keys, expected {}",
                data.genesis_block_hash
            );
        }
        if para_id != data.para_id {
            anyhow::bail!(
                "Parachain id mismatches, saved: {}, in state: {para_id}",
                data.para_id
            );
        }
        info!("Init done.");
        Ok(data)
    }

    fn save_runtime_data(
        &self,
        genesis_block_hash: H256,
        para_id: u32,
        sr25519_sk: sr25519::Pair,
        trusted_sk: bool,
        dev_mode: bool,
    ) -> Result<PersistentRuntimeData> {
        // Put in PresistentRuntimeData
        let sk = sr25519_sk.dump_secret_key();

        let data = PersistentRuntimeData {
            genesis_block_hash,
            para_id,
            sk,
            trusted_sk,
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
                .context("Failed to seal runtime data")?;
            info!("Persistent Runtime Data V2 saved");
        }
        Ok(data)
    }

    fn load_runtime_data(
        platform: &Platform,
        sealing_path: &str,
    ) -> Result<PersistentRuntimeData, Error> {
        let filepath = PathBuf::from(sealing_path).join(RUNTIME_SEALED_DATA_FILE);
        let data = platform
            .unseal_data(filepath)
            .map_err(Into::into)?
            .ok_or(Error::PersistentRuntimeNotFound)?;
        let data: RuntimeDataSeal = Decode::decode(&mut &data[..]).map_err(Error::DecodeError)?;
        match data {
            RuntimeDataSeal::V1(data) => Ok(data),
        }
    }

    pub fn set_netconfig(&mut self, config: NetworkConfig) {
        self.netconfig = Some(config);
        self.reconfigure_network();
    }

    fn reconfigure_network(&self) {
        let config = match &self.netconfig {
            None => return,
            Some(config) => config,
        };
        fn reconfig_one(name: &str, value: &str) {
            if value.is_empty() {
                std::env::remove_var(name);
            } else {
                std::env::set_var(name, value);
            }
        }
        reconfig_one("all_proxy", &config.all_proxy);
        reconfig_one("i2p_proxy", &config.i2p_proxy);
    }
}

impl<P: pal::Platform> Phactory<P> {
    // Restored from checkpoint
    pub fn on_restored(&mut self) -> Result<()> {
        self.check_requirements();
        self.reconfigure_network();
        self.update_runtime_info(|_| {});
        self.trusted_sk =
            Self::load_runtime_data(&self.platform, &self.args.sealing_path)?.trusted_sk;
        if let Some(system) = &mut self.system {
            system.on_restored(self.args.safe_mode_level)?;
        }
        if self.args.safe_mode_level >= 2 {
            if let Some(state) = &mut self.runtime_state {
                info!("Clearing the storage data to save memory");
                state.chain_storage.inner_mut().load_proof(vec![])
            }
        }
        Ok(())
    }

    fn check_requirements(&self) {
        let ver = P::app_version();
        let chain_storage = &self
            .runtime_state
            .as_ref()
            .expect("BUG: no runtime state")
            .chain_storage;
        let min_version = chain_storage.minimum_pruntime_version();

        let measurement = self.platform.measurement().unwrap_or_else(|| vec![0; 32]);
        let in_whitelist = chain_storage.is_pruntime_in_whitelist(&measurement);

        if (ver.major, ver.minor, ver.patch) < min_version && !in_whitelist {
            error!("This pRuntime is outdated. Please update to the latest version.");
            std::process::abort();
        }

        let consensus_version = chain_storage.pruntime_consensus_version();
        if consensus_version > system::MAX_SUPPORTED_CONSENSUS_VERSION {
            error!(
                "{} {}",
                "This pRuntime is outdated and doesn't meet the consensus version requirement.",
                "Please update to the latest version."
            );
            std::process::abort();
        }
    }

    fn update_runtime_info(
        &mut self,
        f: impl FnOnce(&mut phala_types::WorkerRegistrationInfoV2<chain::AccountId>),
    ) {
        let Some(cached_resp) = self.runtime_info.as_mut() else {
            return;
        };
        let mut runtime_info = cached_resp
            .decode_runtime_info()
            .expect("BUG: Decode runtime_info failed");
        runtime_info.version = Self::compat_app_version();
        runtime_info.max_consensus_version = system::MAX_SUPPORTED_CONSENSUS_VERSION;
        f(&mut runtime_info);
        cached_resp.encoded_runtime_info = runtime_info.encode();
        cached_resp.attestation = None;
    }

    fn compat_app_version() -> u32 {
        let version = P::app_version();
        (version.major << 16) + (version.minor << 8) + version.patch
    }
}

impl<Platform: pal::Platform + Serialize + DeserializeOwned> Phactory<Platform> {
    pub fn take_checkpoint(&mut self) -> anyhow::Result<chain::BlockNumber> {
        if self.args.safe_mode_level > 0 {
            anyhow::bail!("Checkpoint is disabled in safe mode");
        }
        let (current_block, _) = self.current_block()?;
        let key = self
            .system
            .as_ref()
            .context("Take checkpoint failed, runtime is not ready")?
            .identity_key
            .dump_secret_key();
        let checkpoint_file = checkpoint_filename_for(current_block, &self.args.storage_path);
        info!("Taking checkpoint to {checkpoint_file}...");
        self.save_checkpoint_info(&checkpoint_file)?;
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
        Ok(current_block)
    }

    pub fn save_checkpoint_info(&self, filename: &str) -> anyhow::Result<()> {
        let info = self.get_info();
        let content =
            serde_json::to_string_pretty(&info).context("Failed to serialize checkpoint info")?;
        let info_filename = checkpoint_info_filename_for(filename);
        let mut file =
            File::create(info_filename).context("Failed to create checkpoint info file")?;
        file.write_all(content.as_bytes())
            .context("Failed to write checkpoint info file")?;
        Ok(())
    }

    pub fn take_checkpoint_to_writer<W: std::io::Write>(
        &mut self,
        key: &[u8],
        writer: W,
    ) -> anyhow::Result<()> {
        let key128 = derive_key_for_checkpoint(key);
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
        args: &InitArgs,
    ) -> anyhow::Result<Option<Self>> {
        let runtime_data = match Self::load_runtime_data(platform, &args.sealing_path) {
            Err(Error::PersistentRuntimeNotFound) => return Ok(None),
            other => other.context("Failed to load persistent data")?,
        };
        let files = glob_checkpoint_files_sorted(&args.storage_path)
            .context("Glob checkpoint files failed")?;
        if files.is_empty() {
            return Ok(None);
        }
        let (_block, ckpt_filename) = &files[0];

        let file = match File::open(ckpt_filename) {
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
                if args.remove_corrupted_checkpoint {
                    error!("Removing {:?}", ckpt_filename);
                    std::fs::remove_file(ckpt_filename)
                        .context("Failed to remove corrupted checkpoint file")?;
                }
                anyhow::bail!(
                    "Failed to open checkpoint file {:?}: {:?}",
                    ckpt_filename,
                    err
                );
            }
        };

        info!("Loading checkpoint from file {:?}", ckpt_filename);
        match Self::restore_from_checkpoint_reader(&runtime_data.sk, file, args) {
            Ok(state) => {
                info!("Succeeded to load checkpoint file {:?}", ckpt_filename);
                Ok(Some(state))
            }
            Err(_err /*Don't leak it into the log*/) => {
                error!("Failed to load checkpoint file {:?}", ckpt_filename);
                if args.remove_corrupted_checkpoint {
                    error!("Removing {:?}", ckpt_filename);
                    std::fs::remove_file(ckpt_filename)
                        .context("Failed to remove corrupted checkpoint file")?;
                }
                anyhow::bail!("Failed to load checkpoint file {:?}", ckpt_filename);
            }
        }
    }

    pub fn restore_from_checkpoint_reader<R: std::io::Read>(
        key: &[u8],
        reader: R,
        args: &InitArgs,
    ) -> anyhow::Result<Self> {
        let key128 = derive_key_for_checkpoint(key);
        let dec_reader = aead::stream::new_aes128gcm_reader(key128, reader);
        system::sidevm_config(args.cores as _);
        let PhactoryLoader(mut factory) =
            serde_cbor::de::from_reader(dec_reader).context("Failed to decode state")?;
        factory.set_args(args.clone());
        factory
            .on_restored()
            .context("Failed to restore Phactory")?;
        Ok(factory)
    }

    fn statistics(
        &mut self,
        request: pb::StatisticsReqeust,
    ) -> anyhow::Result<pb::StatisticsResponse> {
        let uptime = self.started_at.elapsed().as_secs();
        let contracts_query_stats;
        let global_query_stats;
        let contracts_http_stats;
        let global_http_stats;
        if request.all {
            let query_stats = self.query_scheduler.stats();
            contracts_query_stats = query_stats.flows;
            global_query_stats = query_stats.global;
            let http_stats = http_counters::stats();
            contracts_http_stats = http_stats.by_contract;
            global_http_stats = http_stats.global;
        } else {
            let mut query_stats = Vec::new();
            let mut http_stats = BTreeMap::new();
            for contract in request.contracts {
                let contract =
                    AccountId::from_str(&contract).or(Err(anyhow!("Invalid contract address")))?;
                let stat = self.query_scheduler.stats_for(&contract);
                query_stats.push((contract.clone(), stat));
                let stat = http_counters::stats_for(&contract);
                http_stats.insert(contract, stat);
            }
            contracts_query_stats = query_stats;
            global_query_stats = self.query_scheduler.stats_global();
            contracts_http_stats = http_stats;
            global_http_stats = http_counters::stats_global();
        }

        Ok(pb::StatisticsResponse {
            uptime,
            cores: self.args.cores,
            query: Some(pb::QueryStats {
                global: Some(pb::QueryCounters {
                    total: global_query_stats.total,
                    dropped: global_query_stats.dropped,
                    time: global_query_stats.time_ms(),
                }),
                by_contract: contracts_query_stats
                    .into_iter()
                    .map(|(contract, stat)| {
                        (
                            format!("0x{}", hex_fmt::HexFmt(contract)),
                            pb::QueryCounters {
                                total: stat.total,
                                dropped: stat.dropped,
                                time: stat.time_ms(),
                            },
                        )
                    })
                    .collect(),
            }),
            http_egress: Some(pb::HttpEgressStats {
                global: Some(pb::HttpCounters {
                    requests: global_http_stats.requests,
                    failures: global_http_stats.failures,
                    by_status_code: global_http_stats
                        .by_status_code
                        .into_iter()
                        .map(|(s, c)| (s as u32, c))
                        .collect(),
                }),
                by_contract: contracts_http_stats
                    .into_iter()
                    .map(|(contract, stat)| {
                        (
                            format!("0x{}", hex_fmt::HexFmt(contract)),
                            pb::HttpCounters {
                                requests: stat.requests,
                                failures: stat.failures,
                                by_status_code: stat
                                    .by_status_code
                                    .into_iter()
                                    .map(|(s, c)| (s as u32, c))
                                    .collect(),
                            },
                        )
                    })
                    .collect(),
            }),
        })
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
                        "Checkpoint version {version} is not supported"
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
                        .ok_or_else(|| de::Error::custom("Missing runtime_state"))?;

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
        Ok(Self(Phactory::load_state(deserializer)?))
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

fn hex(data: impl AsRef<[u8]>) -> String {
    format!("0x{}", hex_fmt::HexFmt(data))
}
