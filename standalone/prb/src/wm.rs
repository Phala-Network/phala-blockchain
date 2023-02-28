use std::collections::HashMap;
use std::sync::Arc;

use crate::api::{start_api_server, WorkerStatus, WorkerStatusMap};
use anyhow::{anyhow, Result};
use futures::future::try_join_all;
use log::{debug, info};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};

use crate::cli::WorkerManagerCliArgs;
use crate::datasource::{setup_data_source_manager, WrappedDataSourceManager};
use crate::db::{setup_cache_index_db, setup_inventory_db, WrappedDb};
use crate::lifecycle::{WorkerLifecycleManager, WrappedWorkerLifecycleManager};
use crate::wm::WorkerManagerMessage::*;
use crate::worker::{WorkerLifecycleState, WrappedWorkerContext};

pub type GlobalWorkerManagerCommandChannelPair = (
    mpsc::UnboundedSender<WorkerManagerCommand>,
    Arc<Mutex<mpsc::UnboundedReceiver<WorkerManagerCommand>>>,
);

pub type WorkerManagerCommandTx = mpsc::UnboundedSender<WorkerManagerCommand>;
pub type WorkerManagerCommandRx = mpsc::UnboundedReceiver<WorkerManagerCommand>;

pub type WorkerManagerResponseTx = oneshot::Sender<WorkerManagerMessage>;
pub type WorkerManagerResponseRx = oneshot::Receiver<WorkerManagerMessage>;

pub struct WorkerManagerCommand {
    message: WorkerManagerMessage,
    response_tx: Option<WorkerManagerResponseTx>,
}

pub struct WorkerManagerContext {
    pub initialized: bool,
    pub current_lifecycle_manager: Option<WrappedWorkerLifecycleManager>,
    pub inv_db: WrappedDb,
    pub ci_db: WrappedDb,
    pub dsm: WrappedDataSourceManager,
    pub state_map: WorkerStatusMap,
}

pub type WrappedWorkerManagerContext = Arc<RwLock<WorkerManagerContext>>;

pub enum WorkerManagerMessage {
    ResponseOk,
    ResponseErr(String),

    LifecycleManagerStarted,
    ShouldBreakMessageLoop,
    ShouldResetLifecycleManager,

    ShouldStartWorkerLifecycle(WrappedWorkerContext),
    ShoudUpdateWorkerStatus(WrappedWorkerContext),

    AcquireFastSyncLock(String),
    ReleaseFastSyncLock(String),
    FastSyncLockBusy,
}

pub type WrappedReloadTx = mpsc::Sender<()>;

pub async fn do_send_to_main_channel(
    main_tx: WorkerManagerCommandTx,
    message: WorkerManagerMessage,
    response_tx: Option<WorkerManagerResponseTx>,
) -> Result<()> {
    match main_tx.send(WorkerManagerCommand {
        message,
        response_tx,
    }) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Failed to send to main channel! {}", e)),
    }
}

pub async fn send_to_main_channel(
    main_tx: WorkerManagerCommandTx,
    message: WorkerManagerMessage,
) -> Result<()> {
    do_send_to_main_channel(main_tx, message, None).await
}

pub async fn send_to_main_channel_and_wait_for_response(
    main_tx: WorkerManagerCommandTx,
    message: WorkerManagerMessage,
) -> Result<WorkerManagerMessage> {
    let (response_tx, response_rx) = oneshot::channel::<WorkerManagerMessage>();
    do_send_to_main_channel(main_tx, message, Some(response_tx)).await?;
    let res = response_rx.await?;
    Ok(res)
}

pub async fn wm(args: WorkerManagerCliArgs) {
    info!("Staring prb-wm with {:?}", &args);

    let inv_db = setup_inventory_db(&args.db_path);
    let ci_db = setup_cache_index_db(&args.db_path, args.use_persisted_cache_index);
    let (dsm, ds_handles) = setup_data_source_manager(&args.data_source_config_path)
        .await
        .expect("Initialize data source manager");

    let fast_sync_enabled = !args.disable_fast_sync;

    let ctx = Arc::new(RwLock::new(WorkerManagerContext {
        initialized: false,
        current_lifecycle_manager: None,
        inv_db,
        ci_db,
        dsm,
        state_map: HashMap::new(),
    }));

    let _ = tokio::join!(
        tokio::spawn(start_api_server(ctx.clone(), args.clone())),
        try_join_all(ds_handles),
        async {
            loop {
                let (reload_tx, mut reload_rx) = mpsc::channel::<()>(1);
                let main_handle =
                    set_lifecycle_manager(ctx.clone(), reload_tx.clone(), fast_sync_enabled);

                tokio::select! {
                    _ = main_handle => {
                        info!("Task done, exiting!");
                        std::process::exit(0);
                    }
                    _ = reload_rx.recv() => {
                        info!("Reload signal received.");
                    }
                }
            }
        }
    );
}

pub async fn set_lifecycle_manager(
    ctx: WrappedWorkerManagerContext,
    reload_tx: WrappedReloadTx,
    fast_sync_enabled: bool,
) {
    let (tx, rx) = mpsc::unbounded_channel::<WorkerManagerCommand>();

    let ctx_read = ctx.clone();
    let ctx_read = ctx_read.read().await;
    let dsm = ctx_read.dsm.clone();
    let inv_db = ctx_read.inv_db.clone();
    let ci_db = ctx_read.ci_db.clone();
    drop(ctx_read);

    let lm = WorkerLifecycleManager::create(
        tx.clone(),
        ctx.clone(),
        dsm,
        inv_db,
        ci_db,
        fast_sync_enabled,
    )
    .await;

    let mut ctx_write = ctx.write().await;
    ctx_write.current_lifecycle_manager = Some(lm.clone());
    drop(ctx_write);

    let _ = tokio::join!(
        message_loop(ctx.clone(), tx.clone(), rx, reload_tx),
        lm.clone().spawn_lifecycle_tasks()
    );
}

async fn message_loop(
    ctx: WrappedWorkerManagerContext,
    tx: WorkerManagerCommandTx,
    mut rx: WorkerManagerCommandRx,
    reload_tx: WrappedReloadTx,
) {
    let mut fast_sync_lock_owner = "";

    debug!("message_loop start");
    while let Some(WorkerManagerCommand {
        message,
        response_tx,
    }) = rx.recv().await
    {
        match message {
            ShouldBreakMessageLoop => break,
            LifecycleManagerStarted => {
                // todo: setup status map
                info!("LifecycleManagerStarted");
            }
            ShouldResetLifecycleManager => {
                // todo: do some cleanup
                reload_tx
                    .send(())
                    .await
                    .expect("ShouldResetLifecycleManager");
            }

            ShouldStartWorkerLifecycle(c) => {
                let cc = c.clone();
                let cc = cc.read().await;
                let ctx = ctx.clone();
                let mut ctx = ctx.write().await;
                let map = &mut ctx.state_map;
                let id = cc.id.clone();

                map.insert(
                    id,
                    WorkerStatus {
                        worker: cc.worker.clone(),
                        state: WorkerLifecycleState::Starting,
                        phactory_info: None,
                        last_message: String::new(),
                    },
                );

                match cc.pr.get_info(()).await {
                    Ok(_) => {
                        let _ = response_tx.unwrap().send(ResponseOk);
                    }
                    Err(e) => {
                        let _ = response_tx.unwrap().send(ResponseErr(e.to_string()));
                    }
                }
                drop(cc);
                drop(ctx);
            }

            ShoudUpdateWorkerStatus(c) => {
                let cc = c.read().await;
                let ctx = ctx.clone();
                let mut ctx = ctx.write().await;
                let map = &mut ctx.state_map;
                let id = cc.id.as_str();
                let mut status = map.get_mut(id).unwrap();

                status.state = cc.state.clone();
                status.phactory_info = cc.info.clone();
                status.last_message = cc.last_message.clone();
                drop(cc);
                drop(ctx);
            }

            AcquireFastSyncLock(id) => {
                debug!(
                    "AcquireFastSyncLock: {} {} {}",
                    &id,
                    &fast_sync_lock_owner,
                    id == fast_sync_lock_owner
                );
                if (id == fast_sync_lock_owner) {
                    let _ = response_tx.unwrap().send(ResponseOk);
                    return;
                }
                if (fast_sync_lock_owner.len() == 0) {
                    fast_sync_lock_owner = id.as_str();
                    let _ = response_tx.unwrap().send(ResponseOk);
                    return;
                }
                let _ = response_tx.unwrap().send(FastSyncLockBusy);
            }
            ReleaseFastSyncLock(id) => {
                if (id == fast_sync_lock_owner) {
                    fast_sync_lock_owner = "";
                } else {
                    panic!(
                        "Disorded fast_sync_lock_owner! Excepting {}, got {}.",
                        fast_sync_lock_owner, id
                    )
                }
            }

            _ => {}
        }
    }
}
