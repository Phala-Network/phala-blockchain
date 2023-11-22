#![warn(unused_imports)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate log;
extern crate phactory_pal as pal;
extern crate runtime as chain;

use ::pink::{
    runtimes::v1::PinkRuntimeVersion,
    types::{AccountId, ExecSideEffects},
};
use contracts::{
    pink::{http_counters, Cluster},
    ContractsKeeper,
};
use glob::PatternError;
use rand::*;
use serde::{
    de::{self, DeserializeOwned, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use sidevm::service::CommandSender;

use crate::{
    contract_result::{ContractResult, ExecReturnValue, InstantiateReturnValue, StorageDeposit},
    light_validation::LightValidation,
};
use phactory_api::contracts::QueryError;
use phala_types::contract::ContractQueryError;
use sidevm_env::messages::{QueryError as SidevmQueryError, QueryResponse};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    future::Future,
    str::FromStr,
    sync::{Arc, Mutex, Weak},
};
use std::{fs::File, io::ErrorKind, path::PathBuf};
use std::{io::Write, marker::PhantomData};
use std::{path::Path, str};

use anyhow::{anyhow, Context as _, Result};
use core::convert::TryInto;
use parity_scale_codec::{Decode, Encode};
use phala_types::{AttestationProvider, HandoverChallenge};
use ring::rand::SecureRandom;
use scale_info::TypeInfo;
use serde_json::{json, Value};
use sp_core::{blake2_256, crypto::Pair, sr25519, H256};

// use pink::InkModule;

use phactory_api::{
    blocks::{self, SyncCombinedHeadersReq, SyncParachainHeaderReq},
    contracts::QueryType,
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
use phala_mq::{BindTopic, ChannelState, MessageDispatcher, MessageOrigin, MessageSendQueue};
use phala_scheduler::RequestScheduler;
use phala_serde_more as more;
use std::time::Instant;
use types::Error;

pub use chain::BlockNumber;
pub use contracts::pink;
pub use prpc_service::RpcService;
pub use storage::ChainStorage;
pub use system::gk;
pub use types::{BaseBlockInfo, BlockInfo};
pub type PRuntimeLightValidation = LightValidation<chain::Runtime>;

pub mod benchmark;

mod bin_api_service;
mod contract_result;
pub mod contracts;
mod cryptography;
mod im_helpers;
mod light_validation;
mod prpc_service;
mod secret_channel;
mod storage;
mod system;
mod types;

// TODO: Completely remove the reference to Phala/Khala runtime. Instead we can create a minimal
// runtime definition locally.
type RuntimeHasher = <chain::Runtime as frame_system::Config>::Hashing;

#[derive(Serialize, Deserialize, Clone, ::scale_info::TypeInfo)]
struct RuntimeState {
    #[codec(skip)]
    send_mq: MessageSendQueue,

    #[serde(skip)]
    #[codec(skip)]
    recv_mq: MessageDispatcher,

    // chain storage synchonizing
    #[cfg_attr(not(test), codec(skip))]
    storage_synchronizer: Synchronizer<LightValidation<chain::Runtime>>,

    // TODO.kevin: use a better serialization approach
    #[codec(skip)]
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
                info!("Removing {}", filename.display());
                if let Err(e) = std::fs::remove_file(&filename) {
                    error!("Failed to remove {}: {}", filename.display(), e);
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

#[derive(Serialize, Deserialize, Clone, TypeInfo)]
#[serde(bound(deserialize = "Platform: Deserialize<'de>"))]
pub struct Phactory<Platform> {
    platform: Platform,
    #[serde(skip)]
    #[codec(skip)]
    pub args: Arc<InitArgs>,
    dev_mode: bool,
    attestation_provider: Option<AttestationProvider>,
    machine_id: Vec<u8>,
    runtime_info: Option<InitRuntimeResponse>,
    runtime_state: Option<RuntimeState>,
    endpoints: BTreeMap<EndpointType, String>,
    #[serde(skip)]
    #[codec(skip)]
    signed_endpoints: Option<GetEndpointResponse>,
    // The deserialzation of system requires the mq, which inside the runtime_state, to be ready.
    #[serde(skip)]
    system: Option<system::System<Platform>>,

    // tmp key for WorkerKey handover encryption
    #[codec(skip)]
    #[serde(skip)]
    pub(crate) handover_ecdh_key: Option<EcdhKey>,

    #[codec(skip)]
    #[serde(skip)]
    handover_last_challenge: Option<HandoverChallenge<chain::BlockNumber>>,

    #[codec(skip)]
    #[serde(skip)]
    #[serde(default = "Instant::now")]
    last_checkpoint: Instant,

    #[codec(skip)]
    #[serde(skip)]
    query_scheduler: RequestScheduler<AccountId>,

    #[serde(default)]
    netconfig: Option<NetworkConfig>,

    #[codec(skip)]
    #[serde(skip)]
    can_load_chain_state: bool,

    #[codec(skip)]
    #[serde(skip)]
    trusted_sk: bool,

    #[codec(skip)]
    #[serde(skip)]
    pub(crate) rcu_dispatching: bool,

    #[codec(skip)]
    #[serde(skip)]
    pub(crate) pending_effects: Vec<::pink::types::ExecSideEffects>,

    #[codec(skip)]
    #[serde(skip)]
    #[serde(default = "Instant::now")]
    started_at: Instant,

    #[codec(skip)]
    #[serde(skip)]
    pub(crate) cluster_state_to_apply: Option<ClusterState<'static>>,

    #[codec(skip)]
    #[serde(skip)]
    #[serde(default = "sidevm_helper::create_sidevm_service_default")]
    sidevm_spawner: sidevm::service::Spawner,
}

mod sidevm_helper {
    use sidevm::service::{Report, Spawner};
    use std::cell::Cell;

    thread_local! {
        // Used only when deserializing the Spawner.
        static N_WORKERS: Cell<usize> = Cell::new(2);
        static SIDEVM_OUT_TX: Cell<Option<sidevm::OutgoingRequestChannel>> = Cell::new(None);
    }

    pub fn sidevm_config(n_workers: usize, out_tx: sidevm::OutgoingRequestChannel) {
        N_WORKERS.with(|v| v.set(n_workers));
        SIDEVM_OUT_TX.with(|v| v.set(Some(out_tx)));
    }

    pub fn create_sidevm_service_default() -> Spawner {
        let out_tx = SIDEVM_OUT_TX.with(|v| v.take().expect("sidevm_config not called"));
        let n_workers = N_WORKERS.with(|n| n.get());
        create_sidevm_service(n_workers, out_tx)
    }

    pub fn create_sidevm_service(
        worker_threads: usize,
        out_tx: sidevm::OutgoingRequestChannel,
    ) -> Spawner {
        let (service, spawner) = sidevm::service::service(worker_threads, out_tx);
        spawner.spawn(service.run(|report| match report {
            Report::VmTerminated { id, reason } => {
                let id = hex_fmt::HexFmt(&id[..4]);
                tracing::info!(%id, %reason, "Sidevm instance terminated");
            }
        }));
        spawner
    }
}

#[test]
fn show_type_changes_that_affect_the_checkpoint() {
    fn travel_types<T: TypeInfo>() -> String {
        use scale_info::{IntoPortable, PortableRegistry};
        let mut registry = Default::default();
        let _ = T::type_info().into_portable(&mut registry);
        serde_json::to_string_pretty(&PortableRegistry::from(registry).types).unwrap()
    }
    insta::assert_display_snapshot!(travel_types::<Phactory<()>>());
}

#[derive(Serialize, Deserialize, Clone)]
struct ClusterState<'a> {
    block_number: BlockNumber,
    pending_messages: Vec<(MessageOrigin, ChannelState)>,
    contracts: Cow<'a, ContractsKeeper>,
    cluster: Cow<'a, Cluster>,
}

fn create_query_scheduler(cores: u32) -> RequestScheduler<AccountId> {
    const FAIR_QUEUE_BACKLOG: usize = 32;
    RequestScheduler::new(FAIR_QUEUE_BACKLOG, cores + 2)
}

impl<Platform: pal::Platform> Phactory<Platform> {
    pub fn new(platform: Platform, args: InitArgs, weak_self: Weak<Mutex<Self>>) -> Self {
        let machine_id = platform.machine_id();
        let mut me = Phactory {
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
            handover_last_challenge: None,
            last_checkpoint: Instant::now(),
            query_scheduler: Default::default(),
            netconfig: Default::default(),
            can_load_chain_state: false,
            trusted_sk: false,
            rcu_dispatching: false,
            pending_effects: Vec::new(),
            started_at: Instant::now(),
            cluster_state_to_apply: None,
            sidevm_spawner: sidevm_helper::create_sidevm_service(
                args.cores as _,
                create_sidevm_outgoing_channel(weak_self),
            ),
        };
        me.init(args);
        me
    }

    fn init(&mut self, args: InitArgs) {
        if args.init_bench {
            benchmark::resume();
        }

        self.can_load_chain_state = !system::gk_master_key_exists(&args.sealing_path);
        self.args = Arc::new(args);
        self.query_scheduler = create_query_scheduler(self.args.cores);
    }

    pub fn set_args(&mut self, args: InitArgs) {
        self.args = Arc::new(args);
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

    /// Loads the persistent runtime data from the sealing path
    fn persistent_runtime_data(&self) -> Result<PersistentRuntimeData, Error> {
        Self::load_runtime_data(&self.platform, &self.args.sealing_path)
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
            system.on_restored(self.args.safe_mode_level, &self.sidevm_spawner)?;
        }
        if self.args.safe_mode_level >= 2 {
            if let Some(state) = &mut self.runtime_state {
                info!("Clearing the storage data to save memory");
                state.chain_storage.inner_mut().load_proof(vec![])
            }
        }
        self.query_scheduler = create_query_scheduler(self.args.cores);
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

    fn handle_sidevm_ocall(
        &self,
        from: [u8; 32],
        request: sidevm::OutgoingRequest,
        weak_phactory: Weak<Mutex<Phactory<P>>>,
    ) -> impl Future<Output = ()> {
        let sidevm::OutgoingRequest::Query {
            contract_id,
            payload,
            reply_tx,
        } = request;
        let query_scheduler = self.query_scheduler.clone();
        let mut derived_from = from.to_vec();
        derived_from.extend_from_slice(b"/sidevm");
        let origin = AccountId::new(blake2_256(&derived_from));
        let req_id = {
            use std::sync::atomic::{AtomicU64, Ordering};
            // The REQ_IDs would be even numbers to distinguish from the REQ_IDs of queries
            // from RPC, which are odd numbers.
            static REQ_ID: AtomicU64 = AtomicU64::new(0);
            REQ_ID.fetch_add(2, Ordering::Relaxed)
        };
        // Dispatch
        let query_future = self
            .system
            .as_ref()
            .expect("system always exists here")
            .make_query(
                req_id,
                &AccountId::new(contract_id),
                Some(&origin),
                payload,
                query_scheduler,
                &self
                    .runtime_state
                    .as_ref()
                    .expect("runtime state always exists here")
                    .chain_storage,
            );
        let pink_runtime_version = self
            .cluster_runtime_version()
            .expect("BUG: no runtime version");
        async move {
            let result: Result<QueryResponse, SidevmQueryError> = async move {
                let (query_type, output, effects) = query_future
                    .map_err(opaque_to_sidevm_err)?
                    .await
                    .map_err(opaque_to_sidevm_err)?;
                let phactory_api::contracts::Response::Payload(output) =
                    output.map_err(to_sidevm_err)?;
                if let Some(effects) = effects {
                    match weak_phactory.upgrade() {
                        Some(phactory) => {
                            phactory.lock().unwrap().apply_side_effects(effects);
                        }
                        None => {
                            error!("Phactory dropped while processing sidevm query");
                        }
                    }
                }
                match query_type {
                    QueryType::InkMessage => {
                        convert_result::<ExecReturnValue>(pink_runtime_version, &output)
                    }
                    QueryType::InkInstantiate => {
                        convert_result::<InstantiateReturnValue>(pink_runtime_version, &output)
                    }
                    QueryType::SidevmQuery => Ok(QueryResponse::SimpleOutput(output)),
                }
            }
            .await;
            if reply_tx.send(result.encode()).is_err() {
                error!("Failed to send sidevm query reply");
            }
        }
    }
}

fn opaque_to_sidevm_err(err: ContractQueryError) -> SidevmQueryError {
    match err {
        ContractQueryError::InvalidSignature => SidevmQueryError::InvalidSignature,
        ContractQueryError::ContractNotFound => SidevmQueryError::ContractNotFound,
        ContractQueryError::DecodeError => SidevmQueryError::DecodeError,
        ContractQueryError::OtherError(err) => SidevmQueryError::OtherError(err),
    }
}

fn to_sidevm_err(err: QueryError) -> SidevmQueryError {
    match err {
        QueryError::BadOrigin => SidevmQueryError::BadOrigin,
        QueryError::RuntimeError(err) => SidevmQueryError::RuntimeError(err),
        QueryError::SidevmNotFound => SidevmQueryError::SidevmNotFound,
        QueryError::NoResponse => SidevmQueryError::NoResponse,
        QueryError::ServiceUnavailable => SidevmQueryError::ServiceUnavailable,
        QueryError::Timeout => SidevmQueryError::Timeout,
    }
}

fn convert_result<R: Decode + Into<ExecReturnValue>>(
    pink_ver: PinkRuntimeVersion,
    mut exec_output: &[u8],
) -> Result<QueryResponse, SidevmQueryError> {
    use PinkRuntimeVersion::*;
    let result = match pink_ver {
        V1_0 | V1_1 | V1_2 => ContractResult::<R>::decode(&mut exec_output)
            .map_err(|_| SidevmQueryError::InvalidContractExecResult)?,
    };
    match result.result {
        Ok(value) => {
            let value = value.into();

            let storage_deposit_value;
            let storage_deposit_is_charge;
            match result.storage_deposit {
                StorageDeposit::Charge(value) => {
                    storage_deposit_value = value;
                    storage_deposit_is_charge = true;
                }
                StorageDeposit::Refund(value) => {
                    storage_deposit_value = value;
                    storage_deposit_is_charge = false;
                }
            }
            Ok(QueryResponse::OutputWithGasEstimation {
                output: value.data,
                gas_consumed: result.gas_consumed.ref_time,
                gas_required: result.gas_required.ref_time,
                storage_deposit_value,
                storage_deposit_is_charge,
            })
        }
        Err(err) => Err(SidevmQueryError::DispatchError(format!("{err:?}"))),
    }
}

impl<P: pal::Platform> Phactory<P> {
    pub fn apply_side_effects(&mut self, effects: ExecSideEffects) {
        if self.rcu_dispatching {
            const MAX_PENDING: usize = 64;
            if self.pending_effects.len() >= MAX_PENDING {
                error!("Too many pending effects, dropping this");
                return;
            }
            self.pending_effects.push(effects);
            return;
        }
        let Some(state) = self.runtime_state.as_ref() else {
            error!("Failed to apply side effects: chain storage missing");
            return;
        };
        let Some(system) = self.system.as_mut() else {
            error!("Failed to apply side effects: system missing");
            return;
        };
        system.apply_side_effects(effects, &state.chain_storage, &self.sidevm_spawner);
    }

    fn cluster_runtime_version(&self) -> Option<PinkRuntimeVersion> {
        let ver = self
            .system
            .as_ref()?
            .contract_cluster
            .as_ref()?
            .config
            .runtime_version;
        PinkRuntimeVersion::from_ver(ver)
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
        info!("Checkpoint saved to {checkpoint_file}");
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
        serialize_phactory_to_writer(self, &mut enc_writer)
            .context("Failed to write checkpoint")?;
        enc_writer
            .flush()
            .context("Failed to flush encrypted writer")?;
        Ok(())
    }

    pub fn restore_from_checkpoint(
        platform: &Platform,
        args: &InitArgs,
        weak_self: Weak<Mutex<Self>>,
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
                anyhow::bail!("Checkpoint file {ckpt_filename:?} is not found");
            }
            Err(err) => {
                error!("Failed to open checkpoint file {ckpt_filename:?}: {err:?}",);
                if args.remove_corrupted_checkpoint {
                    error!("Removing {ckpt_filename:?}");
                    std::fs::remove_file(ckpt_filename)
                        .context("Failed to remove corrupted checkpoint file")?;
                }
                anyhow::bail!("Failed to open checkpoint file {ckpt_filename:?}: {err:?}");
            }
        };

        info!("Loading checkpoint from file {ckpt_filename:?}");
        match Self::restore_from_checkpoint_reader(
            &runtime_data.sk,
            file,
            args,
            create_sidevm_outgoing_channel(weak_self),
        ) {
            Ok(state) => {
                info!("Succeeded to load checkpoint file {ckpt_filename:?}");
                Ok(Some(state))
            }
            Err(_err /*Don't leak it into the log*/) => {
                error!("Failed to load checkpoint file {ckpt_filename:?}");
                if args.remove_corrupted_checkpoint {
                    error!("Removing {:?}", ckpt_filename);
                    std::fs::remove_file(ckpt_filename)
                        .context("Failed to remove corrupted checkpoint file")?;
                }
                anyhow::bail!("Failed to load checkpoint file {ckpt_filename:?}");
            }
        }
    }

    pub fn restore_from_checkpoint_reader<R: std::io::Read>(
        key: &[u8],
        reader: R,
        args: &InitArgs,
        out_tx: sidevm::OutgoingRequestChannel,
    ) -> anyhow::Result<Self> {
        let key128 = derive_key_for_checkpoint(key);
        let dec_reader = aead::stream::new_aes128gcm_reader(key128, reader);
        sidevm_helper::sidevm_config(args.cores as _, out_tx);

        let mut factory = deserialize_phactory_from_reader(dec_reader, args.safe_mode_level)
            .context("Failed to deserialize Phactory")?;
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

    fn load_state<'de, D: Deserializer<'de>>(
        deserializer: D,
        safe_mode_level: u8,
    ) -> Result<Self, D::Error> {
        struct PhactoryVisitor<Platform> {
            safe_mode_level: u8,
            _marker: PhantomData<Platform>,
        }

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

                if self.safe_mode_level < 2 {
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
                } else {
                    let _: Option<serde::de::IgnoredAny> = seq.next_element()?;
                }
                benchmark::restore_state(state);
                Ok(factory)
            }
        }

        deserializer.deserialize_seq(PhactoryVisitor {
            safe_mode_level,
            _marker: PhantomData,
        })
    }

    pub fn sidevm_command_sender(&self, contract_id: &[u8]) -> Option<CommandSender> {
        let contract_id = AccountId::new(contract_id.try_into().ok()?);
        self.system
            .as_ref()?
            .contracts
            .get(&contract_id)?
            .get_system_message_handler()
    }
}

fn create_sidevm_outgoing_channel<Platform: pal::Platform>(
    weak_phactory: Weak<Mutex<Phactory<Platform>>>,
) -> sidevm::OutgoingRequestChannel {
    let (tx, mut rx) = tokio::sync::mpsc::channel(128);
    tokio::spawn(async move {
        while let Some((from, request)) = rx.recv().await {
            match weak_phactory.upgrade() {
                Some(phactory) => {
                    tokio::spawn(async move {
                        let weak_phactory = Arc::downgrade(&phactory);
                        let fut = phactory.lock().unwrap().handle_sidevm_ocall(
                            from,
                            request,
                            weak_phactory,
                        );
                        fut.await
                    });
                }
                None => {
                    error!("Sidevm outgoing channel: phactory dropped");
                    break;
                }
            }
        }
        info!("Sidevm outgoing channel: stopped");
    });
    tx
}

fn deserialize_phactory_from_reader<Platform, R>(
    reader: R,
    safe_mode_level: u8,
) -> Result<Phactory<Platform>>
where
    Platform: Serialize + DeserializeOwned,
    R: std::io::Read,
{
    let mut deserializer = serde_cbor::Deserializer::from_reader(reader);
    Phactory::load_state(&mut deserializer, safe_mode_level).context("Failed to load factory")
}

fn serialize_phactory_to_writer<Platform, R>(phatory: &Phactory<Platform>, writer: R) -> Result<()>
where
    Platform: Serialize + DeserializeOwned,
    R: std::io::Write,
{
    let mut writer = serde_cbor::ser::IoWrite::new(writer);
    let mut serializer = serde_cbor::Serializer::new(&mut writer);
    phatory.dump_state(&mut serializer)?;
    Ok(())
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

fn derive_key_for_cluster_state(identity_key: &[u8]) -> [u8; 16] {
    sp_core::blake2_128(&(identity_key, b"/cluster_state").encode())
}

fn hex(data: impl AsRef<[u8]>) -> String {
    format!("0x{}", hex_fmt::HexFmt(data))
}

fn try_decode_hex(hex_str: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str))
}

pub fn public_data_dir(storage_path: impl AsRef<Path>) -> PathBuf {
    storage_path.as_ref().to_path_buf().join("public")
}
