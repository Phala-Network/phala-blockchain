use crate::datasource::DataSourceError::*;
use anyhow::{anyhow, Context, Result};
use jsonrpsee::{
    async_client::ClientBuilder,
    client_transport::ws::{Uri, WsTransportClientBuilder},
};
use log::{debug, error, info, warn};
use paste::paste;
use phactory_api::prpc::{HeadersToSync, ParaHeadersToSync};
use phactory_api::{
    blocks::GenesisBlockInfo,
    prpc::{Blocks, InitRuntimeRequest, Message},
};
use phala_types::AttestationProvider;
use phaxt::subxt::rpc::types as subxt_types;
use phaxt::{ChainApi, RpcClient};

use moka::future::Cache;
use pherry::types::ConvertTo;
use pherry::{
    chain_client, get_authority_with_proof_at, get_block_at, get_finalized_header, get_header_hash,
    headers_cache::Client as CacheClient,
};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::size_of_val;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use uuid::Uuid;

static CACHE_SIZE_EXPANSION: f64 = 1.25;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DataSourceConfig {
    pub relaychain: RelaychainDataSourceConfig,
    pub parachain: ParachainDataSourceConfig,
}

impl DataSourceConfig {
    pub fn read_from_file(path: std::path::PathBuf) -> Self {
        let reader = std::fs::File::open(path).expect("Failed to open data source config");
        serde_yaml::from_reader(reader).expect("Failed to read from stream")
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RelaychainDataSourceConfig {
    pub select_policy: SelectPolicy,
    pub data_sources: Vec<DataSource>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ParachainDataSourceConfig {
    pub select_policy: SelectPolicy,
    pub data_sources: Vec<DataSource>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum SelectPolicy {
    Failover,
    Random,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum DataSource {
    SubstrateWebSocketSource(SubstrateWebSocketSource),
    HeadersCacheHttpSource(HeadersCacheHttpSource),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SubstrateWebSocketSource {
    pub endpoint: String,
    pub pruned: bool,
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: usize,
}

fn default_max_concurrent_requests() -> usize {
    1024
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HeadersCacheHttpSource {
    pub endpoint: String,
}

pub struct SubstrateWebSocketSourceInstance {
    pub uuid: Uuid,
    pub uuid_str: String,
    pub client: ChainApi,
    pub endpoint: String,
    pub pruned: bool,
}

pub struct HeadersCacheHttpSourceInstance {
    pub uuid: Uuid,
    pub uuid_str: String,
    pub client: CacheClient,
    pub endpoint: String,
}

pub type WrappedSubstrateWebSocketSourceInstance = Arc<SubstrateWebSocketSourceInstance>;
pub type WrappedHeadersCacheHttpSourceInstance = Arc<HeadersCacheHttpSourceInstance>;

pub type SubstrateWebSocketSourceMap = HashMap<String, WrappedSubstrateWebSocketSourceInstance>;
pub type WrappedSubstrateWebSocketSourceMap = Arc<RwLock<SubstrateWebSocketSourceMap>>;
pub type HeadersCacheHttpSourceMap = HashMap<String, WrappedHeadersCacheHttpSourceInstance>;
pub type WrappedHeadersCacheHttpSourceMap = Arc<RwLock<HeadersCacheHttpSourceMap>>;

pub type DataSourceIdList = Vec<String>;
pub type CachedHeadersToSync = (HeadersToSync, Option<u32>, Option<Vec<Vec<u8>>>);

#[derive(Clone)]
pub enum DataSourceCacheItem {
    InitRuntimeRequest(InitRuntimeRequest),
    StorageChanges(Blocks),
    ParaHeaderByRelayHeight(Option<(u32, Vec<Vec<u8>>)>),
    ParaHeadersToSyncWithoutProof(ParaHeadersToSync),
}

impl DataSourceCacheItem {
    pub fn resident_size(&self) -> usize {
        let ret = match self {
            DataSourceCacheItem::InitRuntimeRequest(e) => {
                (e.encoded_len() as f64 * CACHE_SIZE_EXPANSION) as _
            }
            DataSourceCacheItem::StorageChanges(e) => {
                (e.encoded_len() as f64 * CACHE_SIZE_EXPANSION) as _
            }
            DataSourceCacheItem::ParaHeaderByRelayHeight(e) => match e {
                None => 16,
                Some((_, e)) => {
                    let bytes_len = e.iter().flatten().collect::<Vec<_>>().len();
                    ((e.len() + 1) * size_of_val(e)) + bytes_len
                }
            },
            DataSourceCacheItem::ParaHeadersToSyncWithoutProof(e) => {
                (e.encoded_len() as f64 * CACHE_SIZE_EXPANSION) as _
            },
        };
        ret
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DataSourceError {
    #[error("Unknown error occurred, this is a bug.")]
    UnknownError,
    #[error("Unknown error from cache occurred, this is a bug.")]
    UnknownErrorFromCache,
    #[error("No valid data source found")]
    NoValidDataSource,
    #[error("BlockHashNotFound {0}")]
    BlockHashNotFound(u32),
    #[error("BlockNotFound {0}")]
    BlockNotFound(u32),
    #[error("SearchSetIdChangeInEmptyRange")]
    SearchSetIdChangeInEmptyRange,
    #[error("No authority key found")]
    NoAuthorityKeyFound,
    #[error("Returned value is None")]
    ReturnedNone,
}

pub struct DataSourceManager {
    pub config: DataSourceConfig,
    pub relaychain_rpc_client_ids: DataSourceIdList,
    pub relaychain_full_rpc_client_ids: DataSourceIdList,
    pub relaychain_headers_cache_ids: DataSourceIdList,
    pub relaychain_rpc_client_map: WrappedSubstrateWebSocketSourceMap,
    pub relaychain_headers_cache_map: WrappedHeadersCacheHttpSourceMap,
    pub parachain_rpc_client_ids: DataSourceIdList,
    pub parachain_full_rpc_client_ids: DataSourceIdList,
    pub parachain_headers_cache_ids: DataSourceIdList,
    pub parachain_rpc_client_map: WrappedSubstrateWebSocketSourceMap,
    pub parachain_headers_cache_map: WrappedHeadersCacheHttpSourceMap,
    pub is_relaychain_full: bool,
    pub is_parachain_full: bool,
    pub cache: Cache<String, Arc<DataSourceCacheItem>>,
}

macro_rules! dump_ds_ids_from_config {
    ($source:expr) => {{
        let mut full_rpc_ids = Vec::new();
        let mut rpc_ids = Vec::new();
        let mut hc_ids = Vec::new();

        for c in $source.data_sources.clone() {
            match c {
                DataSource::SubstrateWebSocketSource(c) => {
                    let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, c.endpoint.as_bytes()).to_string();
                    rpc_ids.push(id.clone());
                    if !c.pruned {
                        full_rpc_ids.push(id);
                    }
                }
                DataSource::HeadersCacheHttpSource(c) => {
                    let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, c.endpoint.as_bytes()).to_string();
                    hc_ids.push(id);
                }
            }
        }
        (full_rpc_ids, rpc_ids, hc_ids)
    }};
}

macro_rules! invoke_ds_loops {
    ($config:expr, $dsm:expr, $t:ident) => {{
        paste! {
            $config
                .$t
                .data_sources
                .clone()
                .into_iter()
                .map(|c| {
                    tokio::spawn(Self::[<$t _ds_loop>](c, $dsm.clone()))
                })
                .collect::<Vec<_>>()
        }
    }};
}

macro_rules! ds_loop {
    ($t:ident) => {
        paste! {
            async fn [<$t _ds_loop>](config: DataSource, dsm: WrappedDataSourceManager) {
                match config {
                    DataSource::SubstrateWebSocketSource(config) => {
                        let map = &dsm.[<$t _rpc_client_map>];
                        Self::subxt_loop(config, map.clone()).await;
                    }
                    DataSource::HeadersCacheHttpSource(config) => {
                        let map = &dsm.[<$t _headers_cache_map>];
                        Self::headers_cache_loop(config, map.clone()).await;
                    },
                }
            }
        }
    };
}

impl DataSourceManager {
    pub async fn current_relaychain_rpc_client(
        self: Arc<Self>,
        full: bool,
    ) -> Option<WrappedSubstrateWebSocketSourceInstance> {
        let ids = if full {
            &self.relaychain_full_rpc_client_ids
        } else {
            &self.relaychain_rpc_client_ids
        };
        let mut ids = ids.clone();
        match &self.config.relaychain.select_policy {
            SelectPolicy::Random => ids.shuffle(&mut thread_rng()),
            SelectPolicy::Failover => {}
        };

        let map = self.relaychain_rpc_client_map.clone();
        let map = map.read().await;
        for i in ids {
            if let Some(i) = map.get(&i) {
                return Some(i.clone());
            }
        }
        None
    }

    pub async fn current_parachain_rpc_client(
        self: Arc<Self>,
        full: bool,
    ) -> Option<WrappedSubstrateWebSocketSourceInstance> {
        let ids = if full {
            &self.parachain_full_rpc_client_ids
        } else {
            &self.parachain_rpc_client_ids
        };
        let mut ids = ids.clone();
        match &self.config.parachain.select_policy {
            SelectPolicy::Random => ids.shuffle(&mut thread_rng()),
            SelectPolicy::Failover => {}
        };

        let map = self.parachain_rpc_client_map.clone();
        let map = map.read().await;
        for i in ids {
            if let Some(i) = map.get(&i) {
                return Some(i.clone());
            }
        }
        None
    }

    pub async fn current_relaychain_headers_cache(
        self: Arc<Self>,
    ) -> Option<WrappedHeadersCacheHttpSourceInstance> {
        let ids = &self.relaychain_headers_cache_ids;
        let mut ids = ids.clone();
        match &self.config.relaychain.select_policy {
            SelectPolicy::Random => ids.shuffle(&mut thread_rng()),
            SelectPolicy::Failover => {}
        };

        let map = self.relaychain_headers_cache_map.clone();
        let map = map.read().await;
        for i in ids {
            if let Some(i) = map.get(&i) {
                return Some(i.clone());
            }
        }
        None
    }

    pub async fn current_parachain_headers_cache(
        self: Arc<Self>,
    ) -> Option<WrappedHeadersCacheHttpSourceInstance> {
        let ids = &self.parachain_headers_cache_ids;
        let mut ids = ids.clone();
        match &self.config.parachain.select_policy {
            SelectPolicy::Random => ids.shuffle(&mut thread_rng()),
            SelectPolicy::Failover => {}
        };

        let map = self.parachain_headers_cache_map.clone();
        let map = map.read().await;
        for i in ids {
            if let Some(i) = map.get(&i) {
                return Some(i.clone());
            }
        }
        None
    }

    pub async fn wait_until_rpc_avail(self: Arc<Self>, full: bool) {
        info!("Waiting for Substrate RPC clients to be available...");
        loop {
            if (self.clone().current_relaychain_rpc_client(full).await).is_some()
                && (self.clone().current_parachain_rpc_client(full).await).is_some()
            {
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn from_config(
        config: DataSourceConfig,
        cache_size: usize,
    ) -> Result<(WrappedDataSourceManager, Vec<JoinHandle<()>>)> {
        let relaychain_rpc_client_map: SubstrateWebSocketSourceMap = HashMap::new();
        let relaychain_rpc_client_map = Arc::new(RwLock::new(relaychain_rpc_client_map));

        let relaychain_headers_cache_map = HashMap::new();
        let relaychain_headers_cache_map = Arc::new(RwLock::new(relaychain_headers_cache_map));

        let parachain_rpc_client_map: SubstrateWebSocketSourceMap = HashMap::new();
        let parachain_rpc_client_map = Arc::new(RwLock::new(parachain_rpc_client_map));

        let parachain_headers_cache_map = HashMap::new();
        let parachain_headers_cache_map = Arc::new(RwLock::new(parachain_headers_cache_map));

        let (
            relaychain_full_rpc_client_ids,
            relaychain_rpc_client_ids,
            relaychain_headers_cache_ids,
        ): (DataSourceIdList, DataSourceIdList, DataSourceIdList) =
            dump_ds_ids_from_config!(config.relaychain);
        let (parachain_full_rpc_client_ids, parachain_rpc_client_ids, parachain_headers_cache_ids): (
            DataSourceIdList,
            DataSourceIdList,
            DataSourceIdList,
        ) = dump_ds_ids_from_config!(config.parachain);

        let is_relaychain_full = !relaychain_full_rpc_client_ids.is_empty();
        let is_parachain_full = !parachain_full_rpc_client_ids.is_empty();

        let cache = Cache::builder()
            .weigher(|_key, value: &Arc<DataSourceCacheItem>| -> u32 { value.resident_size() as _ })
            .max_capacity(cache_size as _)
            .time_to_idle(Duration::from_secs(2 * 60));

        let cache = cache.build();

        let dsm = Self {
            config: config.clone(),
            relaychain_rpc_client_ids,
            relaychain_full_rpc_client_ids,
            relaychain_rpc_client_map,
            relaychain_headers_cache_ids,
            relaychain_headers_cache_map,
            parachain_rpc_client_ids,
            parachain_full_rpc_client_ids,
            parachain_rpc_client_map,
            parachain_headers_cache_ids,
            parachain_headers_cache_map,
            is_relaychain_full,
            is_parachain_full,
            cache,
        };
        let dsm = Arc::new(dsm);
        let ret = dsm.clone();
        let _dsm_move = dsm.clone();

        if !(ret.is_relaychain_full && ret.is_parachain_full) {
            warn!("Pruned mode detected hence fast sync feature disabled.");
        }

        let handles = vec![
            invoke_ds_loops!(config, dsm, relaychain),
            invoke_ds_loops!(config, dsm, parachain),
        ];
        let handles = handles.into_iter().flatten().collect::<Vec<_>>();

        Ok((ret, handles))
    }

    ds_loop!(relaychain);
    ds_loop!(parachain);

    async fn headers_cache_loop(
        config: HeadersCacheHttpSource,
        map: WrappedHeadersCacheHttpSourceMap,
    ) {
        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, config.endpoint.as_bytes());
        let uuid_str = uuid.to_string();
        let client = CacheClient::new(config.endpoint.as_str());
        let instance = HeadersCacheHttpSourceInstance {
            uuid,
            uuid_str: uuid_str.clone(),
            client: client.clone(),
            endpoint: config.endpoint.clone(),
        };
        let instance = Arc::new(instance);

        let mut online = false;
        let mut fail_count = 0;
        loop {
            if let Ok(()) = client.ping().await {
                if !online {
                    map.write().await.insert(uuid_str.clone(), instance.clone());
                    online = true;
                    fail_count = 0;
                    info!("Headers cache {}({}) online!", &uuid_str, &config.endpoint);
                }
            } else if online {
                fail_count += 1;
                if fail_count >= 3 {
                    map.write().await.remove(&uuid_str);
                    online = false;
                    warn!("Headers cache {}({}) down!", &uuid_str, &config.endpoint);
                } else {
                    warn!(
                        "Pinging to headers cache {}({}) failed!",
                        &uuid_str, &config.endpoint
                    );
                }
            }
            sleep(Duration::from_secs(6)).await;
        }
    }

    async fn subxt_loop(config: SubstrateWebSocketSource, map: WrappedSubstrateWebSocketSourceMap) {
        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, config.endpoint.as_bytes());
        loop {
            let uuid_str = uuid.to_string();
            match Self::subxt_connect(&config, &uuid, map.clone()).await {
                Ok(_) => {
                    error!(
                        "SubstrateWebSocketSource {}({}) disconnected!",
                        uuid_str.as_str(),
                        config.endpoint.as_str(),
                    );
                }
                Err(e) => {
                    error!(
                        "SubstrateWebSocketSource {}({}) failed: {}",
                        uuid_str.as_str(),
                        config.endpoint.as_str(),
                        e
                    );
                }
            };
            map.write().await.remove(&uuid_str);
            warn!(
                "Reconnecting SubstrateWebSocketSource {}({}) in 30s...",
                uuid_str.as_str(),
                &config.endpoint.as_str()
            );
            sleep(Duration::from_secs(30)).await;
        }
    }

    async fn subxt_connect(
        config: &SubstrateWebSocketSource,
        uuid: &Uuid,
        map: WrappedSubstrateWebSocketSourceMap,
    ) -> Result<()> {
        let uuid_str = uuid.to_string();
        let raw_uri: Uri = config.endpoint.parse().context("Invalid websocket url")?;
        let raw_port = raw_uri.port_u16();
        let scheme = raw_uri.scheme_str().unwrap();
        let port = if let Some(p) = raw_port {
            p
        } else {
            match scheme {
                "ws" => 80,
                "wss" => 443,
                _ => 80,
            }
        };
        let uri = format!(
            "{}://{}:{}{}",
            scheme,
            raw_uri.host().unwrap(),
            &port,
            raw_uri.path_and_query().unwrap().as_str()
        );
        debug!("raw: {}, parsed: {}", &config.endpoint, &uri);
        let uri: Uri = uri.parse().context("Invalid websocket url")?;

        let (sender, receiver) = WsTransportClientBuilder::default()
            .max_request_body_size(u32::MAX)
            .build(uri)
            .await?;
        let ws_client = ClientBuilder::default()
            .max_concurrent_requests(config.max_concurrent_requests)
            .build_with_tokio(sender, receiver);
        let ws_client = Arc::new(ws_client);

        let client = RpcClient::from_rpc_client(ws_client.clone())
            .await
            .context("Failed to connect to substrate")?;
        let update_client = client.updater();
        tokio::spawn(async move {
            let result = update_client.perform_runtime_updates().await;
            error!("Runtime update failed with result={result:?}");
        });

        let client = ChainApi(client);
        let instance = SubstrateWebSocketSourceInstance {
            uuid: *uuid,
            uuid_str: uuid_str.clone(),
            client,
            endpoint: config.endpoint.clone(),
            pruned: config.pruned,
        };

        map.write().await.insert(uuid_str, Arc::new(instance));

        ws_client.on_disconnect().await;
        Ok(())
    }
}

pub type WrappedDataSourceManager = Arc<DataSourceManager>;

pub async fn setup_data_source_manager(
    config_path: &str,
    cache_size: usize,
) -> Result<(WrappedDataSourceManager, Vec<JoinHandle<()>>)> {
    let path = std::path::PathBuf::from(config_path);
    let config = DataSourceConfig::read_from_file(path);
    DataSourceManager::from_config(config, cache_size).await
}

#[macro_export]
macro_rules! use_relaychain_api {
    ($dsm:expr, $full:ident) => {
        $dsm.clone()
            .current_relaychain_rpc_client($full)
            .await
            .map(|i| i.client.clone())
    };
}

#[macro_export]
macro_rules! use_parachain_api {
    ($dsm:expr, $full:ident) => {
        $dsm.clone()
            .current_parachain_rpc_client($full)
            .await
            .map(|i| i.client.clone())
    };
}

#[macro_export]
macro_rules! use_relaychain_hc {
    ($dsm:expr) => {
        $dsm.clone()
            .current_relaychain_headers_cache()
            .await
            .map(|i| i.client.clone())
    };
}

#[macro_export]
macro_rules! use_parachain_hc {
    ($dsm:expr) => {
        $dsm.clone()
            .current_parachain_headers_cache()
            .await
            .map(|i| i.client.clone())
    };
}

impl DataSourceManager {
    pub async fn do_get_init_runtime_default_request(
        self: Arc<Self>,
        provider: Option<AttestationProvider>,
    ) -> Result<Arc<DataSourceCacheItem>> {
        let relay_hc = use_relaychain_hc!(self);

        let relay_api = use_relaychain_api!(self, false).ok_or(NoValidDataSource)?;
        let para_api = use_parachain_api!(self, false).ok_or(NoValidDataSource)?;

        let relaychain_start_block = para_api.clone().relay_parent_number().await? - 1;
        debug!("relaychain_start_block: {relaychain_start_block}");

        let mut genesis_info: Option<GenesisBlockInfo> = match relay_hc {
            Some(relay_hc) => match relay_hc.get_genesis(relaychain_start_block).await {
                Ok(genesis_info) => Some(genesis_info),
                Err(e) => {
                    warn!("get_init_runtime_default_request: {}", e);
                    None
                }
            },
            None => None,
        };
        if genesis_info.is_none() {
            let genesis_block = get_block_at(&relay_api, Some(relaychain_start_block))
                .await?
                .0
                .block;
            let set_proof = get_authority_with_proof_at(&relay_api, &genesis_block.header).await?;
            genesis_info = Some(GenesisBlockInfo {
                block_header: genesis_block.header.clone(),
                authority_set: set_proof.authority_set,
                proof: set_proof.authority_proof,
            });
        }
        let genesis_info = genesis_info.unwrap();
        let genesis_state = chain_client::fetch_genesis_storage(&para_api).await?;
        let ret = InitRuntimeRequest::new(
            false,
            genesis_info.clone(),
            None,
            genesis_state,
            None,
            true,
            provider,
        );
        Ok(Arc::new(DataSourceCacheItem::InitRuntimeRequest(ret)))
    }
    pub async fn get_init_runtime_default_request(
        self: Arc<Self>,
        provider: Option<AttestationProvider>,
    ) -> Result<InitRuntimeRequest> {
        let key = format!("init-{provider:?}");
        let cache = self.cache.clone();
        match cache
            .try_get_with(
                key,
                self.clone().do_get_init_runtime_default_request(provider),
            )
            .await
        {
            Ok(ret) => match *ret {
                DataSourceCacheItem::InitRuntimeRequest(ref data) => Ok(data.clone()),
                _ => Err(UnknownErrorFromCache.into()),
            },
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub async fn do_fetch_storage_changes(
        self: Arc<Self>,
        from: u32,
        to: u32,
    ) -> Result<Arc<DataSourceCacheItem>> {
        let hc = self.clone().current_parachain_headers_cache().await;
        let para_api = if hc.is_some() {
            use_parachain_api!(self, false)
        } else {
            use_parachain_api!(self, true)
        }
        .ok_or(NoValidDataSource)?;
        let ret = if let Some(hc) = hc {
            pherry::fetch_storage_changes(&para_api, Some(&hc.client), from, to).await
        } else {
            pherry::fetch_storage_changes(&para_api, None, from, to).await
        }?;

        let ret = Blocks::new(ret);
        Ok(Arc::new(DataSourceCacheItem::StorageChanges(ret)))
    }
    pub async fn fetch_storage_changes(self: Arc<Self>, from: u32, to: u32) -> Result<Blocks> {
        let key = format!("sc:{from}:{to}");
        let cache = self.cache.clone();
        match cache
            .try_get_with(key, self.clone().do_fetch_storage_changes(from, to))
            .await
        {
            Ok(ret) => match *ret {
                DataSourceCacheItem::StorageChanges(ref data) => Ok(data.clone()),
                _ => Err(UnknownErrorFromCache.into()),
            },
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub async fn do_get_para_header_by_relay_header(
        self: Arc<Self>,
        height: u32,
    ) -> Result<Arc<DataSourceCacheItem>> {
        let hc = use_relaychain_hc!(self);
        if let Some(hc) = hc {
            if let Ok(block_info) = hc.get_header(height).await {
                if let Some(para_header) = block_info.para_header {
                    return Ok(Arc::new(DataSourceCacheItem::ParaHeaderByRelayHeight(
                        Some((para_header.fin_header_num, para_header.proof))
                    )));
                }
            }
        }

        let relay_api = use_relaychain_api!(self, true).ok_or(NoValidDataSource)?;
        let para_api = use_parachain_api!(self, true).ok_or(NoValidDataSource)?;

        let last_header_hash = get_header_hash(&relay_api, Some(height)).await?;
        let header = get_finalized_header(&relay_api, &para_api, last_header_hash)
            .await?
            .map(|(h, proof)| (h.number, proof));
        Ok(Arc::new(DataSourceCacheItem::ParaHeaderByRelayHeight(
            header,
        )))
    }
    pub async fn get_para_header_by_relay_header(
        self: Arc<Self>,
        height: u32,
    ) -> Result<Option<(u32, Vec<Vec<u8>>)>> {
        let key = format!("ph:rh:{height}");
        let cache = self.cache.clone();
        match cache
            .try_get_with(
                key.clone(),
                self.clone().do_get_para_header_by_relay_header(height),
            )
            .await
        {
            Ok(ret) => match *ret {
                DataSourceCacheItem::ParaHeaderByRelayHeight(ref data) => Ok(data.clone()),
                _ => Err(UnknownErrorFromCache.into()),
            },
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub async fn do_get_para_headers(
        self: Arc<Self>,
        from: u32,
        to: u32,
    ) -> Result<Arc<DataSourceCacheItem>> {
        if let Some(hc) = use_parachain_hc!(self) {
            let count = to - from + 1;
            if let Ok(d) = hc.get_parachain_headers(from, count).await {
                let d = ParaHeadersToSync::new(d, vec![]);
                return Ok(Arc::new(
                    DataSourceCacheItem::ParaHeadersToSyncWithoutProof(d),
                ));
            }
        }
        let para_api = use_parachain_api!(self, true).ok_or(NoValidDataSource)?;
        let mut d = vec![];
        for b in from..=to {
            let num = subxt_types::BlockNumber::from(subxt_types::NumberOrHex::Number(b as _));
            let hash = para_api
                .rpc()
                .block_hash(Some(num))
                .await?
                .ok_or(BlockHashNotFound(b))?;
            let header = para_api
                .rpc()
                .header(Some(hash))
                .await?
                .ok_or(BlockNotFound(b))?;
            d.push(header.convert_to());
        }
        let d = ParaHeadersToSync::new(d, vec![]);
        Ok(Arc::new(
            DataSourceCacheItem::ParaHeadersToSyncWithoutProof(d),
        ))
    }
    pub async fn get_para_headers(
        self: Arc<Self>,
        from: u32,
        to: u32,
    ) -> Result<ParaHeadersToSync> {
        let key = format!("ph:{from}:{to}");
        let cache = self.cache.clone();
        match cache
            .try_get_with(key, self.clone().do_get_para_headers(from, to))
            .await
        {
            Ok(ret) => match *ret {
                DataSourceCacheItem::ParaHeadersToSyncWithoutProof(ref data) => Ok(data.clone()),
                _ => Err(UnknownErrorFromCache.into()),
            },
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }
}
