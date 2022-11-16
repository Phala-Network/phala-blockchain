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
) -> anyhow::Result<()> {
    let Some(mut metadata) = db.get_metadata()? else {
        anyhow::bail!("No metadata in the DB, can not grab");
    };
    let Some(highest) = metadata.recent_imported.header else {
        anyhow::bail!("There aren't any headers in the DB, can not grab");
    };

    let mut next_block = highest + 1;

    loop {
        info!("Connecting to {node_uri}...");
        let Ok(api) = pherry::subxt_connect(node_uri).await else {
            warn!("Failed to connect to {node_uri}, try again later");
            sleep(check_interval).await;
            continue;
        };

        info!("Connecting to {para_node_uri}...");
        let Ok(para_api) = pherry::subxt_connect(para_node_uri).await else {
            warn!("Failed to connect to {para_node_uri}, try again later");
            sleep(check_interval).await;
            continue;
        };

        loop {
            info!("Trying to grab next={next_block}, just_interval={justification_interval}");
            let state = api.extra_rpc().system_sync_state().await?;
            info!("Node state: {state:?}");
            if (state.current_block as BlockNumber) < next_block + justification_interval {
                info!("Continue waiting for enough blocks in node");
                sleep(check_interval).await;
                continue;
            }

            info!("Grabbing...");
            let result = cache::grab_headers(
                &api,
                &para_api,
                next_block,
                u32::MAX,
                justification_interval,
                |info| {
                    if info.justification.is_some() {
                        info!("Got justification at {}", info.header.number);
                    }
                    db.put_header(info.header.number, &info.encode())
                        .context("Failed to put record to DB")?;
                    metadata.update_header(info.header.number);
                    next_block = info.header.number + 1;
                    Ok(())
                },
            )
            .await;
            db.put_metadata(&metadata)
                .context("Failed to update metadata")?;
            if let Err(err) = result {
                warn!("Failed to grab header from node: {err:?}");
                sleep(check_interval).await;
                break;
            }
            sleep(check_interval).await;
        }
    }
}

async fn sleep(secs: u64) {
    info!("Sleeping for {secs} seconds...");
    tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
}
