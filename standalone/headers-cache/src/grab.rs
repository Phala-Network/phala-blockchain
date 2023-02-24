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
    BlockNumber,
};

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub grab_headers: bool,
    pub grab_para_headers: bool,
    pub grab_storage_changes: bool,
}

pub async fn run(
    db: CacheDB,
    node_uri: &str,
    para_node_uri: &str,
    check_interval: u64,
    justification_interval: BlockNumber,
    genesis_block: BlockNumber,
    config: Config,
) -> Result<()> {
    let mut metadata = db.get_metadata()?.unwrap_or_default();
    let mut next_header = match metadata.higest.header {
        Some(highest) => highest + 1,
        None => genesis_block,
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

    GENESIS.store(genesis_block, Ordering::Relaxed);

    loop {
        if let Err(err) = Crawler::grab(
            node_uri,
            para_node_uri,
            &db,
            &mut metadata,
            config.grab_headers.then_some(&mut next_header),
            config.grab_para_headers.then_some(&mut next_para_header),
            config.grab_storage_changes.then_some(&mut next_delta),
            justification_interval,
            check_interval,
            genesis_block,
        )
        .await
        {
            error!("Error: {err:?}");
        }
        info!("Sleeping for {check_interval} seconds...");
        sleep(check_interval).await;
    }
}

struct Crawler<'c> {
    db: &'c CacheDB,
    metadata: &'c mut Metadata,
    api: ChainApi,
    para_api: ChainApi,
    next_header: Option<&'c mut BlockNumber>,
    next_para_header: Option<&'c mut BlockNumber>,
    next_delta: Option<&'c mut BlockNumber>,
    justification_interval: BlockNumber,
    check_interval: u64,
}

impl<'c> Crawler<'c> {
    async fn grab<'p>(
        node_uri: &'p str,
        para_node_uri: &'p str,
        db: &'c CacheDB,
        metadata: &'c mut Metadata,
        next_header: Option<&'c mut BlockNumber>,
        next_para_header: Option<&'c mut BlockNumber>,
        next_delta: Option<&'c mut BlockNumber>,
        justification_interval: BlockNumber,
        check_interval: u64,
        genesis_block: BlockNumber,
    ) -> Result<()> {
        info!("Connecting to {node_uri}...");
        let api = pherry::subxt_connect(node_uri)
            .await
            .context(format!("Failed to connect to {node_uri}"))?;
        info!("Connecting to {para_node_uri}...");
        let para_api = pherry::subxt_connect(para_node_uri)
            .await
            .context(format!("Failed to connect to {para_node_uri}"))?;
        if !metadata.genesis.contains(&genesis_block) {
            info!("Fetching genesis at {}", genesis_block);
            let genesis = cache::fetch_genesis_info(&api, genesis_block)
                .await
                .context("Failed to fetch genesis info")?;
            db.put_genesis(genesis_block, &genesis.encode())?;
            metadata.put_genesis(genesis_block);
            db.put_metadata(&metadata)?;
            info!("Got genesis at {}", genesis_block);
        }
        Self {
            db,
            metadata,
            api,
            para_api,
            next_header,
            next_para_header,
            next_delta,
            justification_interval,
            check_interval,
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
        if (state.current_block as BlockNumber) < *next_header + self.justification_interval {
            info!("No enough relaychain headers in node");
            return Ok(());
        }

        info!("Grabbing headers start from {next_header}...");
        cache::grab_headers(
            &self.api,
            &self.para_api,
            *next_header,
            u32::MAX,
            self.justification_interval,
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
        cache::grab_storage_changes(&self.para_api, *next_delta, u32::MAX, 4, |info| {
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
        })
        .await
        .context("Failed to grab storage changes from node")?;
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            self.grab_headers().await?;
            self.grab_para_headers().await?;
            self.grab_storage_changes().await?;
            sleep(self.check_interval).await;
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
