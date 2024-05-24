use crate::api::WorkerStatus;
use crate::worker::WorkerLifecycleState;
use crate::wm::WorkerManagerContext;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::mpsc;

pub enum WorkerStatusUpdate {
    Update(Box<WorkerStatus>),
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
        let mut events = vec![];
        let received_count = rx.recv_many(&mut events, 65536).await;
        if received_count == 0 {
            break
        }

        let status_map = ctx.worker_status_map.clone();
        let mut status_map = status_map.lock().await;

        for (worker_id, update) in events {
            match update {
                WorkerStatusUpdate::Update(status) => {
                    status_map.insert(worker_id, *status);
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
                        if let Some(info) = status.phactory_info.as_mut() {
                            info.headernum = headernum;
                            info.para_headernum = para_headernum;
                            info.blocknum = blocknum;
                        }
                    });

                },
                WorkerStatusUpdate::Delete => {
                    status_map.remove(&worker_id);
                },
            }
        }
        drop(status_map);
    }

    Ok(())
}