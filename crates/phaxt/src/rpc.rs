use phala_node_rpc_ext_types::GetStorageChangesResponse;
use serde::{Deserialize, Serialize};
use serde_json::to_value as to_json_value;
use subxt::{
    sp_core::storage::{StorageData, StorageKey},
    Client, Error, RpcClient, Config
};

use crate::Hash;

pub trait ExtraRpcExt {
    fn extra_rpc(&self) -> ExtraRpcClient;
}

impl<C: Config> ExtraRpcExt for Client<C> {
    fn extra_rpc(&self) -> ExtraRpcClient {
        ExtraRpcClient {
            client: &self.rpc().client,
        }
    }
}

pub struct ExtraRpcClient<'a> {
    client: &'a RpcClient,
}

impl<'a> ExtraRpcClient<'a> {
    /// Query storage changes
    pub async fn get_storage_changes(
        &self,
        from: &Hash,
        to: &Hash,
    ) -> Result<GetStorageChangesResponse, Error> {
        let params = &[to_json_value(from)?, to_json_value(to)?];
        self.client
            .request("pha_getStorageChanges", params)
            .await
            .map_err(Into::into)
    }

    /// Returns the keys with prefix, leave empty to get all the keys
    pub async fn storage_pairs(
        &self,
        prefix: StorageKey,
        hash: Option<Hash>,
    ) -> Result<Vec<(StorageKey, StorageData)>, Error> {
        let params = &[to_json_value(prefix)?, to_json_value(hash)?];
        let data = self.client.request("state_getPairs", params).await?;
        Ok(data)
    }

    /// Fetch block syncing status
    pub async fn system_sync_state(&self) -> Result<SyncState, Error> {
        Ok(self.client.request("system_syncState", &[]).await?)
    }
}

/// System sync state for a Substrate-based runtime
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    /// Height of the block at which syncing started.
    pub starting_block: u64,
    /// Height of the current best block of the node.
    pub current_block: u64,
    /// Height of the highest block learned from the network. Missing if no block is known yet.
    #[serde(default = "Default::default")]
    pub highest_block: Option<u64>,
}
