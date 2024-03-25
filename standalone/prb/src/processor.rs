use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{debug, error, info, trace, warn};
use phactory_api::prpc::{
    self, ChainState, GetEgressMessagesResponse, GetEndpointResponse, GetRuntimeInfoRequest,
    InitRuntimeRequest, InitRuntimeResponse, PhactoryInfo, SignEndpointsRequest,
};
use phala_pallets::pallet_computation::{SessionInfo, WorkerState};
use phala_pallets::registry::WorkerInfoV2;
use sp_core::crypto::{AccountId32, ByteArray};
use sp_core::sr25519::Public as Sr25519Public;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::api::WorkerStatus;
use crate::bus::Bus;
use crate::repository::{RepositoryEvent, SyncRequest, SyncRequestManifest, WorkerSyncInfo};
use crate::messages::MessagesEvent;
use crate::pruntime::PRuntimeClient;
use crate::tx::TxManager;
use crate::worker::{WorkerLifecycleCommand, WorkerLifecycleState};
use crate::worker_status::WorkerStatusUpdate;


enum WorkerLifecycle {
    Init
}

enum SyncStatus {
    Idle,
    Syncing,
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

    pub calling: bool,
    pub accept_sync_request: bool,

    pub client: Arc<PRuntimeClient>,
    pub pending_requests: VecDeque<PRuntimeRequest>,

    pub register_requested: bool,
    pub add_to_pool_requested: bool,
    pub computing_requested: bool,
}

impl WorkerContext {
    pub fn create(
        worker: crate::inv_db::Worker,
        pool: Option<crate::inv_db::Pool>,
        operator: Option<AccountId32>,
    ) -> Self {
        Self {
            uuid: worker.id.clone(),

            pool_id: worker.pid.unwrap_or_default(),
            operator,
            worker_sync_only: worker.sync_only,
            pool_sync_only: pool.map(|p| p.sync_only).unwrap_or(true),

            headernum: 0,
            para_headernum: 0,
            blocknum: 0,

            worker_status: WorkerStatus {
                worker: worker.clone(),
                state: WorkerLifecycleState::Starting,
                phactory_info: None,
                last_message: String::new(),
                session_info: None,
            },
            worker_info: None,
            session_id: None,

            last_message: String::new(),
            last_updated_at: chrono::Utc::now(),

            calling: false,
            accept_sync_request: false,

            client: Arc::new(crate::pruntime::create_client(worker.endpoint.clone())),
            pending_requests: VecDeque::new(),

            register_requested: false,
            add_to_pool_requested: false,
            computing_requested: false,
        }
    }

    pub fn update_message(&mut self, message: &str, updated_at: Option<DateTime<Utc>>) {
        let updated_at = match updated_at {
            Some(updated_at) => updated_at,
            None => chrono::Utc::now(),
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
        relaychain_chaintip: u32,
        parachain_chaintip: u32,
    ) -> bool {
        self.headernum == relaychain_chaintip + 1
            && self.para_headernum == parachain_chaintip + 1
            && self.blocknum == self.para_headernum
    }

    pub fn is_reached_para_chaintip(
        &self,
        parachain_chaintip: u32,
    ) -> bool {
        self.para_headernum == parachain_chaintip + 1
            && self.blocknum == self.para_headernum
    }

    pub fn is_sync_only(&self) -> bool{
        self.worker_sync_only || self.pool_sync_only
    }
}

#[derive(Default)]
pub struct SyncInfo {
    pub headernum: Option<u32>,
    pub para_headernum: Option<u32>,
    pub blocknum: Option<u32>,
}

#[derive(Default)]
pub struct BroadcastInfo {
    //pub sync_info: SyncInfo,
    pub relay_chaintip: u32,
    pub para_chaintip: u32,
}

#[derive(Debug)]
pub enum PRuntimeRequest {
    PrepareLifecycle,
    InitRuntime(InitRuntimeRequest),
    LoadChainState(ChainState),
    Sync(SyncRequest),
    RegularGetInfo,
    PrepareRegister((bool, Option<sp_core::crypto::AccountId32>)),
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

pub enum WorkerEvent {
    UpdateWorker(crate::inv_db::Worker),
    PRuntimeRequest(PRuntimeRequest),
    PRuntimeResponse(Result<PRuntimeResponse, prpc::client::Error>),
    UpdateSessionInfo(SessionInfo),
    WorkerLifecycleCommand(WorkerLifecycleCommand),
    UpdateMessage((DateTime<Utc>, String)),
    MarkError((DateTime<Utc>, String)),
}

pub enum ProcessorEvent {
    AddWorker((crate::inv_db::Worker, Option<crate::inv_db::Pool>, Option<AccountId32>)),
    DeleteWorker(String),
    UpdatePool((u64, Option<crate::inv_db::Pool>)),
    UpdatePoolOperator((u64, Option<AccountId32>)),
    WorkerEvent((String, WorkerEvent)),
    Heartbeat,
    BroadcastSync((SyncRequest, BroadcastInfo)),
}

pub type ProcessorRx = mpsc::UnboundedReceiver<ProcessorEvent>;
pub type ProcessorTx = mpsc::UnboundedSender<ProcessorEvent>;

pub struct Processor {
    pub rx: ProcessorRx,

    pub bus: Arc<Bus>,
    pub txm: Arc<TxManager>,

    pub allow_fast_sync: bool,
    pub pccs_url: String,
    pub pccs_timeout_secs: u64,

    pub init_runtime_request_ias: InitRuntimeRequest,
    pub init_runtime_request_dcap: InitRuntimeRequest,

    pub relaychain_chaintip: u32,
    pub parachain_chaintip: u32,
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
            txm,

            allow_fast_sync: !args.disable_fast_sync,
            pccs_url: args.pccs_url.clone(),
            pccs_timeout_secs: args.pccs_timeout.clone(),

            init_runtime_request_ias: ias_init_runtime_request,
            init_runtime_request_dcap: dcap_init_runtime_request,

            relaychain_chaintip: crate::repository::relaychain_api(dsm.clone(), false).await.latest_finalized_block_number().await.unwrap(),
            parachain_chaintip: crate::repository::parachain_api(dsm.clone(), false).await.latest_finalized_block_number().await.unwrap(),

        }
    }

    pub async fn master_loop(
        &mut self,
    ) -> Result<()> {
        let mut workers = HashMap::<String, WorkerContext>::new();

        let bus = self.bus.clone();
        tokio::spawn(async move {
            loop {
                let now = chrono::Utc::now();
                let _ = bus.send_processor_event(ProcessorEvent::Heartbeat);
                let duration = chrono::Utc::now().signed_duration_since(&now);
                let duration = chrono::Duration::seconds(3) - duration;
                if duration > chrono::Duration::zero() {
                    tokio::time::sleep(duration.to_std().unwrap()).await;
                }
            };
        });

        loop {
            let event = self.rx.recv().await;
            if event.is_none() {
                break
            }

            match event.unwrap() {
                ProcessorEvent::AddWorker((added_worker, pool, operator)) => {
                    let worker_id = added_worker.id.clone();
                    let worker_context = WorkerContext::create(added_worker, pool, operator);
                    if workers.contains_key(&worker_id) {
                        error!("[{}] Failed to add worker because the UUID is existed.", worker_id);
                    } else {
                        workers.insert(worker_id.clone(), worker_context);
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
                            // TODO: need anything?
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
                        if worker.is_sync_only() {
                            trace!("[{}] worker or pool is sync only, skipping get egress messages.", worker.uuid);
                        } else if !worker.is_reached_para_chaintip(self.parachain_chaintip) {
                            trace!("[{}] worker parachain is not at chaintip, skipping get egress messages.", worker.uuid);
                        } else {
                            self.add_pruntime_request(worker, PRuntimeRequest::GetEgressMessages);
                        }
                    }
                },
                ProcessorEvent::BroadcastSync((request, info)) => {
                    for worker in workers.values_mut() {
                        if worker.accept_sync_request && worker.is_match(&request.manifest) {
                            info!("[{}] Accepted BroadcastSyncRequest", worker.uuid);
                            self.add_pruntime_request(worker, PRuntimeRequest::Sync(request.clone()));
                        }
                    }
                    self.relaychain_chaintip = info.relay_chaintip;
                    self.parachain_chaintip = info.para_chaintip;
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
        match event {
            WorkerEvent::UpdateWorker(updated_worker) => {
                if worker.worker_status.worker.endpoint != updated_worker.endpoint {
                    worker.client = Arc::new(crate::pruntime::create_client(updated_worker.endpoint.clone()));
                }
                worker.worker_status.worker = updated_worker;
            },
            WorkerEvent::PRuntimeRequest(request) => {
                self.add_pruntime_request(worker, request);
            },
            WorkerEvent::PRuntimeResponse(result) => {
                worker.calling = false;

                match result {
                    Ok(response) => self.handle_pruntime_response(worker, response),
                    Err(err) => {
                        error!("[{}] met error: {}", worker.uuid, err);
                        self.update_worker_last_message(
                            worker,
                            &err.to_string(),
                            None,
                        );
                    },
                }

                if let Some(request) = worker.pending_requests.pop_front() {
                    self.add_pruntime_request(worker, request);
                }
            },
            WorkerEvent::UpdateSessionInfo(session_info) => {
                worker.worker_status.session_info = Some(session_info);
            },
            WorkerEvent::WorkerLifecycleCommand(command) => {
                self.handle_worker_lifecycle_command(worker, command);
            },
            WorkerEvent::UpdateMessage((timestamp, message)) => {
                self.update_worker_last_message(
                    worker,
                    &message,
                    Some(timestamp),
                );
            },
            WorkerEvent::MarkError((timestamp, error_msg)) => {
                self.update_worker_lifecycle_state(
                    worker,
                    WorkerLifecycleState::HasError(error_msg.clone()),
                    &error_msg,
                    Some(timestamp),
                );
            },
        }

    }

    fn update_worker_lifecycle_state(
        &mut self,
        worker: &mut WorkerContext,
        state: WorkerLifecycleState,
        message: &str,
        updated_at: Option<DateTime<Utc>>,
    ) {
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

    fn update_worker_last_message(
        &mut self,
        worker: &mut WorkerContext,
        message: &str,
        updated_at: Option<DateTime<Utc>>,
    ) {
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
        //trace!("[{}] adding a new pruntime request: {:?}", worker.uuid, request);
        if let PRuntimeRequest::Sync(sync_request) = &request {
            assert!(
                worker.accept_sync_request,
                "[{}] worker does not accept sync request but received one",
                worker.uuid,
            );
            if sync_request.headers.is_none()
                && sync_request.para_headers.is_none()
                && sync_request.combined_headers.is_none()
                && sync_request.blocks.is_none()
                && (worker.blocknum < worker.para_headernum && worker.headernum <= self.relaychain_chaintip || worker.para_headernum <= self.parachain_chaintip)
            {
                warn!("[{}] Worker needs to be sync, but received an empty request. Try again.", worker.uuid);
                self.request_next_sync(worker);
                return;
            }
            worker.accept_sync_request = false;
        }

        if worker.pending_requests.is_empty() {
            self.handle_pruntime_request(worker, request);
        } else {
            worker.pending_requests.push_back(request);
        }
    }

    fn handle_pruntime_request(
        &mut self,
        worker: &mut WorkerContext,
        request: PRuntimeRequest,
    ) {
        worker.calling = true;
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
        match response {
            PRuntimeResponse::PrepareLifecycle(info) => {
                //info!("[{}] PRuntimeResponse, getInfo", worker.uuid);
                worker.worker_status.phactory_info = Some(info.clone());
                worker.headernum = info.headernum;
                worker.para_headernum = info.para_headernum;
                worker.blocknum = info.blocknum;

                self.request_prepare_determination(worker);
                self.send_worker_status(worker);
            },
            PRuntimeResponse::InitRuntime(_response) => {
                self.add_pruntime_request(worker, PRuntimeRequest::PrepareLifecycle);
            },
            PRuntimeResponse::LoadChainState => {
                self.add_pruntime_request(worker, PRuntimeRequest::PrepareLifecycle);
            },
            PRuntimeResponse::Sync(info) => {
                //info!("[{}] PRuntimeResponse, sync", worker.uuid);
                worker.accept_sync_request = true;
                self.handle_pruntime_sync_response(worker, &info);
                self.send_worker_sync_info(worker);
            },
            PRuntimeResponse::RegularGetInfo(phactory_info) => {
                worker.worker_status.phactory_info = Some(phactory_info);
            },
            PRuntimeResponse::PrepareRegister(response) => {
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
            PRuntimeResponse::TakeCheckpoint(_) => todo!(),
        }
    }

    fn handle_pruntime_sync_response(
        &mut self,
        worker: &mut WorkerContext,
        info: &SyncInfo,
    ) {
        if let Some(headernum) = info.headernum {
            worker.headernum = headernum + 1;
            trace!("[{}] headernum updated, next: {}", worker.uuid, worker.headernum);
        }
        if let Some(para_headernum) = info.para_headernum {
            worker.para_headernum = para_headernum + 1;
            trace!("[{}] para_headernum updated, next: {}", worker.uuid, worker.para_headernum);
        }
        if let Some(blocknum) = info.blocknum {
            worker.blocknum = blocknum + 1;
            trace!("[{}] blocknum updated, next: {}", worker.uuid, worker.blocknum);
        }

        if !worker.is_reached_chaintip(self.relaychain_chaintip, self.parachain_chaintip) {
            self.request_next_sync(worker);
        } else {
            self.request_computation_determination(worker);
        }
    }

    fn request_prepare_determination(
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
        worker.accept_sync_request = true;
        self.request_next_sync(worker);
        self.update_worker_lifecycle_state(
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
        if !worker.is_registered() {
            if worker.register_requested {
                trace!("[{}] register has been requested, waiting for register result", worker.uuid);
            } else {
                self.add_pruntime_request(
                    worker,
                    PRuntimeRequest::PrepareRegister((true, worker.operator.clone()))
                );
                worker.register_requested = true;
            }
            return;
        }

        let public_key = match worker.public_key() {
            Some(public_key) => public_key,
            None => {
                trace!("[{}] no public key found. Skip continuing computation determination.", worker.uuid);
                return
            },
        };

        match &worker.worker_info {
            Some(worker_info) => {
                if worker_info.initial_score.is_none() {
                    trace!("[{}] no initial_score yet. Skip continuing computation determination.", worker.uuid);
                    return;
                }
            },
            None => {
                trace!("[{}] no worker_info found yet. Skip continuing computation determination.", worker.uuid);
                return;
            },
        };

        if worker.session_id.is_none() {
            if worker.add_to_pool_requested {
                trace!("[{}] already requested add to pool, waiting for session_id", worker.uuid);
            } else {
                trace!("[{}] requesting add to pool", worker.uuid);
                tokio::spawn(do_add_worker_to_pool(
                    self.bus.clone(),
                    self.txm.clone(),
                    worker.uuid.clone(),
                    worker.pool_id.clone(),
                    public_key.clone(),
                ));
                worker.add_to_pool_requested = true;
            }
            return;
        }

        if !worker.is_computing() {
            if worker.computing_requested {
                trace!("[{}] already requested start computing, waiting...", worker.uuid);
            } else {
                trace!("[{}] requesting start computing", worker.uuid);
                tokio::spawn(do_start_computing(
                    self.bus.clone(),
                    self.txm.clone(),
                    worker.uuid.clone(),
                    worker.pool_id.clone(),
                    public_key.clone(),
                    worker.worker_status.worker.stake.clone()
                ));
                worker.computing_requested = true;
            }
        }
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
            self.update_worker_lifecycle_state(
                worker,
                WorkerLifecycleState::HasError(err_msg.to_string()),
                &err_msg,
                None,
            );
            return;
        };

        self.update_worker_last_message(worker, "Initializing pRuntime...", None);
        self.add_pruntime_request(worker, PRuntimeRequest::InitRuntime(request));
    }

    fn request_fast_sync(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        let _ = self.bus.send_repository_event(
            RepositoryEvent::GenerateFastSyncRequest((worker.uuid.clone(), worker.public_key().unwrap()))
        );
        self.update_worker_last_message(worker, "Trying to load chain state...", None);
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
            },
            WorkerLifecycleCommand::ShouldForceRegister => {
            },
            WorkerLifecycleCommand::ShouldUpdateEndpoint(endpoints) => {
                self.add_pruntime_request(worker, PRuntimeRequest::SignEndpoints(endpoints));
            },
        }
    }
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
                    chrono::Utc::now(),
                    err.to_string(),
                ))
            );
            return;
        },
    };

    let result = txm.register_worker(pool_id, response.encoded_runtime_info, attestation, v2).await;
    match result {
        Ok(_) => {
            info!("[{}] Worker registered successfully.", worker_id);
            let _ = bus.send_worker_event(
                worker_id.clone(),
                WorkerEvent::UpdateMessage((
                    chrono::Utc::now(),
                    "Registered".to_string(),
                ))
            );
            let _ = bus.send_pruntime_request(worker_id.clone(), PRuntimeRequest::RegularGetInfo);
        },
        Err(err) => {
            error!("[{}] Worker registered failed. {}", worker_id, err);
            let _ = bus.send_worker_event(
                worker_id,
                WorkerEvent::MarkError((
                    chrono::Utc::now(),
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
                    chrono::Utc::now(),
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
                    chrono::Utc::now(),
                    err_msg,
                ))
            );
        },
    }
}

async fn dispatch_pruntime_request(
    bus: Arc<Bus>,
    worker_id: String,
    client: Arc<PRuntimeClient>,
    request: PRuntimeRequest,
) {
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
        PRuntimeRequest::PrepareRegister((force_refresh_ra, operator)) => {
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
        PRuntimeRequest::TakeCheckpoint => todo!(),
    };

    let _ = bus.send_processor_event(ProcessorEvent::WorkerEvent((worker_id, WorkerEvent::PRuntimeResponse(result))));
}
