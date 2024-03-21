use crate::api::{start_api_server, WrappedWorkerContexts};
use crate::cli::WorkerManagerCliArgs;
use crate::dataprovider::{DataProvider, DataProviderEvent};
use crate::datasource::{setup_data_source_manager, WrappedDataSourceManager};
use crate::db::{get_all_workers, setup_inventory_db, WrappedDb};
use crate::lifecycle::{WorkerContextMap, WorkerLifecycleManager, WrappedWorkerLifecycleManager};
use crate::offchain_tx::{master_loop as offchain_tx_loop, OffchainMessagesEvent};
use crate::processor::{Processor, ProcessorEvent, ProcessorEventTx, WorkerContext as ProcessorWorkerContext};
use crate::tx::TxManager;
use crate::use_parachain_api;
use crate::wm::WorkerManagerMessage::*;
use crate::worker::{WorkerLifecycleState, WrappedWorkerContext};
use crate::worker_status::{update_worker_status, WorkerStatusUpdate};
use anyhow::{anyhow, Result};
use futures::future::{try_join, try_join3, try_join_all};
use log::{debug, error, info};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex};
use tokio::time::sleep;

pub type GlobalWorkerManagerCommandChannelPair = (
    mpsc::UnboundedSender<WorkerManagerCommand>,
    Arc<TokioMutex<mpsc::UnboundedReceiver<WorkerManagerCommand>>>,
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
    pub current_lifecycle_manager: Arc<Mutex<Option<WrappedWorkerLifecycleManager>>>,
    pub current_lifecycle_tx: Arc<TokioMutex<Option<WorkerManagerCommandTx>>>,
    pub inv_db: WrappedDb,
    pub dsm: WrappedDataSourceManager,
    pub workers: WrappedWorkerContexts,
    pub worker_map: Arc<TokioMutex<WorkerContextMap>>,
    pub processor_tx: Arc<ProcessorEventTx>,
    pub txm: Arc<TxManager>,
    pub pccs_url: String,
    pub pccs_timeout_secs: u64,
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

    let (processor_event_tx, processor_event_rx) = mpsc::unbounded_channel::<ProcessorEvent>();
    let (data_provider_event_tx, data_provider_event_rx) = mpsc::unbounded_channel::<DataProviderEvent>();
    let (offchain_messages_tx, mut offchain_messages_rx) = mpsc::unbounded_channel::<OffchainMessagesEvent>();
    let (worker_status_update_tx, mut worker_status_update_rx) = mpsc::unbounded_channel::<WorkerStatusUpdate>();

    let processor_event_tx = Arc::new(processor_event_tx);
    let data_provider_tx = Arc::new(data_provider_event_tx);
    let offchain_messages_tx = Arc::new(offchain_messages_tx);

    let ctx = Arc::new(WorkerManagerContext {
        initialized: false.into(),
        current_lifecycle_manager: Arc::new(Mutex::new(None)),
        current_lifecycle_tx: Arc::new(TokioMutex::new(None)),
        inv_db: inv_db.clone(),
        dsm: dsm.clone(),
        txm: txm.clone(),
        workers: Arc::new(TokioMutex::new(Vec::new())),
        worker_map: Arc::new(TokioMutex::new(HashMap::new())),
        pccs_url: args.pccs_url.clone(),
        pccs_timeout_secs: args.pccs_timeout,
        processor_tx: processor_event_tx.clone(),
    });

    let headers_db = {
        let opts = crate::tx::get_options(None);
        let path = std::path::Path::new(&args.db_path).join("headers");
        let db = crate::tx::DB::open(&opts, path).unwrap();
        Arc::new(db)
    };

    let mut data_provider = DataProvider {
        dsm: dsm.clone(),
        headers_db: headers_db.clone(),
        rx: data_provider_event_rx,
        tx: data_provider_tx.clone(),
        processor_event_tx: processor_event_tx.clone(),
    };
    data_provider.init().await.unwrap();

    let mut processor = Processor {
        rx: processor_event_rx,
        tx: processor_event_tx.clone(),
        data_provider_event_tx: data_provider_tx.clone(),
        offchain_message_tx: offchain_messages_tx.clone(),
        worker_status_update_tx: Arc::new(worker_status_update_tx),
        relaychain_chaintip: crate::dataprovider::relaychain_api(dsm.clone(), false).await.latest_finalized_block_number().await.unwrap(),
        parachain_chaintip: crate::dataprovider::parachain_api(dsm.clone(), false).await.latest_finalized_block_number().await.unwrap(),
    };

    let workers = get_all_workers(inv_db.clone()).unwrap()
        .into_iter()
        .filter(|w| w.enabled)
        .map(|w| ProcessorWorkerContext {
            uuid: w.id,
            pool_id: w.pid.unwrap_or(0),

            headernum: 0,
            para_headernum: 0,
            blocknum: 0,
            calling: false,
            accept_sync_request: false,
            client: Arc::new(crate::pruntime::create_client(w.endpoint.clone())),
            pending_requests: VecDeque::new(),

            initialized: false,
            registered: false,
            benchmarked: false,
        })
        .collect::<Vec<_>>();

    let join_handle = try_join3(
        tokio::spawn(start_api_server(ctx.clone(), args.clone())),
        tokio::spawn(txm_handle),
        try_join_all(ds_handles),
    );

    tokio::select! {
        _ = processor.master_loop(workers) => {}

        _ = data_provider.master_loop() => {}

        _ = offchain_tx_loop(offchain_messages_rx, offchain_messages_tx.clone(), txm.clone()) => {}

        _ = update_worker_status(ctx.clone(), worker_status_update_rx) => {}

        _ = crate::dataprovider::keep_syncing_headers(dsm.clone(), headers_db.clone(), processor_event_tx.clone()) => {}

        ret = join_handle => {
            info!("wm.join_handle: {:?}", ret);
        }
        _ = async {
            loop {
                let (reload_tx, mut reload_rx) = mpsc::channel::<()>(1);
                let main_handle =
                    set_lifecycle_manager(
                        ctx.clone(),
                        reload_tx.clone(),
                        fast_sync_enabled,
                        args.webhook_url.clone(),
                    );

                tokio::select! {
                    ret = main_handle => {
                        info!("main_handle finished: {:?}", ret);
                        info!("Task done, exiting!");
                        std::process::exit(0);
                    }
                    _ = reload_rx.recv() => {
                        info!("Reload signal received, restarting WM in 15 sec...");
                    }
                }
                sleep(Duration::from_secs(15)).await;
            }
        } => {}
    }
}

#[allow(clippy::await_holding_lock)]
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

    let clm = ctx.current_lifecycle_manager.clone();
    let curr_tx = ctx.current_lifecycle_tx.clone();
    let mut clm = clm.lock().map_err(|e| anyhow!(e.to_string()))?;
    let mut curr_tx = curr_tx.lock().await;
    *clm = Some(lm.clone());
    *curr_tx = Some(tx.clone());
    drop(clm);
    drop(curr_tx);

    try_join(
        message_loop(ctx.clone(), tx.clone(), rx, reload_tx),
        lm.clone().spawn_lifecycle_tasks(),
    )
    .await?;
    Ok(())
}

#[allow(clippy::await_holding_lock)]
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
                let clm = ctx.current_lifecycle_manager.clone();
                let c_tx = ctx.current_lifecycle_tx.clone();
                let mut clm = clm.lock().map_err(|e| anyhow!(e.to_string()))?;
                let mut c_tx = c_tx.lock().await;
                *clm = None;
                *c_tx = None;
                drop(clm);
                drop(c_tx);

                let workers = ctx.workers.clone();
                let worker_map = ctx.worker_map.clone();
                let mut workers = workers.lock().await;
                let mut worker_map = worker_map.lock().await;
                for i in workers.iter() {
                    let ii = i.clone();
                    let ii = ii.read().await;
                    let sm_tx = ii.sm_tx.as_ref().unwrap().clone();
                    drop(ii);
                    sm_tx.send(WorkerLifecycleState::HasError("WM reloaded!".to_string()))?;
                }
                *workers = vec![];
                *worker_map = HashMap::new();
                drop(workers);
                drop(worker_map);
                reload_tx.send(()).await?;
            }

            ShouldStartWorkerLifecycle(c) => {
                let cc = c.clone();
                let cc = cc.read().await;
                let id = cc.worker.id.clone();
                drop(cc);

                let workers = ctx.workers.clone();
                let worker_map = ctx.worker_map.clone();
                let mut workers = workers.lock().await;
                let mut worker_map = worker_map.lock().await;
                if !worker_map.contains_key(id.as_str()) {
                    workers.push(c.clone());
                    worker_map.insert(id, c);
                }
                drop(workers);
                drop(worker_map);

                let _ = response_tx.unwrap().send(ResponseOk);
            }

            _ => {}
        }
    }
    debug!("message_loop end");
    Ok(())
}
