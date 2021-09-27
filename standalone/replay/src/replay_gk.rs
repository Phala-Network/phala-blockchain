mod data_persist;
mod httpserver;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Error;
use anyhow::Result;
use parity_scale_codec::{Decode, Encode};
use phactory::{gk, BlockInfo, SideTaskManager, StorageExt};
use phala_mq::MessageDispatcher;
use phala_trie_storage::TrieStorage;
use phala_types::{messaging::MiningInfoUpdateEvent, WorkerPublicKey};
use pherry::chain_client::StorageKey;
use pherry::types::{BlockNumber, BlockWithChanges, Hashing, NumberOrHex, XtClient};
use tokio::sync::{mpsc, Mutex};

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
            log::warn!("There are {} unhandled messages dropped", n_unhandled);
        }

        Ok(())
    }
}

struct ReplayMsgChannel;

impl gk::MessageChannel for ReplayMsgChannel {
    fn push_message<M: Encode + phala_types::messaging::BindTopic>(&self, message: M) {
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

pub async fn replay(
    node_uri: String,
    genesis_block: BlockNumber,
    db_uri: String,
    bind_addr: String,
) -> Result<()> {
    let mut client = pherry::subxt_connect(&node_uri)
        .await
        .expect("Failed to connect to substrate");
    log::info!("Connected to substrate at: {}", node_uri);

    let genesis_state = fetch_genesis_storage(&client, genesis_block).await?;
    let (event_tx, event_rx) = mpsc::channel(1024 * 5);

    let factory = Arc::new(Mutex::new(ReplayFactory::new(genesis_state, event_tx)));

    let _db_task = tokio::spawn(async move { data_persist::run_persist(event_rx, &db_uri).await });

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
            match pherry::get_block_with_storage_changes(&client, Some(block_number)).await {
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
            let client = match pherry::subxt_connect(&node_uri).await {
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
