use crate::pruntime::PRuntimeClient;

//use phactory_api::blocks::{AuthoritySetChange, HeaderToSyn};
use phactory_api::prpc::{self, Blocks, CombinedHeadersToSync, ParaHeadersToSync, HeadersToSync, PhactoryInfo};

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex as TokioMutex};

enum SyncStatus {
    Idle,
    Syncing,
}

struct WorkerContext {
    pub uuid: String,

    pub headernum: u32,
    pub para_headernum: u32,
    pub blocknum: u32,

    pub syncing: bool,
    //pub self_ref: Option<WrappedWorkerContext>,
    //pub sm_tx: Option<WorkerLifecycleStateTx>,
    //pub worker: Worker,
    //pub state: WorkerLifecycleState,
    //pub tx: WorkerLifecycleCommandTx,
    //pub rx: Arc<TokioMutex<WorkerLifecycleCommandRx>>,
    //pub ctx: WrappedWorkerManagerContext,
    pub client: PRuntimeClient,
    pub info: Option<PhactoryInfo>,
    //pub last_message: String,
    //pub session_info: Option<SessionInfo>,
    //pub pending_sequences: HashSet<u32>,
    pub waiting_for_event: Option<ProcessorEvent>,
}

struct SyncRequest {
    headers: Option<HeadersToSync>,
    para_headers: Option<ParaHeadersToSync>,
    combined_headers: Option<CombinedHeadersToSync>,
    blocks: Option<Blocks>,
};

struct SyncInfo {
    pub headernum: Option<u32>,
    pub para_headernum: Option<u32>,
    pub blocknum: Option<u32>,
};

pub enum ProcessorEvent {
    BroadcastSyncRequestReceived((SyncRequest, SyncInfo)),
    SyncRequestReceived((usize, Result<SyncRequest>)),
    SyncResponseReceived((usize, Result<SyncInfo, prpc::client::Error>)),
}

pub type ProcessorEventRx = mpsc::UnboundedReceiver<ProcessorEvent>;
pub type ProcessorEventTx = mpsc::UnboundedSender<ProcessorEvent>;

pub async fn master_loop(
    mut workers: Vec<WorkerContext>,
) -> Result<()> {
    let (tx, rx) = mpsc::unbounded_channel::<ProcessorEvent>();

    loop {
        let event = rx.recv().await;
        if event.is_none() {
            break;
        }

        match event.unwrap() {
            ProcessorEvent::BroadcastSyncRequestReceived((request, info)) => {
                for (worker_id, worker) in workers.iter().enumerate() {
                    if !worker.syncing && is_match(&worker, &info) {
                        worker.syncing = true;
                        tokio::task::spawn(handle_sync_request(&tx, worker_id, &worker.client, request.clone()));
                    }
                }
            },
            ProcessorEvent::SyncRequestReceived((worker_id, result)) => {
                match result {
                    Ok(request) => {
                        let worker = workers.get_mut(worker_id).unwrap();

                        assert!(!worker.syncing, "Shouldn't receive any sync_request for a syncing worker");
                        worker.syncing = true;

                        tokio::task::spawn(handle_sync_request(&tx, worker_id, &worker.client, request));
                    },
                    Err(_) => todo!(),
                }
            },
            ProcessorEvent::SyncResponseReceived((worker_id, result)) => {
                let worker = workers.get_mut(worker_id).unwrap();

                assert!(worker.syncing, "Shouldn't receive any sync response for a non-syncing worker");
                worker.syncing = false;

                match result {
                    Ok(info) => {
                        if let Some(headernum) = &info.headernum {
                            worker.headernum = headernum + 1;
                        }
                        if let Some(para_headernum) = &info.para_headernum {
                            worker.para_headernum = para_headernum + 1;
                        }
                        if let Some(blocknum) = &info.blocknum {
                            worker.blocknum = blocknum + 1;
                        }
                    },
                    Err(_) => todo!(),
                }
            },
        }

        for worker in &self.workers {
            let info = worker.info.expect("should have value").clone();

            if info.blocknum < info.para_headernum {

            } else if info.waiting_for_paraheaders { // && get_parachain_header_from_relaychain_at

            } else if info.headernum < relay_chaintip_number {

            } else {
                // chaintip
            }
        }
        drop(worker_contexts);
    }

    Ok(());
}

fn is_match(worker: &WorkerContext, info: &SyncInfo) -> bool {
    if let Some(headernum) = info.headernum && headernum != worker.headernum {
        return false;
    }
    if let Some(para_headernum) = info.para_headernum && para_headernum != worker.para_headernum {
        return false;
    }
    if let Some(blocknum) = info.blocknum && blocknum != worker.blocknum {
        return false;
    }
    return true;
}

async fn do_sync_request(
    client: &PRuntimeClient,
    request: SyncRequest,
) -> Result<SyncInfo, prpc::client::Error> {
    let mut response = SyncInfo { ..Default::default() };

    if let Some(headers) = request.headers {
        match client.sync_header(request).await {
            Ok(sync_to) => {
                response.headernum = sync_to.synced_to;
            },
            Err(_) => todo!(),
        }
    }

    if let Some(para_headers) = request.para_headers {
        match client.sync_para_header(para_headers).await {
            Ok(synced_to) => {
                response.para_headernum = sync_to.synced_to;
            },
            Err(_) => todo!(),
        }
    }

    if let Some(combined_headers) = request.combined_headers {
        match client.sync_combined_headers(combined_headers).await {
            Ok(synced_to) => {
                response.headernum = synced_to.relaychain_synced_to;
                response.para_headernum = synced_to.parachain_synced_to;
            },
            Err(_) => todo!(),
        }
    }

    if let Some(blocks) = request.blocks {
        match client.dispatch_blocks(blocks).await {
            Ok(synced_to) => {
                response.blocknum = synced_to.synced_to;
            },
            Err(_) => todo!(),
        }
    }
}

async fn handle_sync_request(
    tx: &mpsc::UnboundedSender<ProcessorEvent>,
    worker_id: usize,
    client: &PRuntimeClient,
    request: SyncRequest,
) -> Result<()> {
    let result = do_sync_request(client, request).await;
    tx.send(ProcessorEvent::SyncResponseReceived((worker_id, result)));
}