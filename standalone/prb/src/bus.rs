use log::error;
use tokio::sync::mpsc::error::SendError;

use crate::processor::{PRuntimeRequest, ProcessorEvent, ProcessorTx, WorkerEvent};
use crate::messages::{MessagesEvent, MessagesTx};
use crate::worker_status::{WorkerStatusEvent, WorkerStatusTx};

#[derive(Clone)]
pub struct Bus {
    pub processor_tx: ProcessorTx,
    pub messages_tx: MessagesTx,
    pub worker_status_tx: WorkerStatusTx,
}

impl Bus {
    pub fn send_processor_event(&self, event: ProcessorEvent) -> Result<(), SendError<ProcessorEvent>> {
        let result = self.processor_tx.send(event);
        if let Err(err) = &result {
            error!("Fail to send message to processor_tx. {}", err);
        }
        result
    }

    pub fn send_worker_event(&self, worker_id: String, event: WorkerEvent) -> Result<(), SendError<ProcessorEvent>> {
        self.send_processor_event(
            ProcessorEvent::WorkerEvent((worker_id, event)),
        )
    }

    pub fn send_worker_update_message(&self, worker_id: String, message: String) -> Result<(), SendError<ProcessorEvent>> {
        self.send_worker_event(
            worker_id,
            WorkerEvent::UpdateMessage((chrono::Utc::now(), message)),
        )
    }

    pub fn send_worker_mark_error(&self, worker_id: String, message: String) -> Result<(), SendError<ProcessorEvent>> {
        self.send_worker_event(
            worker_id,
            WorkerEvent::MarkError((chrono::Utc::now(), message)),
        )
    }

    pub fn send_pruntime_request(&self, worker_id: String, request: PRuntimeRequest) -> Result<(), SendError<ProcessorEvent>> {
        self.send_worker_event(
            worker_id,
            WorkerEvent::PRuntimeRequest(request),
        )
    }

    pub fn send_messages_event(&self, event: MessagesEvent) -> Result<(), SendError<MessagesEvent>> {
        let result = self.messages_tx.send(event);
        if let Err(err) = &result {
            error!("Fail to send message to messages_tx. {}", err);
        }
        result
    }

    pub fn send_worker_status_event(&self, event: WorkerStatusEvent) -> Result<(), SendError<WorkerStatusEvent>>{
        let result = self.worker_status_tx.send(event);
        if let Err(err) = &result {
            error!("Fail to send message to worker_status_update_tx. {}", err);
        }
        result
    }
}