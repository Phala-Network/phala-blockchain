use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context as _, Result};
use log::{error, info};
use scale::Encode;

use pherry::{
    headers_cache as cache,
    types::{phaxt::ChainApi, rpc::ExtraRpcExt},
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
            db.put_metadata(&metadata)?;
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

    async fn grab_headers(&mut self) -> Result<()> {
        let Some(next_header) = self.next_header.as_deref_mut() else {
            return Ok(());
        };
        let state = self
            .api
            .extra_rpc()
            .system_sync_state()
            .await
            .context("Failed to get sync state")?;

        info!("Relaychain node state: {state:?}");
        if (state.current_block as BlockNumber) < *next_header + self.config.justification_interval
        {
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
        let Some(next_para_header) = self.next_para_header.as_deref_mut() else {
            return Ok(());
        };
        info!("Grabbing parachain headers start from {next_para_header}...");
        cache::grab_para_headers(&self.para_api, *next_para_header, u32::MAX, |info| {
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
        let Some(next_delta) = self.next_delta.as_deref_mut() else {
            return Ok(());
        };
        info!("Grabbing storage changes start from {}...", next_delta);
        cache::grab_storage_changes(
            &self.para_api,
            *next_delta,
            u32::MAX,
            self.config.grab_storage_changes_batch,
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

    async fn run(&mut self) -> Result<()> {
        loop {
            self.grab_headers().await?;
            self.grab_para_headers().await?;
            self.grab_storage_changes().await?;
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
