pub mod gk;
mod master_key;
mod side_tasks;

use crate::{
    benchmark,
    contracts::{pink::cluster::Cluster, AnyContract, ContractsKeeper, ExecuteEnv},
    pink::{cluster::ClusterKeeper, ContractEventCallback, Pink},
    secret_channel::{ecdh_serde, SecretReceiver},
    types::{BlockInfo, OpaqueError, OpaqueQuery, OpaqueReply},
};
use anyhow::{anyhow, Context, Result};
use core::fmt;
use log::info;
use phala_scheduler::RequestScheduler;
use pink::runtime::ExecSideEffects;
use runtime::BlockNumber;

use crate::contracts;
use crate::pal;
use chain::pallet_fat::ContractRegistryEvent;
use chain::pallet_registry::RegistryEvent;
pub use master_key::RotatedMasterKey;
use parity_scale_codec::{Decode, Encode};
pub use phactory_api::prpc::{GatekeeperRole, GatekeeperStatus};
use phala_crypto::{
    ecdh::EcdhKey,
    key_share,
    sr25519::{Persistence, Signing, KDF},
};
use phala_mq::{
    traits::MessageChannel, BadOrigin, ContractId, MessageDispatcher, MessageOrigin,
    MessageSendQueue, SignedMessageChannel, TypedReceiver,
};
use phala_serde_more as more;
use phala_types::{
    contract::{
        self,
        messaging::{
            BatchDispatchClusterKeyEvent, ClusterOperation, ContractOperation, ResourceType,
            WorkerClusterReport, WorkerContractReport,
        },
        CodeIndex,
    },
    messaging::{
        AeadIV, BatchRotateMasterKeyEvent, Condition, DispatchMasterKeyEvent,
        DispatchMasterKeyHistoryEvent, GatekeeperChange, GatekeeperLaunch, HeartbeatChallenge,
        KeyDistribution, MiningReportEvent, NewGatekeeperEvent, PRuntimeManagementEvent,
        RemoveGatekeeperEvent, RotateMasterKeyEvent, SystemEvent, WorkerEvent,
    },
    EcdhPublicKey, HandoverChallenge, HandoverChallengePayload, WorkerPublicKey,
};
use serde::{Deserialize, Serialize};
use side_tasks::geo_probe;
use sidevm::service::{Command as SidevmCommand, CommandSender, Report, Spawner, SystemMessage};
use sp_core::{hashing::blake2_256, sr25519, Pair, U256};
use sp_io;

use std::convert::TryFrom;
use std::future::Future;
use std::cell::Cell;

pub type TransactionResult = Result<pink::runtime::ExecSideEffects, TransactionError>;

#[derive(Encode, Decode, Debug, Clone, thiserror::Error)]
#[error("TransactionError: {:?}", self)]
pub enum TransactionError {
    BadInput,
    BadOrigin,
    Other(String),
    // general
    InsufficientBalance,
    NoBalance,
    UnknownError,
    BadContractId,
    BadCommand,
    SymbolExist,
    AssetIdNotFound,
    NotAssetOwner,
    BadSecret,
    BadMachineId,
    FailedToSign,
    BadDecimal,
    DestroyNotAllowed,
    ChannelError,
    // for gatekeeper
    NotGatekeeper,
    MasterKeyLeakage,
    BadSenderSignature,
    // for pdiem
    BadAccountInfo,
    BadLedgerInfo,
    BadTrustedStateData,
    BadEpochChangedProofData,
    BadTrustedState,
    InvalidAccount,
    BadTransactionWithProof,
    FailedToVerify,
    FailedToGetTransaction,
    FailedToCalculateBalance,
    BadChainId,
    TransferringNotAllowed,
    // for contract
    CodeNotFound,
    DuplicatedClusterDeploy,
}

impl From<BadOrigin> for TransactionError {
    fn from(_: BadOrigin) -> TransactionError {
        TransactionError::BadOrigin
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchState {
    start_block: chain::BlockNumber,
    start_time: u64,
    start_iter: u64,
    duration: u32,
}

#[derive(Debug, Serialize, Deserialize)]
enum MiningState {
    Mining,
    Paused,
}

#[derive(Debug, Serialize, Deserialize)]
struct MiningInfo {
    session_id: u32,
    state: MiningState,
    start_time: u64,
    start_iter: u64,
}

// Minimum worker state machine can be reused to replay in GK.
#[derive(Debug, Serialize, Deserialize)]
struct WorkerState {
    #[serde(with = "more::pubkey_bytes")]
    pubkey: WorkerPublicKey,
    hashed_id: U256,
    registered: bool,
    bench_state: Option<BenchState>,
    mining_state: Option<MiningInfo>,
}

impl WorkerState {
    pub fn new(pubkey: WorkerPublicKey) -> Self {
        let raw_pubkey: &[u8] = pubkey.as_ref();
        let pkh = blake2_256(raw_pubkey);
        let hashed_id: U256 = pkh.into();
        Self {
            pubkey,
            hashed_id,
            registered: false,
            bench_state: None,
            mining_state: None,
        }
    }

    pub fn process_event(
        &mut self,
        block: &BlockInfo,
        event: &SystemEvent,
        callback: &mut impl WorkerStateMachineCallback,
        log_on: bool,
    ) {
        match event {
            SystemEvent::WorkerEvent(evt) => {
                if evt.pubkey != self.pubkey {
                    return;
                }

                use MiningState::*;
                use WorkerEvent::*;
                if log_on {
                    info!("System::handle_event: {:?}", evt.event);
                }
                match evt.event {
                    Registered(_) => {
                        self.registered = true;
                    }
                    BenchStart { duration } => {
                        self.bench_state = Some(BenchState {
                            start_block: block.block_number,
                            start_time: block.now_ms,
                            start_iter: callback.bench_iterations(),
                            duration,
                        });
                        callback.bench_resume();
                    }
                    BenchScore(score) => {
                        if log_on {
                            info!("My benchmark score is {}", score);
                        }
                    }
                    MiningStart { session_id, .. } => {
                        self.mining_state = Some(MiningInfo {
                            session_id,
                            state: Mining,
                            start_time: block.now_ms,
                            start_iter: callback.bench_iterations(),
                        });
                        callback.bench_resume();
                    }
                    MiningStop => {
                        self.mining_state = None;
                        if self.need_pause() {
                            callback.bench_pause();
                        }
                    }
                    MiningEnterUnresponsive => {
                        if let Some(info) = &mut self.mining_state {
                            if let Mining = info.state {
                                if log_on {
                                    info!("Enter paused");
                                }
                                info.state = Paused;
                                return;
                            }
                        }
                        if log_on {
                            error!(
                                "Unexpected event received: {:?}, mining_state= {:?}",
                                evt.event, self.mining_state
                            );
                        }
                    }
                    MiningExitUnresponsive => {
                        if let Some(info) = &mut self.mining_state {
                            if let Paused = info.state {
                                if log_on {
                                    info!("Exit paused");
                                }
                                info.state = Mining;
                                return;
                            }
                        }
                        if log_on {
                            error!(
                                "Unexpected event received: {:?}, mining_state= {:?}",
                                evt.event, self.mining_state
                            );
                        }
                    }
                }
            }
            SystemEvent::HeartbeatChallenge(seed_info) => {
                self.handle_heartbeat_challenge(block, seed_info, callback, log_on);
            }
        };
    }

    fn handle_heartbeat_challenge(
        &mut self,
        block: &BlockInfo,
        seed_info: &HeartbeatChallenge,
        callback: &mut impl WorkerStateMachineCallback,
        log_on: bool,
    ) {
        if log_on {
            debug!(
                "System::handle_heartbeat_challenge({}, {:?}), registered={:?}, mining_state={:?}",
                block.block_number, seed_info, self.registered, self.mining_state
            );
        }

        if !self.registered {
            return;
        }

        let mining_state = if let Some(state) = &mut self.mining_state {
            state
        } else {
            return;
        };

        if matches!(mining_state.state, MiningState::Paused) {
            return;
        }

        let x = self.hashed_id ^ seed_info.seed;
        let online_hit = x <= seed_info.online_target;

        // Push queue when necessary
        if online_hit {
            let iterations = callback.bench_iterations() - mining_state.start_iter;
            callback.heartbeat(
                mining_state.session_id,
                block.block_number,
                block.now_ms,
                iterations,
            );
        }
    }

    fn need_pause(&self) -> bool {
        self.bench_state.is_none() && self.mining_state.is_none()
    }

    fn on_block_processed(
        &mut self,
        block: &BlockInfo,
        callback: &mut impl WorkerStateMachineCallback,
    ) {
        // Handle registering benchmark report
        if let Some(BenchState {
            start_block,
            start_time,
            start_iter,
            duration,
        }) = self.bench_state
        {
            if block.block_number - start_block >= duration {
                self.bench_state = None;
                let iterations = callback.bench_iterations() - start_iter;
                callback.bench_report(start_time, iterations);
                if self.need_pause() {
                    callback.bench_pause();
                }
            }
        }
    }
}

trait WorkerStateMachineCallback {
    fn bench_iterations(&self) -> u64 {
        0
    }
    fn bench_resume(&mut self) {}
    fn bench_pause(&mut self) {}
    fn bench_report(&mut self, _start_time: u64, _iterations: u64) {}
    fn heartbeat(
        &mut self,
        _session_id: u32,
        _block_num: chain::BlockNumber,
        _block_time: u64,
        _iterations: u64,
    ) {
    }
}

struct WorkerSMDelegate<'a> {
    egress: &'a SignedMessageChannel,
    n_clusters: u32,
    n_contracts: u32,
}

impl WorkerStateMachineCallback for WorkerSMDelegate<'_> {
    fn bench_iterations(&self) -> u64 {
        benchmark::iteration_counter()
    }
    fn bench_resume(&mut self) {
        benchmark::resume();
    }
    fn bench_pause(&mut self) {
        benchmark::pause();
    }
    fn bench_report(&mut self, start_time: u64, iterations: u64) {
        let report = RegistryEvent::BenchReport {
            start_time,
            iterations,
        };
        info!("Reporting benchmark: {:?}", report);
        self.egress.push_message(&report);
    }
    fn heartbeat(
        &mut self,
        session_id: u32,
        challenge_block: chain::BlockNumber,
        challenge_time: u64,
        iterations: u64,
    ) {
        let event = MiningReportEvent::HeartbeatV2 {
            session_id,
            challenge_block,
            challenge_time,
            iterations,
            n_clusters: self.n_clusters,
            n_contracts: self.n_contracts,
        };
        info!("System: sending {:?}", event);
        self.egress.push_message(&event);
    }
}

#[derive(
    Serialize, Deserialize, Clone, derive_more::Deref, derive_more::DerefMut, derive_more::From,
)]
#[serde(transparent)]
pub(crate) struct WorkerIdentityKey(#[serde(with = "more::key_bytes")] sr25519::Pair);

// By mocking the public key of the identity key pair, we can pretend to be the first Gatekeeper on Khala
// for "shadow-gk" simulation.
#[cfg(feature = "shadow-gk")]
impl WorkerIdentityKey {
    pub(crate) fn public(&self) -> sr25519::Public {
        // The pubkey of the first GK on khala
        sr25519::Public(hex_literal::hex!(
            "60067697c486c809737e50d30a67480c5f0cede44be181b96f7d59bc2116a850"
        ))
    }
}

#[derive(
    Serialize, Deserialize, Clone, derive_more::Deref, derive_more::DerefMut, derive_more::From,
)]
#[serde(transparent)]
pub(crate) struct ContractKey(#[serde(with = "more::key_bytes")] sr25519::Pair);

fn get_contract_key(cluster_key: &sr25519::Pair, contract_id: &ContractId) -> sr25519::Pair {
    // Introduce deployer in key generation to prevent Replay Attacks
    cluster_key
        .derive_sr25519_pair(&[b"contract_key", contract_id.as_ref()])
        .expect("should not fail with valid info")
}

#[derive(Serialize, Deserialize)]
pub struct System<Platform> {
    platform: Platform,
    // Configuration
    pub(crate) sealing_path: String,
    pub(crate) storage_path: String,
    enable_geoprobing: bool,
    pub(crate) geoip_city_db: String,
    // Messageing
    egress: SignedMessageChannel,
    system_events: TypedReceiver<SystemEvent>,
    pruntime_management_events: TypedReceiver<PRuntimeManagementEvent>,
    gatekeeper_launch_events: TypedReceiver<GatekeeperLaunch>,
    gatekeeper_change_events: TypedReceiver<GatekeeperChange>,
    key_distribution_events: TypedReceiver<KeyDistribution<chain::BlockNumber>>,
    cluster_key_distribution_events:
        TypedReceiver<ClusterOperation<chain::AccountId, chain::BlockNumber>>,
    contract_operation_events: TypedReceiver<ContractOperation<chain::Hash, chain::AccountId>>,
    // Worker
    pub(crate) identity_key: WorkerIdentityKey,
    #[serde(with = "ecdh_serde")]
    pub(crate) ecdh_key: EcdhKey,
    pub(crate) trusted_identity_key: bool,
    #[serde(skip)]
    last_challenge: Option<HandoverChallenge<chain::BlockNumber>>,
    worker_state: WorkerState,
    // Gatekeeper
    pub(crate) gatekeeper: Option<gk::Gatekeeper<SignedMessageChannel>>,

    pub(crate) contracts: ContractsKeeper,
    pub(crate) contract_clusters: ClusterKeeper,
    #[serde(skip)]
    #[serde(default = "create_sidevm_service_default")]
    sidevm_spawner: Spawner,

    // Cached for query
    pub(crate) block_number: BlockNumber,
    pub(crate) now_ms: u64,
    retired_versions: Vec<Condition>,
}

thread_local! {
    static N_WORKERS: Cell<usize> = Cell::new(2);
}

pub fn sidevm_config(n_workers: usize) {
    N_WORKERS.with(|v| v.set(n_workers))
}

fn create_sidevm_service_default() -> Spawner {
    create_sidevm_service(N_WORKERS.with(|n| n.get()))
}

fn create_sidevm_service(worker_threads: usize) -> Spawner {
    let (service, spawner) = sidevm::service::service(worker_threads);
    spawner.spawn(service.run(|report| match report {
        Report::VmTerminated { id, reason } => {
            let id = hex_fmt::HexFmt(&id[..4]);
            info!("Sidevm {id} terminated with reason: {reason:?}");
        }
    }));
    spawner
}

impl<Platform: pal::Platform> System<Platform> {
    pub fn new(
        platform: Platform,
        sealing_path: String,
        storage_path: String,
        enable_geoprobing: bool,
        geoip_city_db: String,
        identity_key: sr25519::Pair,
        ecdh_key: EcdhKey,
        trusted_identity_key: bool,
        send_mq: &MessageSendQueue,
        recv_mq: &mut MessageDispatcher,
        contracts: ContractsKeeper,
        worker_threads: usize,
    ) -> Self {
        // Trigger panic early if platform is not properly implemented.
        let _ = Platform::app_version();

        let identity_key = WorkerIdentityKey(identity_key);
        let pubkey = identity_key.public();
        let sender = MessageOrigin::Worker(pubkey);

        System {
            platform,
            sealing_path,
            storage_path,
            enable_geoprobing,
            geoip_city_db,
            egress: send_mq.channel(sender, identity_key.clone().0.into()),
            system_events: recv_mq.subscribe_bound(),
            pruntime_management_events: recv_mq.subscribe_bound(),
            gatekeeper_launch_events: recv_mq.subscribe_bound(),
            gatekeeper_change_events: recv_mq.subscribe_bound(),
            key_distribution_events: recv_mq.subscribe_bound(),
            cluster_key_distribution_events: recv_mq.subscribe_bound(),
            contract_operation_events: recv_mq.subscribe_bound(),
            identity_key,
            ecdh_key,
            trusted_identity_key,
            last_challenge: None,
            worker_state: WorkerState::new(pubkey),
            gatekeeper: None,
            contracts,
            contract_clusters: Default::default(),
            block_number: 0,
            now_ms: 0,
            sidevm_spawner: create_sidevm_service(worker_threads),
            retired_versions: vec![],
        }
    }

    pub fn get_system_message_handler(&mut self, cluster_id: &ContractId) -> Option<CommandSender> {
        let handler_contract_id = self
            .contract_clusters
            .get_cluster_mut(cluster_id)
            .expect("BUG: contract cluster should always exists")
            .config
            .log_handler
            .as_ref()?;
        self.contracts
            .get(handler_contract_id)?
            .get_system_message_handler()
    }

    pub fn get_system_message_handler_for_contract_id(
        &mut self,
        contract_id: &ContractId,
    ) -> Option<CommandSender> {
        let cluster_id = self.contracts.get(contract_id)?.cluster_id();
        self.get_system_message_handler(&cluster_id)
    }

    // The IdentityKey is considered valid in two situations:
    //
    // 1. It's generated by pRuntime thus is safe;
    // 2. It's handovered, but we find out that it was successfully registered as a worker on-chain;
    pub fn validated_identity_key(&self) -> bool {
        self.trusted_identity_key || self.worker_state.registered
    }

    pub fn get_worker_key_challenge(
        &mut self,
        dev_mode: bool,
    ) -> HandoverChallenge<chain::BlockNumber> {
        let payload = HandoverChallengePayload {
            block_number: self.block_number,
            now: self.now_ms,
            dev_mode,
            nonce: crate::generate_random_info(),
        };
        let signature = self.identity_key.sign_data(&payload.encode());
        let challenge = HandoverChallenge { payload, signature };
        self.last_challenge = Some(challenge.clone());
        challenge
    }

    pub fn verify_worker_key_challenge(
        &mut self,
        challenge: &HandoverChallenge<chain::BlockNumber>,
    ) -> bool {
        let challenge_match = if self.last_challenge.as_ref() != Some(challenge) {
            info!("Unknown challenge: {:?}", challenge);
            false
        } else {
            true
        };
        // Clear used one-time challenge
        self.last_challenge = None;
        challenge_match
    }

    pub fn make_query(
        &mut self,
        contract_id: &ContractId,
        origin: Option<&chain::AccountId>,
        query: OpaqueQuery,
        query_scheduler: RequestScheduler<ContractId>,
    ) -> Result<impl Future<Output = Result<OpaqueReply, OpaqueError>>, OpaqueError> {
        use pink::storage::Snapshot as _;

        let contract = self
            .contracts
            .get_mut(contract_id)
            .ok_or(OpaqueError::ContractNotFound)?;
        let cluster_id = contract.cluster_id();
        let storage = self
            .contract_clusters
            .get_cluster_mut(&cluster_id)
            .expect("BUG: contract cluster should always exists")
            .storage
            .snapshot();
        let sidevm_handle = contract.sidevm_handle();
        let contract = contract.snapshot_for_query();
        let mut context = contracts::QueryContext {
            block_number: self.block_number,
            now_ms: self.now_ms,
            storage,
            sidevm_handle,
            log_handler: self.get_system_message_handler(&cluster_id),
            query_scheduler,
        };
        let origin = origin.cloned();
        Ok(async move {
            contract
                .handle_query(origin.as_ref(), query, &mut context)
                .await
        })
    }

    pub fn process_next_message(&mut self, block: &mut BlockInfo) -> anyhow::Result<bool> {
        let ok = phala_mq::select_ignore_errors! {
            (event, origin) = self.system_events => {
                if !origin.is_pallet() {
                    anyhow::bail!("Invalid SystemEvent sender: {}", origin);
                }
                self.process_system_event(block, &event);
            },
            (event, origin) = self.pruntime_management_events => {
                if !origin.is_pallet() {
                    anyhow::bail!("Invalid pRuntime management event sender: {}", origin);
                }
                self.process_pruntime_management_event(event);
            },
            (event, origin) = self.gatekeeper_launch_events => {
                self.process_gatekeeper_launch_event(block, origin, event);
            },
            (event, origin) = self.gatekeeper_change_events => {
                self.process_gatekeeper_change_event(block, origin, event);
            },
            (event, origin) = self.key_distribution_events => {
                self.process_key_distribution_event(block, origin, event);
            },
            (event, origin) = self.cluster_key_distribution_events => {
                self.process_cluster_operation_event(block, origin, event)?;
            },
            (event, origin) = self.contract_operation_events => {
                self.process_contract_operation_event(block, origin, event)?
            },
        };
        Ok(ok.is_none())
    }

    pub fn will_process_block(&mut self, block: &mut BlockInfo) {
        self.block_number = block.block_number;
        self.now_ms = block.now_ms;

        if self.enable_geoprobing {
            geo_probe::process_block(
                block.block_number,
                &self.egress,
                block.side_task_man,
                &self.identity_key,
                self.geoip_city_db.clone(),
            );
        }
        if let Some(gatekeeper) = &mut self.gatekeeper {
            gatekeeper.will_process_block(block);
        }
    }

    pub fn process_messages(&mut self, block: &mut BlockInfo) {
        loop {
            match self.process_next_message(block) {
                Err(err) => {
                    error!("Error processing message: {:?}", err);
                }
                Ok(no_more) => {
                    if no_more {
                        break;
                    }
                }
            }
        }
        if let Some(gatekeeper) = &mut self.gatekeeper {
            gatekeeper.process_messages(block);
        }
    }

    pub fn did_process_block(&mut self, block: &mut BlockInfo) {
        if let Some(gatekeeper) = &mut self.gatekeeper {
            gatekeeper.did_process_block(block);
        }

        self.worker_state
            .on_block_processed(block, &mut WorkerSMDelegate{
                egress: &self.egress,
                n_clusters: self.contract_clusters.len() as _,
                n_contracts: self.contracts.len() as _,
            });

        // Iterate over all contracts to handle their incoming commands.
        //
        // Since the wasm contracts can instantiate new contracts, it means that it will mutate the `self.contracts`.
        // So we can not directly iterate over the self.contracts.values_mut() which would keep borrowing on `self.contracts`
        // in the scope of entire `for loop` body.
        let contract_ids: Vec<_> = self.contracts.keys().cloned().collect();
        'outer: for key in contract_ids {
            // Inner loop to handle commands. One command per iteration and apply the command side-effects to make it
            // availabe for next command.
            loop {
                let log_handler = self.get_system_message_handler_for_contract_id(&key);
                let contract = match self.contracts.get_mut(&key) {
                    None => continue 'outer,
                    Some(v) => v,
                };
                let cluster_id = contract.cluster_id();
                let mut env = ExecuteEnv {
                    block: block,
                    contract_clusters: &mut self.contract_clusters,
                    log_handler: log_handler.clone(),
                };
                let result = match contract.process_next_message(&mut env) {
                    Some(result) => result,
                    None => break,
                };
                handle_contract_command_result(
                    result,
                    cluster_id,
                    &mut self.contracts,
                    &mut self.contract_clusters,
                    block,
                    &self.egress,
                    &self.sidevm_spawner,
                    log_handler,
                );
            }
            let log_handler = self.get_system_message_handler_for_contract_id(&key);
            let contract = match self.contracts.get_mut(&key) {
                None => continue 'outer,
                Some(v) => v,
            };
            let mut env = ExecuteEnv {
                block: block,
                contract_clusters: &mut self.contract_clusters,
                log_handler: log_handler.clone(),
            };
            let result = contract.on_block_end(&mut env);
            let cluster_id = contract.cluster_id();
            handle_contract_command_result(
                result,
                cluster_id,
                &mut self.contracts,
                &mut self.contract_clusters,
                block,
                &self.egress,
                &self.sidevm_spawner,
                log_handler,
            );
        }
        self.contracts.try_restart_sidevms(&self.sidevm_spawner);

        let contract_running = !self.contract_clusters.is_empty();
        benchmark::set_flag(benchmark::Flags::CONTRACT_RUNNING, contract_running);
    }

    fn process_system_event(&mut self, block: &BlockInfo, event: &SystemEvent) {
        self.worker_state
            .process_event(block, event, &mut WorkerSMDelegate {
                egress: &self.egress,
                n_clusters: self.contract_clusters.len() as _,
                n_contracts: self.contracts.len() as _,
            }, true);
    }

    fn process_pruntime_management_event(&mut self, event: PRuntimeManagementEvent) {
        match event {
            PRuntimeManagementEvent::RetirePRuntime(condition) => {
                self.retired_versions.push(condition.clone());
                self.check_retirement();
            }
        }
    }

    fn check_retirement(&mut self) {
        let cur_ver = Platform::app_version();
        for condition in self.retired_versions.iter() {
            let should_retire = match *condition {
                Condition::VersionLessThan(major, minor, patch) => {
                    (cur_ver.major, cur_ver.minor, cur_ver.patch) < (major, minor, patch)
                }
                Condition::VersionIs(major, minor, patch) => {
                    (cur_ver.major, cur_ver.minor, cur_ver.patch) == (major, minor, patch)
                }
            };

            if should_retire {
                error!("This pRuntime is outdated. Please update to the latest version.");
                std::process::abort();
            }
        }
    }

    /// Update local sealed master keys if the received history is longer than existing one.
    ///
    /// Panic if `self.gatekeeper` is None since it implies a need for resync from the start as gk
    fn set_master_key_history(&mut self, master_key_history: Vec<RotatedMasterKey>) {
        if self.gatekeeper.is_none() {
            master_key::seal(
                self.sealing_path.clone(),
                &master_key_history,
                &self.identity_key,
                &self.platform,
            );
            crate::maybe_remove_checkpoints(&self.sealing_path);
            panic!("Received master key, please restart pRuntime and pherry to sync as Gatekeeper");
        }

        if self
            .gatekeeper
            .as_mut()
            .expect("checked; qed.")
            .set_master_key_history(&master_key_history)
        {
            master_key::seal(
                self.sealing_path.clone(),
                &master_key_history,
                &self.identity_key,
                &self.platform,
            );
        }
    }

    fn init_gatekeeper(
        &mut self,
        block: &mut BlockInfo,
        master_key_history: Vec<RotatedMasterKey>,
    ) {
        assert!(
            self.gatekeeper.is_none(),
            "Duplicated gatekeeper initialization"
        );
        assert!(
            !master_key_history.is_empty(),
            "Init gatekeeper with no master key"
        );

        let master_key = sr25519::Pair::restore_from_secret_key(
            &master_key_history
                .first()
                .expect("empty master key history")
                .secret,
        );
        let gatekeeper = gk::Gatekeeper::new(
            master_key_history,
            block.recv_mq,
            block
                .send_mq
                .channel(MessageOrigin::Gatekeeper, master_key.into()),
        );
        self.gatekeeper = Some(gatekeeper);
    }

    fn process_gatekeeper_launch_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: GatekeeperLaunch,
    ) {
        if !origin.is_pallet() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        }

        info!("Incoming gatekeeper launch event: {:?}", event);
        match event {
            GatekeeperLaunch::FirstGatekeeper(event) => {
                self.process_first_gatekeeper_event(block, origin, event)
            }
            GatekeeperLaunch::MasterPubkeyOnChain(event) => {
                info!(
                    "Gatekeeper launches on chain in block {}",
                    block.block_number
                );
                if let Some(gatekeeper) = &mut self.gatekeeper {
                    gatekeeper.master_pubkey_uploaded(event.master_pubkey);
                }
            }
            GatekeeperLaunch::RotateMasterKey(event) => {
                info!(
                    "Master key rotation req round {} in block {}",
                    event.rotation_id, block.block_number
                );
                self.process_master_key_rotation_request(block, origin, event);
            }
            GatekeeperLaunch::MasterPubkeyRotated(event) => {
                info!(
                    "Rotated Master Pubkey {} on chain in block {}",
                    hex::encode(event.master_pubkey),
                    block.block_number
                );
            }
        }
    }

    /// Generate the master key if this is the first gatekeeper
    fn process_first_gatekeeper_event(
        &mut self,
        block: &mut BlockInfo,
        _origin: MessageOrigin,
        event: NewGatekeeperEvent,
    ) {
        // ATTENTION: the first gk cannot resume if its original master_key.seal is lost,
        // since there is no tx recorded on-chain that shares the key to itself
        //
        // Solution: always unregister the first gk after the second gk receives the key,
        // thank god we only need to do this once for each blockchain

        // double check the first gatekeeper is valid on chain
        if !chain_state::is_gatekeeper(&event.pubkey, block.storage) {
            error!(
                "Fatal error: Invalid first gatekeeper registration {:?}",
                event
            );
            panic!("System state poisoned");
        }

        let mut master_key_history = master_key::try_unseal(
            self.sealing_path.clone(),
            &self.identity_key.0,
            &self.platform,
        );
        let my_pubkey = self.identity_key.public();
        if my_pubkey == event.pubkey {
            // if the first gatekeeper reboots, it will possess the master key, and should not re-generate it
            if master_key_history.is_empty() {
                info!("Gatekeeper: generate master key as the first gatekeeper");
                // generate master key as the first gatekeeper, no need to restart
                let master_key = crate::new_sr25519_key();
                master_key_history.push(RotatedMasterKey {
                    rotation_id: 0,
                    block_height: 0,
                    secret: master_key.dump_secret_key(),
                });
                // manually seal the first master key for the first gk
                master_key::seal(
                    self.sealing_path.clone(),
                    &master_key_history,
                    &self.identity_key,
                    &self.platform,
                );
            }

            let master_key = sr25519::Pair::restore_from_secret_key(
                &master_key_history.first().expect("checked; qed.").secret,
            );
            // upload the master key on chain via worker egress
            info!(
                "Gatekeeper: upload master key {} on chain",
                hex::encode(master_key.public())
            );
            let master_pubkey = RegistryEvent::MasterPubkey {
                master_pubkey: master_key.public(),
            };
            self.egress.push_message(&master_pubkey);
        }

        // other gatekeepers will has keys after key sharing and reboot
        // init the gatekeeper if there is any master key to start slient syncing
        if !master_key_history.is_empty() {
            info!("Init gatekeeper in block {}", block.block_number);
            self.init_gatekeeper(block, master_key_history);
        }

        if my_pubkey == event.pubkey {
            self.gatekeeper
                .as_mut()
                .expect("gatekeeper must be initializaed; qed.")
                .register_on_chain();
        }
    }

    /// Rotate the master key
    ///
    /// All the gatekeepers will generate the key, and only one will get published due to the nature of message queue.
    ///
    /// The generated master key will be shared to all the gatekeepers (include this one), and only then will they really
    /// update the master key on-chain.
    fn process_master_key_rotation_request(
        &mut self,
        block: &mut BlockInfo,
        _origin: MessageOrigin,
        event: RotateMasterKeyEvent,
    ) {
        if let Some(gatekeeper) = &mut self.gatekeeper {
            info!("Gatekeeper：Rotate master key");
            gatekeeper.process_master_key_rotation_request(
                block,
                event,
                self.identity_key.0.clone(),
            );
        }
    }

    fn process_gatekeeper_change_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: GatekeeperChange,
    ) {
        info!("Incoming gatekeeper change event: {:?}", event);
        match event {
            GatekeeperChange::Registered(event) => {
                self.process_new_gatekeeper_event(block, origin, event)
            }
            GatekeeperChange::Unregistered(event) => {
                self.process_remove_gatekeeper_event(block, origin, event)
            }
        }
    }

    /// Share the master key to the newly-registered gatekeeper
    /// Tick the state if the registered gatekeeper is this worker
    fn process_new_gatekeeper_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: NewGatekeeperEvent,
    ) {
        if !origin.is_pallet() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        }

        // double check the registered gatekeeper is valid on chain
        if !chain_state::is_gatekeeper(&event.pubkey, block.storage) {
            error!(
                "Fatal error: Invalid first gatekeeper registration {:?}",
                event
            );
            panic!("System state poisoned");
        }

        if let Some(gatekeeper) = &mut self.gatekeeper {
            gatekeeper.share_master_key(&event.pubkey, &event.ecdh_pubkey, block.block_number);

            let my_pubkey = self.identity_key.public();
            if my_pubkey == event.pubkey {
                gatekeeper.register_on_chain();
            }
        }
    }

    /// Turn gatekeeper to silent syncing. The real cleanup will happen in next key rotation since it will have no chance
    /// to continuce syncing.
    ///
    /// There is no meaning to remove the master_key.seal file
    fn process_remove_gatekeeper_event(
        &mut self,
        _block: &mut BlockInfo,
        origin: MessageOrigin,
        event: RemoveGatekeeperEvent,
    ) {
        if !origin.is_pallet() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        }

        let my_pubkey = self.identity_key.public();
        if my_pubkey == event.pubkey && self.gatekeeper.is_some() {
            self.gatekeeper
                .as_mut()
                .expect("checked; qed.")
                .unregister_on_chain();
        }
    }

    fn process_key_distribution_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: KeyDistribution<chain::BlockNumber>,
    ) {
        match event {
            KeyDistribution::MasterKeyDistribution(event) => {
                if let Err(err) = self.process_master_key_distribution(origin, event) {
                    error!("Failed to process master key distribution event: {:?}", err);
                };
            }
            KeyDistribution::MasterKeyRotation(event) => {
                if let Err(err) = self.process_batch_rotate_master_key(block, origin, event) {
                    error!(
                        "Failed to process batch master key rotation event: {:?}",
                        err
                    );
                };
            }
            KeyDistribution::MasterKeyHistory(event) => {
                if let Err(err) = self.process_master_key_history(origin, event) {
                    error!("Failed to process master key history event: {:?}", err);
                };
            }
        }
    }

    fn process_cluster_operation_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: ClusterOperation<chain::AccountId, chain::BlockNumber>,
    ) -> Result<()> {
        match event {
            ClusterOperation::DispatchKeys(event) => {
                let cluster = event.cluster;
                if let Err(err) = self.process_cluster_key_distribution(block, origin, event) {
                    error!(
                        "Failed to process cluster key distribution event: {:?}",
                        err
                    );
                    let message = WorkerClusterReport::ClusterDeploymentFailed { id: cluster };
                    self.egress.push_message(&message);
                }
            }
            ClusterOperation::SetLogReceiver {
                cluster: cluster_id,
                log_handler,
            } => {
                if !origin.is_pallet() {
                    error!("Invalid origin {:?} sent a {:?}", origin, event);
                    anyhow::bail!("Invalid origin");
                }
                let cluster = self.contract_clusters.get_cluster_mut(&cluster_id);
                if let Some(cluster) = cluster {
                    info!(
                        "Set log handler for cluster {}: {:?}",
                        hex_fmt::HexFmt(cluster_id),
                        log_handler
                    );
                    cluster.config.log_handler = Some(log_handler);
                }
            }
            ClusterOperation::DestroyCluster(cluster_id) => {
                if !origin.is_pallet() {
                    error!("Invalid origin {:?} sent a {:?}", origin, event);
                    anyhow::bail!("Invalid origin");
                }
                let cluster = match self.contract_clusters.remove_cluster(&cluster_id) {
                    // The cluster is not deployed on this worker, just ignore it.
                    None => return Ok(()),
                    Some(cluster) => cluster,
                };
                info!("Destroying cluster {}", hex_fmt::HexFmt(&cluster_id));
                for contract in cluster.iter_contracts() {
                    if let Some(contract) = self.contracts.remove(&contract) {
                        contract.destroy(&self.sidevm_spawner);
                    }
                }
            }
            ClusterOperation::UploadResource {
                origin,
                cluster_id,
                resource_type,
                resource_data,
            } => {
                let cluster = self
                    .contract_clusters
                    .get_cluster_mut(&cluster_id)
                    .context("Cluster not deployed")?;
                let _uploader = phala_types::messaging::AccountId(origin.clone().into());
                let hash = cluster
                    .upload_resource(origin, resource_type, resource_data)
                    .map_err(|err| anyhow!("Failed to upload code: {:?}", err))?;
                info!(
                    "Uploaded code to cluster {}, code_hash={:?}",
                    cluster_id, hash
                );
            }
        }
        Ok(())
    }

    fn process_contract_operation_event(
        &mut self,
        block: &mut BlockInfo,
        sender: MessageOrigin,
        event: ContractOperation<chain::Hash, chain::AccountId>,
    ) -> anyhow::Result<()> {
        info!("Incoming contract operation: {:?}", event);
        if !sender.is_pallet() {
            anyhow::bail!("Invalid origin {:?} for contract operation", sender);
        }
        match event {
            ContractOperation::InstantiateCode { contract_info } => {
                let cluster_id = contract_info.cluster_id;
                let cluster = self
                    .contract_clusters
                    .get_cluster_mut(&cluster_id)
                    .context("Cluster not deployed")?;
                // We generate a unique key for each contract instead of
                // sharing the same cluster key to prevent replay attack
                let contract_id = contract_info.contract_id(blake2_256);
                let contract_key = get_contract_key(cluster.key(), &contract_id);
                let contract_pubkey = contract_key.public();
                let ecdh_key = contract_key
                    .derive_ecdh_key()
                    .or(Err(anyhow::anyhow!("Invalid contract key")))?;

                let sender = MessageOrigin::Cluster(cluster_id);
                let cluster_mq: SignedMessageChannel =
                    block.send_mq.channel(sender, cluster.key().clone().into());

                match contract_info.code_index {
                    CodeIndex::NativeCode(code_id) => {
                        use contracts::*;
                        let deployer = phala_types::messaging::AccountId(
                            contract_info.clone().deployer.into(),
                        );

                        macro_rules! match_and_install_contract {
                            ($(($id: path => $contract: expr)),*) => {{
                                match code_id {
                                    $(
                                        $id => {
                                            let id = contract_info.contract_id(blake2_256);
                                            install_contract(
                                                &mut self.contracts,
                                                id,
                                                $contract,
                                                contract_key.clone(),
                                                ecdh_key,
                                                block,
                                                cluster_id,
                                            )?;
                                            id
                                        }
                                    )*
                                    _ => {
                                        anyhow::bail!(
                                            "Invalid contract code id: {:?}",
                                            code_id
                                        );
                                    }
                                }
                            }};
                        }

                        let contract_id = match_and_install_contract! {
                            (BALANCES => balances::Balances::new()),
                            (ASSETS => assets::Assets::new()),
                            (BTC_LOTTERY => btc_lottery::BtcLottery::new(Some(contract_key.to_raw_vec()))),
                            // (GEOLOCATION => geolocation::Geolocation::new()),
                            (GUESS_NUMBER => guess_number::GuessNumber::new())
                            // (BTC_PRICE_BOT => btc_price_bot::BtcPriceBot::new())
                        };

                        let message = ContractRegistryEvent::PubkeyAvailable {
                            contract: contract_id,
                            pubkey: contract_pubkey.clone(),
                        };
                        cluster_mq.push_message(&message);

                        cluster.add_contract(contract_id);

                        let message = WorkerContractReport::ContractInstantiated {
                            id: contract_id,
                            cluster_id,
                            deployer,
                            pubkey: contract_pubkey,
                        };
                        info!("Native contract instantiate status: {:?}", message);
                        self.egress.push_message(&message);
                    }
                    CodeIndex::WasmCode(code_hash) => {
                        let deployer = contract_info.deployer.clone();
                        let contract_id = contract_info.contract_id(blake2_256);

                        let message = ContractRegistryEvent::PubkeyAvailable {
                            contract: contract_id,
                            pubkey: contract_pubkey,
                        };
                        cluster_mq.push_message(&message);

                        let log_handler = self.get_system_message_handler(&cluster_id);

                        let effects = self
                            .contract_clusters
                            .instantiate_contract(
                                cluster_id,
                                deployer.clone(),
                                code_hash,
                                contract_info.instantiate_data,
                                contract_info.salt,
                                block.block_number,
                                block.now_ms,
                                ContractEventCallback::from_log_sender(
                                    &log_handler,
                                    block.block_number,
                                ),
                            )
                            .with_context(|| format!("Contract deployer: {:?}", deployer))?;

                        let cluster = self
                            .contract_clusters
                            .get_cluster_mut(&cluster_id)
                            .expect("Cluster must exist");
                        apply_pink_side_effects(
                            effects,
                            cluster_id,
                            &mut self.contracts,
                            cluster,
                            block,
                            &self.egress,
                            &self.sidevm_spawner,
                            log_handler,
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Decrypt the key encrypted by `encrypt_key_to()`
    ///
    /// This function could panic a lot, thus should only handle data from other pRuntimes.
    fn decrypt_key_from(
        &self,
        ecdh_pubkey: &EcdhPublicKey,
        encrypted_key: &Vec<u8>,
        iv: &AeadIV,
    ) -> sr25519::Pair {
        let my_ecdh_key = self
            .identity_key
            .derive_ecdh_key()
            .expect("Should never failed with valid identity key; qed.");
        let secret =
            key_share::decrypt_secret_from(&my_ecdh_key, &ecdh_pubkey.0, &encrypted_key, &iv)
                .expect("Failed to decrypt dispatched key");
        sr25519::Pair::restore_from_secret_key(&secret)
    }

    /// Process encrypted master key from mq
    fn process_master_key_distribution(
        &mut self,
        origin: MessageOrigin,
        event: DispatchMasterKeyEvent,
    ) -> Result<(), TransactionError> {
        if !origin.is_gatekeeper() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return Err(TransactionError::BadOrigin);
        }

        let my_pubkey = self.identity_key.public();
        if my_pubkey == event.dest {
            let master_pair =
                self.decrypt_key_from(&event.ecdh_pubkey, &event.encrypted_master_key, &event.iv);
            info!("Gatekeeper: successfully decrypt received master key");
            self.set_master_key_history(vec![RotatedMasterKey {
                rotation_id: 0,
                block_height: 0,
                secret: master_pair.dump_secret_key(),
            }]);
        }
        Ok(())
    }

    fn process_master_key_history(
        &mut self,
        origin: MessageOrigin,
        event: DispatchMasterKeyHistoryEvent<chain::BlockNumber>,
    ) -> Result<(), TransactionError> {
        if !origin.is_gatekeeper() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return Err(TransactionError::BadOrigin);
        }

        let my_pubkey = self.identity_key.public();
        if my_pubkey == event.dest {
            let master_key_history: Vec<RotatedMasterKey> = event
                .encrypted_master_key_history
                .iter()
                .map(|(rotation_id, block_height, key)| RotatedMasterKey {
                    rotation_id: *rotation_id,
                    block_height: *block_height,
                    secret: self
                        .decrypt_key_from(&key.ecdh_pubkey, &key.encrypted_key, &key.iv)
                        .dump_secret_key(),
                })
                .collect();
            self.set_master_key_history(master_key_history);
        }
        Ok(())
    }

    /// Decrypt the rotated master key
    ///
    /// The new master key takes effect immediately after the GatekeeperRegistryEvent::RotatedMasterPubkey is sent
    fn process_batch_rotate_master_key(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: BatchRotateMasterKeyEvent,
    ) -> Result<(), TransactionError> {
        // ATTENTION.shelven: There would be a mismatch between on-chain and off-chain master key until the on-chain pubkey
        // is updated, which may cause problem in the future.
        if !origin.is_gatekeeper() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return Err(TransactionError::BadOrigin.into());
        }

        // check the event sender identity and signature to ensure it's not forged with a leaked master key and really from
        // a gatekeeper
        let data = event.data_be_signed();
        let sig = sp_core::sr25519::Signature::try_from(event.sig.as_slice())
            .or(Err(TransactionError::BadSenderSignature))?;
        if !sp_io::crypto::sr25519_verify(&sig, &data, &event.sender) {
            return Err(TransactionError::BadSenderSignature.into());
        }
        // valid master key but from a non-gk
        if !chain_state::is_gatekeeper(&event.sender, block.storage) {
            error!("Fatal error: Forged batch master key rotation {:?}", event);
            return Err(TransactionError::MasterKeyLeakage);
        }

        let my_pubkey = self.identity_key.public();
        // for normal worker
        if self.gatekeeper.is_none() {
            if event.secret_keys.contains_key(&my_pubkey) {
                panic!(
                    "Batch rotate master key to a normal worker {:?}",
                    &my_pubkey
                );
            }
            return Ok(());
        }

        // for gatekeeper (both active or unregistered)
        if event.secret_keys.contains_key(&my_pubkey) {
            let encrypted_key = &event.secret_keys[&my_pubkey];
            let new_master_key = self.decrypt_key_from(
                &encrypted_key.ecdh_pubkey,
                &encrypted_key.encrypted_key,
                &encrypted_key.iv,
            );
            info!("Worker: successfully decrypt received rotated master key");
            let gatekeeper = self.gatekeeper.as_mut().expect("checked; qed.");
            if gatekeeper.append_master_key(RotatedMasterKey {
                rotation_id: event.rotation_id,
                block_height: self.block_number,
                secret: new_master_key.dump_secret_key(),
            }) {
                master_key::seal(
                    self.sealing_path.clone(),
                    &gatekeeper.master_key_history(),
                    &self.identity_key,
                    &self.platform,
                );
            }
        }

        if self
            .gatekeeper
            .as_mut()
            .expect("checked; qed.")
            .switch_master_key(event.rotation_id, self.block_number)
        {
            // This is a valid GK in syncing, the needed master key should already be dispatched before the restart this
            // pRuntime.
            info!("Worker: rotate master key with received master key history");
        } else {
            // This is an unregistered GK whose master key is not outdated yet, it 's still sliently syncing. It cannot
            // do silent syncing anymore since it does not know the rotated key.
            info!("Worker: master key rotation received, stop unregistered gatekeeper silent syncing and cleanup");
            self.gatekeeper = None;
        }
        Ok(())
    }

    fn process_cluster_key_distribution(
        &mut self,
        _block: &mut BlockInfo,
        origin: MessageOrigin,
        event: BatchDispatchClusterKeyEvent<chain::BlockNumber>,
    ) -> anyhow::Result<()> {
        if !origin.is_gatekeeper() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return Err(TransactionError::BadOrigin.into());
        }

        let my_pubkey = self.identity_key.public();
        if event.secret_keys.contains_key(&my_pubkey) {
            let encrypted_key = &event.secret_keys[&my_pubkey];
            let cluster_key = self.decrypt_key_from(
                &encrypted_key.ecdh_pubkey,
                &encrypted_key.encrypted_key,
                &encrypted_key.iv,
            );
            info!("Worker: successfully decrypt received cluster key");

            // TODO(shelven): forget cluster key after expiration time
            let cluster = self.contract_clusters.get_cluster_mut(&event.cluster);
            if cluster.is_some() {
                error!("Cluster {:?} is already deployed", &event.cluster);
                return Err(TransactionError::DuplicatedClusterDeploy.into());
            }
            // register cluster
            self.contract_clusters
                .get_cluster_or_default_mut(&event.cluster, &cluster_key);
            let message = WorkerClusterReport::ClusterDeployed {
                id: event.cluster,
                pubkey: cluster_key.public(),
            };
            self.egress.push_message(&message);
        }
        Ok(())
    }

    pub fn is_registered(&self) -> bool {
        self.worker_state.registered
    }

    pub fn gatekeeper_status(&self) -> GatekeeperStatus {
        let has_gatekeeper = self.gatekeeper.is_some();
        let active = match &self.gatekeeper {
            Some(gk) => gk.registered_on_chain(),
            None => false,
        };
        let role = match (has_gatekeeper, active) {
            (true, true) => GatekeeperRole::Active,
            (true, false) => GatekeeperRole::Dummy,
            _ => GatekeeperRole::None,
        };
        let master_public_key = self
            .gatekeeper
            .as_ref()
            .map(|gk| hex::encode(&gk.master_pubkey()))
            .unwrap_or_default();
        GatekeeperStatus {
            role: role.into(),
            master_public_key,
        }
    }
}

impl<P: pal::Platform> System<P> {
    pub fn on_restored(&mut self) -> Result<()> {
        self.contracts.try_restart_sidevms(&self.sidevm_spawner);
        self.check_retirement();
        Ok(())
    }
}

pub fn handle_contract_command_result(
    result: TransactionResult,
    cluster_id: phala_mq::ContractClusterId,
    contracts: &mut ContractsKeeper,
    clusters: &mut ClusterKeeper,
    block: &mut BlockInfo,
    egress: &SignedMessageChannel,
    spawner: &Spawner,
    log_handler: Option<CommandSender>,
) {
    let effects = match result {
        Err(err) => {
            error!("Run contract command failed: {:?}", err);
            return;
        }
        Ok(effects) => effects,
    };
    let cluster = match clusters.get_cluster_mut(&cluster_id) {
        None => {
            error!(
                "BUG: contract cluster not found, it should always exsists, cluster_id={:?}",
                cluster_id
            );
            return;
        }
        Some(cluster) => cluster,
    };
    apply_pink_side_effects(
        effects,
        cluster_id,
        contracts,
        cluster,
        block,
        egress,
        spawner,
        log_handler,
    );
}

pub fn apply_pink_side_effects(
    effects: ExecSideEffects,
    cluster_id: phala_mq::ContractClusterId,
    contracts: &mut ContractsKeeper,
    cluster: &mut Cluster,
    block: &mut BlockInfo,
    egress: &SignedMessageChannel,
    spawner: &Spawner,
    log_handler: Option<CommandSender>,
) {
    for (deployer, address) in effects.instantiated {
        let pink = Pink::from_address(address.clone(), cluster_id);
        let contract_id = ContractId::from(address.as_ref());
        let contract_key = get_contract_key(cluster.key(), &contract_id);
        let ecdh_key = contract_key
            .derive_ecdh_key()
            .expect("Derive ecdh_key should not fail");
        let id = pink.id();
        let result = install_contract(
            contracts,
            id,
            pink,
            contract_key.clone(),
            ecdh_key.clone(),
            block,
            cluster_id,
        );

        if let Err(err) = result {
            error!("BUG: Install contract failed: {:?}", err);
            error!(" address: {:?}", address);
            error!(" cluster_id: {:?}", cluster_id);
            error!(" deployer: {:?}", deployer);
            continue;
        };

        cluster.add_contract(id);

        let message = WorkerContractReport::ContractInstantiated {
            id,
            cluster_id,
            deployer: phala_types::messaging::AccountId(deployer.into()),
            pubkey: EcdhPublicKey(ecdh_key.public()),
        };

        info!("pink instantiate status: {:?}", message);
        egress.push_message(&message);
    }

    for (address, event) in effects.pink_events {
        let id = contracts::contract_address_to_id(&address);
        let contract = match contracts.get_mut(&id) {
            Some(contract) => contract,
            None => {
                panic!(
                    "BUG: Unknown contract sending pink event, address={:?}, cluster_id={:?}",
                    address, cluster_id
                );
            }
        };
        let vmid = sidevm::ShortId(address.as_ref());
        use pink::runtime::PinkEvent;
        match event {
            PinkEvent::Message(message) => {
                contract.push_message(message.payload, message.topic);
            }
            PinkEvent::OspMessage(message) => {
                contract.push_osp_message(
                    message.message.payload,
                    message.message.topic,
                    message.remote_pubkey.as_ref(),
                );
            }
            PinkEvent::OnBlockEndSelector(selector) => {
                contract.set_on_block_end_selector(selector);
            }
            PinkEvent::StartSidevm {
                code_hash,
                auto_restart,
            } => {
                let code_hash = code_hash.into();
                let wasm_code = match cluster.get_resource(ResourceType::SidevmCode, &code_hash) {
                    Some(code) => code,
                    None => {
                        error!(target: "sidevm", "[{vmid}] Start sidevm failed: code not found, code_hash={code_hash:?}");
                        continue;
                    }
                };
                if let Err(err) = contract.start_sidevm(&spawner, wasm_code, auto_restart) {
                    error!(target: "sidevm", "[{vmid}] Start sidevm failed: {:?}", err);
                }
            }
            PinkEvent::SidevmMessage(payload) => {
                if let Err(err) = contract.push_message_to_sidevm(payload) {
                    error!(target: "sidevm", "[{vmid}] Push message to sidevm failed: {:?}", err);
                }
            }
            PinkEvent::CacheOp(op) => {
                pink::local_cache::local_cache_op(&address, op);
            }
        }
    }

    if let Some(log_handler) = log_handler {
        for (contract, topics, payload) in effects.ink_events.into_iter() {
            if let Err(_) =
                log_handler.try_send(SidevmCommand::PushSystemMessage(SystemMessage::PinkEvent {
                    contract: contract.into(),
                    block_number: block.block_number,
                    payload,
                    topics: topics.into_iter().map(Into::into).collect(),
                }))
            {
                warn!("Cluster [{cluster_id}] emit ink event to log handler failed");
            }
        }
    }
}

#[must_use]
pub fn install_contract(
    contracts: &mut ContractsKeeper,
    contract_id: phala_mq::ContractId,
    contract: impl Into<AnyContract>,
    contract_key: sr25519::Pair,
    ecdh_key: EcdhKey,
    block: &mut BlockInfo,
    cluster_id: phala_mq::ContractClusterId,
) -> anyhow::Result<()> {
    if contracts.get(&contract_id).is_some() {
        return Err(anyhow::anyhow!("Contract already exists"));
    }
    let sender = MessageOrigin::Contract(contract_id);
    let mq = block.send_mq.channel(sender, contract_key.into());
    let cmd_mq = SecretReceiver::new_secret(
        block
            .recv_mq
            .subscribe(contract::command_topic(contract_id))
            .into(),
        ecdh_key.clone(),
    );
    let wrapped = contracts::FatContract::new(
        contract,
        mq,
        cmd_mq,
        ecdh_key.clone(),
        cluster_id,
        contract_id,
    );
    contracts.insert(wrapped);
    Ok(())
}

#[derive(Encode, Decode, Debug)]
pub enum Error {
    NotAuthorized,
    TxHashNotFound,
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::TxHashNotFound => write!(f, "transaction hash not found"),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

pub mod chain_state {
    use super::*;
    use crate::light_validation::utils::{storage_map_prefix_twox_64_concat, storage_prefix};
    use crate::storage::{Storage, StorageExt};
    use parity_scale_codec::Decode;

    pub fn is_gatekeeper(pubkey: &WorkerPublicKey, chain_storage: &Storage) -> bool {
        let key = storage_prefix("PhalaRegistry", "Gatekeeper");
        let gatekeepers = chain_storage
            .get(&key)
            .map(|v| {
                Vec::<WorkerPublicKey>::decode(&mut &v[..])
                    .expect("Decode value of Gatekeeper Failed. (This should not happen)")
            })
            .unwrap_or_default();

        gatekeepers.contains(pubkey)
    }

    /// Return `None` if given pruntime hash is not allowed on-chain
    pub fn get_pruntime_added_at(
        chain_storage: &Storage,
        runtime_hash: &Vec<u8>,
    ) -> Option<chain::BlockNumber> {
        let key =
            storage_map_prefix_twox_64_concat(b"PhalaRegistry", b"PRuntimeAddedAt", runtime_hash);
        chain_storage.get_decoded(&key).unwrap_or(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_runtime::AccountId32;

    const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

    #[test]
    fn test_on_block_end() {
        let cluster_key = sp_core::Pair::from_seed(&Default::default());
        let mut contracts = ContractsKeeper::default();
        let mut keeper = ClusterKeeper::default();
        let wasm_bin = pink::load_test_wasm("hooks_test");
        let cluster_id = phala_mq::ContractClusterId(Default::default());
        let cluster = keeper.get_cluster_or_default_mut(&cluster_id, &cluster_key);
        let code_hash = cluster
            .upload_resource(ALICE.clone(), ResourceType::InkCode, wasm_bin)
            .unwrap();
        let effects = keeper
            .instantiate_contract(
                cluster_id,
                ALICE,
                code_hash,
                vec![0xed, 0x4b, 0x9d, 0x1b],
                Default::default(),
                1,
                1,
                None,
            )
            .unwrap();
        insta::assert_debug_snapshot!(effects);

        let cluster = keeper.get_cluster_mut(&cluster_id).unwrap();
        let mut builder = BlockInfo::builder().block_number(1).now_ms(1);
        let signer = sr25519::Pair::from_seed(&Default::default());
        let egress = builder
            .send_mq
            .channel(MessageOrigin::Gatekeeper, signer.into());
        let mut block_info = builder.build();
        let spawner = create_sidevm_service(4);

        apply_pink_side_effects(
            effects,
            cluster_id,
            &mut contracts,
            cluster,
            &mut block_info,
            &egress,
            &spawner,
            None,
        );

        insta::assert_display_snapshot!(contracts.len());

        let mut env = ExecuteEnv {
            block: &mut block_info,
            contract_clusters: &mut keeper,
            log_handler: None,
        };

        for contract in contracts.values_mut() {
            let effects = contract.on_block_end(&mut env).unwrap();
            insta::assert_debug_snapshot!(effects);
        }

        let messages: Vec<_> = builder
            .send_mq
            .all_messages()
            .into_iter()
            .map(|msg| (msg.sequence, msg.message))
            .collect();
        // WorkerContractReport::ContractInstantiated
        insta::assert_debug_snapshot!(messages);
    }
}
