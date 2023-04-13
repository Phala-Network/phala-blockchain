use crate::api::{start_api_server, WrappedWorkerContexts};
use crate::cli::WorkerManagerCliArgs;
use crate::datasource::{setup_data_source_manager, WrappedDataSourceManager};
use crate::db::{setup_inventory_db, WrappedDb};
use crate::lifecycle::{WorkerLifecycleManager, WrappedWorkerLifecycleManager};
use crate::tx::TxManager;
use crate::use_parachain_api;
use crate::wm::WorkerManagerMessage::*;
use crate::worker::WrappedWorkerContext;
use anyhow::{anyhow, Result};
use futures::future::{try_join, try_join3, try_join_all};
use log::{debug, info};

use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

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
    pub initialized: AtomicBool,
    pub current_lifecycle_manager: AtomicPtr<Option<WrappedWorkerLifecycleManager>>,
    pub inv_db: WrappedDb,
    pub dsm: WrappedDataSourceManager,
    pub workers: WrappedWorkerContexts,
    pub txm: Arc<TxManager>,
}

pub type WrappedWorkerManagerContext = Arc<WorkerManagerContext>;

pub enum WorkerManagerMessage {
    ResponseOk,
    ResponseErr(String),

    LifecycleManagerStarted,
    ShouldBreakMessageLoop,
    ShouldResetLifecycleManager,

    ShouldStartWorkerLifecycle(WrappedWorkerContext),
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
    let (dsm, ds_handles) =
        setup_data_source_manager(&args.data_source_config_path, args.cache_size)
            .await
            .expect("Initialize data source manager");

    let fast_sync_enabled = !args.disable_fast_sync;

    dsm.clone().wait_until_rpc_avail(false).await;
    let _api = use_parachain_api!(dsm, false).unwrap();

    let (txm, txm_handle) = TxManager::new(&args.db_path, dsm.clone()).expect("TxManager");

    let mut current_lifecycle_manager_inner = None;
    let current_lifecycle_manager = AtomicPtr::new(&mut current_lifecycle_manager_inner);
    let ctx = Arc::new(WorkerManagerContext {
        initialized: false.into(),
        current_lifecycle_manager,
        inv_db,
        dsm: dsm.clone(),
        txm: txm.clone(),
        workers: Arc::new(Mutex::new(Vec::new())),
    });

    let join_handle = try_join3(
        tokio::spawn(start_api_server(ctx.clone(), args.clone())),
        tokio::spawn(txm_handle),
        try_join_all(ds_handles),
    );

    tokio::select! {
        ret = join_handle => {
            info!("wm.join_handle: {:?}", ret);
        }
        _ = async {
            loop {
                let (reload_tx, mut reload_rx) = mpsc::channel::<()>(1);
                let main_handle =
                    set_lifecycle_manager(ctx.clone(), reload_tx.clone(), fast_sync_enabled, args.webhook_url.clone());

                tokio::select! {
                    ret = main_handle => {
                        info!("main_handle finished: {:?}", ret);
                        info!("Task done, exiting!");
                        std::process::exit(0);
                    }
                    _ = reload_rx.recv() => {
                        info!("Reload signal received.");
                    }
                }
            }
        } => {}
    }
}

pub async fn set_lifecycle_manager(
    ctx: WrappedWorkerManagerContext,
    reload_tx: WrappedReloadTx,
    fast_sync_enabled: bool,
    webhook_url: Option<String>,
) -> Result<()> {
    let (tx, rx) = mpsc::unbounded_channel::<WorkerManagerCommand>();

    let lm = WorkerLifecycleManager::create(
        tx.clone(),
        ctx.clone(),
        ctx.dsm.clone(),
        ctx.inv_db.clone(),
        fast_sync_enabled,
        webhook_url,
        ctx.txm.clone(),
    )
    .await;
    let mut lm_inner = Some(lm.clone());
    ctx.current_lifecycle_manager
        .store(&mut lm_inner, Ordering::SeqCst);

    try_join(
        message_loop(ctx.clone(), tx.clone(), rx, reload_tx),
        lm.clone().spawn_lifecycle_tasks(),
    )
    .await?;
    Ok(())
}

async fn message_loop(
    ctx: WrappedWorkerManagerContext,
    _tx: WorkerManagerCommandTx,
    mut rx: WorkerManagerCommandRx,
    reload_tx: WrappedReloadTx,
) -> Result<()> {
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
                reload_tx.send(()).await?;
            }

            ShouldStartWorkerLifecycle(c) => {
                let workers = ctx.workers.clone();
                let mut workers = workers.lock().await;
                workers.push(c);
                drop(workers);
                let _ = response_tx.unwrap().send(ResponseOk);
            }

            _ => {}
        }
    }
    debug!("message_loop end");
    Ok(())
}
