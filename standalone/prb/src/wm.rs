use crate::api::{start_api_server, WorkerStatus};
use crate::bus::Bus;
use crate::cli::WorkerManagerCliArgs;
use crate::repository::Repository;
use crate::datasource::setup_data_source_manager;
use crate::inv_db::{get_all_workers, setup_inventory_db, WrappedDb};
use crate::messages::{master_loop as message_master_loop, MessagesEvent};
use crate::pool_operator::PoolOperatorAccess;
use crate::processor::{Processor, ProcessorEvent};
use crate::tx::TxManager;
use crate::worker_status::{update_worker_status, WorkerStatusEvent};
use chrono::{Timelike, Utc};
use futures::future::{try_join4, try_join_all};
use log::{error, info};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex as TokioMutex};

pub struct WorkerManagerContext {
    pub inv_db: WrappedDb,
    pub worker_status_map: Arc<TokioMutex<HashMap<String, WorkerStatus>>>,
    pub txm: Arc<TxManager>,
    pub bus: Arc<Bus>,
}

pub type WrappedWorkerManagerContext = Arc<WorkerManagerContext>;

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

    let mut repository = Repository::create(
        bus.clone(),
        dsm.clone(),
        headers_db.clone(),
    ).await.unwrap();
    let _ = repository.background(true, args.verify_saved_headers).await.unwrap();

    if args.download_headers_only {
        headers_db.cancel_all_background_work(true);
        drop(headers_db);
        return;
    }

    let inv_db = setup_inventory_db(&args.db_path);
    let (txm, txm_handle) = TxManager::new(&args.db_path, dsm.clone()).expect("TxManager");
    let ctx = Arc::new(WorkerManagerContext {
        inv_db: inv_db.clone(),
        txm: txm.clone(),
        worker_status_map: Arc::new(TokioMutex::new(HashMap::new())),
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

        _ = repository.background(false, false) => {}

        ret = join_handle => {
            info!("wm.join_handle: {:?}", ret);
        }
    }
}