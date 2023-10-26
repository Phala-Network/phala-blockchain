use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{anyhow, bail, Context as _, Result};
use log::{error, info, warn};
use scale::{Decode, Encode};

use pherry::{
    headers_cache as cache,
    types::{phaxt::ChainApi, Header},
};

use crate::{
    db::{CacheDB, Metadata},
    BlockNumber, Serve,
};

pub(crate) async fn run(db: CacheDB, config: Serve) -> Result<()> {
    let mut metadata = db.get_metadata()?.unwrap_or_default();
    let mut next_header = match metadata.higest.header {
        Some(highest) => highest + 1,
        None => config.genesis_block,
    };
    let mut next_para_header = metadata
        .higest
        .para_header
        .map(|i| i + 1)
        .unwrap_or_default();
    let mut next_delta = metadata
        .higest
        .storage_changes
        .map(|i| i + 1)
        .unwrap_or_default();

    GENESIS.store(config.genesis_block, Ordering::Relaxed);

    loop {
        if let Err(err) = Crawler::grab(
            &config,
            &db,
            &mut metadata,
            &mut next_header,
            &mut next_para_header,
            &mut next_delta,
        )
        .await
        {
            error!("Error: {err:?}");
        }
        sleep(config.interval).await;
    }
}

struct Crawler<'c> {
    config: &'c Serve,
    db: &'c CacheDB,
    metadata: &'c mut Metadata,
    api: ChainApi,
    para_api: ChainApi,
    next_header: Option<&'c mut BlockNumber>,
    next_para_header: Option<&'c mut BlockNumber>,
    next_delta: Option<&'c mut BlockNumber>,
}

impl<'c> Crawler<'c> {
    async fn grab<'p>(
        config: &'c Serve,
        db: &'c CacheDB,
        metadata: &'c mut Metadata,
        next_header: &'c mut BlockNumber,
        next_para_header: &'c mut BlockNumber,
        next_delta: &'c mut BlockNumber,
    ) -> Result<()> {
        info!("Connecting to {}...", config.node_uri);
        let api = pherry::subxt_connect(&config.node_uri)
            .await
            .context(format!("Failed to connect to {}", config.node_uri))?;
        info!("Connecting to {}...", config.para_node_uri);
        let para_api = pherry::subxt_connect(&config.para_node_uri)
            .await
            .context(format!("Failed to connect to {}", config.para_node_uri))?;
        if !metadata.genesis.contains(&config.genesis_block) {
            info!("Fetching genesis at {}", config.genesis_block);
            let genesis = cache::fetch_genesis_info(&api, config.genesis_block)
                .await
                .context("Failed to fetch genesis info")?;
            db.put_genesis(config.genesis_block, &genesis.encode())?;
            metadata.put_genesis(config.genesis_block);
            db.put_metadata(metadata)?;
            info!("Got genesis at {}", config.genesis_block);
        }
        Self {
            config,
            db,
            metadata,
            api,
            para_api,
            next_header: config.grab_headers.then_some(next_header),
            next_para_header: config.grab_para_headers.then_some(next_para_header),
            next_delta: config.grab_storage_changes.then_some(next_delta),
        }
        .run()
        .await
    }

    async fn finalized_header_number(&self, para: bool) -> Result<BlockNumber> {
        let api = if para { &self.para_api } else { &self.api };
        let hash = api.rpc().finalized_head().await?;
        let header = api.rpc().header(Some(hash)).await?;
        let header_number = header.map(|h| h.number).unwrap_or_default();
        Ok(header_number)
    }

    async fn grab_headers(&mut self) -> Result<()> {
        let latest_finalized = self.finalized_header_number(false).await?;
        let Some(next_header) = self.next_header.as_deref_mut() else {
            return Ok(());
        };
        info!("Relaychain finalized: {latest_finalized}");
        if latest_finalized < *next_header + self.config.justification_interval {
            info!("No enough relaychain headers in node");
            return Ok(());
        }

        info!("Grabbing headers start from {next_header}...");
        cache::grab_headers(
            &self.api,
            &self.para_api,
            *next_header,
            u32::MAX,
            self.config.justification_interval,
            |info| {
                if info.justification.is_some() {
                    info!("Got justification at {}", info.header.number);
                    LATEST_JUSTFICATION.store(info.header.number as _, Ordering::Relaxed);
                }
                self.db
                    .put_header(info.header.number, &info.encode())
                    .context("Failed to put record to DB")?;
                self.metadata.update_header(info.header.number);
                self.db
                    .put_metadata(self.metadata)
                    .context("Failed to update metadata")?;
                *next_header = info.header.number + 1;
                Ok(())
            },
        )
        .await
        .context("Failed to grab headers from node")?;
        Ok(())
    }

    async fn grab_para_headers(&mut self) -> Result<()> {
        let latest_finalized = self.finalized_header_number(true).await?;
        let Some(next_para_header) = self.next_para_header.as_deref_mut() else {
            return Ok(());
        };
        if latest_finalized < *next_para_header {
            return Ok(());
        }
        let count = latest_finalized - *next_para_header + 1;
        info!("Grabbing {count} parachain headers start from {next_para_header}...");
        cache::grab_para_headers(&self.para_api, *next_para_header, count, |info| {
            self.db
                .put_para_header(info.number, &info.encode())
                .context("Failed to put record to DB")?;
            self.metadata.update_para_header(info.number);
            self.db
                .put_metadata(self.metadata)
                .context("Failed to update metadata")?;
            *next_para_header = info.number + 1;
            Ok(())
        })
        .await
        .context("Failed to grab para headers from node")?;
        Ok(())
    }

    async fn grab_storage_changes(&mut self) -> Result<()> {
        let latest_finalized = self.finalized_header_number(true).await?;
        let Some(next_delta) = self.next_delta.as_deref_mut() else {
            return Ok(());
        };
        if latest_finalized < *next_delta {
            return Ok(());
        }
        let count = latest_finalized - *next_delta + 1;
        info!("Grabbing {count} storage changes start from {next_delta}...",);
        cache::grab_storage_changes(
            &self.para_api,
            *next_delta,
            count,
            self.config.grab_storage_changes_batch,
            !self.config.no_state_root,
            |info| {
                self.db
                    .put_storage_changes(info.block_header.number, &info.encode())
                    .context("Failed to put record to DB")?;
                self.metadata
                    .update_storage_changes(info.block_header.number);
                self.db
                    .put_metadata(self.metadata)
                    .context("Failed to update metadata")?;
                *next_delta = info.block_header.number + 1;
                Ok(())
            },
        )
        .await
        .context("Failed to grab storage changes from node")?;
        Ok(())
    }

    async fn continue_check_headers(&mut self) -> Result<()> {
        let db = self.db;
        let config = self.config;
        let metadata = &mut *self.metadata;

        {
            let relay_start = metadata.checked.header.unwrap_or(config.genesis_block);
            let relay_end = metadata
                .recent_imported
                .header
                .unwrap_or(0)
                .min(relay_start + config.check_batch);
            if relay_start < relay_end {
                check_and_fix_headers(db, config, "relay", relay_start, Some(relay_end), None)
                    .await
                    .context("Failed to check relay headers")?;
                metadata.checked.header = Some(relay_end);
                db.put_metadata(metadata)
                    .context("Failed to update metadata")?;
            }
        }

        {
            let para_start = metadata.checked.para_header.unwrap_or(0);
            let para_end = metadata
                .recent_imported
                .para_header
                .unwrap_or(0)
                .min(para_start + config.check_batch);
            if para_start < para_end {
                check_and_fix_headers(db, config, "para", para_start, Some(para_end), None)
                    .await
                    .context("Failed to check para headers")?;
                metadata.checked.para_header = Some(para_end);
                db.put_metadata(metadata)
                    .context("Failed to update metadata")?;
            }
        }

        if !config.no_state_root {
            let changes_start = metadata.checked.storage_changes.unwrap_or(1);
            let max_checked_header = metadata.checked.para_header.unwrap_or_default();
            let changes_end = metadata
                .recent_imported
                .storage_changes
                .unwrap_or(0)
                .min(changes_start + config.check_batch)
                .min(max_checked_header);
            if changes_start < changes_end {
                check_and_fix_storages_changes(
                    db,
                    Some(self.para_api.clone()),
                    config,
                    changes_start,
                    Some(changes_end),
                    None,
                    config.allow_empty_state_root,
                )
                .await
                .context("Failed to check storage changes")?;
                metadata.checked.storage_changes = Some(changes_end);
                db.put_metadata(metadata)
                    .context("Failed to update metadata")?;
            }
        }
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            self.grab_headers().await?;
            self.grab_para_headers().await?;
            self.grab_storage_changes().await?;
            if let Err(err) = self.continue_check_headers().await {
                error!("Error fixing headers: {err:?}");
            }
            sleep(self.config.interval).await;
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

pub(crate) async fn check_and_fix_headers(
    db: &CacheDB,
    config: &Serve,
    chain: &str,
    from: BlockNumber,
    to: Option<BlockNumber>,
    count: Option<BlockNumber>,
) -> Result<String> {
    let parachain = match chain {
        "relay" => false,
        "para" => true,
        _ => bail!("Unknown check type {chain}"),
    };
    let to = to.unwrap_or(from + count.unwrap_or(1));
    info!("Checking {chain} headers from {from} to {to}");
    if to < from {
        bail!("Invalid range");
    }
    let from = from.saturating_sub(1);
    let mut prev = load_header_or_regrab(db, config, parachain, from).await?;
    let mut mismatches = 0;
    for block in (from + 1)..=to {
        let cur_header = load_header_or_regrab(db, config, parachain, block).await?;
        if prev.hash() != cur_header.parent_hash {
            mismatches += 1;
            let prev = regrab_header(db, config, prev.number, parachain)
                .await
                .context("Failed to regrab header")?;
            let cur_header = regrab_header(db, config, cur_header.number, parachain).await?;
            if prev.hash() != cur_header.parent_hash {
                bail!("Cannot fix mismatch at {block}");
            }
        }
        prev = cur_header;
    }
    let from = from + 1;
    let response = if mismatches > 0 {
        format!("Checked blocks from {from} to {to}, {mismatches} mismatches")
    } else {
        format!("Checked blocks from {from} to {to}, All OK")
    };
    info!("{}", response);
    Ok(response)
}

pub(crate) async fn check_and_fix_storages_changes(
    db: &CacheDB,
    api: Option<ChainApi>,
    config: &Serve,
    from: BlockNumber,
    to: Option<BlockNumber>,
    count: Option<BlockNumber>,
    allow_empty_root: bool,
) -> Result<u32> {
    let api = match api {
        Some(api) => api,
        None => pherry::subxt_connect(&config.para_node_uri)
            .await
            .context(format!("Failed to connect to {}", config.para_node_uri))?,
    };
    let to = to.unwrap_or(from + count.unwrap_or(1));
    info!("Checking storage changes from {from} to {to}");
    let mut state_root_mismatches = 0_u32;
    for block in from..to {
        let header = db
            .get_para_header(block)
            .ok_or(anyhow!("Header {block} not found"))?;
        let header = decode_header(&header)?;
        let changes = db
            .get_storage_changes(block)
            .ok_or(anyhow!("Storage changes {block} not found"))?;
        let actual_root = decode_header(&changes)
            .map(|h| h.state_root)
            .unwrap_or_default();
        if allow_empty_root && actual_root == Default::default() {
            continue;
        }
        let expected_root = header.state_root;
        if expected_root != actual_root {
            info!("Storage changes {block} mismatch, expected={expected_root} actual={actual_root}, trying to regrab");
            cache::grab_storage_changes(&api, header.number, 1, 1, true, |info| {
                db.put_storage_changes(info.block_header.number, &info.encode())
                    .context("Failed to put record to DB")?;
                Ok(())
            })
            .await
            .context("Failed to grab storage changes from node")?;
            state_root_mismatches += 1;
        }
    }
    Ok(state_root_mismatches)
}

fn decode_header(data: &[u8]) -> Result<Header> {
    let header = Header::decode(&mut &data[..]).context("Failed to decode header")?;
    Ok(header)
}

async fn load_header_or_regrab(
    db: &CacheDB,
    config: &Serve,
    parachain: bool,
    block: BlockNumber,
) -> Result<Header> {
    let header = if parachain {
        db.get_para_header(block)
    } else {
        db.get_header(block)
    }
    .and_then(|header| decode_header(&header).ok());

    let header = match header {
        Some(header) => header,
        None => {
            warn!("Header {block} not found, trying to regrab");
            regrab_header(db, config, block, parachain).await?
        }
    };
    Ok(header)
}

async fn regrab_header(
    db: &CacheDB,
    config: &Serve,
    number: BlockNumber,
    parachain: bool,
) -> Result<Header> {
    let chain = if parachain { "para" } else { "relay" };
    let enabled = if parachain {
        config.grab_para_headers
    } else {
        config.grab_headers
    };
    if !enabled {
        warn!("Trying to regrab {chain} header {number} while grab headers disabled");
        bail!("Grab {chain} headers disabled");
    }
    info!("Regrabbing {chain}chain header {}", number);
    let para_api = pherry::subxt_connect(&config.para_node_uri)
        .await
        .context(format!("Failed to connect to {}", config.para_node_uri))?;
    let mut grabed = None;
    if parachain {
        cache::grab_para_headers(&para_api, number, 1, |header| {
            db.put_para_header(header.number, &header.encode())
                .context("Failed to put record to DB")?;
            grabed = Some(header);
            Ok(())
        })
        .await?;
    } else {
        let api = pherry::subxt_connect(&config.node_uri)
            .await
            .context(format!("Failed to connect to {}", config.node_uri))?;
        cache::grab_headers(&api, &para_api, number, 1, 1, |info| {
            db.put_header(info.header.number, &info.encode())
                .context("Failed to put record to DB")?;
            grabed = Some(info.header);
            Ok(())
        })
        .await?;
    };
    grabed.ok_or(anyhow!("Failed to grab {chain}chain header {number}"))
}
