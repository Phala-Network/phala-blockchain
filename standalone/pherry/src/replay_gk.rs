use super::*;
use anyhow::Error;
use chrono::TimeZone as _;
use phactory::{gk, BlockInfo, SideTaskManager, StorageExt};
use phala_mq::MessageDispatcher;
use phala_trie_storage::TrieStorage;
use phala_types::{messaging::MiningInfoUpdateEvent, WorkerPublicKey};
use sqlx::types::Decimal;
use sqlx::{postgres::PgPoolOptions, Row};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::types::Hashing;

struct EventRecord {
    sequence: i64,
    pubkey: WorkerPublicKey,
    block_number: BlockNumber,
    time_ms: u64,
    event: gk::FinanceEvent,
    v: gk::FixedPoint,
    p: gk::FixedPoint,
}

pub struct ReplayFactory {
    event_seq: i64,
    current_block: BlockNumber,
    event_tx: mpsc::Sender<EventRecord>,
    storage: TrieStorage<Hashing>,
    recv_mq: MessageDispatcher,
    gk: gk::MiningFinance<ReplayMsgChannel>,
}

impl ReplayFactory {
    fn new(genesis_state: Vec<(Vec<u8>, Vec<u8>)>, event_tx: mpsc::Sender<EventRecord>) -> Self {
        let mut recv_mq = MessageDispatcher::new();
        let mut storage = TrieStorage::default();
        storage.load(genesis_state.into_iter());
        let gk = gk::MiningFinance::new(&mut recv_mq, ReplayMsgChannel);
        Self {
            event_seq: 0,
            current_block: 0,
            event_tx,
            storage,
            recv_mq,
            gk,
        }
    }

    async fn dispatch_block(&mut self, block: BlockWithChanges) -> Result<(), &'static str> {
        let (state_root, transaction) = self.storage.calc_root_if_changes(
            &block.storage_changes.main_storage_changes,
            &block.storage_changes.child_storage_changes,
        );
        let header = &block.block.block.header;

        if header.state_root != state_root {
            return Err("State root mismatch");
        }

        self.storage.apply_changes(state_root, transaction);
        self.handle_inbound_messages(header.number).await?;
        self.current_block = block.block.block.header.number;
        Ok(())
    }

    async fn handle_inbound_messages(
        &mut self,
        block_number: BlockNumber,
    ) -> Result<(), &'static str> {
        // Dispatch events
        let messages = self
            .storage
            .mq_messages()
            .map_err(|_| "Can not get mq messages from storage")?;

        self.recv_mq.reset_local_index();

        for message in messages {
            self.recv_mq.dispatch(message);
        }

        let now_ms = self
            .storage
            .timestamp_now()
            .ok_or_else(|| "No timestamp found in block")?;

        let mut block = BlockInfo {
            block_number,
            now_ms,
            storage: &self.storage,
            recv_mq: &mut self.recv_mq,
            side_task_man: &mut SideTaskManager::default(),
        };

        let seq = &mut self.event_seq;

        let mut records = vec![];

        self.gk.process_messages_with_event_listener(
            &mut block,
            &mut |event: gk::FinanceEvent, state: &gk::WorkerInfo| {
                let record = EventRecord {
                    sequence: *seq as _,
                    pubkey: state.pubkey().clone(),
                    block_number,
                    time_ms: now_ms,
                    event,
                    v: state.tokenomic_info().v,
                    p: state.tokenomic_info().p_instant,
                };
                records.push(record);
                *seq += 1;
            },
        );

        for record in records {
            match self.event_tx.send(record).await {
                Ok(()) => (),
                Err(err) => {
                    log::error!("Can not send event to replay: {}", err);
                }
            }
        }

        let n_unhandled = self.recv_mq.clear();
        if n_unhandled > 0 {
            warn!("There are {} unhandled messages dropped", n_unhandled);
        }

        Ok(())
    }
}

struct ReplayMsgChannel;

impl gk::MessageChannel for ReplayMsgChannel {
    fn push_message<M: codec::Encode + phala_types::messaging::BindTopic>(&self, message: M) {
        if let Ok(msg) = MiningInfoUpdateEvent::<BlockNumber>::decode(&mut &message.encode()[..]) {
            log::debug!("Report mining event: {:#?}", msg);
        }
    }

    fn set_dummy(&self, _dummy: bool) {}
}

pub async fn fetch_genesis_storage(
    client: &XtClient,
    pos: BlockNumber,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let pos = subxt::BlockNumber::from(NumberOrHex::Number(pos.into()));
    let hash = client.block_hash(Some(pos)).await?;
    let response = client.rpc.storage_pairs(StorageKey(vec![]), hash).await?;
    let storage = response.into_iter().map(|(k, v)| (k.0, v.0)).collect();
    Ok(storage)
}

async fn wait_for_block(client: &XtClient, block: BlockNumber) -> Result<()> {
    loop {
        let state = client.rpc.system_sync_state().await?;
        if state.current_block as BlockNumber >= block {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

mod httpserver {
    use super::*;
    use actix_web::{get, web, App, HttpResponse, HttpServer};

    struct AppState {
        factory: Arc<Mutex<ReplayFactory>>,
    }

    #[get("/worker-state/{pubkey}")]
    async fn get_worker_state(
        web::Path(pubkey): web::Path<String>,
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let factory = data.factory.lock().await;
        let pubkey = match AccountId32::from_str(pubkey.as_str()) {
            Ok(accid) => WorkerPublicKey(accid.into()),
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid pubkey"
                }));
            }
        };

        match factory.gk.worker_state(&pubkey) {
            None => HttpResponse::NotFound().json(serde_json::json!({
                "error": "Worker not found"
            })),
            Some(state) => HttpResponse::Ok().json(serde_json::json!({
                "current_block": factory.current_block,
                "benchmarking": state.bench_state.is_some(),
                "mining": state.mining_state.is_some(),
                "unresponsive": state.unresponsive,
                "last_heartbeat_for_block": state.last_heartbeat_for_block,
                "last_heartbeat_at_block": state.last_heartbeat_at_block,
                "waiting_heartbeats": state.waiting_heartbeats,
                "v": state.tokenomic_info.as_ref().map(|info| info.v.clone()),
                "v_init": state.tokenomic_info.as_ref().map(|info| info.v_init.clone()),
                "p_instant": state.tokenomic_info.as_ref().map(|info| info.p_instant.clone()),
                "p_init": state.tokenomic_info.as_ref().map(|info| info.p_bench.clone()),
            })),
        }
    }

    pub async fn serve(bind_addr: String, factory: Arc<Mutex<ReplayFactory>>) {
        HttpServer::new(move || {
            let factory = factory.clone();
            App::new()
                .data(AppState { factory })
                .service(get_worker_state)
        })
        .disable_signals()
        .bind(&bind_addr)
        .expect("Can not bind http server")
        .run()
        .await
        .expect("Http server failed");
    }
}

pub async fn replay(
    node_uri: String,
    genesis_block: BlockNumber,
    db_uri: String,
    bind_addr: String,
) -> Result<()> {
    let mut client = crate::subxt_connect(&node_uri)
        .await
        .expect("Failed to connect to substrate");
    log::info!("Connected to substrate at: {}", node_uri);

    let genesis_state = fetch_genesis_storage(&client, genesis_block).await?;
    let (event_tx, event_rx) = mpsc::channel(1024 * 5);

    let factory = Arc::new(Mutex::new(ReplayFactory::new(genesis_state, event_tx)));

    let _db_task = tokio::spawn(async move { save_data_task(event_rx, &db_uri).await });

    let _http_task = std::thread::spawn({
        let factory = factory.clone();
        move || {
            let mut system = actix_rt::System::new("api-server");
            system.block_on(httpserver::serve(bind_addr, factory))
        }
    });

    let mut block_number = genesis_block + 1;

    loop {
        loop {
            log::info!("Fetching block {}", block_number);
            match get_block_with_storage_changes(&client, Some(block_number)).await {
                Ok(block) => {
                    log::info!("Replaying block {}", block_number);
                    factory
                        .lock()
                        .await
                        .dispatch_block(block)
                        .await
                        .expect("Block is valid");
                    block_number += 1;
                }
                Err(err) => {
                    log::error!("{}", err);
                    if restart_required(&err) {
                        break;
                    }
                    if let Err(err) = wait_for_block(&client, block_number).await {
                        log::error!("{}", err);
                        if restart_required(&err) {
                            break;
                        }
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        }

        client = loop {
            log::info!("Reconnecting to substrate");
            let client = match crate::subxt_connect(&node_uri).await {
                Ok(client) => client,
                Err(err) => {
                    log::error!("Failed to connect to substrate: {}", err);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
            break client;
        }
    }
}

fn restart_required(error: &Error) -> bool {
    format!("{}", error).contains("restart required")
}

async fn save_data_task(mut rx: mpsc::Receiver<EventRecord>, uri: &str) {
    log::info!("Connecting to {}", uri);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(uri)
        .await
        .expect("Connect to database failed");

    let mut stopped = false;

    while !stopped {
        let mut records = vec![];
        loop {
            match tokio::time::timeout(Duration::from_secs(2), rx.recv()).await {
                Ok(Some(record)) => {
                    records.push(record);

                    const BATCH_SIZE: usize = 1000;
                    if records.len() >= BATCH_SIZE {
                        break;
                    }
                }
                Ok(None) => {
                    log::info!("data channel closed");
                    stopped = true;
                    break;
                }
                Err(_) => {
                    // Did not receive anything for 2 seconds,
                    break;
                }
            };
        }
        if !records.is_empty() {
            log::info!("Inserting {} records.", records.len());
            'try_insert: loop {
                match insert_records(&pool, &records).await {
                    Ok(()) => {
                        break;
                    }
                    Err(err) => {
                        log::error!("Insert {} records error.", records.len());
                        log::error!("{}", err);
                        match get_last_sequence(&pool).await {
                            Ok(last_sequence) => {
                                log::info!("last_sequence={}", last_sequence);
                                if last_sequence
                                    >= records.last().expect("records can not be empty").sequence
                                {
                                    log::info!("Insert succeeded, let's move on");
                                    break;
                                }
                                records.retain(|r| r.sequence > last_sequence);
                                log::info!("Insert records failed, try again");
                                continue 'try_insert;
                            }
                            Err(err) => {
                                // Error, let's try to insert again later.
                                let delay = 5;
                                log::error!("{}", err);
                                log::error!("Try again in {}s", delay);
                                tokio::time::sleep(Duration::from_secs(delay)).await;
                                continue 'try_insert;
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn insert_records(
    pool: &sqlx::Pool<sqlx::Postgres>,
    records: &[EventRecord],
) -> Result<(), sqlx::Error> {
    let mut sequences = vec![];
    let mut pubkeys = vec![];
    let mut block_numbers = vec![];
    let mut timestamps = vec![];
    let mut events = vec![];
    let mut vs = vec![];
    let mut ps = vec![];
    let mut payouts = vec![];

    for rec in records {
        sequences.push(rec.sequence);
        pubkeys.push(rec.pubkey.0.to_vec());
        block_numbers.push(rec.block_number);
        timestamps.push(chrono::Utc.timestamp_millis(rec.time_ms as _));
        events.push(rec.event.event_string());
        vs.push(cvt_fp(rec.v));
        ps.push(cvt_fp(rec.p));
        payouts.push(cvt_fp(rec.event.payout()));
    }

    sqlx::query(
        r#"
        INSERT INTO worker_finance_events
            (sequence, pubkey, block, time, event, v, p, payout)
        SELECT *
        FROM UNNEST($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(&sequences)
    .bind(&pubkeys)
    .bind(&block_numbers)
    .bind(&timestamps)
    .bind(&events)
    .bind(&vs)
    .bind(&ps)
    .bind(&payouts)
    .execute(pool)
    .await?;

    log::debug!("Inserted {} records.", records.len());

    Ok(())
}

fn cvt_fp(v: gk::FixedPoint) -> Decimal {
    Decimal::from_i128_with_scale((v * 10000000000).to_num(), 10)
}

async fn get_last_sequence(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<i64> {
    let latest_row =
        sqlx::query("SELECT sequence FROM worker_finance_events ORDER BY sequence DESC LIMIT 1")
            .fetch_one(pool)
            .await?;
    Ok(latest_row.get(0))
}
