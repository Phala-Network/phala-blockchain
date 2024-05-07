use crate::api::WorkerStatus;
use crate::bus::Bus;
use crate::compute_management::*;
use crate::datasource::DataSourceManager;
use crate::mainline::{RepoRequest, MainlineContext};
use crate::repository::{do_request_headers, do_request_para_headers, do_request_blocks, do_request_chain_state, ChaintipInfo, SyncRequest, SyncRequestManifest, WorkerSyncInfo};
use crate::messages::MessagesEvent;
use crate::pool_operator::DB;
use crate::pruntime::PRuntimeClient;
use crate::tx::TxManager;
use crate::{use_parachain_api, use_relaychain_api};
use crate::worker::{WorkerLifecycleCommand, WorkerLifecycleState};
use crate::worker_status::WorkerStatusUpdate;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use derive_more::Display;
use log::{debug, error, info, trace, warn};
use parity_scale_codec::Encode;
use phactory_api::prpc::{
    self, Blocks, ChainState, CombinedHeadersToSync, GetEgressMessagesResponse, GetEndpointResponse, GetRuntimeInfoRequest, HeadersSyncedTo, HeadersToSync, InitRuntimeRequest, InitRuntimeResponse, ParaHeadersToSync, PhactoryInfo, SignEndpointsRequest, SyncedTo
};
use phala_pallets::pallet_computation::{SessionInfo, WorkerState};
use phala_pallets::registry::WorkerInfoV2;
use phala_types::messaging::MessageOrigin;
use sp_core::crypto::{AccountId32, ByteArray};
use sp_core::sr25519::Public as Sr25519Public;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, mpsc};
use std::time::Instant;

#[allow(deprecated)]
const UPDATE_PHACTORY_INFO_INTERVAL: Duration = Duration::seconds(5);
#[allow(deprecated)]
const RESTART_WORKER_COOL_PERIOD: Duration = Duration::seconds(15);

#[derive(Default)]
enum RepoStatus {
    #[default]
    Pending,
    Sent,
    PartialReceived,
    Received,
}

#[derive(Default)]
struct RepoUnitStatus {
    worker_ids: HashSet<String>,
    status: RepoStatus,
}

#[derive(Default)]
struct Repo {
    chain_state_requests: HashMap<u32, RepoUnitStatus>,
    headers_requests: HashMap<u32, RepoUnitStatus>,
    para_headers_requests: HashMap<(u32, u32), RepoUnitStatus>,
    blocks_requests: HashMap<(u32, u32), RepoUnitStatus>,
    has_pending_request: bool,
}

impl Repo {
    pub fn add_request(
        &mut self,
        worker_id: String,
        request: RepoRequest,
    ) {
        match request {
            RepoRequest::ChainState(at) => todo!(),
            RepoRequest::Headers(from) => todo!(),
            RepoRequest::ParaHeaders((from, to)) => todo!(),
            RepoRequest::Blocks((from, to)) => todo!(),
        }
        self.has_pending_request = true;
    }

    pub fn get_all_requests(
        &mut self,
    ) -> Vec<RepoRequest> {
        let mut requests = vec![];
        if !self.has_pending_request {
            return requests;
        }

        for (key, value) in self.headers_requests.iter_mut() {
            if matches!(value.status, RepoStatus::Pending) {
                requests.push(RepoRequest::Headers(key.clone()));
                value.status = RepoStatus::Sent;
            }
        }
        for (key, value) in self.para_headers_requests.iter_mut() {
            if matches!(value.status, RepoStatus::Pending) {
                requests.push(RepoRequest::ParaHeaders(key.clone()));
                value.status = RepoStatus::Sent;
            }
        }
        for (key, value) in self.blocks_requests.iter_mut() {
            if matches!(value.status, RepoStatus::Pending) {
                requests.push(RepoRequest::Blocks(key.clone()));
                value.status = RepoStatus::Sent;
            }
        }
        for (key, value) in self.chain_state_requests.iter_mut() {
            if matches!(value.status, RepoStatus::Pending) {
                requests.push(RepoRequest::ChainState(key.clone()));
                value.status = RepoStatus::Sent;
            }
        }
        self.has_pending_request = false;
        return requests;
    }

    pub fn put_received_chain_state(
        &mut self,
        at: u32,
    ) -> Vec<String> {
        todo!()
    }

    pub fn put_received_headers(
        &mut self,
        from: u32,
    ) -> Vec<String> {
        todo!()
    }

    pub fn put_received_para_headers(
        &mut self,
        para_from: u32,
        relay_num: u32,
    ) -> Vec<String> {
        todo!()
    }

    pub fn put_received_blocks(
        &mut self,
        from: u32,
        to: u32,
    ) -> Vec<String> {
        todo!()
    }
}

pub struct WorkerContext {
    pub uuid: String,

    pub pool_id: u64,
    pub operator: Option<AccountId32>,
    pub worker_sync_only: bool,
    pub pool_sync_only: bool,

    pub mainline_context: Option<MainlineContext>,

    pub headernum: u32,
    pub para_headernum: u32,
    pub blocknum: u32,
    pub pending_broadcast: bool,

    pub worker_status: WorkerStatus,
    pub worker_info: Option<WorkerInfoV2<AccountId32>>,
    pub session_id: Option<AccountId32>,

    pub last_message: String,
    pub last_updated_at: DateTime<Utc>,

    pub client: Arc<PRuntimeClient>,
    pub current_pruntime_request: Option<Arc<PRuntimeRequest>>,

    pub register_request: Option<bool>,
    pub sign_endpoints_request: Option<Vec<String>>,
    pub checkpoint_request: Option<()>,

    //pub pending_requests: VecDeque<PRuntimeRequest>,
    //pub pruntime_recent_error_count: usize,
    //pub last_worker_lifecycle: Option<WorkerLifecycleState>,

    //pub phactory_info_requested: bool,
    // pub phactory_info_requested_at: DateTime<Utc>,

    // pub stopped: bool,

    pub compute_management_context: Option<ComputeManagementContext>,
    pub update_session_id_count: usize,
    pub update_session_info_count: usize,
}

impl WorkerContext {
    pub fn create(
        worker: crate::inv_db::Worker,
        pool_sync_only: Option<bool>,
        operator: Option<AccountId32>,
        pruntime_client: PRuntimeClient,
    ) -> Self {
        Self {
            uuid: worker.id.clone(),

            pool_id: worker.pid.unwrap_or_default(),
            operator,
            worker_sync_only: worker.sync_only,
            pool_sync_only: pool_sync_only.unwrap_or(true),
            
            mainline_context: None,

            headernum: 0,
            para_headernum: 0,
            blocknum: 0,
            pending_broadcast: false,

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

            client: Arc::new(pruntime_client),
            current_pruntime_request: None,

            register_request: None,
            sign_endpoints_request: None,
            checkpoint_request: None,

            // stopped: false,

            compute_management_context: None,
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
            .and_then(|info| info.public_key.clone())
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

    // pub fn is_match(&self, manifest: &SyncRequestManifest) -> bool {
    //     if let Some((from, _)) = manifest.headers {
    //         if self.headernum != from {
    //             return false;
    //         }
    //     }
    //     if let Some((from, _)) = manifest.para_headers {
    //         if self.para_headernum != from {
    //             return false;
    //         }
    //     }
    //     if let Some((from, _)) = manifest.blocks {
    //         if self.blocknum != from {
    //             return false;
    //         }
    //     }
    //     true
    // }

    // pub fn is_reached_chaintip(
    //     &self,
    //     chaintip: &ChaintipInfo,
    // ) -> bool {
    //     self.blocknum == self.para_headernum
    //         && self.headernum == chaintip.relaychain + 1
    //         && self.para_headernum == chaintip.parachain + 1
    // }

    // pub fn is_reached_para_chaintip(
    //     &self,
    //     chaintip: &ChaintipInfo,
    // ) -> bool {
    //     self.blocknum == self.para_headernum
    //         &&self.para_headernum == chaintip.parachain + 1
    // }

    // pub fn is_sync_only(&self) -> bool {
    //     self.worker_sync_only || self.pool_sync_only
    // }

    // pub fn is_updating_phactory_info_due(&self) -> bool {
    //     !self.phactory_info_requested
    //         && Utc::now().signed_duration_since(self.phactory_info_requested_at) >= UPDATE_PHACTORY_INFO_INTERVAL
    // }
}

#[derive(Default)]
pub struct SyncInfo {
    pub headernum: Option<u32>,
    pub para_headernum: Option<u32>,
    pub blocknum: Option<u32>,
}

#[derive(Display)]
pub enum PRuntimeRequest {
    #[display(fmt = "GetInfo")]
    GetInfo,
    #[display(fmt = "InitRuntime")]
    InitRuntime(InitRuntimeRequest),
    #[display(fmt = "LoadChainState")]
    LoadChainState(ChainState),
    #[display(fmt = "SyncHeaders")]
    SyncHeaders(HeadersToSync),
    #[display(fmt = "SyncParaHeaders")]
    SyncParaHeaders(ParaHeadersToSync),
    #[display(fmt = "SyncCombinedHeaders")]
    SyncCombinedHeaders(CombinedHeadersToSync),
    #[display(fmt = "SyncBlocks")]
    SyncBlocks(Blocks),
    #[display(fmt = "GetEgressMessages")]
    GetEgressMessages,
    #[display(fmt = "GetRuntimeInfo")]
    GetRuntimeInfo(GetRuntimeInfoRequest),

    #[display(fmt = "SignEndpoints")]
    SignEndpoints(SignEndpointsRequest),
    #[display(fmt = "TakeCheckpoint")]
    TakeCheckpoint,
}

#[derive(Display)]
pub enum PRuntimeResponse {
    #[display(fmt = "GetInfo")]
    GetInfo(PhactoryInfo),
    #[display(fmt = "InitRuntime")]
    InitRuntime(InitRuntimeResponse),
    #[display(fmt = "LoadChainState")]
    LoadChainState,
    #[display(fmt = "SyncHeaders")]
    SyncHeaders(SyncedTo),
    #[display(fmt = "SyncParaHeaders")]
    SyncParaHeaders(SyncedTo),
    #[display(fmt = "SyncCombinedHeaders")]
    SyncCombinedHeaders(HeadersSyncedTo),
    #[display(fmt = "SyncBlocks")]
    SyncBlocks(SyncedTo),
    #[display(fmt = "GetEgressMessages")]
    GetEgressMessages(GetEgressMessagesResponse),
    #[display(fmt = "GetRuntimeInfo")]
    GetRuntimeInfo(InitRuntimeResponse),

    #[display(fmt = "SignEndpoints")]
    SignEndpoints(GetEndpointResponse),
    #[display(fmt = "TakeCheckpoint")]
    TakeCheckpoint(u32),
}

#[derive(Display)]
pub enum WorkerEvent {
    #[display(fmt = "UpdateWorker")]
    UpdateWorker(crate::inv_db::Worker),
    // #[display(fmt = "PRuntimeRequest::{}", "_0")]
    // PRuntimeRequest(PRuntimeRequest),
    #[display(fmt = "PRuntimeResponse")]
    PRuntimeResponse(Result<PRuntimeResponse, prpc::client::Error>),
    #[display(fmt = "UpdateWorkerInfo")]
    UpdateWorkerInfo(WorkerInfoV2<AccountId32>),
    #[display(fmt = "UpdateSessionId")]
    UpdateSessionId(Option<AccountId32>),
    #[display(fmt = "UpdateSessionInfo")]
    UpdateSessionInfo(Option<SessionInfo>),
    #[display(fmt = "WorkerLifecycleCommand")]
    WorkerLifecycleCommand(WorkerLifecycleCommand),
    #[display(fmt = "UpdateMessage")]
    UpdateMessage((DateTime<Utc>, String)),
    #[display(fmt = "MarkError")]
    MarkError((DateTime<Utc>, String)),
}

#[derive(Display)]
pub enum ProcessorEvent {
    #[display(fmt = "AddWorker({})", "_0.0.id")]
    AddWorker((crate::inv_db::Worker, Option<bool>, Option<AccountId32>, PRuntimeClient)),
    #[display(fmt = "DeleteWorker({})", "_0")]
    DeleteWorker(String),
    #[display(fmt = "UpdatePool({})", "_0.0")]
    UpdatePool((u64, Option<crate::inv_db::Pool>)),
    #[display(fmt = "UpdatePoolOperator({})", "_0.0")]
    UpdatePoolOperator((u64, Option<AccountId32>)),
    #[display(fmt = "WorkerEvent({}, {})", "_0.0", "_0.1")]
    WorkerEvent((String, WorkerEvent)),
    #[display(fmt = "Heartbeat")]
    Heartbeat,
    #[display(fmt = "BroadcastSync")]
    BroadcastSync((SyncRequest, ChaintipInfo)),
    #[display(fmt = "RequestUpdateSessionInfo")]
    RequestUpdateSessionInfo,

    #[display(fmt = "ReceivedChainState")]
    ReceivedChainState((u32, Result<Arc<Vec<(Vec<u8>, Vec<u8>)>>>)),
    #[display(fmt = "ReceivedHeaders")]
    ReceivedHeaders((u32, Arc<Vec<phactory_api::blocks::HeaderToSync>>)),
    #[display(fmt = "ReceivedParaHeaders")]
    ReceivedParaHeaders((u32, u32, Result<(Arc<Vec<phactory_api::blocks::BlockHeader>>, Arc<phactory_api::blocks::StorageProof>)>)),
    #[display(fmt = "ReceivedBlocks")]
    ReceivedBlocks((u32, u32, Result<Vec<Arc<phactory_api::blocks::BlockHeaderWithChanges>>>)),
}

pub type ProcessorRx = mpsc::Receiver<ProcessorEvent>;
pub type ProcessorTx = mpsc::Sender<ProcessorEvent>;

type Storage = phala_trie_storage::TrieStorage<phactory_api::blocks::RuntimeHasher>;

pub struct Processor {
    pub rx: ProcessorRx,

    pub bus: Arc<Bus>,
    pub dsm: Arc<DataSourceManager>,
    pub txm: Arc<TxManager>,
    pub headers_db: Arc<DB>,

    pub allow_fast_sync: bool,
    pub pccs_url: String,
    pub pccs_timeout_secs: u64,

    pub init_runtime_request_ias: InitRuntimeRequest,
    pub init_runtime_request_dcap: InitRuntimeRequest,

    pub chaintip: ChaintipInfo,
    pub storage: Storage,
    pub repo: Repo,
}

impl Processor {
    pub async fn create(
        rx: ProcessorRx,
        bus: Arc<Bus>,
        txm: Arc<TxManager>,
        headers_db: Arc<DB>,
        dsm: Arc<crate::datasource::DataSourceManager>,
        args: &crate::cli::WorkerManagerCliArgs,
    ) -> Self {
        let ias_init_runtime_request = dsm.clone().get_init_runtime_default_request(Some(phala_types::AttestationProvider::Ias)).await.unwrap();
        let dcap_init_runtime_request = dsm.clone().get_init_runtime_default_request(Some(phala_types::AttestationProvider::Dcap)).await.unwrap();

        let storage = Storage::default();

        Self {
            rx,

            bus,
            dsm: dsm.clone(),
            txm,
            headers_db,

            allow_fast_sync: !args.disable_fast_sync,
            pccs_url: args.pccs_url.clone(),
            pccs_timeout_secs: args.pccs_timeout.clone(),

            init_runtime_request_ias: ias_init_runtime_request,
            init_runtime_request_dcap: dcap_init_runtime_request,

            chaintip: ChaintipInfo {
                relaychain: use_relaychain_api!(dsm, false).unwrap().latest_finalized_block_number().await.unwrap(),
                parachain: use_parachain_api!(dsm, false).unwrap().latest_finalized_block_number().await.unwrap(),
            },
            storage,
            repo: Repo::default(),
        }
    }

    pub fn master_loop(&mut self) {
        let _ = thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Max);

        let mut workers = HashMap::<String, WorkerContext>::new();

        loop {
            let event = match self.rx.recv() {
                Ok(event) => event,
                Err(err) => break,
            };

            let start_time = Instant::now();
            let event_display = format!("{event}");

            let mut updated_workers = HashSet::<String>::new();

            match event {
                ProcessorEvent::AddWorker((added_worker, pool_sync_only, operator, pruntime_client)) => {
                    let worker_id = added_worker.id.clone();
                    let worker_context = WorkerContext::create(added_worker, pool_sync_only, operator, pruntime_client);
                    if workers.contains_key(&worker_id) {
                        error!("[{}] Failed to add worker because the UUID is existed.", worker_id);
                    } else {
                        workers.insert(worker_id.clone(), worker_context);
                        let _ = updated_workers.insert(worker_id.clone());
                        let worker_context = workers.get_mut(&worker_id).unwrap();

                        trace!("[{}] Added worker into processor. Starting", worker_id);
                        self.send_worker_status(worker_context);
                        self.execute_pruntime_request(worker_context, PRuntimeRequest::GetInfo);
                    }
                },
                ProcessorEvent::DeleteWorker(worker_id) => {
                    match workers.remove(&worker_id) {
                        Some(removed_worker) => {
                            if let Some(public_key) = removed_worker.public_key() {
                                trace!("[{}] Requesting remove MessageOrigin::Worker({})", worker_id, public_key);
                                let _ = self.bus.send_messages_event(
                                    MessagesEvent::RemoveSender(MessageOrigin::Worker(public_key))
                                );
                            }
                            let _ = self.bus.send_worker_status_event((
                                removed_worker.uuid.clone(),
                                WorkerStatusUpdate::Delete,
                            ));
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
                            // TODO; register
                            // self.add_pruntime_request(
                            //     worker,
                            //     PRuntimeRequest::PrepareRegister((
                            //         true,
                            //         worker.operator.clone(),
                            //         false,
                            //     ))
                            // );
                        }
                    }
                },
                ProcessorEvent::WorkerEvent((worker_id, worker_event)) => {
                    match workers.get_mut(&worker_id) {
                        Some(worker_context) => {
                            self.handle_worker_event(worker_context, worker_event);
                            let _ = updated_workers.insert(worker_id.clone());
                        },
                        None => {
                            warn!("[{}] Worker does not found.", worker_id);
                        },
                    }
                },
                ProcessorEvent::Heartbeat => {
                    // for worker in workers.values_mut() {
                    //     if worker.is_updating_phactory_info_due() {
                    //         worker.phactory_info_requested = true;
                    //         worker.phactory_info_requested_at = Utc::now();
                    //         self.add_pruntime_request(worker, PRuntimeRequest::RegularGetInfo);
                    //     }
                    // }
                },
                ProcessorEvent::BroadcastSync((request, info)) => {
                    // for worker in workers.values_mut() {
                    //     if !worker.pending_broadcast {
                    //         continue;
                    //     }

                    //     if worker.is_reached_chaintip(&self.chaintip) {
                    //         if worker.is_match(&request.manifest) {
                    //             worker.pending_broadcast = false;
                    //             trace!("[{}] Accepted BroadcastSyncRequest", worker.uuid);
                    //             self.add_pruntime_request(worker, PRuntimeRequest::Sync(request.clone()));
                    //         }
                    //     } else {
                    //         worker.pending_broadcast = false;
                    //         trace!("[{}] Not at chaintip but pending for broadcase. Need to re-trigger sync.", worker.uuid);
                    //         self.request_next_sync(worker);
                    //     }
                    // }
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
                        if worker.blocknum.saturating_add(8) < self.chaintip.parachain {
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
                ProcessorEvent::ReceivedChainState((number, result)) => {
                    let worker_ids = self.repo.put_received_chain_state(number);
                    for worker_id in worker_ids {
                        let Some(worker) = workers.get_mut(&worker_id) else {
                            continue;
                        };
                        let Some(ref mut mainline_context) = worker.mainline_context else {
                            continue;
                        };
                        match &result {
                            Ok(state) => {
                                mainline_context.buffer_chain_state(number, state.clone());
                            },
                            Err(err) => todo!(),
                        }
                        let _ = updated_workers.insert(worker_id);
                    }
                },
                ProcessorEvent::ReceivedHeaders((from, headers)) => {
                    let worker_ids = self.repo.put_received_headers(from);
                    for worker_id in worker_ids {
                        let Some(worker) = workers.get_mut(&worker_id) else {
                            continue;
                        };
                        let Some(ref mut mainline_context) = worker.mainline_context else {
                            continue;
                        };
                        mainline_context.buffer_headers(headers.clone());
                        let _ = updated_workers.insert(worker_id);
                    }
                },
                ProcessorEvent::ReceivedParaHeaders((para_from, relay_num, result)) => {
                    let worker_ids = self.repo.put_received_para_headers(para_from, relay_num);
                    for worker_id in worker_ids {
                        let Some(worker) = workers.get_mut(&worker_id) else {
                            continue;
                        };
                        let Some(ref mut mainline_context) = worker.mainline_context else {
                            continue;
                        };
                        match &result {
                            Ok((para_headers, proof)) => {
                                mainline_context.buffer_para_headers(para_headers.clone(), proof.clone());
                            },
                            Err(err) => todo!(),
                        }
                        let _ = updated_workers.insert(worker_id);
                    }
                },
                ProcessorEvent::ReceivedBlocks((from, to, result)) => {
                    let worker_ids = self.repo.put_received_blocks(from, to);
                    for worker_id in worker_ids {
                        let Some(worker) = workers.get_mut(&worker_id) else {
                            continue;
                        };
                        let Some(ref mut mainline_context) = worker.mainline_context else {
                            continue;
                        };
                        match &result {
                            Ok(blocks) => {
                                mainline_context.buffer_blocks(blocks.clone());
                            },
                            Err(err) => todo!(),
                        }
                        let _ = updated_workers.insert(worker_id);
                    }
                },
            }

            for worker_id in updated_workers {
                let Some(worker) = workers.get_mut(&worker_id) else {
                    continue;
                };
                let Some(ref mut mainline_context) = worker.mainline_context else {
                    continue;
                };

                if let Some(request) = mainline_context.next_repo_request() {
                    self.repo.add_request(worker_id, request);
                }

                if worker.current_pruntime_request.is_none() {
                    let pruntime_request = mainline_context.next_pruntime_request();
                    if let Some(request) = pruntime_request {
                        self.execute_pruntime_request(worker, request)
                    }
                }
            }

            let repo_requests = self.repo.get_all_requests();
            for request in repo_requests {
                match request {
                    RepoRequest::ChainState(number) => {
                        tokio::spawn(do_request_chain_state(
                            self.bus.clone(),
                            self.dsm.clone(),
                            number,
                        ));
                    },
                    RepoRequest::Headers(from) => {
                        tokio::spawn(do_request_headers(
                            self.bus.clone(),
                            self.headers_db.clone(),
                            from,
                        ));
                    },
                    RepoRequest::ParaHeaders((para_from, relay_num)) => {
                        tokio::spawn(do_request_para_headers(
                            self.bus.clone(),
                            self.dsm.clone(),
                            para_from,
                            relay_num,
                        ));
                    },
                    RepoRequest::Blocks((from, to)) => {
                        tokio::spawn(do_request_blocks(
                            self.bus.clone(),
                            self.dsm.clone(),
                            from,
                            to
                        ));
                    },
                }
            }

            let cost = start_time.elapsed().as_micros();
            debug!("measuring {event_display} cost {cost} microseconds.");
        }
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
            // WorkerEvent::PRuntimeRequest(request) => {
            //     self.handle_pruntime_request(worker, request);
            // },
            WorkerEvent::PRuntimeResponse(result) => {
                worker.current_pruntime_request = None;
                match result {
                    Ok(response) => {
                        // if worker.pruntime_recent_error_count >= 3 && worker.last_worker_lifecycle.is_some() {
                        //     warn!("{:?}", worker.last_worker_lifecycle);
                        //     match &worker.worker_status.state {
                        //         WorkerLifecycleState::HasError(msg) if msg.starts_with("Continously received") => {
                        //             let msg = format!(
                        //                 "Recovered from RpcError. Previously had error {} times.",
                        //                 worker.pruntime_recent_error_count
                        //             );
                        //             info!("[{}] {}", worker.uuid, msg);
                        //             let last_state = worker.last_worker_lifecycle.take().unwrap();
                        //             self.update_worker_state_and_message(
                        //                 worker,
                        //                 last_state,
                        //                 &msg,
                        //                 None,
                        //             );
                        //         },
                        //         _ => (),
                        //     }
                        // }
                        // worker.pruntime_recent_error_count = 0;
                        // worker.last_worker_lifecycle = None;
                        self.handle_pruntime_response(worker, response);
                    },
                    Err(err) => {
                        // match &err {
                        //     ::prpc::client::Error::DecodeError(_) | ::prpc::client::Error::ServerError(_) => {
                        //         let msg = format!("pRuntime returned an error: {}", err);
                        //         self.update_worker_state(
                        //             worker,
                        //             WorkerLifecycleState::HasError(msg),
                        //         );
                        //     },
                        //     ::prpc::client::Error::RpcError(_) => {
                        //         worker.pruntime_recent_error_count += 1;
                        //         if worker.pruntime_recent_error_count >= 3 {
                        //             match &worker.worker_status.state {
                        //                 WorkerLifecycleState::HasError(msg) if msg.starts_with("Continously received") => (),
                        //                 _ => {
                        //                     worker.last_worker_lifecycle = Some(worker.worker_status.state.clone());
                        //                 }
                        //             }
                        //             error!(
                        //                 "[{}] Continously received {} RpcError from pRuntime, marking HasError",
                        //                 worker.uuid,
                        //                 worker.pruntime_recent_error_count,
                        //             );
                        //             let msg = format!(
                        //                 "Continously received {} RpcError from pRuntime. Last one: {}",
                        //                 worker.pruntime_recent_error_count,
                        //                 err,
                        //             );
                        //             self.update_worker_state(
                        //                 worker,
                        //                 WorkerLifecycleState::HasError(msg),
                        //             );
                        //         }
                        //     },
                        // }
                    },
                }
            },
            WorkerEvent::UpdateWorkerInfo(worker_info) => {
                trace!("[{}] Received UpdateWorkerInfo", worker.uuid);
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

    pub fn update_worker_state_and_message(
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

    pub fn update_worker_message(
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

    pub fn send_worker_status(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        let _ = self.bus.send_worker_status_event((
            worker.uuid.clone(),
            WorkerStatusUpdate::Update(worker.worker_status.clone()),
        ));
    }

    pub fn send_worker_sync_info(
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

    pub fn handle_pruntime_request(
        &mut self,
        worker: &mut WorkerContext,
        request: PRuntimeRequest,
    ) {
        // if worker.stopped {
        //     match &request {
        //         PRuntimeRequest::TakeCheckpoint => {
        //             trace!("[{}] Stop Special: Immediately handle {}", worker.uuid, request);
        //             self.execute_pruntime_request(worker, request);
        //             return;
        //         },
        //         _ => {
        //             info!("[{}] worker was stopped, skip the request.", worker.uuid);
        //             return;
        //         },
        //     }
        // }

        // trace!("[{}] Adding {}", worker.uuid, request);
        // if let PRuntimeRequest::Sync(sync_request) = &request {
        //     if sync_request.is_empty() {
        //         if !worker.is_reached_chaintip(&self.chaintip) && sync_request.is_empty() {
        //             warn!("[{}] Worker needs to be sync, but received an empty request. Try again.", worker.uuid);
        //             self.request_next_sync(worker);
        //         } else {
        //             trace!("[{}] Ignoring the empty sync request.", worker.uuid);
        //         }
        //         return;
        //     } else if !worker.is_match(&sync_request.manifest) {
        //         warn!("[{}] Ignoring not match Syncing: {}", worker.uuid, request);
        //         self.request_next_sync(worker);
        //         return;
        //     }
        // }

        // if !worker.pruntime_lock && worker.pending_requests.is_empty() {
        //     trace!("[{}] Immediately handle {}", worker.uuid, request);
        //     self.execute_pruntime_request(worker, request);
        // } else {
        //     trace!("[{}] Enqueuing {} because: pruntime_lock {}, pendings: {}",
        //         worker.uuid,
        //         request,
        //         worker.pruntime_lock,
        //         worker.pending_requests.len()
        //     );
        //     worker.pending_requests.push_back(request);
        // }
    }

    fn execute_pruntime_request(
        &mut self,
        worker: &mut WorkerContext,
        request: PRuntimeRequest,
    ) {
        if worker.current_pruntime_request.is_some() {
            error!("[{}] executing new pruntime request but current not None", worker.uuid);
            return;
        }

        let request = Arc::new(request);
        worker.current_pruntime_request = Some(request.clone());
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
            PRuntimeResponse::GetInfo(info) => {
                if let Some(ref mut mainline_context) = worker.mainline_context {
                    // TODO: verify, if the incoming info has different numbers, need to restart
                    mainline_context.update_phactory_info(info);
                } else {
                    if !info.initialized {
                        self.init_pruntime(worker);
                    } else {
                        worker.mainline_context = Some(MainlineContext::create(&info));
                    }
                }
            },
            PRuntimeResponse::InitRuntime(_) => {
                self.update_worker_message(worker, "InitRuntime Completed.", None);
                self.execute_pruntime_request(worker, PRuntimeRequest::GetInfo);
            },
            PRuntimeResponse::LoadChainState => {
                self.update_worker_message(worker, "LoadChainState Completed.", None);
                self.execute_pruntime_request(worker, PRuntimeRequest::GetInfo);
            },
            PRuntimeResponse::SyncHeaders(response) => {
                if let Some(ref mut mainline_context) = worker.mainline_context {
                    mainline_context.update_headers_synced_to(response.synced_to);
                }
            },
            PRuntimeResponse::SyncParaHeaders(response) => {
                if let Some(ref mut mainline_context) = worker.mainline_context {
                    mainline_context.update_para_headers_synced_to(response.synced_to);
                }
            },
            PRuntimeResponse::SyncCombinedHeaders(response) => {
                if let Some(ref mut mainline_context) = worker.mainline_context {
                    mainline_context.update_headers_synced_to(response.relaychain_synced_to);
                    mainline_context.update_para_headers_synced_to(response.parachain_synced_to);
                }
            },
            PRuntimeResponse::SyncBlocks(response) => {
                if let Some(ref mut mainline_context) = worker.mainline_context {
                    mainline_context.update_blocks_synced_to(response.synced_to);
                }
            },
            PRuntimeResponse::GetRuntimeInfo(response) => {
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

    // fn handle_pruntime_sync_response(
    //     &mut self,
    //     worker: &mut WorkerContext,
    //     info: &SyncInfo,
    // ) {
    //     if let Some(headernum) = info.headernum {
    //         worker.headernum = headernum + 1;
    //         trace!("[{}] Synced headernum, next: {}", worker.uuid, worker.headernum);
    //     }
    //     if let Some(para_headernum) = info.para_headernum {
    //         worker.para_headernum = para_headernum + 1;
    //         trace!("[{}] Synced para_headernum, next: {}", worker.uuid, worker.para_headernum);
    //     }
    //     if let Some(blocknum) = info.blocknum {
    //         worker.blocknum = blocknum + 1;
    //         trace!("[{}] Synced updated, next: {}", worker.uuid, worker.blocknum);
    //     }

    //     if !worker.is_reached_chaintip(&self.chaintip) {
    //         trace!("[{}] Not at chaintip, requesting next sync", worker.uuid);
    //         self.request_next_sync(worker);
    //     } else {
    //         trace!("[{}] Reached to chaintip!", worker.uuid);
    //         worker.pending_broadcast = true;
    //         if worker.is_registered() && info.blocknum.is_some() {
    //             trace!("[{}] Dispatched a block, requesting EgressMessages", worker.uuid);
    //             self.add_pruntime_request(worker, PRuntimeRequest::GetEgressMessages);
    //         }
    //         if worker.is_compute_management_needed() {
    //             trace!("[{}] Requesting compute management", worker.uuid);
    //             self.request_compute_management(worker);
    //         }
    //     }
    // }

    // fn request_prepare_lifecycle(
    //     &mut self,
    //     worker: &mut WorkerContext,
    // ) {
    //     trace!("[{}] checking worker.phactory_info:", worker.uuid);
    //     if worker.worker_status.phactory_info.is_none() {
    //         self.add_pruntime_request(worker, PRuntimeRequest::PrepareLifecycle);
    //         return;
    //     }

    //     trace!("[{}] checking worker.phactory_info.initialized", worker.uuid);
    //     let info  = worker.worker_status.phactory_info.as_ref().unwrap();
    //     if !info.initialized {
    //         self.request_init(worker);
    //         return;
    //     }

    //     trace!("[{}] checking worker fast sync", worker.uuid);
    //     if self.allow_fast_sync && info.can_load_chain_state && self.chaintip.parachain > 1 {
    //         let _ = self.bus.send_worker_event(
    //             worker.uuid.clone(),
    //             WorkerEvent::RepositoryLoadStateRequest
    //         );
    //         return;
    //     }

    //     if worker.is_reached_chaintip(&self.chaintip) {
    //         worker.pending_broadcast = true;
    //         trace!("[{}] Already at chaintip. Requesting compute management.", worker.uuid);
    //         self.request_compute_management(worker);
    //         self.update_worker_state_and_message(
    //             worker,
    //             WorkerLifecycleState::Preparing,
    //             "Start Preparing...",
    //             None,
    //         );
    //     } else {
    //         trace!("[{}] requesting next sync", worker.uuid);
    //         self.request_next_sync(worker);
    //         self.update_worker_state_and_message(
    //             worker,
    //             WorkerLifecycleState::Synchronizing,
    //             "Start Synchronizing...",
    //             None,
    //         );
    //     }
    // }

    fn init_pruntime(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        let info  = worker.worker_status.phactory_info.as_ref().unwrap();
        // pRuntime versions lower than 2.2.0 always returns an empty list.
        let supported = &info.supported_attestation_methods;
        let mut request = if supported.is_empty() || supported.contains(&"epid".into()) {
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
        request.encoded_operator = worker.operator.as_ref().map(|op| op.encode());
        self.execute_pruntime_request(worker, PRuntimeRequest::InitRuntime(request));
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
            if messages.is_empty() {
                trace!("[{}] Received empty messages for sender {}", worker.uuid, sender);
                continue;
            }
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
                self.update_worker_message(worker, &"Requesting ForceRegister...", None);
                worker.register_request = Some(true);
            },
            WorkerLifecycleCommand::ShouldUpdateEndpoint(endpoints) => {
                self.update_worker_message(worker, &"Requesting UpdateEndpoint...", None);
                worker.sign_endpoints_request = Some(endpoints);
            },
            WorkerLifecycleCommand::ShouldTakeCheckpoint => {
                self.update_worker_message(worker, &"Requesting TakeCheckpoint...", None);
                worker.checkpoint_request = Some(());
            },
        }
    }
}

async fn dispatch_pruntime_request(
    bus: Arc<Bus>,
    worker_id: String,
    client: Arc<PRuntimeClient>,
    request: Arc<PRuntimeRequest>,
) {
    let start_time = Instant::now();
    trace!("[{}] Start to dispatch {}", worker_id, request);

    let result = match &*request {
        PRuntimeRequest::GetInfo => {
            client.get_info(&())
                .await
                .map(|response| PRuntimeResponse::GetInfo(response))
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
        PRuntimeRequest::SyncHeaders(headers) => {
            client.sync_header(headers)
                .await
                .map(|response| PRuntimeResponse::SyncHeaders(response))
        },
        PRuntimeRequest::SyncParaHeaders(para_headers) => {
            client.sync_para_header(para_headers)
                .await
                .map(|response| PRuntimeResponse::SyncParaHeaders(response))
        },
        PRuntimeRequest::SyncCombinedHeaders(combined_headers) => {
            client.sync_combined_headers(combined_headers)
                .await
                .map(|response| PRuntimeResponse::SyncCombinedHeaders(response))
        },
        PRuntimeRequest::SyncBlocks(blocks) => {
            client.dispatch_blocks(blocks)
                .await
                .map(|response| PRuntimeResponse::SyncBlocks(response))
        },
        PRuntimeRequest::GetEgressMessages => {
            client.get_egress_messages(&())
                .await
                .map(|response| {
                    PRuntimeResponse::GetEgressMessages(response)
                })
        },
        PRuntimeRequest::GetRuntimeInfo(request) => {
            client.get_runtime_info(request)
                .await
                .map(|response| PRuntimeResponse::GetRuntimeInfo(response))
        },

        PRuntimeRequest::SignEndpoints(request) => {
            client.sign_endpoint_info(&request)
                .await
                .map(|response| {
                    PRuntimeResponse::SignEndpoints(response)
                })
        },
        PRuntimeRequest::TakeCheckpoint => {
            client.take_checkpoint(&())
                .await
                .map(|response| {
                    PRuntimeResponse::TakeCheckpoint(response.synced_to)
                })
        },
    };

    // if let Err(err) = &result {
    //     let msg = format!("pRuntime returned an error: {}", err);
    //     error!("[{}] {}", worker_id, msg);
    //     if is_critical {
    //         let _ = bus.send_worker_mark_error(worker_id.clone(), msg);
    //     } else {
    //         let _ = bus.send_worker_update_message(worker_id.clone(), msg);
    //     }
    // }
    let _ = bus.send_processor_event(ProcessorEvent::WorkerEvent((worker_id.clone(), WorkerEvent::PRuntimeResponse(result))));
    trace!("[{}] Completed {}. Cost {} microseconds", worker_id, request, start_time.elapsed().as_micros());
}


async fn do_restart(
    bus: Arc<Bus>,
    worker: crate::inv_db::Worker,
    pool_sync_only: bool,
    operator: Option<AccountId32>,

) {
    let worker_id = worker.id.clone();
    let _ = bus.send_processor_event(ProcessorEvent::DeleteWorker(worker_id.clone()));
    info!("[{}] Restarting: Remove WorkerContext command sent, wait {} seconds and then add back",
        worker_id, RESTART_WORKER_COOL_PERIOD.num_seconds());
    tokio::time::sleep(RESTART_WORKER_COOL_PERIOD.to_std().unwrap()).await;
    let client = crate::pruntime::create_client(worker.endpoint.clone());
    let _ = bus.send_processor_event(ProcessorEvent::AddWorker((
        worker,
        Some(pool_sync_only),
        operator,
        client,
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