mod data_persist;
mod httpserver;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Error;
use anyhow::Result;
use phactory::{gk, BlockInfo, SideTaskManager, StorageExt};
use phala_mq::MessageDispatcher;
use phala_mq::MqResult;
use phala_mq::Path;
use phala_trie_storage::TrieStorage;
use phala_types::WorkerPublicKey;
use phaxt::rpc::ExtraRpcExt as _;
use pherry::types::{
    phaxt, subxt, BlockNumber, BlockWithChanges, Hashing, NumberOrHex, ParachainApi, StorageKey,
};
use tokio::sync::{mpsc, Mutex};

use crate::Args;

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
    next_event_seq: i64,
    current_block: BlockNumber,
    event_tx: Option<mpsc::Sender<EventRecord>>,
    storage: TrieStorage<Hashing>,
    recv_mq: MessageDispatcher,
    gk: gk::MiningEconomics<ReplayMsgChannel>,
}

impl ReplayFactory {
    fn new(
        genesis_state: Vec<(Vec<u8>, Vec<u8>)>,
        event_tx: Option<mpsc::Sender<EventRecord>>,
    ) -> Self {
        let mut recv_mq = MessageDispatcher::new();
        let mut storage = TrieStorage::default();
        storage.load(genesis_state.into_iter());
        let gk = gk::MiningEconomics::new(&mut recv_mq, ReplayMsgChannel);
        Self {
            next_event_seq: 1,
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
            send_mq: &mut Default::default(),
            side_task_man: &mut SideTaskManager::default(),
        };

        let next_seq = &mut self.next_event_seq;

        let mut records = vec![];

        self.gk.process_messages_with_event_listener(
            &mut block,
            &mut |event: gk::FinanceEvent, state: &gk::WorkerInfo| {
                let record = EventRecord {
                    sequence: *next_seq as _,
                    pubkey: state.pubkey().clone(),
                    block_number,
                    time_ms: now_ms,
                    event,
                    v: state.tokenomic_info().v,
                    p: state.tokenomic_info().p_instant,
                };
                records.push(record);
                *next_seq += 1;
            },
        );

        if let Some(tx) = self.event_tx.as_ref() {
            for record in records {
                match tx.send(record).await {
                    Ok(()) => (),
                    Err(err) => {
                        log::error!("Can not send event to replay: {}", err);
                    }
                }
            }
        }

        let n_unhandled = self.recv_mq.clear();
        if n_unhandled > 0 {
            log::warn!("There are {} unhandled messages dropped", n_unhandled);
        }

        Ok(())
    }
}

struct ReplayMsgChannel;

impl phala_mq::traits::MessageChannel for ReplayMsgChannel {
    fn push_data(&self, _data: Vec<u8>, _to: impl Into<Path>, _hash: phala_mq::MqHash) {}

    fn make_appointment(&self) -> MqResult<u64> {
        Ok(0)
    }
}

pub async fn fetch_genesis_storage(
    api: &ParachainApi,
    pos: BlockNumber,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let pos = subxt::BlockNumber::from(NumberOrHex::Number(pos.into()));
    let hash = api.client.rpc().block_hash(Some(pos)).await?;
    let response = api
        .client
        .extra_rpc()
        .storage_pairs(StorageKey(vec![]), hash)
        .await?;
    let storage = response.into_iter().map(|(k, v)| (k.0, v.0)).collect();
    Ok(storage)
}

async fn finalized_number(api: &ParachainApi) -> Result<BlockNumber> {
    let hash = api.client.rpc().finalized_head().await?;
    let header = api.client.rpc().header(Some(hash)).await?;
    Ok(header.ok_or(anyhow::anyhow!("Header not found"))?.number)
}

async fn wait_for_block(
    api: &ParachainApi,
    block: BlockNumber,
    assume_finalized: u32,
) -> Result<()> {
    loop {
        let finalized = finalized_number(api).await.unwrap_or(0);
        let state = api.client.extra_rpc().system_sync_state().await?;
        if block <= state.current_block as BlockNumber && block <= finalized.max(assume_finalized) {
            return Ok(());
        }
        log::info!(
            "Waiting for {} to be finalized. (finalized={}, assume_finalized={}, latest={})",
            block,
            finalized,
            assume_finalized,
            state.current_block
        );
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

pub async fn replay(args: Args) -> Result<()> {
    let db_uri = args.persist_events_to;
    let bind_addr = args.bind_addr;
    let assume_finalized = args.assume_finalized;

    let mut api: ParachainApi = pherry::subxt_connect(&args.node_uri)
        .await
        .expect("Failed to connect to substrate")
        .into();
    log::info!("Connected to substrate at: {}", args.node_uri);

    let genesis_state = fetch_genesis_storage(&api, args.start_at).await?;
    let event_tx = if !db_uri.is_empty() {
        let (event_tx, event_rx) = mpsc::channel(1024 * 5);
        let _db_task =
            tokio::spawn(async move { data_persist::run_persist(event_rx, &db_uri).await });
        Some(event_tx)
    } else {
        None
    };
    let factory = Arc::new(Mutex::new(ReplayFactory::new(genesis_state, event_tx)));

    let _http_task = std::thread::spawn({
        let factory = factory.clone();
        move || {
            let mut system = actix_rt::System::new("api-server");
            system.block_on(httpserver::serve(bind_addr, factory))
        }
    });

    let mut block_number = args.start_at + 1;

    loop {
        loop {
            if let Err(err) = wait_for_block(&api, block_number, assume_finalized).await {
                log::error!("{}", err);
                if restart_required(&err) {
                    break;
                }
            }
            log::info!("Fetching block {}", block_number);
            match pherry::get_block_with_storage_changes(&api, Some(block_number)).await {
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
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }

        api = loop {
            log::info!("Reconnecting to substrate");
            let api = match pherry::subxt_connect(&args.node_uri).await {
                Ok(client) => client.into(),
                Err(err) => {
                    log::error!("Failed to connect to substrate: {}", err);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
            break api;
        }
    }
}

fn restart_required(error: &Error) -> bool {
    format!("{}", error).contains("restart required")
}
