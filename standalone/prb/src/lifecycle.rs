use crate::api::WorkerStatus;
use crate::datasource::WrappedDataSourceManager;
use crate::inv_db::{get_all_workers, Worker, WrappedDb};
use crate::tx::TxManager;
use crate::wm::{
    send_to_main_channel, send_to_main_channel_and_wait_for_response, WorkerManagerCommandTx,
    WorkerManagerMessage, WrappedWorkerManagerContext,
};
use crate::worker::{WorkerContext, WrappedWorkerContext};
use anyhow::Result;
use log::{debug, info, warn};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinSet;

pub struct WorkerLifecycleManager {
    pub main_tx: WorkerManagerCommandTx,
    pub main_ctx: WrappedWorkerManagerContext,
    pub dsm: WrappedDataSourceManager,
    pub should_stop: bool,
    pub inv_db: WrappedDb,
    pub txm: Arc<TxManager>,
    pub workers: Vec<Worker>,
    pub worker_context_vec: Vec<WrappedWorkerContext>,
    pub worker_context_map: WorkerContextMap,
    pub fast_sync_enabled: bool,
    pub fast_sync_semaphore: Arc<Semaphore>,
    pub webhook_url: Option<String>,
    pub reqwest: Client,
}
pub type WrappedWorkerLifecycleManager = Arc<WorkerLifecycleManager>;

pub type WorkerContextMap = HashMap<String, WrappedWorkerContext>; // HashMap<UuidString, WrappedWorkerContext>