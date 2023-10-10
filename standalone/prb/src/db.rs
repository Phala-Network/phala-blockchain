use crate::cli::ConfigCommands;
use anyhow::{anyhow, Context, Result};
use indradb::{
    Datastore, EdgeDirection, EdgeKey, Identifier, MemoryDatastore, PipeEdgeQuery,
    PropertyValueVertexQuery, RangeVertexQuery, RocksdbDatastore, SpecificEdgeQuery,
    SpecificVertexQuery, VertexProperties, VertexPropertyQuery, VertexQuery,
};
use log::{debug, warn};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

pub const ID_VERTEX_INVENTORY_WORKER: &str = "iv_w";
pub const ID_VERTEX_INVENTORY_POOL: &str = "iv_p";

pub const ID_EDGE_BELONG_TO: &str = "ie_bt"; // Direction: from worker to pool

pub const ID_PROP_CREATED_AT: &str = "created_at";

pub const ID_PROP_WORKER_NAME: &str = "name";
pub const ID_PROP_WORKER_ENDPOINT: &str = "endpoint";
pub const ID_PROP_WORKER_STAKE: &str = "stake";
pub const ID_PROP_WORKER_ENABLED: &str = "enabled";
pub const ID_PROP_WORKER_SYNC_ONLY: &str = "sync_only";
pub const ID_PROP_WORKER_GATEKEEPER: &str = "gatekeeper";

// Account-related settings moved to trade service
pub const ID_PROP_POOL_NAME: &str = "name";
pub const ID_PROP_POOL_PID: &str = "pid";
pub const ID_PROP_POOL_ENABLED: &str = "enabled";
pub const ID_PROP_POOL_SYNC_ONLY: &str = "sync_only";

pub const ID_VERTEX_INVENTORY_DATA_SOURCE: &str = "iv_ds";

pub type Db = dyn Datastore + Send + Sync + 'static;
pub type WrappedDb = Arc<Db>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pool {
    pub id: String,
    pub name: String,
    pub pid: u64,
    pub enabled: bool,
    pub sync_only: bool,
    pub workers: Option<Vec<Worker>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Worker {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub stake: String,
    pub pid: Option<u64>,
    pub enabled: bool,
    pub sync_only: bool,
    pub gatekeeper: bool,
}

impl From<VertexProperties> for Pool {
    fn from(value: VertexProperties) -> Self {
        let mut ret = Self {
            id: value.vertex.id.to_string(),
            name: "".to_string(),
            pid: 0,
            enabled: true,
            sync_only: false,
            workers: None,
        };
        value.props.iter().for_each(|p| match p.name.as_str() {
            ID_PROP_POOL_NAME => {
                ret.name = p.value.as_str().unwrap().to_string();
            }
            ID_PROP_POOL_PID => {
                ret.pid = p.value.as_u64().unwrap();
            }
            ID_PROP_POOL_ENABLED => {
                ret.enabled = p.value.as_bool().unwrap();
            }
            ID_PROP_POOL_SYNC_ONLY => {
                ret.sync_only = p.value.as_bool().unwrap();
            }
            &_ => {}
        });
        ret
    }
}

impl From<VertexProperties> for Worker {
    fn from(value: VertexProperties) -> Self {
        let mut ret = Self {
            id: value.vertex.id.to_string(),
            name: "".to_string(),
            stake: "".to_string(),
            endpoint: "".to_string(),
            pid: None,
            enabled: true,
            sync_only: false,
            gatekeeper: false,
        };
        value.props.iter().for_each(|p| match p.name.as_str() {
            ID_PROP_WORKER_NAME => {
                ret.name = p.value.as_str().unwrap().to_string();
            }
            ID_PROP_WORKER_ENDPOINT => {
                ret.endpoint = p.value.as_str().unwrap().to_string();
            }
            ID_PROP_WORKER_STAKE => {
                ret.stake = p.value.as_str().unwrap().to_string();
            }
            ID_PROP_WORKER_ENABLED => {
                ret.enabled = p.value.as_bool().unwrap();
            }
            ID_PROP_WORKER_SYNC_ONLY => {
                ret.sync_only = p.value.as_bool().unwrap();
            }
            ID_PROP_WORKER_GATEKEEPER => {
                ret.gatekeeper = p.value.as_bool().unwrap();
            }
            &_ => {}
        });
        ret
    }
}

pub fn validate_worker_name_existence(db: WrappedDb, name: String) -> Result<String> {
    let w = get_raw_worker_by_name(db.clone(), name.clone())?;
    if w.is_some() {
        return Err(anyhow!("Duplicated worker name!"));
    };
    Ok(name)
}

pub fn validate_bn_string(s: String) -> Result<String> {
    let s = s.parse::<u128>()?;
    Ok(s.to_string())
}

pub fn validate_endpoint(s: String) -> Result<String> {
    let mut url = Url::parse(&s)?;

    match url.scheme() {
        "http" => {
            if url.port().is_none() {
                let _ = url.set_port(Some(80));
            }
        }
        "https" => {
            if url.port().is_none() {
                let _ = url.set_port(Some(443));
            }
        }
        _ => return Err(anyhow!("Invalid URL scheme")),
    }

    Ok(url.as_str().to_string())
}

pub fn setup_inventory_db(db_path: &str) -> WrappedDb {
    let db_path = Path::new(db_path).join("inventory");
    let db = RocksdbDatastore::new(&db_path, None).expect("Failed to open inventory database.");
    let db = Arc::new(db);
    debug!("Opened inventory database in {:?}", db_path);

    [
        ID_PROP_WORKER_NAME,
        ID_PROP_WORKER_ENABLED,
        ID_PROP_WORKER_SYNC_ONLY,
        ID_PROP_WORKER_GATEKEEPER,
        ID_PROP_POOL_NAME,
        ID_PROP_POOL_PID,
        ID_PROP_POOL_ENABLED,
        ID_PROP_POOL_SYNC_ONLY,
    ]
    .iter()
    .for_each(|i| {
        let ii = Identifier::new(*i).unwrap();
        db.index_property(ii)
            .unwrap_or_else(|_| panic!("inv_db: Failed to index property {:?}", *i));
        debug!("inv_db: Indexing property {:?}", *i);
    });

    db
}

pub fn get_pool_by_pid(db: WrappedDb, pid: u64) -> Result<Option<Pool>> {
    let v = get_raw_pool_by_pid(db, pid)?;
    match v {
        Some(v) => Ok(Some(v.into())),
        None => Ok(None),
    }
}

pub fn get_pool_by_pid_with_workers(db: WrappedDb, pid: u64) -> Result<Option<Pool>> {
    let p = get_pool_by_pid(db.clone(), pid)?;
    match p {
        Some(mut p) => {
            let workers = get_workers_by_pid(db, pid)?;
            p.workers = Some(workers);
            Ok(Some(p))
        }
        None => Ok(None),
    }
}

pub fn get_raw_pool_by_pid(db: WrappedDb, pid: u64) -> Result<Option<VertexProperties>> {
    let q = PropertyValueVertexQuery {
        name: Identifier::new(ID_PROP_POOL_PID).unwrap(),
        value: serde_json::Value::Number(serde_json::Number::from(pid)),
    }
    .into();
    let v: Vec<VertexProperties> = db.get_all_vertex_properties(q)?;
    let v = v.get(0);
    match v {
        Some(v) => Ok(Some(v.clone())),
        None => Ok(None),
    }
}

pub fn get_all_pools(db: WrappedDb) -> Result<Vec<Pool>> {
    let v = get_all_raw_pools(db)?
        .into_iter()
        .map(|v| v.into())
        .collect::<Vec<_>>();
    Ok(v)
}

pub fn get_all_pools_with_workers(db: WrappedDb) -> Result<Vec<Pool>> {
    let v = get_all_raw_pools(db.clone())?
        .into_iter()
        .map(|v| {
            let mut v: Pool = v.into();
            let workers = get_workers_by_pid(db.clone(), v.pid);
            v.workers = match workers {
                Ok(w) => Some(w),
                Err(e) => {
                    warn!("Failed to get workers: {}", e);
                    None
                }
            };
            v
        })
        .collect::<Vec<_>>();
    Ok(v)
}

pub fn get_all_raw_pools(db: WrappedDb) -> Result<Vec<VertexProperties>> {
    let q = RangeVertexQuery {
        limit: 4_294_967_294u32,
        t: Some(Identifier::new(ID_VERTEX_INVENTORY_POOL).unwrap()),
        start_id: None,
    }
    .into();
    let v: Vec<VertexProperties> = db.get_all_vertex_properties(q)?;
    Ok(v)
}

pub fn get_all_workers(db: WrappedDb) -> Result<Vec<Worker>> {
    let workers = get_all_pools_with_workers(db)?
        .into_iter()
        .flat_map(|p| p.workers.unwrap_or_default())
        .collect::<Vec<_>>();
    Ok(workers)
}

pub fn add_pool(db: WrappedDb, cmd: ConfigCommands) -> Result<Uuid> {
    match cmd {
        ConfigCommands::AddPool {
            name,
            pid,
            disabled,
            sync_only,
        } => {
            let p = get_pool_by_pid(db.clone(), pid)?;
            if p.is_some() {
                anyhow::bail!("Pool already exists!")
            }
            let id =
                db.create_vertex_from_type(Identifier::new(ID_VERTEX_INVENTORY_POOL).unwrap())?;
            let uq: VertexQuery = SpecificVertexQuery { ids: vec![id] }.into();
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_POOL_NAME).unwrap(),
                },
                serde_json::Value::String(name),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_POOL_PID).unwrap(),
                },
                serde_json::Value::Number(serde_json::Number::from(pid)),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_POOL_ENABLED).unwrap(),
                },
                serde_json::Value::Bool(!disabled),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq,
                    name: Identifier::new(ID_PROP_POOL_SYNC_ONLY).unwrap(),
                },
                serde_json::Value::Bool(sync_only),
            )?;
            Ok(id)
        }
        _ => Err(anyhow!("Invalid command!")),
    }
}

pub fn update_pool(db: WrappedDb, cmd: ConfigCommands) -> Result<Uuid> {
    match cmd {
        ConfigCommands::UpdatePool {
            name,
            pid,
            disabled,
            sync_only,
        } => {
            let p = get_raw_pool_by_pid(db.clone(), pid)?;
            let Some(v) = p else {
                anyhow::bail!("Pool not found!")

            };
            let id = v.vertex.id;
            let uq: VertexQuery = SpecificVertexQuery { ids: vec![id] }.into();
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_POOL_NAME).unwrap(),
                },
                serde_json::Value::String(name),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_POOL_ENABLED).unwrap(),
                },
                serde_json::Value::Bool(!disabled),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_POOL_SYNC_ONLY).unwrap(),
                },
                serde_json::Value::Bool(sync_only),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq,
                    name: Identifier::new(ID_PROP_POOL_SYNC_ONLY).unwrap(),
                },
                serde_json::Value::Bool(sync_only),
            )?;
            Ok(id)
        }
        _ => Err(anyhow!("Invalid command!")),
    }
}

pub fn remove_pool(db: WrappedDb, pid: u64) -> Result<()> {
    let p = get_raw_pool_by_pid(db.clone(), pid)?;
    let workers = get_raw_workers_by_pid(db.clone(), pid)?;
    if !workers.is_empty() {
        anyhow::bail!("Pool not empty!")
    };
    let Some(p) = p else {
        anyhow::bail!("Pool not found!")
    };
    let q = SpecificVertexQuery {
        ids: vec![p.vertex.id],
    }
    .into();
    db.delete_vertices(q)?;
    Ok(())
}

pub fn get_raw_worker_by_name(db: WrappedDb, name: String) -> Result<Option<VertexProperties>> {
    let q = PropertyValueVertexQuery {
        name: Identifier::new(ID_PROP_WORKER_NAME).unwrap(),
        value: serde_json::Value::String(name),
    }
    .into();
    Ok(db.get_all_vertex_properties(q)?.first().cloned())
}

pub fn get_worker_by_name(db: WrappedDb, name: String) -> Result<Option<Worker>> {
    Ok(get_raw_worker_by_name(db, name)?.map(Into::into))
}

pub fn get_raw_workers_by_pid(db: WrappedDb, pid: u64) -> Result<Vec<VertexProperties>> {
    let pool = get_raw_pool_by_pid(db.clone(), pid)?.context("Pool not found")?;
    let pool = pool.vertex.id;

    let edges = SpecificVertexQuery { ids: vec![pool] }.into();
    let edges = PipeEdgeQuery {
        inner: Box::new(edges),
        direction: EdgeDirection::Inbound,
        limit: 4_294_967_294u32,
        t: Some(Identifier::new(ID_EDGE_BELONG_TO)?),
        high: None,
        low: None,
    }
    .into();
    let edges = db
        .get_edges(edges)?
        .into_iter()
        .map(|e| e.key.outbound_id)
        .collect::<Vec<_>>();

    let v = SpecificVertexQuery { ids: edges }.into();
    let v = db.get_all_vertex_properties(v)?;

    Ok(v)
}

pub fn get_workers_by_pid(db: WrappedDb, pid: u64) -> Result<Vec<Worker>> {
    let workers = get_raw_workers_by_pid(db, pid)?
        .into_iter()
        .map(|w| {
            let mut w: Worker = w.into();
            w.pid = Some(pid);
            w
        })
        .collect::<Vec<_>>();
    Ok(workers)
}

pub fn add_worker(db: WrappedDb, cmd: ConfigCommands) -> Result<Uuid> {
    match cmd {
        ConfigCommands::AddWorker {
            name,
            endpoint,
            stake,
            pid,
            disabled,
            sync_only,
            gatekeeper,
        } => {
            let stake = validate_bn_string(stake)?;
            let name = validate_worker_name_existence(db.clone(), name)?;
            let endpoint = validate_endpoint(endpoint)?;

            let p = get_raw_pool_by_pid(db.clone(), pid)?.context("Pool not found!")?;
            let p = p.vertex.id;

            let id = db.create_vertex_from_type(Identifier::new(ID_VERTEX_INVENTORY_WORKER)?)?;
            let uq: VertexQuery = SpecificVertexQuery { ids: vec![id] }.into();

            let setup = || -> Result<()> {
                db.set_vertex_properties(
                    VertexPropertyQuery {
                        inner: uq.clone(),
                        name: Identifier::new(ID_PROP_WORKER_NAME).unwrap(),
                    },
                    serde_json::Value::String(name),
                )?;
                db.set_vertex_properties(
                    VertexPropertyQuery {
                        inner: uq.clone(),
                        name: Identifier::new(ID_PROP_WORKER_ENDPOINT).unwrap(),
                    },
                    serde_json::Value::String(endpoint),
                )?;
                db.set_vertex_properties(
                    VertexPropertyQuery {
                        inner: uq.clone(),
                        name: Identifier::new(ID_PROP_WORKER_STAKE).unwrap(),
                    },
                    serde_json::Value::String(stake),
                )?;
                db.set_vertex_properties(
                    VertexPropertyQuery {
                        inner: uq.clone(),
                        name: Identifier::new(ID_PROP_WORKER_ENABLED).unwrap(),
                    },
                    serde_json::Value::Bool(!disabled),
                )?;
                db.set_vertex_properties(
                    VertexPropertyQuery {
                        inner: uq.clone(),
                        name: Identifier::new(ID_PROP_WORKER_SYNC_ONLY).unwrap(),
                    },
                    serde_json::Value::Bool(sync_only),
                )?;
                db.set_vertex_properties(
                    VertexPropertyQuery {
                        inner: uq.clone(),
                        name: Identifier::new(ID_PROP_WORKER_GATEKEEPER).unwrap(),
                    },
                    serde_json::Value::Bool(gatekeeper),
                )?;
                let e = EdgeKey {
                    outbound_id: id,
                    t: Identifier::new(ID_EDGE_BELONG_TO)?,
                    inbound_id: p,
                };
                db.create_edge(&e)?;
                Ok(())
            };
            let setup = setup();
            if setup.is_err() {
                db.delete_vertices(uq.clone())?;
                return Err(setup.err().unwrap());
            }

            Ok(id)
        }
        _ => Err(anyhow!("Bad arguments")),
    }
}

pub fn update_worker(db: WrappedDb, cmd: ConfigCommands) -> Result<Uuid> {
    match cmd {
        ConfigCommands::UpdateWorker {
            name,
            new_name,
            endpoint,
            pid,
            stake,
            disabled,
            sync_only,
            gatekeeper,
        } => {
            let worker =
                get_raw_worker_by_name(db.clone(), name.clone())?.context("Worker not found!")?;
            let id = worker.vertex.id;

            let name = match new_name {
                None => name,
                Some(nn) => nn,
            };

            let edges = SpecificVertexQuery { ids: vec![id] }.into();
            let edges = PipeEdgeQuery {
                inner: Box::new(edges),
                direction: EdgeDirection::Outbound,
                limit: 1,
                t: Some(Identifier::new(ID_EDGE_BELONG_TO)?),
                high: None,
                low: None,
            }
            .into();
            let edges = db.get_edges(edges)?;
            let ids = edges
                .clone()
                .into_iter()
                .map(|e| (e.key.inbound_id))
                .collect::<Vec<_>>();
            let keys = edges.into_iter().map(|e| e.key).collect::<Vec<_>>();

            let v = SpecificVertexQuery { ids }.into();
            let v = db.get_all_vertex_properties(v)?;
            let v = v.get(0).context("Worker is not attached to a pool!")?;
            let current_pool: Pool = v.clone().into();

            if current_pool.pid != pid {
                let new_pool =
                    get_raw_pool_by_pid(db.clone(), pid)?.context("New pool not found!")?;
                db.delete_edges(SpecificEdgeQuery { keys }.into())?;
                let e = EdgeKey {
                    outbound_id: id,
                    t: Identifier::new(ID_EDGE_BELONG_TO)?,
                    inbound_id: new_pool.vertex.id,
                };
                db.create_edge(&e)?;
            };

            let uq: VertexQuery = SpecificVertexQuery { ids: vec![id] }.into();
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_WORKER_NAME).unwrap(),
                },
                serde_json::Value::String(name),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_WORKER_ENDPOINT).unwrap(),
                },
                serde_json::Value::String(endpoint),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_WORKER_STAKE).unwrap(),
                },
                serde_json::Value::String(stake),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_WORKER_ENABLED).unwrap(),
                },
                serde_json::Value::Bool(!disabled),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq.clone(),
                    name: Identifier::new(ID_PROP_WORKER_SYNC_ONLY).unwrap(),
                },
                serde_json::Value::Bool(sync_only),
            )?;
            db.set_vertex_properties(
                VertexPropertyQuery {
                    inner: uq,
                    name: Identifier::new(ID_PROP_WORKER_GATEKEEPER).unwrap(),
                },
                serde_json::Value::Bool(gatekeeper),
            )?;

            Ok(id)
        }
        _ => Err(anyhow!("Bad arguments!")),
    }
}

pub fn remove_worker(db: WrappedDb, name: String) -> Result<()> {
    let worker = get_raw_worker_by_name(db.clone(), name)?.context("Worker not found!")?;
    let q = SpecificVertexQuery {
        ids: vec![worker.vertex.id],
    }
    .into();
    db.delete_vertices(q)?;
    Ok(())
}

pub fn setup_cache_index_db(db_path: &str, use_persisted_cache_index: bool) -> WrappedDb {
    let db: WrappedDb = if use_persisted_cache_index {
        let db_path = Path::new(db_path).join("index");
        let db = RocksdbDatastore::new(&db_path, None).expect("Failed to open inventory database.");
        debug!("Opened cache index database in {:?}", db_path);
        Arc::new(db)
    } else {
        let db = MemoryDatastore::default();
        debug!("Using in-memory database for cache index");
        Arc::new(db)
    };
    // TODO: setup indexes
    db
}
