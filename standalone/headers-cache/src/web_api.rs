use std::pin::pin;

use anyhow::{bail, Context, Result};
use log::{debug, error, info};
use pherry::{
    headers_cache::{read_items_stream, BlockInfo},
    types::Header,
};
use rand::Rng;
use rocket::{
    data::ToByteUnit,
    futures::StreamExt,
    get, put,
    response::status::{BadRequest, NotFound},
    routes, Data, State,
};

use scale::{Decode, Encode};

use super::Serve as ServeConfig;
use crate::{db::CacheDB, BlockNumber};
use auth::Authorized;

mod auth;

struct App {
    db: CacheDB,
    config: ServeConfig,
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
fn get_headers(app: &State<App>, start: BlockNumber) -> Result<Vec<u8>, NotFound<()>> {
    let latest_just = crate::grab::latest_justification();
    if start > latest_just {
        log::debug!("No more justification yet");
        return Err(NotFound(()));
    }
    let mut headers = vec![];
    for block in start..start + 10000 {
        match app.db.get_header(block) {
            Some(data) => {
                let info = crate::cache::BlockInfo::decode(&mut &data[..]).map_err(|_| {
                    log::error!("Failed to decode block fetched from db");
                    NotFound(())
                })?;
                let end = info.justification.is_some();
                headers.push(info);
                if end {
                    break;
                }
            }
            None => {
                if start >= crate::grab::genesis_block() {
                    crate::grab::update_404_block(start);
                }
                if block == start {
                    log::debug!("{start} not found");
                    return Err(NotFound(()));
                } else {
                    log::debug!("Justification not found till block {block}");
                    return Err(NotFound(()));
                }
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
                let header =
                    Header::decode(&mut &data[..]).map_err(|_| NotFound("Codec error".into()))?;
                headers.push(header);
            }
            None => {
                log::warn!("Header at {} not found", block);
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
                log::warn!("Changes at {} not found", block);
                return Err(NotFound("header not found".into()));
            }
        }
    }
    log::info!("Got {} storage changes", changes.len());
    Ok(changes.encode())
}

async fn process_items(
    app: &State<App>,
    data: Data<'_>,
    handler: impl Fn(&State<App>, BlockNumber, &[u8]),
) -> Result<(), BadRequest<String>> {
    let input = data.open(10.gibibytes());
    let mut stream = pin!(read_items_stream(input));
    while let Some(result) = stream.next().await {
        match result {
            Ok(record) => {
                let number = record
                    .header()
                    .map_err(|e| BadRequest(Some(format!("Decode error: {e}"))))?
                    .number;
                handler(app, number, record.payload());
            }
            Err(e) => return Err(BadRequest(Some(format!("Decode error: {e}")))),
        }
    }
    Ok(())
}

#[put("/headers", data = "<data>")]
async fn put_headers(
    _auth: Authorized,
    app: &State<App>,
    data: Data<'_>,
) -> Result<(), BadRequest<String>> {
    process_items(app, data, |app, number, data| {
        log::info!("Importing header {}", number);
        app.db
            .put_header(number, data)
            .expect("Failed to put headers into DB");
    })
    .await
}

#[put("/parachain-headers", data = "<data>")]
async fn put_parachain_headers(
    _auth: Authorized,
    app: &State<App>,
    data: Data<'_>,
) -> Result<(), BadRequest<String>> {
    process_items(app, data, |app, number, data| {
        log::info!("Importing parachain header {}", number);
        app.db
            .put_para_header(number, data)
            .expect("Failed to put para headers into DB");
    })
    .await
}

#[put("/storage-changes", data = "<data>")]
async fn put_storage_changes(
    _auth: Authorized,
    app: &State<App>,
    data: Data<'_>,
) -> Result<(), BadRequest<String>> {
    process_items(app, data, |app, number, data| {
        log::info!("Importing changes {}", number);
        app.db
            .put_storage_changes(number, data)
            .expect("Failed to put storage changes into DB");
    })
    .await
}

#[get("/check?<chain>&<from>&<to>&<count>")]
async fn api_check_blocks(
    _auth: Authorized,
    app: &State<App>,
    chain: &str,
    from: BlockNumber,
    to: Option<BlockNumber>,
    count: Option<BlockNumber>,
) -> Result<String, String> {
    if chain == "state" {
        let mismatches = crate::grab::check_and_fix_storages_changes(
            &app.db,
            None,
            &app.config,
            from,
            to,
            count,
            false,
        )
        .await
        .map_err(|e| e.to_string())?;
        if mismatches == 0 {
            Ok("No mismatches".into())
        } else {
            Ok(format!("Mismatches: {:?}", mismatches))
        }
    } else {
        crate::grab::check_and_fix_headers(&app.db, &app.config, chain, from, to, count)
            .await
            .map_err(|e| e.to_string())
    }
}

pub(crate) async fn serve(db: CacheDB, config: ServeConfig, token: Option<String>) -> Result<()> {
    let token = token.unwrap_or_else(|| {
        let token: [u8; 16] = rand::thread_rng().gen();
        let token = hex::encode(token);
        log::warn!("No token provided, generated a random one: {}", token);
        token
    });
    let _ = rocket::build()
        .manage(App {
            db: db.clone(),
            config: config.clone(),
        })
        .manage(auth::Token { value: token })
        .mount(
            "/",
            routes![
                state,
                get_genesis,
                get_header,
                get_headers,
                get_parachain_headers,
                get_storage_changes,
                put_headers,
                put_parachain_headers,
                put_storage_changes,
                api_check_blocks,
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
