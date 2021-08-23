use std::marker::PhantomData;
use std::sync::Arc;

use jsonrpc_derive::rpc;
use node_rpc::IoHandler;
use sc_client_api::blockchain::{HeaderBackend, HeaderMetadata};
use sc_client_api::{backend, Backend, BlockBackend, StorageProvider};
use serde::{Deserialize, Serialize};
use sp_api::{ApiExt, Core, ProvideRuntimeApi, StateBackend};
use sp_runtime::traits::Header;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::fmt::Display;
use storage_changes::Error as StorageChangesError;

pub use storage_changes::{GetStorageChangesResponse, StorageChanges, MakeInto};

mod storage_changes;
mod mq_seq;

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
}

/// Stuffs for custom RPC
struct NodeRpcExt<BE, Block: BlockT, Client> {
    client: Arc<Client>,
    backend: Arc<BE>,
    is_archive_mode: bool,
    _phantom: PhantomData<Block>,
}

impl<BE, Block: BlockT, Client> NodeRpcExt<BE, Block, Client> {
    fn new(client: Arc<Client>, backend: Arc<BE>, is_archive_mode: bool) -> Self {
        Self {
            client,
            backend,
            is_archive_mode,
            _phantom: Default::default(),
        }
    }
}

impl<BE: 'static, Block: BlockT, Client: 'static> NodeRpcExtApi<Block::Hash>
    for NodeRpcExt<BE, Block, Client>
where
    BE: Backend<Block>,
    Client: StorageProvider<Block, BE>
        + HeaderBackend<Block>
        + BlockBackend<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>
        + ProvideRuntimeApi<Block>,
    Client::Api:
        sp_api::Metadata<Block> + ApiExt<Block, StateBackend = backend::StateBackendFor<BE, Block>>,
    Block: BlockT + 'static,
    <<Block as BlockT>::Header as Header>::Number: Into<u64>,
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
}

pub fn extend_rpc<Client, BE, Block>(
    io: &mut IoHandler,
    client: Arc<Client>,
    backend: Arc<BE>,
    is_archive_mode: bool,
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
    <<Block as BlockT>::Header as Header>::Number: Into<u64>,
{
    io.extend_with(NodeRpcExtApi::to_delegate(NodeRpcExt::new(
        client,
        backend,
        is_archive_mode,
    )));
}
