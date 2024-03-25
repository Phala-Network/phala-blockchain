use crate::api::WorkerStatus;
use crate::processor::SyncInfo;
use crate::worker::WorkerLifecycleState;
use crate::wm::WorkerManagerContext;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::mpsc;

pub enum WorkerStatusUpdate {
    Update(WorkerStatus),
    UpdateMessage(String),
    UpdateStateAndMessage((WorkerLifecycleState, String)),
    UpdateSyncInfo(SyncInfo),
    Delete,
}

pub type WorkerStatusEvent = (String, WorkerStatusUpdate);

pub type WorkerStatusRx = mpsc::UnboundedReceiver<WorkerStatusEvent>;
pub type WorkerStatusTx = mpsc::UnboundedSender<WorkerStatusEvent>;

pub async fn update_worker_status(
    ctx: Arc<WorkerManagerContext>,
    mut rx: WorkerStatusRx,
) -> Result<()> {
    loop {
        let event = rx.recv().await;
        if event.is_none() {
            break
        }

        let (worker_id, update) = event.unwrap();
        let status_map = ctx.worker_status_map.clone();
        let mut status_map = status_map.lock().await;
        match update {
            WorkerStatusUpdate::Update(status) => {
                status_map.insert(worker_id, status);
            },
            WorkerStatusUpdate::UpdateMessage(message) => {
                status_map.entry(worker_id).and_modify(|status| {
                    status.last_message = message;
                });
            },
            WorkerStatusUpdate::UpdateStateAndMessage((state, message)) => {
                status_map.entry(worker_id).and_modify(|status| {
                    status.state = state;
                    status.last_message = message;
                });
            },
            WorkerStatusUpdate::UpdateSyncInfo(sync_info) => {
                status_map.entry(worker_id).and_modify(|status| {
                    if let Some(headernum) = sync_info.headernum {
                        status.phactory_info.as_mut().map(|info| {
                            info.headernum = headernum + 1;
                        });
                    }
                    if let Some(para_headernum) = sync_info.para_headernum {
                        status.phactory_info.as_mut().map(|info| {
                            info.para_headernum = para_headernum + 1;
                        });
                    }
                    if let Some(blocknum) = sync_info.blocknum {
                        status.phactory_info.as_mut().map(|info| {
                            info.blocknum = blocknum + 1;
                        });
                    }
                });

            },
            WorkerStatusUpdate::Delete => {
                status_map.remove(&worker_id);
            },
        }
        drop(status_map);
    }

    Ok(())
}