use crate::datasource::WrappedDataSourceManager;
use crate::db::{get_all_workers, Worker, WrappedDb};
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
use crate::api::WorkerStatus;
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

impl WorkerLifecycleManager {
    pub async fn create(
        main_tx: WorkerManagerCommandTx,
        main_ctx: WrappedWorkerManagerContext,
        dsm: WrappedDataSourceManager,
        inv_db: WrappedDb,
        fast_sync_enabled: bool,
        webhook_url: Option<String>,
        txm: Arc<TxManager>,
    ) -> WrappedWorkerLifecycleManager {
        let workers =
            get_all_workers(inv_db.clone()).expect("Failed to load workers from local database");
        let count = workers.len();
        let workers = workers
            .into_iter()
            .filter(|w| w.enabled)
            .collect::<Vec<_>>();
        let count_enabled = workers.len();
        if count_enabled == 0 {
            info!("Got {} worker(s) from local database.", count);
            panic!("There are no worker enabled!");
        }
        debug!(
            "Got workers:\n{}",
            serde_json::to_string_pretty(&workers).unwrap()
        );
        info!(
            "Starting lifecycle for {} of {} worker(s).",
            count_enabled, count
        );

        let mut join_set = JoinSet::new();
        for w in workers.clone() {
            join_set.spawn(WorkerContext::create(w, main_ctx.clone()));
        }
        let mut worker_context_vec = Vec::new();
        let mut worker_context_map: WorkerContextMap = HashMap::new();
        while let Some(c) = join_set.join_next().await {
            match c {
                Ok(c) => match c {
                    Ok(c) => {
                        let cc = Arc::new(RwLock::new(c));
                        let c = cc.clone();
                        let mut c = c.write().await;
                        c.self_ref = Some(cc.clone());
                        worker_context_map.insert(c.id.clone(), cc.clone());
                        worker_context_vec.push(cc.clone());
                        drop(c)
                    }
                    Err(e) => panic!("create_worker_contexts: {}", e),
                },
                Err(e) => panic!("create_worker_contexts: {}", e),
            }
        }

        let fast_sync_semaphore = Arc::new(Semaphore::new(2));

        let dd = dsm.clone();
        dd.clone().wait_until_rpc_avail(false).await;

        let lm = Self {
            main_tx,
            main_ctx,
            dsm,
            should_stop: false,
            inv_db: inv_db.clone(),
            txm,
            workers,
            worker_context_map,
            worker_context_vec,
            fast_sync_enabled,
            fast_sync_semaphore,
            webhook_url,
            reqwest: Client::new(),
        };
        Arc::new(lm)
    }

    pub async fn webhook_send(self: Arc<Self>, c: WrappedWorkerContext) -> Result<()> {
        if self.webhook_url.is_none() {
            return Ok(());
        }
        let cc = c.read().await;
        let s = WorkerStatus {
            worker: cc.worker.clone(),
            state: cc.state.clone(),
            phactory_info: cc.info.clone(),
            last_message: cc.last_message.clone(),
        };
        let body = serde_json::to_string(&s)?;
        if let Err(e) = self
            .reqwest
            .post(self.webhook_url.as_ref().unwrap())
            .body(body)
            .send()
            .await
        {
            warn!("Error while sending to webhook: {e}")
        }
        Ok(())
    }

    pub async fn spawn_lifecycle_tasks(self: Arc<Self>) {
        debug!("spawn_lifecycle_tasks start");
        let v = &self.clone().worker_context_map;
        let v = v.iter().map(|(_, c)| c.clone()).collect::<Vec<_>>();
        let mut join_set = JoinSet::new();
        for c in v {
            join_set.spawn(WorkerContext::start(c));
        }
        while (join_set.join_next().await).is_some() {} // wait tasks to be done
        self.clone()
            .send_to_main_channel(WorkerManagerMessage::ShouldBreakMessageLoop)
            .await
            .expect("spawn_lifecycle_tasks -> ShouldBreakMessageLoop");
    }

    pub async fn send_to_main_channel(
        self: Arc<Self>,
        message: WorkerManagerMessage,
    ) -> Result<()> {
        send_to_main_channel(self.main_tx.clone(), message).await
    }

    pub async fn send_to_main_channel_and_wait_for_response(
        self: Arc<Self>,
        message: WorkerManagerMessage,
    ) -> Result<WorkerManagerMessage> {
        send_to_main_channel_and_wait_for_response(self.main_tx.clone(), message).await
    }
}
