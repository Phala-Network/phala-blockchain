use anyhow::{bail, Context, Result};
use log::{debug, error, info};
use pherry::headers_cache::BlockInfo;
use rocket::{get, response::status::NotFound, routes, State};

use scale::{Decode, Encode};

use crate::db::CacheDB;
use crate::BlockNumber;

struct App {
    db: CacheDB,
}

#[get("/state")]
fn state(app: &State<App>) -> String {
    let metadata = app.db.get_metadata().ok().flatten().unwrap_or_default();
    serde_json::to_string_pretty(&metadata).unwrap_or("{}".into())
}

#[get("/genesis/<block_number>")]
fn get_genesis(app: &State<App>, block_number: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    app.db
        .get_genesis(block_number)
        .ok_or_else(|| NotFound("genesis not found".into()))
}

#[get("/header/<block_number>")]
fn get_header(app: &State<App>, block_number: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    app.db
        .get_header(block_number)
        .ok_or_else(|| NotFound("header not found".into()))
}

#[get("/headers/<start>")]
fn get_headers(app: &State<App>, start: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    let mut headers = vec![];
    for block in start..start + 10000 {
        match app.db.get_header(block) {
            Some(data) => {
                let info = crate::cache::BlockInfo::decode(&mut &data[..])
                    .map_err(|_| NotFound("Codec error".into()))?;
                let end = info.justification.is_some();
                headers.push(info);
                if end {
                    break;
                }
            }
            None => {
                log::warn!("{} not found", block);
                return Err(NotFound("header not found".into()));
            }
        }
    }
    log::info!("Got {} headers", headers.len());
    Ok(headers.encode())
}

#[get("/parachain-headers/<start>/<count>")]
fn get_parachain_headers(
    app: &State<App>,
    start: BlockNumber,
    count: BlockNumber,
) -> Result<Vec<u8>, NotFound<String>> {
    let mut headers = vec![];
    for block in start..start + count {
        match app.db.get_para_header(block) {
            Some(data) => {
                use pherry::types::Header;
                let header =
                    Header::decode(&mut &data[..]).map_err(|_| NotFound("Codec error".into()))?;
                headers.push(header);
            }
            None => {
                log::warn!("header at {} not found", block);
                return Err(NotFound("header not found".into()));
            }
        }
    }
    log::info!("Got {} parachain headers", headers.len());
    Ok(headers.encode())
}

#[get("/storage-changes/<start>/<count>")]
fn get_storage_changes(
    app: &State<App>,
    start: BlockNumber,
    count: BlockNumber,
) -> Result<Vec<u8>, NotFound<String>> {
    let mut changes = vec![];
    for block in start..start + count {
        match app.db.get_storage_changes(block) {
            Some(data) => {
                let header = crate::cache::BlockHeaderWithChanges::decode(&mut &data[..])
                    .map_err(|_| NotFound("Codec error".into()))?;
                changes.push(header);
            }
            None => {
                log::warn!("changes at {} not found", block);
                return Err(NotFound("header not found".into()));
            }
        }
    }
    log::info!("Got {} storage changes", changes.len());
    Ok(changes.encode())
}

pub(crate) async fn serve(db: CacheDB) -> Result<()> {
    let _rocket = rocket::build()
        .manage(App { db })
        .mount(
            "/",
            routes![
                state,
                get_genesis,
                get_header,
                get_headers,
                get_parachain_headers,
                get_storage_changes,
            ],
        )
        .attach(phala_rocket_middleware::TimeMeter)
        .launch()
        .await?;
    Ok(())
}

async fn http_get(client: &reqwest::Client, url: &str) -> Result<Option<Vec<u8>>> {
    let response = client.get(url).send().await?;
    if response.status() == 404 {
        return Ok(None);
    }
    if !response.status().is_success() {
        bail!("Http status error {}", response.status());
    }
    let body = response.bytes().await?;
    Ok(Some(body.to_vec()))
}

pub(crate) async fn sync_from(
    db: CacheDB,
    base_uri: &str,
    check_interval: u64,
    genesis_block: BlockNumber,
) -> Result<()> {
    let mut metadata = db
        .get_metadata()
        .context("Failed to get metadata")?
        .unwrap_or_default();
    let highest = metadata.recent_imported.header.unwrap_or(genesis_block);

    let http_client = reqwest::Client::builder()
        .build()
        .context("Failed to build HTTP client")?;

    'sync_genesis: {
        if metadata.genesis.is_empty() {
            let url = format!("{base_uri}/genesis/{genesis_block}");
            let body = match http_get(&http_client, &url).await {
                Ok(Some(body)) => body,
                Ok(None) => {
                    info!("Genesis {genesis_block} not found in upstream cache");
                    break 'sync_genesis;
                }
                Err(err) => {
                    error!("Failed to sync genesis from {url}: {err:?}");
                    break 'sync_genesis;
                }
            };
            db.put_genesis(genesis_block, &body)
                .context("Failed to put genesis")?;
            metadata.put_genesis(genesis_block);
            db.put_metadata(&metadata)
                .context("Failed to put metadata")?;
            info!("Synced genesis block {genesis_block}");
        }
    }

    let mut next_block = highest + 1;
    loop {
        loop {
            info!("Syncing {next_block}");
            let url = format!("{base_uri}/headers/{next_block}");
            let body = match http_get(&http_client, &url).await {
                Ok(Some(body)) => body,
                Ok(None) => {
                    debug!("Block {next_block} not found in upstream cache");
                    break;
                }
                Err(err) => {
                    error!("Failed to sync blocks from {url}: {err:?}");
                    break;
                }
            };
            let headers: Vec<BlockInfo> = match Decode::decode(&mut &body[..]) {
                Ok(headers) => headers,
                Err(_) => {
                    error!("Failed to decode the received blocks");
                    break;
                }
            };
            for info in headers {
                db.put_header(info.header.number, &info.encode())
                    .context("Failed to put record to DB")?;
                metadata.update_header(info.header.number);
                next_block = info.header.number + 1;
            }
            db.put_metadata(&metadata)
                .context("Failed to update metadata")?;
            info!("Synced to {} from upstream cache", next_block - 1);
        }
        info!("Sleeping for {check_interval} seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(check_interval)).await;
    }
}
