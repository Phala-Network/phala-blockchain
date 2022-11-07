use anyhow::Context as _;
use log::{info, warn};

use pherry::headers_cache as cache;

use crate::db::CacheDB;

pub async fn run(
    db: CacheDB,
    node_uri: &str,
    para_node_uri: &str,
    interval: u64,
) -> anyhow::Result<()> {
    let Some(mut metadata) = db.get_metadata()? else {
        info!("No metadata in the DB, can not grab");
        return Ok(());
    };
    let Some(highest) = metadata.higest.header else {
        info!("There aren't any headers in the DB, can not grab");
        return Ok(());
    };

    let mut from_block = highest + 1;

    loop {
        info!("Sleeping for {interval} seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

        let Ok(api) = pherry::subxt_connect(node_uri).await else {
            warn!("Failed to connect to {node_uri}, try again later");
            continue;
        };
        let Ok(para_api) = pherry::subxt_connect(para_node_uri).await else {
            warn!("Failed to connect to {para_node_uri}, try again later");
            continue;
        };

        loop {
            info!("Starting to grab headers from {from_block}...");
            let mut buffer = vec![];
            let result =
                cache::grap_headers_to_file(&api, &para_api, from_block, 10, 0, &mut buffer).await;
            match result {
                Ok(count) => {
                    if count == 0 {
                        break;
                    }
                    let count = cache::read_items(&buffer[..], |record| {
                        let header = record.header().context("Failed to decode record header")?;
                        db.put_header(header.number, record.payload())
                            .context("Failed to put record to DB")?;
                        metadata.update_header(header.number);
                        Ok(false)
                    })?;
                    db.put_metadata(&metadata)
                        .context("Failed to update metadata")?;
                    from_block += count;
                }
                Err(err) => {
                    warn!("Failed to grab header from node: {err:?}");
                    break;
                }
            }
        }
    }
}
