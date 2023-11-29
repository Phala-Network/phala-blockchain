use anyhow::Result;
use phactory_api::blocks::BlockHeaderWithChanges;
use phaxt::{BlockNumber, RpcClient};
use tokio::task::JoinHandle;

struct StoragePrefetchState {
    from: BlockNumber,
    to: BlockNumber,
    handle: JoinHandle<Result<Vec<BlockHeaderWithChanges>>>,
}

pub struct PrefetchClient {
    prefetching_storage_changes: Option<StoragePrefetchState>,
}

impl PrefetchClient {
    pub fn new() -> Self {
        Self {
            prefetching_storage_changes: None,
        }
    }

    pub async fn fetch_storage_changes(
        &mut self,
        client: &RpcClient,
        cache: Option<&crate::CacheClient>,
        from: BlockNumber,
        to: BlockNumber,
    ) -> Result<Vec<BlockHeaderWithChanges>> {
        let count = to + 1 - from;
        let result = if let Some(state) = self.prefetching_storage_changes.take() {
            if state.from == from && state.to == to {
                log::info!("use prefetched storage changes ({from}-{to})",);
                state.handle.await?.ok()
            } else {
                log::info!(
                    "cancelling the prefetch ({}-{}), requesting ({from}-{to})",
                    state.from,
                    state.to,
                );
                state.handle.abort();
                None
            }
        } else {
            None
        };

        let result = if let Some(result) = result {
            result
        } else {
            crate::fetch_storage_changes(client, cache, from, to).await?
        };
        let next_from = from + count;
        let next_to = next_from + count - 1;
        let client = client.clone();
        let cache = cache.cloned();
        self.prefetching_storage_changes = Some(StoragePrefetchState {
            from: next_from,
            to: next_to,
            handle: tokio::spawn(async move {
                log::info!("prefetching ({next_from}-{next_to})");
                crate::fetch_storage_changes(&client, cache.as_ref(), next_from, next_to)
                    .await
            }),
        });
        Ok(result)
    }
}
