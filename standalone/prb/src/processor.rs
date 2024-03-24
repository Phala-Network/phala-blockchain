use anyhow::{anyhow, Result};
use log::{debug, error, info, trace, warn};
use sp_core::crypto::{AccountId32, ByteArray};
use sp_core::sr25519::Public as Sr25519Public;
use std::collections::{HashMap, VecDeque};
use std::future::IntoFuture;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::bus::Bus;
use crate::repository::{RepositoryEvent, RepositoryTx, WorkerSyncInfo};
use crate::messages::{MessagesEvent, MessagesTx};
use crate::pruntime::PRuntimeClient;
use crate::tx::TxManager;
use crate::worker::{WorkerLifecycleCommand, WorkerLifecycleState};
use crate::worker_status::{WorkerStatusUpdate, WorkerStatusUpdateTx};

//use phactory_api::blocks::{AuthoritySetChange, HeaderToSyn};
use phactory_api::prpc::{self, Blocks, ChainState, CombinedHeadersToSync, GetEgressMessagesResponse, GetRuntimeInfoRequest, HeadersToSync, InitRuntimeRequest, InitRuntimeResponse, ParaHeadersToSync, PhactoryInfo};
use phala_pallets::pallet_computation::{SessionInfo, WorkerState};

enum WorkerLifecycle {
    Init
}

enum SyncStatus {
    Idle,
    Syncing,
}

pub struct WorkerContext {
    pub uuid: String,
    pub worker: crate::inv_db::Worker,

    pub pool_id: u64,
    pub operator: Option<AccountId32>,
    pub worker_sync_only: bool,
    pub pool_sync_only: bool,

    pub headernum: u32,
    pub para_headernum: u32,
    pub blocknum: u32,

    pub phactory_info: Option<PhactoryInfo>,
    pub session_info: Option<SessionInfo>,

    pub calling: bool,
    pub accept_sync_request: bool,

    pub client: Arc<PRuntimeClient>,
    pub pending_requests: VecDeque<PRuntimeRequest>,

    pub register_requested: bool,
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
            worker: worker.clone(),

            pool_id: worker.pid.unwrap_or_default(),
            operator,
            worker_sync_only: worker.sync_only,
            pool_sync_only: pool.map(|p| p.sync_only).unwrap_or(true),

            headernum: 0,
            para_headernum: 0,
            blocknum: 0,

            phactory_info: None,
            session_info: None,

            calling: false,
            accept_sync_request: false,

            client: Arc::new(crate::pruntime::create_client(worker.endpoint.clone())),
            pending_requests: VecDeque::new(),

            register_requested: false,
            computing_requested: false,
        }
    }

    pub fn public_key(&self) -> Option<Sr25519Public> {
        self.phactory_info
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
        self.phactory_info
            .as_ref()
            .and_then(|info| info.system.as_ref())
            .map(|info| info.registered)
            .unwrap_or(false)
    }

    pub fn is_computing(&self) -> bool {
        let state = self.session_info
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

    pub fn is_match(&self, info: &SyncInfo) -> bool {
        if let Some(headernum) = info.headernum {
            if headernum != self.headernum {
                return false;
            }
        }
        if let Some(para_headernum) = info.para_headernum {
            if para_headernum != self.para_headernum {
                return false;
            }
        }
        if let Some(blocknum) = info.blocknum {
            if blocknum != self.blocknum {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Default)]
pub struct SyncRequest {
    pub headers: Option<HeadersToSync>,
    pub para_headers: Option<ParaHeadersToSync>,
    pub combined_headers: Option<CombinedHeadersToSync>,
    pub blocks: Option<Blocks>,
}

#[derive(Default)]
pub struct SyncInfo {
    pub headernum: Option<u32>,
    pub para_headernum: Option<u32>,
    pub blocknum: Option<u32>,
}

#[derive(Default)]
pub struct BroadcastInfo {
    pub sync_info: SyncInfo,
    pub relay_chaintip: u32,
    pub para_chaintip: u32,
}

pub enum PRuntimeRequest {
    PrepareLifecycle,
    InitRuntime(InitRuntimeRequest),
    LoadChainState(ChainState),
    Sync(SyncRequest),
    RegularGetInfo,
    PrepareRegister((bool, Option<sp_core::crypto::AccountId32>)),
    GetEgressMessages,
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
    TakeCheckpoint(u32),
}

pub enum WorkerEvent {
    UpdateWorker(crate::inv_db::Worker),
    PRuntimeRequest(PRuntimeRequest),
    PRuntimeResponse(Result<PRuntimeResponse, prpc::client::Error>),
    UpdateSessionInfo(SessionInfo),
    WorkerLifecycleCommand(WorkerLifecycleCommand),
    UpdateMessage((i64, String)),
    MarkError((i64, String)),
}

pub enum ProcessorEvent {
    AddWorker((crate::inv_db::Worker, Option<crate::inv_db::Pool>, Option<AccountId32>)),
    DeleteWorker(String),
    UpdatePool((u64, Option<crate::inv_db::Pool>)),
    UpdatePoolOperator((u64, Option<AccountId32>)),
    WorkerEvent((String, WorkerEvent)),
    BroadcastSyncRequest((SyncRequest, BroadcastInfo)),
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
        mut rx: ProcessorRx,
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
        loop {
            let event = self.rx.recv().await;
            if event.is_none() {
                break
            }

            match event.unwrap() {
                ProcessorEvent::BroadcastSyncRequest((request, info)) => {
                    for (worker_id, worker) in workers.iter_mut() {
                        debug!("[{}] Looking to see BroadcastSyncRequest", worker.uuid);
                        if worker.accept_sync_request && worker.is_match(&info.sync_info) {
                            info!("[{}] Meet BroadcastSyncRequest", worker.uuid);
                            self.add_pruntime_request(worker, PRuntimeRequest::Sync(request.clone()));
                        }
                    }
                    self.relaychain_chaintip = info.relay_chaintip;
                    self.parachain_chaintip = info.para_chaintip;
                },
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
                        Some(removed_worker) => {
                        },
                        None => {
                            error!("[{}] Failed to delete worker because the UUID is not existed.", worker_id);
                        },
                    }
                },
                ProcessorEvent::UpdatePool((pool_id, pool)) => {
                    let pool_sync_only = pool.map(|p| p.sync_only).unwrap_or(true);
                    for (worker_id, worker) in workers.iter_mut() {
                        if worker.pool_id == pool_id && worker.pool_sync_only != pool_sync_only {
                            worker.pool_sync_only = pool_sync_only;
                            // TODO: need anything?
                        }
                    }
                },
                ProcessorEvent::UpdatePoolOperator((pool_id, operator)) => {
                    for (worker_id, worker) in workers.iter_mut() {
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
                if worker.worker.endpoint != updated_worker.endpoint {
                    worker.client = Arc::new(crate::pruntime::create_client(updated_worker.endpoint.clone()));
                }
                worker.worker = updated_worker;
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
                worker.session_info = Some(session_info);
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
        updated_at: Option<i64>,
    ) {
    }

    fn update_worker_last_message(
        &mut self,
        worker: &mut WorkerContext,
        message: &str,
        updated_at: Option<i64>,
    ) {
    }

    fn send_worker_status(
        &mut self,
        worker: &mut WorkerContext,
    ) {
    }

    fn send_worker_sync_info(
        &mut self,
        worker: &mut WorkerContext,
    ) {
    }

    fn add_pruntime_request(
        &mut self,
        worker: &mut WorkerContext,
        request: PRuntimeRequest,
    ) {
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
        tokio::task::spawn(
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
                worker.phactory_info = Some(info.clone());
                worker.headernum = info.headernum;
                worker.para_headernum = info.para_headernum;
                worker.blocknum = info.blocknum;

                self.request_prepare_determination(worker);
                self.send_worker_status(worker);
            },
            PRuntimeResponse::InitRuntime(response) => {
                self.add_pruntime_request(worker, PRuntimeRequest::PrepareLifecycle);
            },
            PRuntimeResponse::LoadChainState => {
                self.add_pruntime_request(worker, PRuntimeRequest::PrepareLifecycle);
                self.request_next_sync(worker);
            },
            PRuntimeResponse::Sync(info) => {
                //info!("[{}] PRuntimeResponse, sync", worker.uuid);
                worker.accept_sync_request = true;
                self.handle_pruntime_sync_response(worker, &info);
                self.send_worker_sync_info(worker);
            },
            PRuntimeResponse::RegularGetInfo(phactory_info) => {
                worker.phactory_info = Some(phactory_info);
            },
            PRuntimeResponse::PrepareRegister(response) => {
                let bus = self.bus.clone();
                let txm = self.txm.clone();
                let worker_id = worker.uuid.clone();
                let pool_id = worker.pool_id;
                let pccs_url = self.pccs_url.clone();
                let pccs_timeout_secs = self.pccs_timeout_secs;

                tokio::task::spawn(async move {
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
                            bus.send_worker_event(
                                worker_id,
                                WorkerEvent::MarkError((
                                    chrono::Utc::now().timestamp_millis(),
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
                            bus.send_worker_event(
                                worker_id,
                                WorkerEvent::UpdateMessage((
                                    chrono::Utc::now().timestamp_millis(),
                                    "Registered".to_string(),
                                ))
                            );
                        },
                        Err(err) => {
                            error!("[{}] Worker registered failed. {}", worker_id, err);
                            bus.send_worker_event(
                                worker_id,
                                WorkerEvent::MarkError((
                                    chrono::Utc::now().timestamp_millis(),
                                    err.to_string(),
                                ))
                            );
                            return;
                        },
                    }
                });
            },
            PRuntimeResponse::GetEgressMessages(response) => {
                self.handle_pruntime_egress_messages(worker, response)
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
            debug!("[{}] updated headernum: {}", worker.uuid, worker.headernum);
        }
        if let Some(para_headernum) = info.para_headernum {
            worker.para_headernum = para_headernum + 1;
            debug!("[{}] updated para_headernum: {}", worker.uuid, worker.para_headernum);
        }
        if let Some(blocknum) = info.blocknum {
            worker.blocknum = blocknum + 1;
            debug!("[{}] updated blocknum: {}", worker.uuid, worker.blocknum);
        }

        if worker.headernum <= self.relaychain_chaintip || worker.para_headernum <= self.parachain_chaintip || worker.blocknum < worker.para_headernum {
            self.request_next_sync(worker);
        } else {
            if !worker.register_requested && !worker.is_registered() {
                self.add_pruntime_request(
                    worker,
                    PRuntimeRequest::PrepareRegister((true, worker.operator.clone()))
                );
                worker.register_requested = true;
            }

            let public_key = worker.public_key();
            if public_key.is_some() && !worker.computing_requested && !worker.is_computing() {
                let bus = self.bus.clone();
                let txm = self.txm.clone();
                let worker_id = worker.uuid.clone();
                let pool_id = worker.pool_id;
                let stake = worker.worker.stake.clone();
                tokio::spawn(async move {
                    let result = txm.start_computing(pool_id, public_key.unwrap(), stake).await;
                    if let Err(err) = result {
                        let err_msg = format!("Failed to start computing. {}", err);
                        error!("[{}] {}", worker_id, err_msg);
                        bus.send_worker_mark_error(worker_id, err_msg);
                    }
                });
                worker.computing_requested = true;
            }
        }
    }

    fn request_prepare_determination(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        if worker.phactory_info.is_none() {
            self.add_pruntime_request(worker, PRuntimeRequest::PrepareLifecycle);
            return;
        }

        let info  = worker.phactory_info.as_ref().unwrap();
        if !info.initialized {
            self.request_init(worker);
            return;
        }

        if self.allow_fast_sync && info.can_load_chain_state {
            self.request_fast_sync(worker);
            return;
        }

        self.request_next_sync(worker);
        self.update_worker_lifecycle_state(
            worker,
            WorkerLifecycleState::Synchronizing,
            "Start Synchronizing...",
            None, 
        );
    }

    fn request_init(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        let info  = worker.phactory_info.as_ref().unwrap();
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
        self.bus.send_repository_event(
            RepositoryEvent::GenerateFastSyncRequest((worker.uuid.clone(), worker.public_key().unwrap()))
        );
        self.update_worker_last_message(worker, "Trying to load chain state...", None);
    }

    fn request_next_sync(
        &mut self,
        worker: &WorkerContext,
    ) {
        self.bus.send_repository_event(RepositoryEvent::UpdateWorkerSyncInfo(
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
            let result = self.bus.send_messages_event(
                MessagesEvent::SyncMessages((
                    worker.pool_id,
                    sender,
                    messages,
                ))
            );
            if let Err(err) = result {
                error!("[{}] fail to send offchain messages. {}", worker.uuid, err);
            }
        }
    }

    fn handle_worker_lifecycle_command(
        &mut self,
        worker: &WorkerContext,
        command: WorkerLifecycleCommand,
    ) {
        match command {
            WorkerLifecycleCommand::ShouldRestart => {
            },
            WorkerLifecycleCommand::ShouldForceRegister => todo!(),
            WorkerLifecycleCommand::ShouldUpdateEndpoint(_) => todo!(),
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
        PRuntimeRequest::TakeCheckpoint => todo!(),
    };

    bus.send_processor_event(ProcessorEvent::WorkerEvent((worker_id, WorkerEvent::PRuntimeResponse(result))));
}
