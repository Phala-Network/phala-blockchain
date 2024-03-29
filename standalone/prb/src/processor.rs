use anyhow::{anyhow, Result};
use chrono::{DateTime, Timelike, Utc};
use futures::future::try_join_all;
use log::{error, info, trace, warn};
use parity_scale_codec::Decode;
use phactory_api::prpc::{
    self, ChainState, GetEgressMessagesResponse, GetEndpointResponse, GetRuntimeInfoRequest,
    InitRuntimeRequest, InitRuntimeResponse, PhactoryInfo, SignEndpointsRequest,
};
use phala_pallets::pallet_computation::{SessionInfo, WorkerState};
use phala_pallets::registry::WorkerInfoV2;
use sp_core::crypto::{AccountId32, ByteArray};
use sp_core::sr25519::Public as Sr25519Public;
use subxt::ext::sp_runtime::traits::EnsureAdd;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::api::WorkerStatus;
use crate::bus::Bus;
use crate::datasource::DataSourceManager;
use crate::repository::{ChaintipInfo, RepositoryEvent, SyncRequest, SyncRequestManifest, WorkerSyncInfo};
use crate::messages::MessagesEvent;
use crate::pruntime::PRuntimeClient;
use crate::tx::TxManager;
use crate::use_parachain_api;
use crate::worker::{WorkerLifecycleCommand, WorkerLifecycleState};
use crate::worker_status::WorkerStatusUpdate;

const UPDATE_PHACTORY_INFO_INTERVAL: chrono::Duration = chrono::Duration::seconds(5);
const RESTART_WORKER_COOL_PERIOD: chrono::Duration = chrono::Duration::seconds(15);

pub enum SyncStage {
    NotStart,
    Init,
    LoadChainState,
    Sync,
    Completed,
}

pub enum ComputationStage {
    NotStart,
    Register,
    AddToPool,
    StartComputing,
    Completed,
}

impl fmt::Display for ComputationStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ComputationStage::")?;
        match &self {
            ComputationStage::NotStart => write!(f, "NotStart"),
            ComputationStage::Register => write!(f, "Register"),
            ComputationStage::AddToPool => write!(f, "AddToPool"),
            ComputationStage::StartComputing => write!(f, "StartComputing"),
            ComputationStage::Completed => write!(f, "Completed"),
        }
    }
}

pub enum ExecutionStatus {
    Ok((usize, String)),
    Error((usize, String)),
}

pub struct WorkerContext {
    pub uuid: String,

    pub pool_id: u64,
    pub operator: Option<AccountId32>,
    pub worker_sync_only: bool,
    pub pool_sync_only: bool,

    pub headernum: u32,
    pub para_headernum: u32,
    pub blocknum: u32,

    pub worker_status: WorkerStatus,
    pub worker_info: Option<WorkerInfoV2<AccountId32>>,
    pub session_id: Option<AccountId32>,

    pub last_message: String,
    pub last_updated_at: DateTime<Utc>,

    pub pruntime_lock: bool,
    pub client: Arc<PRuntimeClient>,
    pub pending_requests: VecDeque<PRuntimeRequest>,

    pub phactory_info_execution_status: ExecutionStatus,
    pub phactory_info_requested: bool,
    pub phactory_info_requested_at: DateTime<Utc>,

    pub sync_stage: SyncStage,
    pub sync_execution_status: ExecutionStatus,

    pub stopped: bool,

    pub computation_stage: ComputationStage,
    pub computation_exeuction_status: ExecutionStatus,
    pub update_worker_info_count: usize,
    pub update_session_id_count: usize,
    pub update_session_info_count: usize,
}

impl WorkerContext {
    pub fn create(
        worker: crate::inv_db::Worker,
        pool_sync_only: Option<bool>,
        operator: Option<AccountId32>,
    ) -> Self {
        Self {
            uuid: worker.id.clone(),

            pool_id: worker.pid.unwrap_or_default(),
            operator,
            worker_sync_only: worker.sync_only,
            pool_sync_only: pool_sync_only.unwrap_or(true),

            headernum: 0,
            para_headernum: 0,
            blocknum: 0,

            worker_status: WorkerStatus {
                worker: worker.clone(),
                state: if worker.enabled {
                    WorkerLifecycleState::Starting
                } else {
                    WorkerLifecycleState::Disabled
                },
                phactory_info: None,
                last_message: String::new(),
                session_info: None,
            },
            worker_info: None,
            session_id: None,

            last_message: String::new(),
            last_updated_at: Utc::now(),

            pruntime_lock: false,
            client: Arc::new(crate::pruntime::create_client(worker.endpoint.clone())),
            pending_requests: VecDeque::new(),

            phactory_info_execution_status: ExecutionStatus::Ok((0, String::new())),
            phactory_info_requested: false,
            phactory_info_requested_at: DateTime::<Utc>::MIN_UTC,

            sync_stage: SyncStage::NotStart,
            sync_execution_status: ExecutionStatus::Ok((0, String::new())),

            stopped: false,

            computation_stage: ComputationStage::NotStart,
            computation_exeuction_status: ExecutionStatus::Ok((0, String::new())),
            update_worker_info_count: 0,
            update_session_id_count: 0,
            update_session_info_count: 0,
        }
    }

    pub fn update_message(&mut self, message: &str, updated_at: Option<DateTime<Utc>>) {
        let updated_at = match updated_at {
            Some(updated_at) => updated_at,
            None => Utc::now(),
        };
        self.last_message = message.to_string();
        self.last_updated_at = updated_at;
        self.worker_status.last_message = self.display_last_message();
    }

    pub fn display_last_message(&self) -> String {
        format!(
            "[{}] {}",
            self.last_updated_at.format("%m-%d %H:%M:%S"),
            self.last_message
        )
    }

    pub fn public_key(&self) -> Option<Sr25519Public> {
        self.worker_status.phactory_info
            .as_ref()
            .and_then(|info| info.system.as_ref())
            .map(|info| info.public_key.clone())
            .map(|str|
                hex::decode(str).map(
                    |vec| Sr25519Public::from_slice(vec.as_slice()).unwrap()
                )
            )
            .and_then(|result| result.ok())
    }
    
    pub fn is_registered(&self) -> bool {
        self.worker_status.phactory_info
            .as_ref()
            .and_then(|info| info.system.as_ref())
            .map(|info| info.registered)
            .unwrap_or(false)
    }

    pub fn is_computing(&self) -> bool {
        let state = self.worker_status.session_info
            .as_ref()
            .map(|info| &info.state);
        match state {
            Some(state) => match state {
                WorkerState::Ready => false,
                _ => true,
            },
            None => false,
        }
    }

    pub fn is_match(&self, manifest: &SyncRequestManifest) -> bool {
        if let Some((from, _)) = manifest.headers {
            if self.headernum != from {
                return false;
            }
        }
        if let Some((from, _)) = manifest.para_headers {
            if self.para_headernum != from {
                return false;
            }
        }
        if let Some((from, _)) = manifest.blocks {
            if self.blocknum != from {
                return false;
            }
        }
        true
    }

    pub fn is_reached_chaintip(
        &self,
        chaintip: &ChaintipInfo,
    ) -> bool {
        self.blocknum == self.para_headernum
            && self.headernum == chaintip.relaychain + 1
            && self.para_headernum == chaintip.parachain + 1
    }

    pub fn is_reached_para_chaintip(
        &self,
        chaintip: &ChaintipInfo,
    ) -> bool {
        self.blocknum == self.para_headernum
            &&self.para_headernum == chaintip.parachain + 1
    }

    pub fn is_sync_only(&self) -> bool {
        self.worker_sync_only || self.pool_sync_only
    }

    pub fn is_updating_phactory_info_due(&self) -> bool {
        !self.phactory_info_requested
            && Utc::now().signed_duration_since(self.phactory_info_requested_at) >= UPDATE_PHACTORY_INFO_INTERVAL
    }
}

#[derive(Default)]
pub struct SyncInfo {
    pub headernum: Option<u32>,
    pub para_headernum: Option<u32>,
    pub blocknum: Option<u32>,
}

#[derive(Debug)]
pub enum PRuntimeRequest {
    PrepareLifecycle,
    InitRuntime(InitRuntimeRequest),
    LoadChainState(ChainState),
    Sync(SyncRequest),
    RegularGetInfo,
    PrepareRegister((bool, Option<sp_core::crypto::AccountId32>, bool)),
    GetEgressMessages,
    SignEndpoints(Vec<String>),
    TakeCheckpoint,
}

pub enum PRuntimeResponse {
    PrepareLifecycle(PhactoryInfo),
    InitRuntime(InitRuntimeResponse),
    LoadChainState,
    Sync(SyncInfo),
    RegularGetInfo(PhactoryInfo),
    PrepareRegister(InitRuntimeResponse),
    GetEgressMessages(GetEgressMessagesResponse),
    SignEndpoints(GetEndpointResponse),
    TakeCheckpoint(u32),
}

impl fmt::Display for PRuntimeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PRuntimeRequest::")?;
        match &self {
            PRuntimeRequest::PrepareLifecycle => write!(f, "PrepareLifecycle"),
            PRuntimeRequest::InitRuntime(_) => write!(f, "InitRuntime"),
            PRuntimeRequest::LoadChainState(_) => write!(f, "LoadChainState"),
            PRuntimeRequest::Sync(info) => {
                write!(f, "Sync(")?;
                if let Some((from, to)) = info.manifest.headers {
                    write!(f, "headers({}-{})", from, to)?;
                }
                if let Some((from, to)) = info.manifest.para_headers {
                    write!(f, "para_headers({}-{})", from, to)?;
                }
                if let Some((from, to)) = info.manifest.blocks {
                    write!(f, "blocks({}-{})", from, to)?;
                }
                write!(f, ")")
            },
            PRuntimeRequest::RegularGetInfo => write!(f, "RegularGetInfo"),
            PRuntimeRequest::PrepareRegister(_) => write!(f, "PrepareRegister"),
            PRuntimeRequest::GetEgressMessages => write!(f, "GetEgressMessages"),
            PRuntimeRequest::SignEndpoints(_) => write!(f, "SignEndpoints"),
            PRuntimeRequest::TakeCheckpoint => write!(f, "TakeCheckpoint"),
        }
    }
}

impl fmt::Display for PRuntimeResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PRuntimeResponse::")?;
        match &self {
            PRuntimeResponse::PrepareLifecycle(_) => write!(f, "PrepareLifecycle"),
            PRuntimeResponse::InitRuntime(_) => write!(f, "InitRuntime"),
            PRuntimeResponse::LoadChainState => write!(f, "LoadChainState"),
            PRuntimeResponse::Sync(info) => {
                write!(f, "Sync(")?;
                if let Some(to) = info.headernum {
                    write!(f, "headers({})", to)?;
                }
                if let Some(to) = info.para_headernum {
                    write!(f, "para_headers({})", to)?;
                }
                if let Some(to) = info.blocknum {
                    write!(f, "blocks({})", to)?;
                }
                write!(f, ")")
            },
            PRuntimeResponse::RegularGetInfo(_) => write!(f, "RegularGetInfo"),
            PRuntimeResponse::PrepareRegister(_) => write!(f, "PrepareRegister"),
            PRuntimeResponse::GetEgressMessages(_) => write!(f, "GetEgressMessages"),
            PRuntimeResponse::SignEndpoints(_) => write!(f, "SignEndpoints"),
            PRuntimeResponse::TakeCheckpoint(_) => write!(f, "TakeCheckpoint"),
        }
    }
}

pub enum WorkerEvent {
    UpdateWorker(crate::inv_db::Worker),
    PRuntimeRequest(PRuntimeRequest),
    PRuntimeResponse(Result<PRuntimeResponse, prpc::client::Error>),
    UpdateWorkerInfo(WorkerInfoV2<AccountId32>),
    UpdateSessionId(Option<AccountId32>),
    UpdateSessionInfo(Option<SessionInfo>),
    WorkerLifecycleCommand(WorkerLifecycleCommand),
    UpdateMessage((DateTime<Utc>, String)),
    MarkError((DateTime<Utc>, String)),
}

pub enum ProcessorEvent {
    AddWorker((crate::inv_db::Worker, Option<bool>, Option<AccountId32>)),
    DeleteWorker(String),
    UpdatePool((u64, Option<crate::inv_db::Pool>)),
    UpdatePoolOperator((u64, Option<AccountId32>)),
    WorkerEvent((String, WorkerEvent)),
    Heartbeat,
    BroadcastSync((SyncRequest, ChaintipInfo)),
    RequestUpdateSessionInfo,
}

pub type ProcessorRx = mpsc::UnboundedReceiver<ProcessorEvent>;
pub type ProcessorTx = mpsc::UnboundedSender<ProcessorEvent>;

pub struct Processor {
    pub rx: ProcessorRx,

    pub bus: Arc<Bus>,
    pub dsm: Arc<DataSourceManager>,
    pub txm: Arc<TxManager>,

    pub allow_fast_sync: bool,
    pub pccs_url: String,
    pub pccs_timeout_secs: u64,

    pub init_runtime_request_ias: InitRuntimeRequest,
    pub init_runtime_request_dcap: InitRuntimeRequest,

    pub chaintip: ChaintipInfo,
}

impl Processor {
    pub async fn create(
        rx: ProcessorRx,
        bus: Arc<Bus>,
        txm: Arc<TxManager>,
        dsm: Arc<crate::datasource::DataSourceManager>,
        args: &crate::cli::WorkerManagerCliArgs,
    ) -> Self {
        let ias_init_runtime_request = dsm.clone().get_init_runtime_default_request(Some(phala_types::AttestationProvider::Ias)).await.unwrap();
        let dcap_init_runtime_request = dsm.clone().get_init_runtime_default_request(Some(phala_types::AttestationProvider::Dcap)).await.unwrap();

        Self {
            rx,

            bus,
            dsm: dsm.clone(),
            txm,

            allow_fast_sync: !args.disable_fast_sync,
            pccs_url: args.pccs_url.clone(),
            pccs_timeout_secs: args.pccs_timeout.clone(),

            init_runtime_request_ias: ias_init_runtime_request,
            init_runtime_request_dcap: dcap_init_runtime_request,

            chaintip: ChaintipInfo {
                relaychain: crate::repository::relaychain_api(dsm.clone(), false).await.latest_finalized_block_number().await.unwrap(),
                parachain: crate::repository::parachain_api(dsm.clone(), false).await.latest_finalized_block_number().await.unwrap(),
            },
        }
    }

    pub async fn master_loop(
        &mut self,
    ) -> Result<()> {
        let mut workers = HashMap::<String, WorkerContext>::new();

        let bus = self.bus.clone();
        tokio::spawn(async move {
            loop {
                let _ = bus.send_processor_event(ProcessorEvent::Heartbeat);
                let nanos = 1_000_000_000 - Utc::now().nanosecond() % 1_000_000_000;
                tokio::time::sleep(std::time::Duration::from_nanos(nanos.into())).await;
            };
        });

        loop {
            let event = self.rx.recv().await;
            if event.is_none() {
                break
            }

            match event.unwrap() {
                ProcessorEvent::AddWorker((added_worker, pool_sync_only, operator)) => {
                    let worker_id = added_worker.id.clone();
                    let worker_context = WorkerContext::create(added_worker, pool_sync_only, operator);
                    if workers.contains_key(&worker_id) {
                        error!("[{}] Failed to add worker because the UUID is existed.", worker_id);
                    } else {
                        workers.insert(worker_id.clone(), worker_context);
                        self.send_worker_status(workers.get_mut(&worker_id).unwrap());
                        trace!("[{}] Added worker into processor. Starting", worker_id);
                        self.add_pruntime_request(
                            workers.get_mut(&worker_id).unwrap(),
                            PRuntimeRequest::PrepareLifecycle
                        );
                    }
                },
                ProcessorEvent::DeleteWorker(worker_id) => {
                    match workers.remove(&worker_id) {
                        Some(_removed_worker) => {
                        },
                        None => {
                            error!("[{}] Failed to delete worker because the UUID is not existed.", worker_id);
                        },
                    }
                },
                ProcessorEvent::UpdatePool((pool_id, pool)) => {
                    let pool_sync_only = pool.map(|p| p.sync_only).unwrap_or(true);
                    for worker in workers.values_mut() {
                        if worker.pool_id == pool_id && worker.pool_sync_only != pool_sync_only {
                            worker.pool_sync_only = pool_sync_only;
                        }
                    }
                },
                ProcessorEvent::UpdatePoolOperator((pool_id, operator)) => {
                    for worker in workers.values_mut() {
                        if worker.pool_id == pool_id && worker.operator != operator {
                            worker.operator = operator.clone();
                            self.add_pruntime_request(
                                worker,
                                PRuntimeRequest::PrepareRegister((
                                    true,
                                    worker.operator.clone(),
                                    false,
                                ))
                            );
                        }
                    }
                },
                ProcessorEvent::WorkerEvent((worker_id, worker_event)) => {
                    match workers.get_mut(&worker_id) {
                        Some(worker_context) => {
                            self.handle_worker_event(worker_context, worker_event);
                        },
                        None => {
                            warn!("[{}] Worker does not found.", worker_id);
                        },
                    }
                },
                ProcessorEvent::Heartbeat => {
                    for worker in workers.values_mut() {
                        if worker.is_updating_phactory_info_due() {
                            worker.phactory_info_requested = true;
                            worker.phactory_info_requested_at = Utc::now();
                            self.add_pruntime_request(worker, PRuntimeRequest::RegularGetInfo);
                        }

                    }
                },
                ProcessorEvent::BroadcastSync((request, info)) => {
                    for worker in workers.values_mut() {
                        if worker.is_match(&request.manifest) && worker.is_reached_chaintip(&self.chaintip) {
                            trace!("[{}] Accepted BroadcastSyncRequest", worker.uuid);
                            self.add_pruntime_request(worker, PRuntimeRequest::Sync(request.clone()));
                        }
                    }
                    self.chaintip = info;
                },
                ProcessorEvent::RequestUpdateSessionInfo => {
                    info!("Received RequestUpdateSessionInfo");
                    let mut worker_info_requests = Vec::<(String, Sr25519Public)>::new();
                    let mut session_id_requests = Vec::<(String, Sr25519Public)>::new();
                    let mut session_info_requests = Vec::<(String, AccountId32)>::new();
                    for worker in workers.values_mut() {
                        if !worker.is_registered() {
                            continue;
                        }
                        let public_key = match worker.public_key() {
                            Some(key) => key,
                            None => continue,
                        };

                        let initial_score = worker.worker_info.as_ref().and_then(|info| info.initial_score);
                        if initial_score.is_none() {
                            trace!("[{}] Requesting ChainStatus: WorkerInfoV2", worker.uuid);
                            worker_info_requests.push((worker.uuid.clone(), public_key));
                            continue;
                        }
                        match &worker.session_id {
                            Some(session_id) => {
                                trace!("[{}] Requesting ChainStatus: SessionInfo", worker.uuid);
                                session_info_requests.push((worker.uuid.clone(), session_id.clone()));
                            },
                            None => {
                                trace!("[{}] Requesting ChainStatus: SessionId", worker.uuid);
                                session_id_requests.push((worker.uuid.clone(), public_key));
                            },
                        }
                    }

                    if !worker_info_requests.is_empty() {
                        tokio::spawn(do_update_worker_info(
                            self.bus.clone(),
                            self.dsm.clone(),
                            worker_info_requests,
                        ));
                    }
                    if !session_id_requests.is_empty() {
                        tokio::spawn(do_update_session_id(
                            self.bus.clone(),
                            self.dsm.clone(),
                            session_id_requests,
                        ));
                    }
                    if !session_info_requests.is_empty() {
                        tokio::spawn(do_update_session_info(
                            self.bus.clone(),
                            self.dsm.clone(),
                            session_info_requests,
                        ));
                    }
                },
            }
        }

        Ok(())
    }

    fn handle_worker_event(
        &mut self,
        worker: &mut WorkerContext,
        event: WorkerEvent,
    ) {
        if !worker.worker_status.worker.enabled {
            match &event {
                WorkerEvent::UpdateWorker(_) => (),
                _ => {
                    warn!("[{}] Worker disabled but received event.", worker.uuid);
                    return;
                },
            }
        }

        match event {
            WorkerEvent::UpdateWorker(updated_worker) => {
                if worker.worker_status.worker.endpoint != updated_worker.endpoint {
                    worker.client = Arc::new(crate::pruntime::create_client(updated_worker.endpoint.clone()));
                }
                if worker.worker_status.worker.enabled != updated_worker.enabled {
                    let message = format!("Restarting due to switching {}, need to wait about {} seconds",
                        if updated_worker.enabled { "enabled" } else { "disabled" },
                        RESTART_WORKER_COOL_PERIOD.num_seconds() + 5
                    );
                    self.update_worker_state_and_message(worker, WorkerLifecycleState::Restarting, &message, None);
                    tokio::spawn(do_restart(
                        self.bus.clone(),
                        updated_worker,
                        worker.pool_sync_only,
                        worker.operator.clone(),
                    ));
                } else {
                    worker.worker_status.worker = updated_worker;
                }
            },
            WorkerEvent::PRuntimeRequest(request) => {
                self.add_pruntime_request(worker, request);
            },
            WorkerEvent::PRuntimeResponse(result) => {
                worker.pruntime_lock = false;
                match result {
                    Ok(response) => self.handle_pruntime_response(worker, response),
                    Err(err) => {
                        error!("[{}] pRuntime returned an error: {}", worker.uuid, err);
                        self.update_worker_message(
                            worker,
                            &err.to_string(),
                            None,
                        );
                    },
                }

                trace!("[{}] Pending PRuntimeRequest Count: {}", worker.uuid, worker.pending_requests.len());
                if let Some(request) = worker.pending_requests.pop_front() {
                    self.execute_pruntime_request(worker, request);
                }
            },
            WorkerEvent::UpdateWorkerInfo(worker_info) => {
                trace!("[{}] Received UpdateWorkerInfo", worker.uuid);
                worker.update_worker_info_count = worker.update_worker_info_count.saturating_add(1);
                worker.worker_info = Some(worker_info);
            },
            WorkerEvent::UpdateSessionId(session_id) => {
                trace!("[{}] Received UpdateSessionId", worker.uuid);
                worker.update_session_id_count = worker.update_session_id_count.saturating_add(1);
                if session_id.is_some() {
                    worker.session_id = session_id;
                    self.send_worker_status(worker);
                } else if worker.session_id.is_some() {
                    warn!("[{}] Received a none session_id, but we already have.", worker.uuid);
                }
            },
            WorkerEvent::UpdateSessionInfo(session_info) => {
                trace!("[{}] Received UpdateSessionInfo", worker.uuid);
                worker.update_session_info_count = worker.update_session_info_count.saturating_add(1);
                if session_info.is_some() {
                    worker.worker_status.session_info = session_info;
                    self.send_worker_status(worker);
                } else if worker.worker_status.session_info.is_some() {
                    warn!("[{}] Received a none session_info, but we already have.", worker.uuid);
                }
            },
            WorkerEvent::WorkerLifecycleCommand(command) => {
                self.handle_worker_lifecycle_command(worker, command);
            },
            WorkerEvent::UpdateMessage((timestamp, message)) => {
                self.update_worker_message(
                    worker,
                    &message,
                    Some(timestamp),
                );
            },
            WorkerEvent::MarkError((timestamp, error_msg)) => {
                self.update_worker_state_and_message(
                    worker,
                    WorkerLifecycleState::HasError(error_msg.clone()),
                    &error_msg,
                    Some(timestamp),
                );
            },
        }

    }

    fn update_worker_state(
        &mut self,
        worker: &mut WorkerContext,
        state: WorkerLifecycleState,
    ) {
        worker.worker_status.state = state;
        let _ = self.bus.send_worker_status_event((
            worker.uuid.clone(),
            WorkerStatusUpdate::UpdateStateAndMessage((
                worker.worker_status.state.clone(),
                worker.worker_status.last_message.clone(),
            )),
        ));
    }

    fn update_worker_state_and_message(
        &mut self,
        worker: &mut WorkerContext,
        state: WorkerLifecycleState,
        message: &str,
        updated_at: Option<DateTime<Utc>>,
    ) {
        info!("[{}] WORKER_MESSAGE: {}", worker.uuid, message);
        worker.worker_status.state = state;
        worker.update_message(message, updated_at);
        let _ = self.bus.send_worker_status_event((
            worker.uuid.clone(),
            WorkerStatusUpdate::UpdateStateAndMessage((
                worker.worker_status.state.clone(),
                worker.worker_status.last_message.clone(),
            )),
        ));
    }

    fn update_worker_message(
        &mut self,
        worker: &mut WorkerContext,
        message: &str,
        updated_at: Option<DateTime<Utc>>,
    ) {
        info!("[{}] WORKER_MESSAGE: {}", worker.uuid, message);
        worker.update_message(message, updated_at);
        let _ = self.bus.send_worker_status_event((
            worker.uuid.clone(),
            WorkerStatusUpdate::UpdateMessage(worker.worker_status.last_message.clone()),
        ));
    }

    fn send_worker_status(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        let _ = self.bus.send_worker_status_event((
            worker.uuid.clone(),
            WorkerStatusUpdate::Update(worker.worker_status.clone()),
        ));
    }

    fn send_worker_sync_info(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        let _ = self.bus.send_worker_status_event((
            worker.uuid.clone(),
            WorkerStatusUpdate::UpdateSyncInfo(SyncInfo {
                headernum: Some(worker.headernum),
                para_headernum: Some(worker.para_headernum),
                blocknum: Some(worker.blocknum),
            }),
        ));
    }

    fn add_pruntime_request(
        &mut self,
        worker: &mut WorkerContext,
        request: PRuntimeRequest,
    ) {
        if worker.stopped {
            match &request {
                PRuntimeRequest::PrepareLifecycle
                    | PRuntimeRequest::PrepareRegister((_, _, true))
                    | PRuntimeRequest::SignEndpoints(_)
                    | PRuntimeRequest::TakeCheckpoint
                    => (),
                _ => {
                    warn!("[{}] worker was stopped, skip the request.", worker.uuid);
                },
            }
        }

        trace!("[{}] Adding {}", worker.uuid, request);
        if let PRuntimeRequest::Sync(sync_request) = &request {
            if sync_request.is_empty() {
                if !worker.is_reached_chaintip(&self.chaintip) && sync_request.is_empty() {
                    warn!("[{}] Worker needs to be sync, but received an empty request. Try again.", worker.uuid);
                    self.request_next_sync(worker);
                } else {
                    trace!("[{}] Ignoring the empty sync request.", worker.uuid);
                }
                return;
            } else if !worker.is_match(&sync_request.manifest) {
                warn!("[{}] Ignoring not match Syncing: {}", worker.uuid, request);
                return;
            }
        }

        if !worker.pruntime_lock && worker.pending_requests.is_empty() {
            trace!("[{}] Immediately handle {}", worker.uuid, request);
            self.execute_pruntime_request(worker, request);
        } else {
            trace!("[{}] Enqueuing {} because: pruntime_lock {}, pendings: {}",
                worker.uuid,
                request,
                worker.pruntime_lock,
                worker.pending_requests.len()
            );
            worker.pending_requests.push_back(request);
        }
    }

    fn execute_pruntime_request(
        &mut self,
        worker: &mut WorkerContext,
        request: PRuntimeRequest,
    ) {
        worker.pruntime_lock = true;
        tokio::spawn(
            dispatch_pruntime_request(
                self.bus.clone(),
                worker.uuid.clone(),
                worker.client.clone(),
                request,
            )
        );
    }

    fn handle_pruntime_response(
        &mut self,
        worker: &mut WorkerContext,
        response: PRuntimeResponse,
    ) {
        trace!("[{}] Received OK {}", worker.uuid, response);
        match response {
            PRuntimeResponse::PrepareLifecycle(info) => {
                worker.worker_status.phactory_info = Some(info.clone());
                self.send_worker_status(worker);

                worker.headernum = info.headernum;
                worker.para_headernum = info.para_headernum;
                worker.blocknum = info.blocknum;

                self.request_prepare_lifecycle(worker);
            },
            PRuntimeResponse::InitRuntime(_response) => {
                self.update_worker_message(worker, "InitRuntime Completed.", None);
                self.add_pruntime_request(worker, PRuntimeRequest::PrepareLifecycle);
            },
            PRuntimeResponse::LoadChainState => {
                self.update_worker_message(worker, "LoadChainState Completed.", None);
                self.add_pruntime_request(worker, PRuntimeRequest::PrepareLifecycle);
            },
            PRuntimeResponse::Sync(info) => {
                self.handle_pruntime_sync_response(worker, &info);
                self.send_worker_sync_info(worker);
            },
            PRuntimeResponse::RegularGetInfo(phactory_info) => {
                worker.phactory_info_requested = false;
                worker.worker_status.phactory_info = Some(phactory_info);
                self.send_worker_status(worker);
            },
            PRuntimeResponse::PrepareRegister(response) => {
                self.update_worker_message(worker, "Register Starting...", None);
                tokio::spawn(do_register(
                    self.bus.clone(),
                    self.txm.clone(),
                    worker.uuid.clone(),
                    worker.pool_id.clone(),
                    response,
                    self.pccs_url.clone(),
                    self.pccs_timeout_secs.clone(),
                ));
            },
            PRuntimeResponse::GetEgressMessages(response) => {
                self.handle_pruntime_egress_messages(worker, response)
            },
            PRuntimeResponse::SignEndpoints(response) => {
                tokio::spawn(do_update_endpoints(
                    self.bus.clone(),
                    self.txm.clone(),
                    worker.uuid.clone(),
                    worker.pool_id.clone(),
                    response,
                ));
            },
            PRuntimeResponse::TakeCheckpoint(synced_to) => {
                self.update_worker_message(
                    worker,
                    &format!("Checkpoint saved to #{}.", synced_to),
                    None
                );
            },
        }
        trace!("[{}] Handled PRuntimeResponse", worker.uuid);
    }

    fn handle_pruntime_sync_response(
        &mut self,
        worker: &mut WorkerContext,
        info: &SyncInfo,
    ) {
        if let Some(headernum) = info.headernum {
            worker.headernum = headernum + 1;
            trace!("[{}] Synced headernum, next: {}", worker.uuid, worker.headernum);
        }
        if let Some(para_headernum) = info.para_headernum {
            worker.para_headernum = para_headernum + 1;
            trace!("[{}] Synced para_headernum, next: {}", worker.uuid, worker.para_headernum);
        }
        if let Some(blocknum) = info.blocknum {
            worker.blocknum = blocknum + 1;
            trace!("[{}] Synced updated, next: {}", worker.uuid, worker.blocknum);
        }

        if !worker.is_reached_chaintip(&self.chaintip) {
            trace!("[{}] Not at chaintip, requesting next sync", worker.uuid);
            self.request_next_sync(worker);
        } else {
            trace!("[{}] Reached to chaintip!", worker.uuid);
            if worker.is_registered() && info.blocknum.is_some() {
                trace!("[{}] Dispatched a block, requesting EgressMessages", worker.uuid);
                self.add_pruntime_request(worker, PRuntimeRequest::GetEgressMessages);
            }
            if !matches!(worker.computation_stage, ComputationStage::Completed) {
                trace!("[{}] Requesting computation determination", worker.uuid);
                self.request_computation_determination(worker);
            }
        }
    }

    fn request_prepare_lifecycle(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        trace!("[{}] checking worker.phactory_info:", worker.uuid);
        if worker.worker_status.phactory_info.is_none() {
            self.add_pruntime_request(worker, PRuntimeRequest::PrepareLifecycle);
            return;
        }

        trace!("[{}] checking worker.phactory_info.initialized", worker.uuid);
        let info  = worker.worker_status.phactory_info.as_ref().unwrap();
        if !info.initialized {
            self.request_init(worker);
            return;
        }

        trace!("[{}] checking worker fast sync", worker.uuid);
        if self.allow_fast_sync && info.can_load_chain_state {
            self.request_fast_sync(worker);
            return;
        }

        trace!("[{}] requesting next sync", worker.uuid);
        self.request_next_sync(worker);
        self.update_worker_state_and_message(
            worker,
            WorkerLifecycleState::Synchronizing,
            "Start Synchronizing...",
            None, 
        );
    }

    fn request_computation_determination(
        &mut self,
        worker: &mut WorkerContext
    ) {
        if matches!(worker.computation_stage, ComputationStage::Completed) {
            return;
        }
        if worker.is_sync_only() {
            trace!("[{}] Worker or Pool is sync only mode, skip computation determination.", worker.uuid);
            return;
        }

        if matches!(worker.worker_status.state, WorkerLifecycleState::Synchronizing) {
            self.update_worker_state_and_message(
                worker,
                WorkerLifecycleState::Preparing,
                "Worker reached to ChainTip, trying to start computation.",
                None,
            );
        }

        if !worker.is_registered() {
            match &worker.computation_stage {
                ComputationStage::NotStart => {
                    trace!("[{}] Requesting PRuntime InitRuntimeResponse for register", worker.uuid);
                    self.add_pruntime_request(
                        worker,
                        PRuntimeRequest::PrepareRegister((true, worker.operator.clone(), false))
                    );
                    worker.computation_stage = ComputationStage::Register;
                },
                ComputationStage::Register => {
                    trace!("[{}] Register has been requested, waiting for register result", worker.uuid);
                },
                _ => {
                    error!("[{}] Worker is not registered but has Stage={}", worker.uuid, worker.computation_stage);
                },
            }
            return;
        }

        if worker.worker_status.worker.gatekeeper {
            trace!("[{}] Worker is in Gatekeeper mode. Completing computation determination.", worker.uuid);
            worker.computation_stage = ComputationStage::Completed;
            return;
        }

        let public_key = match worker.public_key() {
            Some(public_key) => public_key,
            None => {
                trace!("[{}] No public key found. Skip continuing computation determination.", worker.uuid);
                return
            },
        };

        if worker.update_worker_info_count == 0 {
            trace!("[{}] Not updating WorkerInfo yet. Skip continuing computation determination.", worker.uuid);
            return;
        }
        match &worker.worker_info {
            Some(worker_info) => {
                if worker_info.initial_score.is_none() {
                    trace!("[{}] No initial_score yet. Skip continuing computation determination.", worker.uuid);
                    return;
                }
            },
            None => {
                trace!("[{}] No worker_info found yet. Skip continuing computation determination.", worker.uuid);
                return;
            },
        };

        if worker.update_session_id_count == 0 {
            trace!("[{}] Not updating SessionId yet. Skip continuing computation determination.", worker.uuid);
            return;
        }
        if worker.session_id.is_none() {
            match &worker.computation_stage {
                ComputationStage::NotStart | ComputationStage::Register => {
                    trace!("[{}] Requesting add to pool", worker.uuid);
                    tokio::spawn(do_add_worker_to_pool(
                        self.bus.clone(),
                        self.txm.clone(),
                        worker.uuid.clone(),
                        worker.pool_id.clone(),
                        public_key.clone(),
                    ));
                    worker.computation_stage = ComputationStage::AddToPool;
                },
                ComputationStage::AddToPool => {
                    trace!("[{}] AddToPool has been requested, waiting for session_id", worker.uuid);
                },
                _ => {
                    error!("[{}] Worker has no SessionId but has Stage={}", worker.uuid, worker.computation_stage);
                }
            }
            return;
        }

        if worker.update_session_info_count == 0 {
            trace!("[{}] Not updating SessionInfo yet. Skip continuing computation determination.", worker.uuid);
            return;
        }
        if !worker.is_computing() {
            match &worker.computation_stage {
                ComputationStage::NotStart | ComputationStage::Register | ComputationStage::AddToPool => {
                    trace!("[{}] requesting start computing", worker.uuid);
                    tokio::spawn(do_start_computing(
                        self.bus.clone(),
                        self.txm.clone(),
                        worker.uuid.clone(),
                        worker.pool_id.clone(),
                        public_key.clone(),
                        worker.worker_status.worker.stake.clone()
                    ));
                    worker.computation_stage = ComputationStage::StartComputing;
                },
                ComputationStage::StartComputing => {
                    trace!("[{}] StartComputing has been requested, waiting...", worker.uuid);
                },
                _ => {
                    error!("[{}] Worker is not computing but has Stage={}", worker.uuid, worker.computation_stage);
                },
            }
            return;
        }

        worker.computation_stage = ComputationStage::Completed;
        self.update_worker_state_and_message(
            worker,
            WorkerLifecycleState::Working,
            "Computing Now!",
            None,
        );
    }

    fn request_init(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        let info  = worker.worker_status.phactory_info.as_ref().unwrap();
        // pRuntime versions lower than 2.2.0 always returns an empty list.
        let supported = &info.supported_attestation_methods;
        let request = if supported.is_empty() || supported.contains(&"epid".into()) {
            self.init_runtime_request_ias.clone()
        } else if supported.contains(&"dcap".into()) {
            self.init_runtime_request_dcap.clone()
        } else {
            let err_msg = "Supported attestation methods does not include epid or dcap.";
            error!("[{}] {}", worker.uuid, err_msg);
            self.update_worker_state_and_message(
                worker,
                WorkerLifecycleState::HasError(err_msg.to_string()),
                &err_msg,
                None,
            );
            return;
        };

        self.update_worker_message(worker, "InitRuntime Starting...", None);
        self.add_pruntime_request(worker, PRuntimeRequest::InitRuntime(request));
    }

    fn request_fast_sync(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        let _ = self.bus.send_repository_event(
            RepositoryEvent::GenerateFastSyncRequest((worker.uuid.clone(), worker.public_key().unwrap()))
        );
        self.update_worker_message(worker, "LoadChainState Starting...", None);
    }

    fn request_next_sync(
        &mut self,
        worker: &WorkerContext,
    ) {
        let _ = self.bus.send_repository_event(RepositoryEvent::UpdateWorkerSyncInfo(
            WorkerSyncInfo {
                worker_id: worker.uuid.clone(),
                headernum: worker.headernum,
                para_headernum: worker.para_headernum,
                blocknum: worker.blocknum,
            }
        ));
    }

    fn handle_pruntime_egress_messages(
        &mut self,
        worker: &WorkerContext,
        response: GetEgressMessagesResponse,
    ) {
        let messages = match response.decode_messages() {
            Ok(messages) => messages,
            Err(err) => {
                error!("[{}] failed to decode egress messages. {}", worker.uuid, err);
                return;
            },
        };

        for (sender, messages) in messages {
            let _ = self.bus.send_messages_event(
                MessagesEvent::SyncMessages((
                    worker.uuid.clone(),
                    worker.pool_id,
                    sender,
                    messages,
                ))
            );
        }
    }

    fn handle_worker_lifecycle_command(
        &mut self,
        worker: &mut WorkerContext,
        command: WorkerLifecycleCommand,
    ) {
        match command {
            WorkerLifecycleCommand::ShouldRestart => {
                info!("[{}] Restarting...", worker.uuid);
                self.update_worker_state_and_message(
                    worker,
                    WorkerLifecycleState::Restarting,
                    &format!("Restarting, need to wait about {} seconds",
                        RESTART_WORKER_COOL_PERIOD.num_seconds() + 5
                    ),
                    None,
                );
                tokio::spawn(do_restart(
                    self.bus.clone(),
                    worker.worker_status.worker.clone(),
                    worker.pool_sync_only,
                    worker.operator.clone(),
                ));
            },
            WorkerLifecycleCommand::ShouldForceRegister => {
                self.add_pruntime_request(
                    worker,
                    PRuntimeRequest::PrepareRegister((true, worker.operator.clone(), true)),
                );
            },
            WorkerLifecycleCommand::ShouldUpdateEndpoint(endpoints) => {
                self.add_pruntime_request(worker, PRuntimeRequest::SignEndpoints(endpoints));
            },
            WorkerLifecycleCommand::ShouldTakeCheckpoint => {
                self.add_pruntime_request(worker, PRuntimeRequest::TakeCheckpoint);
            },
        }
    }
}

async fn dispatch_pruntime_request(
    bus: Arc<Bus>,
    worker_id: String,
    client: Arc<PRuntimeClient>,
    request: PRuntimeRequest,
) {
    trace!("[{}] Start to dispatch PRuntimeRequest: {}", worker_id, request);
    let need_mark_error = matches!(
        &request,
        PRuntimeRequest::PrepareLifecycle
            | PRuntimeRequest::InitRuntime(_)
            | PRuntimeRequest::LoadChainState(_)
            | PRuntimeRequest::PrepareRegister(_)
    );
    let result = match request {
        PRuntimeRequest::PrepareLifecycle => {
            client.get_info(())
                .await
                .map(|response| PRuntimeResponse::PrepareLifecycle(response))
        },
        PRuntimeRequest::InitRuntime(request) => {
            client.init_runtime(request)
                .await
                .map(|response| PRuntimeResponse::InitRuntime(response))
        },
        PRuntimeRequest::LoadChainState(request) => {
            client.load_chain_state(request)
                .await
                .map(|_| PRuntimeResponse::LoadChainState)
        },
        PRuntimeRequest::Sync(request) => {
            do_sync_request(client, request)
                .await
                .map(|response| PRuntimeResponse::Sync(response))
        },
        PRuntimeRequest::RegularGetInfo => {
            client.get_info(())
                .await
                .map(|response| PRuntimeResponse::RegularGetInfo(response))
        },
        PRuntimeRequest::PrepareRegister((force_refresh_ra, operator, _)) => {
            let request = GetRuntimeInfoRequest::new(force_refresh_ra, operator);
            client.get_runtime_info(request)
                .await
                .map(|response| PRuntimeResponse::PrepareRegister(response))
        },
        PRuntimeRequest::GetEgressMessages => {
            client.get_egress_messages(())
                .await
                .map(|response| {
                    PRuntimeResponse::GetEgressMessages(response)
                })
        },
        PRuntimeRequest::SignEndpoints(endpoints) => {
            client.sign_endpoint_info(SignEndpointsRequest::new(endpoints))
                .await
                .map(|response| {
                    PRuntimeResponse::SignEndpoints(response)
                })
        },
        PRuntimeRequest::TakeCheckpoint => {
            client.take_checkpoint(())
                .await
                .map(|response| {
                    PRuntimeResponse::TakeCheckpoint(response.synced_to)
                })
        },
    };

    if need_mark_error {
        if let Err(err) = &result {
            let _ = bus.send_worker_mark_error(worker_id.clone(), err.to_string());
        }
    }
    let _ = bus.send_processor_event(ProcessorEvent::WorkerEvent((worker_id.clone(), WorkerEvent::PRuntimeResponse(result))));
    trace!("[{}] Completed to dispatch PRuntimeRequest", worker_id);
}

async fn do_sync_request(
    client: Arc<PRuntimeClient>,
    request: SyncRequest,
) -> Result<SyncInfo, prpc::client::Error> {
    let mut response = SyncInfo { ..Default::default() };

    if let Some(headers) = request.headers {
        match client.sync_header(headers).await {
            Ok(synced_to) => {
                response.headernum = Some(synced_to.synced_to);
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    if let Some(para_headers) = request.para_headers {
        match client.sync_para_header(para_headers).await {
            Ok(synced_to) => {
                response.para_headernum = Some(synced_to.synced_to);
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    if let Some(combined_headers) = request.combined_headers {
        match client.sync_combined_headers(combined_headers).await {
            Ok(synced_to) => {
                response.headernum = Some(synced_to.relaychain_synced_to);
                response.para_headernum = Some(synced_to.parachain_synced_to);
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    if let Some(blocks) = request.blocks {
        match client.dispatch_blocks(blocks).await {
            Ok(synced_to) => {
                response.blocknum = Some(synced_to.synced_to);
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    Ok(response)
}

async fn do_register(
    bus: Arc<Bus>,
    txm: Arc<TxManager>,
    worker_id: String,
    pool_id: u64,
    response: InitRuntimeResponse,
    pccs_url: String,
    pccs_timeout_secs: u64,
) {
    let attestation = match response.attestation {
        Some(attestation) => attestation,
        None => {
            error!("[{}] Worker has no attestation.", worker_id);
            return;
        },
    };
    let v2 = attestation.payload.is_none();
    let attestation = pherry::attestation_to_report(
        attestation,
        &pccs_url,
        pccs_timeout_secs,
    )
    .await;
    let attestation = match attestation {
        Ok(attestation) => attestation,
        Err(err) => {
            error!("[{}] Failed to get attestation report. {}", worker_id, err);
            let _ = bus.send_worker_event(
                worker_id,
                WorkerEvent::MarkError((
                    Utc::now(),
                    err.to_string(),
                ))
            );
            return;
        },
    };

    let result = txm.register_worker(pool_id, response.encoded_runtime_info, attestation, v2).await;
    match result {
        Ok(_) => {
            info!("[{}] Worker Register Completed.", worker_id);
            let _ = bus.send_worker_event(
                worker_id.clone(),
                WorkerEvent::UpdateMessage((
                    Utc::now(),
                    "Worker Register Completed.".to_string(),
                ))
            );
            let _ = bus.send_pruntime_request(worker_id.clone(), PRuntimeRequest::RegularGetInfo);
        },
        Err(err) => {
            error!("[{}] Worker Register Failed: {}", worker_id, err);
            let _ = bus.send_worker_event(
                worker_id,
                WorkerEvent::MarkError((
                    Utc::now(),
                    err.to_string(),
                ))
            );
            return;
        },
    }
}

async fn do_add_worker_to_pool(
    bus: Arc<Bus>,
    txm: Arc<TxManager>,
    worker_id: String,
    pool_id: u64,
    worker_public_key: Sr25519Public,
) {
    let result = txm.add_worker(pool_id, worker_public_key).await;
    if let Err(err) = result {
        let err_msg = format!("Failed to add_worker_to_pool. {}", err);
        error!("[{}] {}", worker_id, err_msg);
        let _ = bus.send_worker_mark_error(worker_id, err_msg);
    }
}

async fn do_start_computing(
    bus: Arc<Bus>,
    txm: Arc<TxManager>,
    worker_id: String,
    pool_id: u64,
    worker_public_key: Sr25519Public,
    stake: String,
) {
    let result = txm.start_computing(pool_id, worker_public_key, stake).await;
    if let Err(err) = result {
        let err_msg = format!("Failed to start computing. {}", err);
        error!("[{}] {}", worker_id, err_msg);
        let _ = bus.send_worker_mark_error(worker_id, err_msg);
    }
}

async fn do_restart(
    bus: Arc<Bus>,
    worker: crate::inv_db::Worker,
    pool_sync_only: bool,
    operator: Option<AccountId32>,

) {
    let worker_id = worker.id.clone();
    let _ = bus.send_messages_event(MessagesEvent::RemoveWorker(worker_id.clone()));
    let _ = bus.send_processor_event(ProcessorEvent::DeleteWorker(worker_id.clone()));
    info!("[{}] Restarting: Remove WorkerContext command sent, wait {} seconds and then add back",
        worker_id, RESTART_WORKER_COOL_PERIOD.num_seconds());
    tokio::time::sleep(RESTART_WORKER_COOL_PERIOD.to_std().unwrap()).await;
    let _ = bus.send_processor_event(ProcessorEvent::AddWorker((
        worker,
        Some(pool_sync_only),
        operator,
    )));
    info!("[{}] Restart: Add WorkerContext command sent.", worker_id);

}

async fn do_update_endpoints(
    bus: Arc<Bus>,
    txm: Arc<TxManager>,
    worker_id: String,
    pool_id: u64,
    response: GetEndpointResponse,
) {
    let result = txm.update_worker_endpoint(pool_id, response).await;
    match result {
        Ok(_) => {
            let _ = bus.send_worker_event(
                worker_id,
                WorkerEvent::UpdateMessage((
                    Utc::now(),
                    "Updated endpoints.".to_string(),
                ))
            );
        },
        Err(err) => {
            let err_msg = format!("ShouldUpdateEndpoint failed. {}", err);
            error!("[{}] {}", worker_id, err_msg);
            let _ = bus.send_worker_event(
                worker_id,
                WorkerEvent::UpdateMessage((
                    Utc::now(),
                    err_msg,
                ))
            );
        },
    }
}

async fn do_update_worker_info(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    requests: Vec<(String, Sr25519Public)>,
) -> Result<()> {
    let para_api = use_parachain_api!(dsm, false).ok_or(anyhow!("parachain_api not found"))?;
    let metadata = para_api.metadata();
    let hash = para_api.rpc().finalized_head().await?;
    let storage = para_api.storage().at(hash);

    let futures = requests
        .iter()
        .map(|(_, public_key)| {
            let address = subxt::dynamic::storage(
                "PhalaRegistry",
                "Workers",
                vec![subxt::dynamic::Value::from_bytes(public_key)],
            );
            let storage = storage.clone();
            let lookup_bytes = crate::utils::storage_address_bytes(&address, &metadata).unwrap();
            tokio::spawn(async move {
                storage.fetch_raw(&lookup_bytes).await
            })
        })
        .collect::<Vec<_>>();

    let results = match try_join_all(futures).await {
        Ok(results) => results,
        Err(err) => {
            error!("Met error during update worker info: {}", err);
            return Err(err.into());
        },
    };

    let mut session_ids_requests = Vec::<(String, Sr25519Public)>::new();
    for (idx, (worker_id, public_key)) in requests.iter().enumerate() {
        let result = match results.get(idx) {
            Some(res) => res,
            None => {
                error!("WorkerInfo: #{} result not exists.", idx);
                continue;
            },
        };
        match result {
            Ok(result) => {
                if let Some(bytes) = result {
                    let worker_info = match WorkerInfoV2::<AccountId32>::decode(&mut bytes.as_slice()) {
                        Ok(info) => info,
                        Err(err) => {
                            error!("Failed to decode WorkerInvoV2: {:?}. {:?}", err, bytes);
                            continue;
                        },
                    };
                    let _ = bus.send_worker_event(
                        worker_id.clone(),
                        WorkerEvent::UpdateWorkerInfo(worker_info.clone()),
                    );
                    session_ids_requests.push((worker_id.clone(), public_key.clone()))
                }
            },
            Err(err) => {
                error!("{}", err);
            },
        }
    }

    if session_ids_requests.is_empty() {
        Ok(())
    } else {
        do_update_session_id(bus, dsm, session_ids_requests).await
    }
}

async fn do_update_session_id(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    requests: Vec<(String, Sr25519Public)>,
) -> Result<()> {
    let para_api = use_parachain_api!(dsm, false).ok_or(anyhow!("parachain_api not found"))?;
    let metadata = para_api.metadata();
    let hash = para_api.rpc().finalized_head().await?;
    let storage = para_api.storage().at(hash);

    let futures = requests
        .iter()
        .map(|(_, public_key)| {
            let address = subxt::dynamic::storage(
                "PhalaComputation",
                "WorkerBindings",
                vec![subxt::dynamic::Value::from_bytes(public_key)],
            );
            let storage = storage.clone();
            let lookup_bytes = crate::utils::storage_address_bytes(&address, &metadata).unwrap();
            tokio::spawn(async move {
                storage.fetch_raw(&lookup_bytes).await
            })
        })
        .collect::<Vec<_>>();

    let results = match try_join_all(futures).await {
        Ok(results) => results,
        Err(err) => {
            error!("Met error during update session id: {}", err);
            return Err(err.into());
        },
    };

    let mut session_info_requests = Vec::<(String, AccountId32)>::new();
    for (idx, (worker_id, _)) in requests.iter().enumerate() {
        let result = match results.get(idx) {
            Some(res) => res,
            None => {
                error!("SessionId: #{} result not exists.", idx);
                continue;
            },
        };
        match result {
            Ok(result) => {
                let session_id = match result {
                    Some(bytes) => match AccountId32::decode(&mut bytes.as_slice()) {
                        Ok(id) => Some(id),
                        Err(err) => {
                            error!("Failed to decode AccountId32: {:?}. {:?}", err, bytes);
                            continue;
                        },
                    },
                    None => None,
                };
                let _ = bus.send_worker_event(
                    worker_id.clone(),
                    WorkerEvent::UpdateSessionId(session_id.clone()),
                );
                if let Some(session_id) = session_id {
                    session_info_requests.push((worker_id.clone(), session_id))
                }
            },
            Err(err) => {
                error!("{}", err);
            },
        }
    }

    if session_info_requests.is_empty() {
        Ok(())
    } else {
        do_update_session_info(bus, dsm, session_info_requests).await
    }
}

async fn do_update_session_info(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    requests: Vec<(String, AccountId32)>,
) -> Result<()> {
    let para_api = use_parachain_api!(dsm, false).ok_or(anyhow!("parachain_api not found"))?;
    let metadata = para_api.metadata();
    let hash = para_api.rpc().finalized_head().await?;
    let storage = para_api.storage().at(hash);

    let futures = requests
        .iter()
        .map(|(_, session_id)| {
            let address = subxt::dynamic::storage(
                "PhalaComputation",
                "Sessions",
                vec![subxt::dynamic::Value::from_bytes(session_id)],
            );
            let storage = storage.clone();
            let lookup_bytes = crate::utils::storage_address_bytes(&address, &metadata).unwrap();
            tokio::spawn(async move {
                storage.fetch_raw(&lookup_bytes).await
            })
        })
        .collect::<Vec<_>>();

    let results = match try_join_all(futures).await {
        Ok(results) => results,
        Err(err) => {
            error!("Met error during update session id: {}", err);
            return Err(err.into());
        },
    };

    for (idx, (worker_id, _)) in requests.iter().enumerate() {
        let result = match results.get(idx) {
            Some(res) => res,
            None => {
                error!("SessionInfo: #{} result not exists.", idx);
                continue;
            },
        };
        match result {
            Ok(result) => {
                let session_info = match result {
                    Some(bytes) => {
                        let session_info = match SessionInfo::decode(&mut bytes.as_slice()) {
                            Ok(id) => id,
                            Err(err) => {
                                error!("Failed to decode SessionInfo: {:?}. {:?}", err, bytes);
                                continue;
                            },
                        };
                        Some(session_info)
                    }
                    _ => None
                };
                let _ = bus.send_worker_event(
                    worker_id.clone(),
                    WorkerEvent::UpdateSessionInfo(session_info),
                );
            },
            Err(err) => {
                error!("{}", err);
            },
        }
    }
    Ok(())
}