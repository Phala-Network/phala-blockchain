use crate::api::{start_api_server, WorkerStatus, WrappedWorkerContexts};
use crate::bus::Bus;
use crate::cli::WorkerManagerCliArgs;
use crate::repository::Repository;
use crate::datasource::{setup_data_source_manager, WrappedDataSourceManager};
use crate::inv_db::{get_all_workers, setup_inventory_db, WrappedDb};
use crate::lifecycle::{WorkerContextMap, WrappedWorkerLifecycleManager};
use crate::messages::{master_loop as message_master_loop, MessagesEvent};
use crate::pool_operator::PoolOperatorAccess;
use crate::processor::{Processor, ProcessorEvent};
use crate::tx::TxManager;
use crate::use_parachain_api;
use crate::worker::WrappedWorkerContext;
use crate::worker_status::{update_worker_status, WorkerStatusEvent};
use anyhow::{anyhow, Result};
use chrono::{Timelike, Utc};
use futures::future::{try_join4, try_join_all};
use log::{error, info};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex};

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
    pub worker_status_map: Arc<TokioMutex<HashMap<String, WorkerStatus>>>,
    pub txm: Arc<TxManager>,
    pub pccs_url: String,
    pub pccs_timeout_secs: u64,
    pub bus: Arc<Bus>,
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

    let (dsm, ds_handles) =
        setup_data_source_manager(&args.data_source_config_path, args.cache_size)
            .await
            .expect("Initialize data source manager");
    let ds_join_handle = tokio::spawn(try_join_all(ds_handles));

    dsm.clone().wait_until_rpc_avail(false).await;

    let (processor_tx, processor_rx) = std::sync::mpsc::channel::<ProcessorEvent>();
    let (messages_tx, messages_rx) = mpsc::unbounded_channel::<MessagesEvent>();
    let (worker_status_tx, worker_status_rx) = mpsc::unbounded_channel::<WorkerStatusEvent>();

    let bus = Arc::new(Bus {
        processor_tx: processor_tx.clone(),
        messages_tx: messages_tx.clone(),
        worker_status_tx: worker_status_tx.clone(),
    });

    let headers_db = {
        let opts = crate::pool_operator::get_options(None);
        let path = std::path::Path::new(&args.db_path).join("headers");
        let db = crate::pool_operator::DB::open(&opts, path).unwrap();
        Arc::new(db)
    };

    let _ = Repository::create(
        bus.clone(),
        dsm.clone(),
        headers_db.clone(),
    ).await.unwrap();

    if args.download_headers_only {
        headers_db.cancel_all_background_work(true);
        drop(headers_db);
        return;
    }

    let inv_db = setup_inventory_db(&args.db_path);
    let (txm, txm_handle) = TxManager::new(&args.db_path, dsm.clone()).expect("TxManager");
    let ctx = Arc::new(WorkerManagerContext {
        initialized: false.into(),
        current_lifecycle_manager: Arc::new(Mutex::new(None)),
        current_lifecycle_tx: Arc::new(TokioMutex::new(None)),
        inv_db: inv_db.clone(),
        dsm: dsm.clone(),
        txm: txm.clone(),
        workers: Arc::new(TokioMutex::new(Vec::new())),
        worker_map: Arc::new(TokioMutex::new(HashMap::new())),
        worker_status_map: Arc::new(TokioMutex::new(HashMap::new())),
        pccs_url: args.pccs_url.clone(),
        pccs_timeout_secs: args.pccs_timeout,
        bus: bus.clone(),
    });

    let workers = get_all_workers(inv_db.clone()).unwrap();
    let workers = workers
        .into_par_iter()
        .map(|worker| {
            let client = crate::pruntime::create_client(worker.endpoint.clone());
            match worker.pid {
                Some(pid) => {
                    let pool = match crate::inv_db::get_pool_by_pid(inv_db.clone(), pid) {
                        Ok(pool) => pool,
                        Err(err) => {
                            error!("Fail to get pool #{}. {}", pid, err);
                            None
                        },
                    };
                    let operator = match txm.clone().db.get_po(pid) {
                        Ok(po) => po.map(|po| po.operator()),
                        Err(err) => {
                            error!("Fail to get pool operator #{}. {}", pid, err);
                            None
                        },
                    };
                    (worker, pool, operator, client)
                },
                None => (worker, None, None, client),
            }
        })
        .collect::<Vec<_>>();

    for (worker, pool, operator, client) in workers {
        let _ = bus.send_processor_event(ProcessorEvent::AddWorker((
            worker,
            pool.map(|p| p.sync_only),
            operator,
            client,
        )));
    }

    let timer_future = {
        let bus = bus.clone();
        tokio::spawn(async move {
            loop {
                let _ = bus.send_processor_event(ProcessorEvent::Heartbeat);
                let nanos = 1_000_000_000 - Utc::now().nanosecond() % 1_000_000_000;
                tokio::time::sleep(std::time::Duration::from_nanos(nanos.into())).await;
            };
        })
    };

    let join_handle = try_join4(
        tokio::spawn(start_api_server(ctx.clone(), args.clone())),
        tokio::spawn(txm_handle),
        ds_join_handle,
        timer_future,
    );

    let mut processor = Processor::create(
        processor_rx,
        bus.clone(),
        txm.clone(),
        headers_db.clone(),
        dsm.clone(),
        &args,
    ).await;

    tokio::select! {
        _ = tokio::task::spawn_blocking(move || {
            processor.master_loop();
        }) => {}

        _ = message_master_loop(messages_rx, bus.clone(), dsm.clone(), txm.clone()) => {}

        _ = update_worker_status(ctx.clone(), worker_status_rx) => {}

        _ = crate::repository::keep_syncing_headers(bus.clone(), dsm.clone(), headers_db.clone()) => {}

        ret = join_handle => {
            info!("wm.join_handle: {:?}", ret);
        }
        /*
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
        */
    }
}

/*
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
 */
