use anyhow::{anyhow, Context, Result};
use jsonrpsee::{
    async_client::ClientBuilder,
    client_transport::ws::{Uri, WsTransportClientBuilder},
};
use log::{error, info, warn};
use paste::paste;
use phaxt::{ChainApi, RpcClient};
use pherry::headers_cache::Client as CacheClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DataSourceConfig {
    relaychain: RelaychainDataSourceConfig,
    parachain: ParachainDataSourceConfig,
}

impl DataSourceConfig {
    pub fn read_from_file(path: std::path::PathBuf) -> Self {
        let reader = std::fs::File::open(path).expect("Failed to open data source config");
        serde_yaml::from_reader(reader).expect("Failed to read from stream")
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RelaychainDataSourceConfig {
    data_sources: Vec<DataSource>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ParachainDataSourceConfig {
    para_id: u32,
    data_sources: Vec<DataSource>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum DataSource {
    SubstrateWebSocketSource(SubstrateWebSocketSource),
    HeadersCacheHttpSource(HeadersCacheHttpSource),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SubstrateWebSocketSource {
    endpoint: String,
    pruned: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HeadersCacheHttpSource {
    endpoint: String,
    pruned: bool,
}

impl Into<CacheClient> for HeadersCacheHttpSource {
    fn into(self) -> CacheClient {
        todo!()
    }
}

pub struct SubstrateWebSocketSourceInstance {
    pub uuid: Uuid,
    pub uuid_str: String,
    pub client: ChainApi,
    pub endpoint: String,
    pub pruned: bool,
}

pub type WrappedSubstrateWebSocketSourceInstance = Arc<SubstrateWebSocketSourceInstance>;

pub type SubstrateWebSocketSourceMap = HashMap<String, WrappedSubstrateWebSocketSourceInstance>;
pub type WrappedSubstrateWebSocketSourceMap = Arc<RwLock<SubstrateWebSocketSourceMap>>;

pub type SubstrateWebSocketSourceIdList = Vec<String>;

pub struct DataSourceManager {
    pub config: DataSourceConfig,
    pub relaychain_rpc_client_ids: SubstrateWebSocketSourceIdList,
    pub relaychain_full_rpc_client_ids: SubstrateWebSocketSourceIdList,
    pub relaychain_rpc_client_map: WrappedSubstrateWebSocketSourceMap,
    pub parachain_rpc_client_ids: SubstrateWebSocketSourceIdList,
    pub parachain_full_rpc_client_ids: SubstrateWebSocketSourceIdList,
    pub parachain_rpc_client_map: WrappedSubstrateWebSocketSourceMap,
}

macro_rules! dump_substrate_rpc_ids_from_config {
    ($source:expr) => {{
        let groups = $source
            .data_sources
            .clone()
            .into_iter()
            .filter_map(|c| match c {
                DataSource::SubstrateWebSocketSource(c) => Some((
                    Uuid::new_v5(&Uuid::NAMESPACE_URL, c.endpoint.as_bytes()).to_string(),
                    c.pruned,
                )),
                _ => None,
            });
        let ret2 = groups
            .clone()
            .into_iter()
            .filter_map(|(id, pruned)| if pruned { None } else { Some(id) })
            .collect();
        let ret1 = groups.into_iter().map(|(id, _)| id).collect();
        (ret1, ret2)
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
                    DataSource::HeadersCacheHttpSource(_) => todo!(),
                }
            }
        }
    };
}

impl DataSourceManager {
    pub async fn current_relaychain_ds(&self) -> WrappedSubstrateWebSocketSourceInstance {
        todo!()
    }

    pub async fn current_relaychain_rpc_client(
        self: Arc<Self>,
    ) -> Result<WrappedSubstrateWebSocketSourceInstance> {
        let ids = &self.relaychain_rpc_client_ids;
        let map = self.relaychain_rpc_client_map.clone();
        let map = map.read().await;
        for i in ids {
            if let Some(i) = map.get(i) {
                return Ok(i.clone());
            }
        }
        Err(anyhow!("No rpc client available."))
    }

    pub async fn current_parachain_rpc_client(
        self: Arc<Self>,
    ) -> Result<WrappedSubstrateWebSocketSourceInstance> {
        let ids = &self.parachain_rpc_client_ids;
        let map = self.parachain_rpc_client_map.clone();
        let map = map.read().await;
        for i in ids {
            if let Some(i) = map.get(i) {
                return Ok(i.clone());
            }
        }
        Err(anyhow!("No rpc client available."))
    }

    pub async fn wait_until_rpc_avail(self: Arc<Self>) {
        info!("Waiting for Substrate RPC clients to be available...");
        loop {
            if let Ok(_) = self.clone().current_relaychain_rpc_client().await {
                if let Ok(_) = self.clone().current_parachain_rpc_client().await {
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn from_config(
        config: DataSourceConfig,
    ) -> (WrappedDataSourceManager, Vec<JoinHandle<()>>) {
        let relaychain_rpc_client_map: SubstrateWebSocketSourceMap = HashMap::new();
        let relaychain_rpc_client_map = Arc::new(RwLock::new(relaychain_rpc_client_map));

        let parachain_rpc_client_map: SubstrateWebSocketSourceMap = HashMap::new();
        let parachain_rpc_client_map = Arc::new(RwLock::new(parachain_rpc_client_map));

        let (relaychain_full_rpc_client_ids, relaychain_rpc_client_ids): (
            SubstrateWebSocketSourceIdList,
            SubstrateWebSocketSourceIdList,
        ) = dump_substrate_rpc_ids_from_config!(config.relaychain);
        let (parachain_full_rpc_client_ids, parachain_rpc_client_ids): (
            SubstrateWebSocketSourceIdList,
            SubstrateWebSocketSourceIdList,
        ) = dump_substrate_rpc_ids_from_config!(config.parachain);

        if (relaychain_full_rpc_client_ids.len() + parachain_full_rpc_client_ids.len()) < 2 {
            panic!("Running with only pruned nodes is not supported yet!")
        }

        // TODO: support headers-cache + pruned nodes

        let dsm = Self {
            config: config.clone(),
            relaychain_rpc_client_ids,
            relaychain_full_rpc_client_ids,
            relaychain_rpc_client_map,
            parachain_rpc_client_ids,
            parachain_full_rpc_client_ids,
            parachain_rpc_client_map,
        };
        let dsm = Arc::new(dsm);
        let ret = dsm.clone();

        let handles = vec![
            invoke_ds_loops!(config, dsm, relaychain),
            invoke_ds_loops!(config, dsm, parachain),
        ];
        let handles = handles.into_iter().flatten().collect::<Vec<_>>();

        (ret, handles)
    }

    ds_loop!(relaychain);
    ds_loop!(parachain);

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
) -> Result<(WrappedDataSourceManager, Vec<JoinHandle<()>>)> {
    let path = std::path::PathBuf::from(config_path);
    let config = DataSourceConfig::read_from_file(path);
    Ok(DataSourceManager::from_config(config).await)
}
