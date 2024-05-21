use crate::api::WorkerStatus;
use crate::worker::WorkerLifecycleState;
use crate::wm::WorkerManagerContext;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::mpsc;

pub enum WorkerStatusUpdate {
    Update(WorkerStatus),
    UpdateMessage(String),
    UpdateStateAndMessage((WorkerLifecycleState, String)),
    UpdateSyncInfo((u32, u32, u32)),
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
            WorkerStatusUpdate::UpdateSyncInfo((headernum, para_headernum, blocknum)) => {
                status_map.entry(worker_id).and_modify(|status| {
                    status.phactory_info.as_mut().map(|info| {
                        info.headernum = headernum;
                        info.para_headernum = para_headernum;
                        info.blocknum = blocknum;
                    });
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