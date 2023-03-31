use crate::db::Worker;
use crate::wm::{WorkerManagerMessage, WrappedWorkerManagerContext};
use crate::worker::WorkerLifecycleCommand::*;
use anyhow::Result;
use chrono::prelude::*;
use futures::future::join;
use log::{debug, error, info, warn};
use phactory_api::prpc::PhactoryInfo;
use phactory_api::pruntime_client::{new_pruntime_client, PRuntimeClient};
use pherry::chain_client::search_suitable_genesis_for_worker;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::sleep;

pub type WorkerLifecycleCommandTx = mpsc::UnboundedSender<WorkerLifecycleCommand>;
pub type WorkerLifecycleCommandRx = mpsc::UnboundedReceiver<WorkerLifecycleCommand>;

pub enum WorkerLifecycleCommand {
    ShouldRestart,
    ShouldStart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerLifecycleState {
    Starting,
    Synchronizing,
    Preparing,
    Working,

    HasError(String),
}

pub type WrappedWorkerContext = Arc<RwLock<WorkerContext>>;
pub type WorkerLifecycleStateTx = mpsc::UnboundedSender<WorkerLifecycleState>;
pub type WorkerLifecycleStateRx = mpsc::UnboundedReceiver<WorkerLifecycleState>;

macro_rules! lifecycle_loop_state_handle_error {
    ($f:expr, $c:expr) => {{
        if let Err(e) = $f($c.clone()).await {
            let (_, worker, _, sm_tx) = extract_essential_values!($c);
            let cc = $c.clone();
            let mut cc = cc.write().await;
            cc.set_last_message(&e.to_string());
            drop(cc);
            info!(
                "Worker {}({}, {}) stopped.",
                &worker.name, &worker.id, &worker.endpoint
            );
            let _ = sm_tx
                .clone()
                .send(WorkerLifecycleState::HasError(e.to_string()));
        }
    }};
}

macro_rules! set_worker_message {
    ($c:expr, $m:expr) => {{
        let cc = $c.clone();
        let mut cc = cc.write().await;
        let ctx = cc.ctx.clone();
        let ctx = ctx.read().await;
        let lm = ctx.current_lifecycle_manager.as_ref();
        let lm = lm.unwrap().clone();
        drop(ctx);
        cc.set_last_message($m);
        drop(cc);
        let _ = lm
            .send_to_main_channel(WorkerManagerMessage::ShouldUpdateWorkerStatus($c.clone()))
            .await;
    }};
}

#[macro_export]
macro_rules! extract_essential_values {
    ($c:expr) => {{
        let cc = $c.clone();
        let cc = cc.read().await;
        let sm_tx = cc.sm_tx.as_ref().unwrap().clone();
        let pr = cc.pr.clone();
        let ctx = cc.ctx.clone();
        let ctx = ctx.read().await;
        let lm = ctx.current_lifecycle_manager.as_ref();
        let lm = lm.unwrap().clone();
        let worker = cc.worker.clone();
        drop(ctx);
        drop(cc);
        (lm, worker, pr, sm_tx)
    }};
}

pub struct WorkerContext {
    pub id: String,
    pub self_ref: Option<WrappedWorkerContext>,
    pub sm_tx: Option<WorkerLifecycleStateTx>,
    pub worker: Worker,
    pub state: WorkerLifecycleState,
    pub tx: WorkerLifecycleCommandTx,
    pub rx: Arc<Mutex<WorkerLifecycleCommandRx>>,
    pub ctx: WrappedWorkerManagerContext,
    pub pr: Arc<PRuntimeClient>,
    pub info: Option<PhactoryInfo>,
    pub last_message: String,
}

impl WorkerContext {
    pub async fn create(w: Worker, ctx: WrappedWorkerManagerContext) -> Result<Self> {
        let pr = new_pruntime_client(w.endpoint.clone());
        let (tx, rx) = mpsc::unbounded_channel::<WorkerLifecycleCommand>();

        let mut ret = Self {
            id: w.id.clone(),
            self_ref: None,
            sm_tx: None,
            worker: w,
            state: WorkerLifecycleState::Starting,
            tx,
            rx: Arc::new(Mutex::new(rx)),
            ctx,
            pr: Arc::new(pr),
            info: None,
            last_message: String::new(),
        };
        ret.set_last_message("Starting lifecycle...");
        Ok(ret)
    }

    pub fn set_last_message(&mut self, m: &str) {
        let time: DateTime<Local> = Local::now();
        let worker = &self.worker;
        self.last_message = format!("[{time}] {m}");
        info!(
            "Worker {}({}, {}): {}",
            &worker.name, &worker.id, &worker.endpoint, &m
        );
    }

    pub async fn start(c: WrappedWorkerContext) {
        debug!("WorkerContext::start");
        let cc = c.clone();
        let cc = cc.read().await;
        let ctx = cc.ctx.clone();
        let ctx = ctx.read().await;
        let lm = ctx.current_lifecycle_manager.clone().unwrap().clone();
        let worker = cc.worker.clone();
        drop(ctx);
        drop(cc);
        if let WorkerManagerMessage::ResponseErr(err_str) = lm
            .clone()
            .send_to_main_channel_and_wait_for_response(
                WorkerManagerMessage::ShouldStartWorkerLifecycle(c.clone()),
            )
            .await
            .unwrap_or_else(|_| panic!("Failed to start worker {}", worker.name))
        {
            let cc = c.clone();
            let mut cc = cc.write().await;
            cc.state = WorkerLifecycleState::HasError(err_str);
            drop(cc);
        }
        let _ = lm
            .send_to_main_channel(WorkerManagerMessage::ShouldUpdateWorkerStatus(c.clone()))
            .await;

        let _ = join(
            tokio::spawn(Self::do_start(c.clone())),
            Self::message_loop(c.clone()),
        )
        .await;
    }

    async fn restart(_c: WrappedWorkerContext) {}

    async fn do_start(c: WrappedWorkerContext) {
        let (tx, rx) = mpsc::unbounded_channel::<WorkerLifecycleState>();
        let _cc = c.clone();
        let mut cc = c.write().await;
        cc.sm_tx = Some(tx.clone());
        drop(cc);
        let _ = join(
            Self::lifecycle_loop(c.clone(), rx),
            Self::update_info_loop(c.clone()),
        )
        .await;
    }

    async fn send_status(c: WrappedWorkerContext) {
        let (lm, _, _pr, _) = extract_essential_values!(c);
        let _ = lm
            .clone()
            .send_to_main_channel(WorkerManagerMessage::ShouldUpdateWorkerStatus(c.clone()))
            .await;
    }

    async fn lifecycle_loop(c: WrappedWorkerContext, mut sm_rx: WorkerLifecycleStateRx) {
        let (_lm, worker, _pr, sm_tx) = extract_essential_values!(c);

        let _ = sm_tx.clone().send(WorkerLifecycleState::Starting);
        Self::send_status(c.clone()).await;

        while let Some(s) = sm_rx.recv().await {
            let _cc = c.clone();
            let mut cc = c.write().await;
            cc.state = s.clone();
            drop(cc);

            Self::send_status(c.clone()).await;

            match s {
                WorkerLifecycleState::Starting => {
                    lifecycle_loop_state_handle_error!(Self::handle_on_starting, c);
                }
                WorkerLifecycleState::Synchronizing => {
                    lifecycle_loop_state_handle_error!(Self::handle_on_synchronizing, c);
                }
                WorkerLifecycleState::Preparing => {}
                WorkerLifecycleState::Working => {}
                WorkerLifecycleState::HasError(e) => {
                    error!(
                        "Worker {}({}, {}) stopped due to error: {}",
                        &worker.name, &worker.id, &worker.endpoint, e
                    );
                    // todo: may need some cleanups
                }
            }

            Self::send_status(c.clone()).await;
        }
    }

    async fn handle_on_starting(c: WrappedWorkerContext) -> Result<()> {
        let (lm, worker, pr, sm_tx) = extract_essential_values!(c);
        let dsm = lm.dsm.clone();

        let mut i = pr.get_info(()).await?;

        if !i.initialized {
            set_worker_message!(c, "Initializing pRuntime...");
            let res = pr
                .init_runtime(dsm.get_init_runtime_default_request().await?)
                .await?;
            set_worker_message!(c, "Initialized pRuntime.");
            debug!(
                "Worker {}({}, {}) init_runtime resp: {:?}",
                &worker.name, &worker.id, &worker.endpoint, res
            )
        }

        if i.public_key.is_none() {
            i = pr.get_info(()).await?;
        }

        if lm.fast_sync_enabled
            && i.can_load_chain_state
            && lm.dsm.is_relaychain_full
            && lm.dsm.is_parachain_full
        {
            let para_api = &lm
                .dsm
                .clone()
                .current_parachain_rpc_client(true)
                .await
                .expect("No online rpc client")
                .client;
            let pubkey = &i.public_key.unwrap();
            let pubkey = hex::decode(pubkey)?;
            set_worker_message!(c, "Trying to load chain state...");
            let lock = lm.fast_sync_semaphore.clone();
            let lock = lock.acquire().await?;
            let search = search_suitable_genesis_for_worker(para_api, &pubkey, None).await;
            drop(lock);
            match search {
                Ok((block_number, state)) => {
                    pr.load_chain_state(phactory_api::prpc::ChainState::new(block_number, state))
                        .await?;
                    set_worker_message!(c, "Loaded chain state!");
                }
                Err(e) => {
                    set_worker_message!(c, "Failed to get suitable genesis.");
                    error!(
                        "Worker {}({}, {}) search_suitable_genesis_for_worker: {}",
                        &worker.name, &worker.id, &worker.endpoint, e
                    );
                }
            }
        }

        let _ = sm_tx.clone().send(WorkerLifecycleState::Synchronizing);
        Ok(())
    }

    async fn handle_on_synchronizing(c: WrappedWorkerContext) -> Result<()> {
        set_worker_message!(c, "Now start synchronizing!");
        Ok(())
    }

    async fn update_info_loop(c: WrappedWorkerContext) {
        let (lm, worker, pr, sm_tx) = extract_essential_values!(c);

        let mut retry_count: u8 = 0;

        loop {
            let cc = c.clone();
            let cc = cc.read().await;
            if let WorkerLifecycleState::HasError(_) = &cc.state {
                break;
            }
            drop(cc);

            sleep(Duration::from_secs(6)).await;
            let get_info_req = pr.get_info(()).await;
            match get_info_req {
                Ok(p) => {
                    retry_count = 0;
                    let cc = c.clone();
                    let mut cc = cc.write().await;
                    cc.info = Some(p);
                    drop(cc);
                }
                Err(e) => {
                    warn!("Failed to get_info from {}: {}", &worker.endpoint, &e);
                    if retry_count > 3 {
                        let cc = c.clone();
                        let mut cc = cc.write().await;
                        let m = format!("Failed to get_info from {}: {}", &worker.endpoint, &e);
                        cc.set_last_message(m.as_str());
                        drop(cc);
                        let _ = sm_tx
                            .clone()
                            .send(WorkerLifecycleState::HasError(e.to_string()));
                    }
                    retry_count += 1;
                }
            }
            let _ = lm
                .clone()
                .send_to_main_channel(WorkerManagerMessage::ShouldUpdateWorkerStatus(c.clone()))
                .await;
        }
    }

    async fn message_loop(c: WrappedWorkerContext) {
        let cc = c.clone();
        let cc = cc.read().await;
        let rx = cc.rx.clone();
        drop(cc);
        let mut rx = rx.lock().await;
        while let Some(cmd) = rx.recv().await {
            match cmd {
                ShouldRestart => {}
                ShouldStart => {}
            }
        }
        drop(rx);
    }
}
