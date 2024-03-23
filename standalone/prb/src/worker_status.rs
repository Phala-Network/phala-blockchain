use std::sync::Arc;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::processor::SyncInfo;
use crate::worker::WorkerLifecycleState;
use crate::wm::WorkerManagerContext;

use phactory_api::prpc::PhactoryInfo;
use phala_pallets::pallet_computation::SessionInfo;

#[derive(Default)]
pub struct WorkerStatusUpdate {
    pub uuid: String,
    pub state: Option<WorkerLifecycleState>,
    pub phactory_info: Option<PhactoryInfo>,
    pub last_message: Option<String>,
    pub session_info: Option<SessionInfo>,
    pub sync_info: Option<SyncInfo>,
}

pub type WorkerStatusUpdateRx = mpsc::UnboundedReceiver<WorkerStatusUpdate>;
pub type WorkerStatusUpdateTx = mpsc::UnboundedSender<WorkerStatusUpdate>;

pub async fn update_worker_status(
    ctx: Arc<WorkerManagerContext>,
    mut rx: WorkerStatusUpdateRx,
) -> Result<()> {
    loop {
        let update = rx.recv().await;
        if update.is_none() {
            break
        }

        let update = update.unwrap();

        let worker_map = ctx.worker_map.clone();
        let worker_map = worker_map.lock().await;
        let worker = worker_map.get(&update.uuid);
        if let Some(worker) = worker {
            let worker = worker.clone();
            drop(worker_map);
            let mut worker = worker.write().await;
            if let Some(state) = update.state {
                worker.state = state;
            }
            if let Some(phactory_info) = update.phactory_info {
                worker.info = Some(phactory_info);
            }
            if let Some(last_message) = update.last_message {
                worker.last_message = last_message;
            }
            if let Some(session_info) = update.session_info {
                worker.session_info = Some(session_info);
            }
            if let Some(sync_info) = update.sync_info {
                if let Some(headernum) = sync_info.headernum {
                    worker.info.as_mut().map(|info| {
                        info.headernum = headernum + 1;
                    });
                }
                if let Some(para_headernum) = sync_info.para_headernum {
                    worker.info.as_mut().map(|info| {
                        info.para_headernum = para_headernum + 1;
                    });
                }
                if let Some(blocknum) = sync_info.blocknum {
                    worker.info.as_mut().map(|info| {
                        info.blocknum = blocknum + 1;
                    });
                }
            }
            drop(worker);
        }
    }

    Ok(())
}