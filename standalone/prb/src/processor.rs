use log::{debug, error, info};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::pruntime::PRuntimeClient;

//use phactory_api::blocks::{AuthoritySetChange, HeaderToSyn};
use phactory_api::prpc::{self, Blocks, CombinedHeadersToSync, ParaHeadersToSync, HeadersToSync, PhactoryInfo};

enum SyncStatus {
    Idle,
    Syncing,
}

struct WorkerContext {
    pub uuid: String,

    pub headernum: u32,
    pub para_headernum: u32,
    pub blocknum: u32,

    pub calling: bool,
    pub accept_sync_request: bool,
    pub syncing: bool,

    pub client: Arc<PRuntimeClient>,
    pub info: Option<PhactoryInfo>,
    //pub last_message: String,
    //pub session_info: Option<SessionInfo>,
    //pub pending_sequences: HashSet<u32>,
    pub pending_requests: VecDeque<PRuntimeRequest>,
}

#[derive(Clone)]
struct SyncRequest {
    headers: Option<HeadersToSync>,
    para_headers: Option<ParaHeadersToSync>,
    combined_headers: Option<CombinedHeadersToSync>,
    blocks: Option<Blocks>,
}

#[derive(Default)]
struct SyncInfo {
    pub headernum: Option<u32>,
    pub para_headernum: Option<u32>,
    pub blocknum: Option<u32>,
}

enum PRuntimeRequest {
    Sync(SyncRequest),
    GetEgressMessages,
    TakeCheckpoint,
}

enum PRuntimeResponse {
    Sync(SyncInfo),
    GetEgressMessages(),
    TakeCheckpoint(u32),
}

pub enum ProcessorEvent {
    GetEgressMsgTimerReceived(),
    BroadcastSyncRequestReceived((SyncRequest, SyncInfo)),
    PRuntimeRequest((usize, PRuntimeRequest)),
    PRuntimeResponse((usize, Result<PRuntimeResponse, prpc::client::Error>))
}

pub type ProcessorEventRx = mpsc::UnboundedReceiver<ProcessorEvent>;
pub type ProcessorEventTx = mpsc::UnboundedSender<ProcessorEvent>;

pub type ChainDataRequestType = (usize, u32 /* head */, u32 /* parahead */, u32 /* parablock */);
pub type ChainDataRequestTx = mpsc::UnboundedSender<ChainDataRequestType>;

pub struct Processor {
    //workers: Vec<WorkerContext>,
    processor_event_rx: ProcessorEventRx,
    processor_event_tx: Arc<ProcessorEventTx>,
    chain_data_reqeust_tx: ChainDataRequestTx,

    relaychain_chaintip: u32,
    parachain_chaintip: u32,
}

impl Processor {
    pub async fn master_loop(
        &mut self,
        mut workers: Vec<WorkerContext>
    ) {
        loop {
            let event = self.processor_event_rx.recv().await;
            if event.is_none() {
                break;
            }

            match event.unwrap() {
                ProcessorEvent::GetEgressMsgTimerReceived() => {
                    for (worker_id, worker) in workers.iter().enumerate() {
                    }
                },
                ProcessorEvent::BroadcastSyncRequestReceived((request, info)) => {
                    for (worker_id, worker) in workers.iter_mut().enumerate() {
                        if worker.accept_sync_request && is_match(&worker, &info) {
                            self.add_pruntime_request(worker_id, worker, PRuntimeRequest::Sync(request.clone())).await;
                        }
                    }
                },
                ProcessorEvent::PRuntimeRequest((worker_id, request)) => {
                    let worker = workers.get_mut(worker_id).unwrap();
                    self.add_pruntime_request(worker_id, worker, request).await;
                },
                ProcessorEvent::PRuntimeResponse((worker_id, result)) => {
                    let worker = workers.get_mut(worker_id).unwrap();
                    worker.calling = false;

                    match result {
                        Ok(response) => self.handle_pruntime_response(worker_id, worker, response),
                        Err(error) => match error {
                            prpc::client::Error::DecodeError(_) => todo!(),
                            prpc::client::Error::ServerError(_) => todo!(),
                            prpc::client::Error::RpcError(_) => todo!(),
                        },
                    }

                    if let Some(request) = worker.pending_requests.pop_front() {
                        self.add_pruntime_request(worker_id, worker, request).await;
                    }
                },
            }
        }
    }


    async fn add_pruntime_request(
        &mut self,
        worker_id: usize,
        worker: &mut WorkerContext,
        request: PRuntimeRequest,
    ) {
        if let PRuntimeRequest::Sync(_) = request {
            assert!(
                worker.accept_sync_request,
                "worker {} does not accept sync request but received one",
                worker.uuid,
            );
            worker.accept_sync_request = false;
        }

        if worker.pending_requests.is_empty() {
            self.handle_pruntime_request(worker_id, worker, request).await;
        } else {
            worker.pending_requests.push_back(request);
        }
    }

    async fn handle_pruntime_request(
        &mut self,
        worker_id: usize,
        worker: &mut WorkerContext,
        request: PRuntimeRequest,
    ) {
        worker.calling = true;
        tokio::task::spawn(dispatch_pruntime_request(self.processor_event_tx.clone(), worker_id, worker.client.clone(), request));
    }

    fn handle_pruntime_response(
        &mut self,
        worker_id: usize,
        worker: &mut WorkerContext,
        response: PRuntimeResponse,
    ) {
        match response {
            PRuntimeResponse::Sync(info) => {
                worker.accept_sync_request = true;
                self.handle_pruntime_sync_response(worker_id, worker, info);
            },
            PRuntimeResponse::GetEgressMessages() => todo!(),
            PRuntimeResponse::TakeCheckpoint(_) => todo!(),
        }
    }

    fn handle_pruntime_sync_response(
        &mut self,
        worker_id: usize,
        worker: &mut WorkerContext,
        info: SyncInfo,
    ) {
        if let Some(headernum) = &info.headernum {
            worker.headernum = headernum + 1;
        }
        if let Some(para_headernum) = &info.para_headernum {
            worker.para_headernum = para_headernum + 1;
        }
        if let Some(blocknum) = &info.blocknum {
            worker.blocknum = blocknum + 1;
        }

        if worker.headernum <= self.relaychain_chaintip || worker.para_headernum <= self.parachain_chaintip || worker.blocknum <= self.parachain_chaintip {
            self.request_next_sync(worker_id, worker);
        }
    }

    fn request_next_sync(
        &mut self,
        worker_id: usize,
        worker: &WorkerContext,
    ) {
        let send_result = self.chain_data_reqeust_tx.send((
            worker_id,
            worker.headernum,
            worker.para_headernum,
            worker.blocknum,
        ));
        if let Err(send_error) = send_result {
            error!("{:?}", send_error);
            std::process::exit(255);
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
            Err(_) => todo!(),
        }
    }

    if let Some(para_headers) = request.para_headers {
        match client.sync_para_header(para_headers).await {
            Ok(synced_to) => {
                response.para_headernum = Some(synced_to.synced_to);
            },
            Err(_) => todo!(),
        }
    }

    if let Some(combined_headers) = request.combined_headers {
        match client.sync_combined_headers(combined_headers).await {
            Ok(synced_to) => {
                response.headernum = Some(synced_to.relaychain_synced_to);
                response.para_headernum = Some(synced_to.parachain_synced_to);
            },
            Err(_) => todo!(),
        }
    }

    if let Some(blocks) = request.blocks {
        match client.dispatch_blocks(blocks).await {
            Ok(synced_to) => {
                response.blocknum = Some(synced_to.synced_to);
            },
            Err(_) => todo!(),
        }
    }

    Ok(response)
}

async fn dispatch_pruntime_request(
    tx: Arc<ProcessorEventTx>,
    worker_id: usize,
    client: Arc<PRuntimeClient>,
    request: PRuntimeRequest,
) {
    let result = match request {
        PRuntimeRequest::Sync(request) => {
            do_sync_request(client, request)
                .await
                .map(|response| PRuntimeResponse::Sync(response))
        },
        PRuntimeRequest::GetEgressMessages => todo!(),
        PRuntimeRequest::TakeCheckpoint => todo!(),
    };

    send_processor_event(tx, ProcessorEvent::PRuntimeResponse((worker_id, result)));
}

fn send_processor_event(tx: Arc<ProcessorEventTx>, event: ProcessorEvent) {
    let result = tx.send(event);
    if let Err(error) = result {
        error!("{:?}", error);
        std::process::exit(255);
    }
}

fn is_match(worker: &WorkerContext, info: &SyncInfo) -> bool {
    if let Some(headernum) = info.headernum {
        if headernum != worker.headernum {
            return false;
        }
    }
    /*
    if let Some(para_headernum) = info.para_headernum && para_headernum != worker.para_headernum {
        return false;
    }
    if let Some(blocknum) = info.blocknum && blocknum != worker.blocknum {
        return false;
    }
    */
    return true;
}