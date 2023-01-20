use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::Context as _;
use log::{info, warn};
use scale::Encode;

use pherry::{headers_cache as cache, types::rpc::ExtraRpcExt};

use crate::{db::CacheDB, BlockNumber};

pub async fn run(
    db: CacheDB,
    node_uri: &str,
    para_node_uri: &str,
    check_interval: u64,
    justification_interval: BlockNumber,
    genesis_block: BlockNumber,
) -> anyhow::Result<()> {
    let mut metadata = db.get_metadata()?.unwrap_or_default();
    let highest = metadata.recent_imported.header.unwrap_or(genesis_block);
    let mut next_block = highest + 1;

    GENESIS.store(genesis_block, Ordering::Relaxed);

    loop {
        info!("Connecting to {node_uri}...");
        let api = match pherry::subxt_connect(node_uri).await {
            Ok(api) => api,
            Err(err) => {
                warn!("Failed to connect to {node_uri}: {err:?}");
                sleep(check_interval).await;
                continue;
            }
        };
        info!("Connecting to {para_node_uri}...");
        let para_api = match pherry::subxt_connect(para_node_uri).await {
            Ok(api) => api,
            Err(err) => {
                warn!("Failed to connect to {para_node_uri}: {err:?}");
                sleep(check_interval).await;
                continue;
            }
        };
        if !metadata.genesis.contains(&genesis_block) {
            info!("Fetching genesis at {}", genesis_block);
            let genesis = match cache::fetch_genesis_info(&api, genesis_block).await {
                Ok(genesis) => genesis,
                Err(err) => {
                    warn!("Failed to fetch genesis info: {err}");
                    sleep(check_interval).await;
                    continue;
                }
            };
            db.put_genesis(genesis_block, &genesis.encode())?;
            metadata.put_genesis(genesis_block);
            db.put_metadata(&metadata)?;
            info!("Got genesis at {}", genesis_block);
        }

        loop {
            info!("Trying to grab next={next_block}, just_interval={justification_interval}");
            let state = match api.extra_rpc().system_sync_state().await {
                Ok(state) => state,
                Err(err) => {
                    warn!("Failed to get node state: {err:?}");
                    sleep(check_interval).await;
                    break;
                }
            };
            info!("Node state: {state:?}");
            if (state.current_block as BlockNumber) < next_block + justification_interval {
                info!("Continue waiting for enough blocks in node");
                sleep(check_interval).await;
                continue;
            }

            info!("Grabbing headers...");
            let result = cache::grab_headers(
                &api,
                &para_api,
                next_block,
                u32::MAX,
                justification_interval,
                |info| {
                    if info.justification.is_some() {
                        info!("Got justification at {}", info.header.number);
                        LATEST_JUSTFICATION.store(info.header.number as _, Ordering::Relaxed);
                    }
                    db.put_header(info.header.number, &info.encode())
                        .context("Failed to put record to DB")?;
                    metadata.update_header(info.header.number);
                    db.put_metadata(&metadata)
                        .context("Failed to update metadata")?;
                    next_block = info.header.number + 1;
                    Ok(())
                },
            )
            .await;
            if let Err(err) = result {
                warn!("Failed to grab header from node: {err:?}");
                sleep(check_interval).await;
                break;
            }
            sleep(check_interval).await;
        }
    }
}

static GENESIS: AtomicU32 = AtomicU32::new(u32::MAX);
static LATEST_JUSTFICATION: AtomicU32 = AtomicU32::new(u32::MAX);

pub(crate) fn genesis_block() -> BlockNumber {
    GENESIS.load(Ordering::Relaxed)
}

pub(crate) fn latest_justification() -> BlockNumber {
    LATEST_JUSTFICATION.load(Ordering::Relaxed)
}

pub(crate) fn update_404_block(block: BlockNumber) {
    LATEST_JUSTFICATION.fetch_min(block.saturating_sub(1), Ordering::Relaxed);
}

async fn sleep(secs: u64) {
    info!("Sleeping for {secs} seconds...");
    tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
}
