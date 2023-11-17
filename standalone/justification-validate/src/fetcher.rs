use anyhow::Result;

use reqwest::Response;
use serde::{Deserialize, Serialize};

use phactory_api::blocks::AuthoritySetChange;
use pherry::types::Header;
use scale::{Decode, Encode};

#[derive(Decode, Encode, Debug, Clone)]
pub struct BlockInfo {
    pub header: Header,
    pub justification: Option<Vec<u8>>,
    pub para_header: Option<ParaHeader>,
    pub authority_set_change: Option<AuthoritySetChange>,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct ParaHeader {
    /// Finalized parachain header number
    pub fin_header_num: u32,
    pub proof: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CacheInfo {
    genesis: Vec<u32>,
    recent_imported: Progress,
    higest: Progress,
    checked: Progress,
}

#[derive(Serialize, Deserialize, Debug)]
struct Progress {
    header: u32,
    para_header: u32,
    storage_changes: u32,
}

pub struct BlockFetcher {
    base_url: String,
    client: reqwest::Client,
}

impl BlockFetcher {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_genesis(&self) -> Result<Vec<u8>> {
        let info = self.info().await?;
        let genesis = *info.genesis.first().ok_or(anyhow::anyhow!("No genesis"))?;
        let genesis = self
            .fetch(&format!("genesis/{}", genesis))
            .await?
            .bytes()
            .await?;
        Ok(genesis.to_vec())
    }

    async fn info(&self) -> Result<CacheInfo> {
        let info = self.fetch("state").await?.json().await?;
        Ok(info)
    }

    pub async fn fetch_headers(&self, start: u32) -> Result<Vec<u8>> {
        let headers = self
            .fetch(&format!("headers/{}", start))
            .await?
            .bytes()
            .await?;
        Ok(headers.to_vec())
    }

    pub async fn fetch(&self, path: &str) -> Result<Response> {
        let url = format!("{}/{}", self.base_url, path);
        let resp = self.client.get(&url).send().await?;
        if resp.status().is_success() {
            Ok(resp)
        } else {
            Err(anyhow::anyhow!("Failed to fetch {}", url))
        }
    }
}
