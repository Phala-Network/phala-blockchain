use std::marker::PhantomData;
use std::sync::Arc;

use jsonrpc_derive::rpc;
use mq_seq::Error as MqSeqError;
use pallet_mq_runtime_api::MqApi;
use sc_client_api::blockchain::{HeaderBackend, HeaderMetadata};
use sc_client_api::{backend, Backend, BlockBackend, StorageProvider};
use sc_transaction_pool_api::{InPoolTransaction, TransactionPool};
use serde::{Deserialize, Serialize};
use sp_api::{ApiExt, Core, ProvideRuntimeApi, StateBackend};
use sp_runtime::traits::Header;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::fmt::Display;
use storage_changes::Error as StorageChangesError;

pub use storage_changes::{GetStorageChangesResponse, MakeInto, StorageChanges};

mod mq_seq;
mod storage_changes;

/// Base code for all errors.
const CUSTOM_RPC_ERROR: i64 = 10000;

#[rpc]
pub trait NodeRpcExtApi<BlockHash> {
    /// Return the storage changes made by each block one by one from `from` to `to`(both inclusive).
    /// To get better performance, the client should limit the amount of requested block properly.
    /// 100 blocks for each call should be OK. REQUESTS FOR TOO LARGE NUMBER OF BLOCKS WILL BE REJECTED.
    #[rpc(name = "pha_getStorageChanges")]
    fn get_storage_changes(
        &self,
        from: BlockHash,
        to: BlockHash,
    ) -> Result<GetStorageChangesResponse, StorageChangesError>;

    /// Return the next mq sequence number for given sender which take the ready transactions in count.
    #[rpc(name = "pha_getMqNextSequence")]
    fn get_mq_seq(&self, sender_hex: String) -> Result<u64, MqSeqError>;
}

/// Stuffs for custom RPC
struct NodeRpcExt<BE, Block: BlockT, Client, P> {
    client: Arc<Client>,
    backend: Arc<BE>,
    is_archive_mode: bool,
    pool: Arc<P>,
    _phantom: PhantomData<Block>,
}

impl<BE, Block: BlockT, Client, P> NodeRpcExt<BE, Block, Client, P> {
    fn new(client: Arc<Client>, backend: Arc<BE>, is_archive_mode: bool, pool: Arc<P>) -> Self {
        Self {
            client,
            backend,
            is_archive_mode,
            pool,
            _phantom: Default::default(),
        }
    }
}

impl<BE: 'static, Block: BlockT, Client: 'static, P> NodeRpcExtApi<Block::Hash>
    for NodeRpcExt<BE, Block, Client, P>
where
    BE: Backend<Block>,
    Client: StorageProvider<Block, BE>
        + HeaderBackend<Block>
        + BlockBackend<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>
        + ProvideRuntimeApi<Block>,
    Client::Api:
        sp_api::Metadata<Block> + ApiExt<Block, StateBackend = backend::StateBackendFor<BE, Block>>,
    Client::Api: MqApi<Block>,
    Block: BlockT + 'static,
    <<Block as BlockT>::Header as Header>::Number: Into<u64>,
    P: TransactionPool + 'static,
{
    fn get_storage_changes(
        &self,
        from: Block::Hash,
        to: Block::Hash,
    ) -> Result<GetStorageChangesResponse, StorageChangesError> {
        if !self.is_archive_mode {
            Err(StorageChangesError::Unavailable(
                r#"Add "--pruning=archive" to the command line to enable this RPC"#.into(),
            ))
        } else {
            storage_changes::get_storage_changes(
                self.client.as_ref(),
                self.backend.as_ref(),
                from,
                to,
            )
        }
    }

    fn get_mq_seq(&self, sender_hex: String) -> Result<u64, MqSeqError> {
        mq_seq::get_mq_seq(&*self.client, &self.pool, sender_hex)
    }
}

pub fn extend_rpc<Client, BE, Block, P>(
    io: &mut jsonrpc_core::IoHandler<sc_rpc::Metadata>,
    client: Arc<Client>,
    backend: Arc<BE>,
    is_archive_mode: bool,
    pool: Arc<P>,
) where
    BE: Backend<Block> + 'static,
    Client: StorageProvider<Block, BE>
        + HeaderBackend<Block>
        + BlockBackend<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>
        + ProvideRuntimeApi<Block>
        + 'static,
    Block: BlockT + 'static,
    Client::Api:
        sp_api::Metadata<Block> + ApiExt<Block, StateBackend = backend::StateBackendFor<BE, Block>>,
    Client::Api: MqApi<Block>,
    <<Block as BlockT>::Header as Header>::Number: Into<u64>,
    P: TransactionPool + 'static,
{
    io.extend_with(NodeRpcExtApi::to_delegate(NodeRpcExt::new(
        client,
        backend,
        is_archive_mode,
        pool,
    )));
}
