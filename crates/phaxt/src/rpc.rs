use phala_node_rpc_ext_types::{GetStorageChangesResponse, GetStorageChangesResponseWithRoot};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::to_value as to_json_value;
use subxt::{
    rpc::{rpc_params, RpcClient},
    Config, Error, OnlineClient,
};

pub use sp_core::storage::{StorageData, StorageKey};

pub trait ExtraRpcExt {
    type Config: Config;
    fn extra_rpc(&self) -> ExtraRpcClient<Self::Config>;
}

impl<C: Config> ExtraRpcExt for OnlineClient<C> {
    type Config = C;
    fn extra_rpc(&self) -> ExtraRpcClient<Self::Config> {
        ExtraRpcClient {
            client: self.rpc(),
            _config: Default::default(),
        }
    }
}

pub struct ExtraRpcClient<'a, Config> {
    client: &'a RpcClient,
    _config: std::marker::PhantomData<Config>,
}

impl<'a, T: Config> ExtraRpcClient<'a, T> {
    /// Query storage changes
    pub async fn get_storage_changes(
        &self,
        from: &T::Hash,
        to: &T::Hash,
    ) -> Result<GetStorageChangesResponse, Error> {
        let params = rpc_params![to_json_value(from)?, to_json_value(to)?];
        self.client
            .request("pha_getStorageChanges", params)
            .await
            .map_err(Into::into)
    }

    /// Query storage changes with root
    pub async fn get_storage_changes_with_root(
        &self,
        from: &T::Hash,
        to: &T::Hash,
    ) -> Result<GetStorageChangesResponseWithRoot, Error> {
        let params = rpc_params![to_json_value(from)?, to_json_value(to)?];
        self.client
            .request("pha_getStorageChangesWithRoot", params)
            .await
            .map_err(Into::into)
    }

    /// Returns the keys with prefix, leave empty to get all the keys
    pub async fn storage_pairs(
        &self,
        prefix: StorageKey,
        hash: Option<T::Hash>,
    ) -> Result<Vec<(StorageKey, StorageData)>, Error> {
        let params = rpc_params![to_json_value(prefix)?, to_json_value(hash)?];
        let data = self.client.request("state_getPairs", params).await?;
        Ok(data)
    }

    /// Fetch block syncing status
    pub async fn system_sync_state(&self) -> Result<SyncState, Error> {
        self.client.request("system_syncState", rpc_params![]).await
    }
}

impl<'a, T: Config> ExtraRpcClient<'a, T>
where
    T::AccountId: Serialize,
    T::Index: DeserializeOwned,
{
    /// Reads the next nonce of an account, considering the pending extrinsics in the txpool
    pub async fn account_nonce(&self, account: &T::AccountId) -> Result<T::Index, Error> {
        let params = rpc_params![to_json_value(account)?];
        self.client.request("system_accountNextIndex", params).await
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
