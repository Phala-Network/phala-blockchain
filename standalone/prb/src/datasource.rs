use anyhow::{anyhow, Context, Result};
use jsonrpsee::{
    async_client::ClientBuilder,
    client_transport::ws::{Uri, WsTransportClientBuilder},
};
use log::{debug, error, info, warn};
use memory_lru::{MemoryLruCache, ResidentSize};
use paste::paste;
use phactory_api::{blocks::GenesisBlockInfo, prpc::InitRuntimeRequest};
use phala_types::AttestationProvider;
use phaxt::subxt::rpc::types as subxt_types;
use phaxt::{ChainApi, RpcClient};
use pherry::{get_authority_with_proof_at, get_block_at, headers_cache::Client as CacheClient};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, mem::size_of};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use uuid::Uuid;

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

pub type DataSourceCache = MemoryLruCache<String, DataSourceCacheItem>;
pub type WrappedDataSourceCache = Arc<RwLock<DataSourceCache>>;
pub type DataSourceCacheLockSpace = LockSpace;
pub type WrappedDataSourceCacheLockSpace = Arc<Mutex<DataSourceCacheLockSpace>>;

pub struct LockSpace {
    locks: HashMap<String, Arc<Semaphore>>,
}

impl LockSpace {
    pub fn new() -> WrappedDataSourceCacheLockSpace {
        Arc::new(Mutex::new(Self {
            locks: HashMap::new(),
        }))
    }

    pub async fn get_lock(s: WrappedDataSourceCacheLockSpace, key: String) -> Arc<Semaphore> {
        let s = s.clone();
        let mut s = s.lock().await;
        let lock = s.get_or_create(key);
        drop(s);
        lock.clone()
    }
    pub async fn clean(s: WrappedDataSourceCacheLockSpace) {
        let s = s.clone();
        let mut s = s.lock().await;
        s.do_clean();
        drop(s);
    }

    pub fn get_or_create(&mut self, key: String) -> Arc<Semaphore> {
        if let Some(ret) = self.locks.get(&key) {
            return ret.clone();
        } else {
            let ret = Arc::new(Semaphore::new(1));
            self.locks.insert(key, ret.clone());
            ret.clone()
        }
    }
    pub fn do_clean(&mut self) {
        let locks = &mut self.locks;
        locks.retain(|k, s| s.available_permits() != 0);
    }
}

#[derive(Clone)]
pub enum DataSourceCacheItem {
    InitRuntimeRequest(InitRuntimeRequest),
}

impl ResidentSize for DataSourceCacheItem {
    fn resident_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DataStourceError {
    #[error("No valid data source found (key: {key})")]
    NoValidDataSource { key: String },
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
    pub lock_space: WrappedDataSourceCacheLockSpace,
    pub cache: WrappedDataSourceCache,
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

    pub async fn wait_until_rpc_avail(self: Arc<Self>, full: bool) {
        info!("Waiting for Substrate RPC clients to be available...");
        loop {
            if let Some(_) = self.clone().current_relaychain_rpc_client(full).await {
                if let Some(_) = self.clone().current_parachain_rpc_client(full).await {
                    break;
                }
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

        let is_relaychain_full = relaychain_full_rpc_client_ids.len() > 0;
        let is_parachain_full = parachain_full_rpc_client_ids.len() > 0;

        let lock_space = LockSpace::new();
        let cache = MemoryLruCache::new(cache_size);
        let cache = Arc::new(RwLock::new(cache));

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
            lock_space,
            cache,
        };
        let dsm = Arc::new(dsm);
        let ret = dsm.clone();
        let dsm_move = dsm.clone();

        if !(*&ret.is_relaychain_full && *&ret.is_relaychain_full) {
            warn!("Pruned mode detected hence fast sync feature disabled.");
        }

        let handles = vec![
            invoke_ds_loops!(config, dsm, relaychain),
            invoke_ds_loops!(config, dsm, parachain),
        ];
        let mut handles = handles.into_iter().flatten().collect::<Vec<_>>();

        handles.push(tokio::spawn(async move {
            loop {
                let dsm = dsm_move.clone();
                let locks = dsm.lock_space.clone();
                LockSpace::clean(locks).await;
                let cache = dsm.cache.clone();
                let cache = cache.read().await;
                debug!("Cache used {}/{} bytes", cache.current_size(), cache_size);
                drop(cache);
                sleep(Duration::from_secs(18)).await;
            }
        }));

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
                    let m = map.clone();
                    let mut m = map.write().await;
                    m.insert(uuid_str.clone(), instance.clone());
                    online = true;
                    fail_count = 0;
                    drop(m);
                    info!("Headers cache {}({}) online!", &uuid_str, &config.endpoint);
                }
            } else {
                if online {
                    fail_count += 1;
                    if fail_count >= 3 {
                        let m = map.clone();
                        let mut m = map.write().await;
                        m.remove(&uuid_str);
                        online = false;
                        drop(m);
                        warn!("Headers cache {}({}) down!", &uuid_str, &config.endpoint);
                    } else {
                        warn!(
                            "Pinging to headers cache {}({}) failed!",
                            &uuid_str, &config.endpoint
                        );
                    }
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
            let map = map.clone();
            let mut map = map.write().await;
            map.remove(&uuid_str);
            drop(map);
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
        let url: Uri = config.endpoint.parse().context("Invalid websocket url")?;
        let (sender, receiver) = WsTransportClientBuilder::default()
            .max_request_body_size(u32::MAX)
            .build(url)
            .await
            .context("Failed to build ws transport")?;
        let ws_client = ClientBuilder::default().build_with_tokio(sender, receiver);
        let ws_client = Arc::new(ws_client);

        let client = RpcClient::from_rpc_client(ws_client.clone())
            .await
            .context("Failed to connect to substrate")?;
        let update_client = client.updater();
        tokio::spawn(async move {
            let result = update_client.perform_runtime_updates().await;
            eprintln!("Runtime update failed with result={result:?}");
        });

        let client = ChainApi(client);
        let instance = SubstrateWebSocketSourceInstance {
            uuid: uuid.clone(),
            uuid_str: uuid_str.clone(),
            client,
            endpoint: config.endpoint.clone(),
            pruned: config.pruned,
        };

        let mut map = map.write().await;
        map.insert(uuid_str, Arc::new(instance));
        drop(map);

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
    ($dsm:ident, $full:ident) => {
        $dsm.clone()
            .current_relaychain_rpc_client($full)
            .await
            .unwrap()
            .client
            .clone()
    };
}

#[macro_export]
macro_rules! use_parachain_api {
    ($dsm:ident, $full:ident) => {
        $dsm.clone()
            .current_parachain_rpc_client($full)
            .await
            .unwrap()
            .client
            .clone()
    };
}

impl DataSourceManager {
    pub async fn get_init_runtime_default_request(self: Arc<Self>) -> Result<InitRuntimeRequest> {
        let mut key = "init".to_string();

        let lock = (&self.lock_space).clone();
        let lock = LockSpace::get_lock(lock, key.clone()).await;
        let lock = lock.acquire().await;

        let cache = self.cache.clone();
        let mut cache = cache.write().await;
        if let Some(DataSourceCacheItem::InitRuntimeRequest(data)) = cache.get(&key) {
            return Ok(data.clone());
        };
        drop(cache);

        let relay_api = use_relaychain_api!(self, true);
        let para_api = use_parachain_api!(self, true);

        let relaychain_start_block = para_api
            .clone()
            .relay_parent_number()
            .await
            .expect("Relaychain start block not found")
            - 1;
        let genesis_block = get_block_at(&relay_api, Some(relaychain_start_block))
            .await
            .expect("Genesis block not found")
            .0
            .block;
        let genesis_hash = relay_api
            .rpc()
            .block_hash(Some(subxt_types::BlockNumber::from(
                subxt_types::NumberOrHex::Number(relaychain_start_block as u64),
            )))
            .await
            .expect("Genesis hash not found")
            .unwrap();
        let set_proof = get_authority_with_proof_at(&relay_api, genesis_hash)
            .await
            .expect("get_authority_with_proof_at Failed");
        let genesis_info = GenesisBlockInfo {
            block_header: genesis_block.header.clone(),
            authority_set: set_proof.authority_set,
            proof: set_proof.authority_proof,
        };
        let genesis_state = pherry::chain_client::fetch_genesis_storage(&para_api)
            .await
            .expect("fetch_genesis_storage failed");
        let ret = InitRuntimeRequest::new(
            false,
            genesis_info.clone(),
            None,
            genesis_state.clone(),
            None,
            true,
            Some(AttestationProvider::Ias),
        );

        let cache = self.cache.clone();
        let mut cache = cache.write().await;
        debug!(
            "InitRuntimeRequest: {} bytes: {:?}",
            std::mem::size_of_val(&ret),
            ret
        );
        cache.insert(key, DataSourceCacheItem::InitRuntimeRequest(ret.clone()));
        drop(cache);

        return Ok(ret);

        drop(lock);
        Err(DataStourceError::NoValidDataSource { key }.into())
    }
}
