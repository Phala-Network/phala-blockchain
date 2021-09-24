use super::*;
use phactory::{gk, BlockInfo, SideTaskManager, StorageExt};
use phala_mq::MessageDispatcher;
use phala_trie_storage::TrieStorage;
use phala_types::messaging::MiningInfoUpdateEvent;

use crate::types::Hashing;

struct ReplayFactory {
    storage: TrieStorage<Hashing>,
    recv_mq: MessageDispatcher,
    gk: gk::MiningFinance<ReplayMsgChannel>,
}

impl ReplayFactory {
    fn new(genesis_state: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        let mut recv_mq = MessageDispatcher::new();
        let mut storage = TrieStorage::default();
        storage.load(genesis_state.into_iter());
        let gk = gk::MiningFinance::new(&mut recv_mq, ReplayMsgChannel);
        Self {
            storage,
            recv_mq,
            gk,
        }
    }

    fn dispatch_block(&mut self, block: BlockWithChanges) -> Result<(), &'static str> {
        let (state_root, transaction) = self.storage.calc_root_if_changes(
            &block.storage_changes.main_storage_changes,
            &block.storage_changes.child_storage_changes,
        );
        let header = &block.block.block.header;

        if header.state_root != state_root {
            return Err("State root mismatch");
        }

        self.storage.apply_changes(state_root, transaction);
        self.handle_inbound_messages(header.number)?;
        Ok(())
    }

    fn handle_inbound_messages(&mut self, block_number: BlockNumber) -> Result<(), &'static str> {
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

        self.gk.process_messages(&mut block);

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
            info!("Report mining event: {:#?}", msg);
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

pub(super) async fn replay(client: &XtClient, genesis_block: BlockNumber) -> Result<()> {
    log::info!("Fetching genesis storage");
    let genesis_state = fetch_genesis_storage(client, genesis_block).await?;
    let mut factory = ReplayFactory::new(genesis_state);

    let mut block_number = genesis_block + 1;

    loop {
        log::info!("Fetching block {}", block_number);
        match get_block_with_storage_changes(client, Some(block_number)).await {
            Ok(block) => {
                log::info!("Replaying block {}", block_number);
                factory.dispatch_block(block).expect("Block is valid");
                block_number += 1;
            }
            Err(err) => {
                log::error!("{}", err);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }

    println!("pubkey,v_init,v,p_init,p_instant,total_slashed,total_slash_times,total_payout,total_payout_times");
    for (pubkey, state) in factory.gk.dump_workers_state() {
        let tk = match &state.tokenomic_info {
            Some(info) => info,
            None => continue,
        };
        println!(
            "{:?},{},{},{},{},{},{},{},{}",
            pubkey,
            tk.v_init,
            tk.v,
            tk.p_bench,
            tk.p_instant,
            tk.total_slash,
            tk.total_slash_count,
            tk.total_payout,
            tk.total_payout_count,
        );
    }
    Ok(())
}
