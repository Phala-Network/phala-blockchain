use crate::datasource::WrappedDataSourceManager;
use crate::tx::TxManagerError::*;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use phaxt::subxt::dynamic::{self, Value};
use phaxt::subxt::tx::DynamicTxPayload;
use rocksdb::{DBCompactionStyle, DBWithThreadMode, MultiThreaded, Options};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

static TX_QUEUE_CHUNK_SIZE: usize = 30;
static TX_QUEUE_CHUNK_TIMEOUT_IN_MS: u64 = 1000;

type DB = DBWithThreadMode<MultiThreaded>;
type TxGroupMap = HashMap<u64, TxGroup>;
type TxGroup = Vec<Transaction>;

type DynamicTxPayloadStatic = DynamicTxPayload<'static>;

fn get_options(max_open_files: Option<i32>) -> Options {
    // Current tuning based off of the total ordered example, flash
    // storage example on
    // https://github.com/facebook/rocksdb/wiki/RocksDB-Tuning-Guide
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_compaction_style(DBCompactionStyle::Level);
    opts.set_write_buffer_size(67_108_864); // 64mb
    opts.set_max_write_buffer_number(3);
    opts.set_target_file_size_base(67_108_864); // 64mb
    opts.set_level_zero_file_num_compaction_trigger(8);
    opts.set_level_zero_slowdown_writes_trigger(17);
    opts.set_level_zero_stop_writes_trigger(24);
    opts.set_num_levels(4);
    opts.set_max_bytes_for_level_base(536_870_912); // 512mb
    opts.set_max_bytes_for_level_multiplier(8.0);

    if let Some(max_open_files) = max_open_files {
        opts.set_max_open_files(max_open_files);
    }

    opts
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TransactionState {
    Pending,
    Running,
    Success(TransactionSuccess),
    Error(TransactionErrorMessage),
}

#[derive(thiserror::Error, Clone, Debug, Serialize)]
pub enum TxManagerError {
    #[error("Operator of pool #{0} not set")]
    PoolOperatorNotSet(u64),

    #[error("There is no valid substrate data source")]
    NoValidSubstrateDataSource,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionSuccess {
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionErrorMessage {
    pub updated_at: DateTime<Utc>,
    pub message: String,
}

impl Debug for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(r) => write!(f, "{r}"),
            Err(e) => {
                panic!("{:?}", &e);
            }
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(r) => write!(f, "{r}"),
            Err(e) => {
                panic!("{:?}", &e);
            }
        }
    }
}

#[derive(Serialize)]
pub struct Transaction {
    pub id: usize,
    pub state: TransactionState,
    pub desc: String,
    pub pid: u64,
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub tx_payload: Option<DynamicTxPayloadStatic>,
    #[serde(skip)]
    pub shot: Option<oneshot::Sender<Result<()>>>,
}

impl Clone for Transaction {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            state: self.state.clone(),
            desc: self.desc.clone(),
            pid: self.pid,
            created_at: self.created_at,
            tx_payload: None,
            shot: None,
        }
    }
}

impl Transaction {
    pub fn new(
        id: usize,
        pid: u64,
        tx_payload: DynamicTxPayloadStatic,
        desc: String,
        shot: oneshot::Sender<Result<()>>,
    ) -> Self {
        Self {
            id,
            state: TransactionState::Pending,
            desc,
            pid,
            created_at: Utc::now(),
            tx_payload: Some(tx_payload),
            shot: Some(shot),
        }
    }
}

pub struct TxManager {
    pub db: Arc<DB>,
    pub total_tx_count: Arc<Mutex<usize>>,
    dsm: WrappedDataSourceManager,
    queue_tx: mpsc::UnboundedSender<Transaction>,
    pub pending_tx_count: Arc<Mutex<usize>>,
    pub running_txs: Arc<RwLock<Vec<Transaction>>>,
    pub finished_txs: Arc<RwLock<VecDeque<Transaction>>>,
}

impl TxManager {
    pub fn new(
        path_base: &str,
        dsm: WrappedDataSourceManager,
    ) -> Result<(Arc<Self>, JoinHandle<Result<()>>)> {
        let opts = get_options(None);
        let path = Path::new(path_base).join("po");
        let db = DB::open(&opts, path)?;
        let (tx, rx) = mpsc::unbounded_channel::<Transaction>();

        let txm = Arc::new(TxManager {
            db: Arc::new(db),
            queue_tx: tx,
            dsm,
            total_tx_count: Arc::new(Mutex::new(0)),
            pending_tx_count: Arc::new(Mutex::new(0)),
            running_txs: Arc::new(RwLock::new(Vec::new())),
            finished_txs: Arc::new(RwLock::new(VecDeque::new())),
        });
        let handle = tokio::spawn(txm.clone().start_trader(rx));

        Ok((txm, handle))
    }
    async fn start_trader(self: Arc<Self>, rx: mpsc::UnboundedReceiver<Transaction>) -> Result<()> {
        let rx_stream = UnboundedReceiverStream::new(rx).chunks_timeout(
            TX_QUEUE_CHUNK_SIZE,
            Duration::from_millis(TX_QUEUE_CHUNK_TIMEOUT_IN_MS),
        );
        tokio::pin!(rx_stream);

        while let Some(i) = rx_stream.next().await {
            let rt = self.running_txs.clone();
            let mut rt = rt.write().await;
            let ft = self.finished_txs.clone();
            let mut ft = ft.write().await;
            while let Some(t) = rt.pop() {
                ft.push_front(t);
            }
            drop(rt);
            drop(ft);

            let mut tx_map: TxGroupMap = HashMap::new();
            for i in i {
                let pid = i.pid;
                if let Some(group) = tx_map.get_mut(&pid) {
                    group.push(i);
                } else {
                    let group = vec![i];
                    let _ = tx_map.insert(pid, group);
                };
            }
            join_all(
                tx_map
                    .into_values()
                    .map(|g| self.clone().send_tx_group(g))
                    .collect::<Vec<_>>(),
            )
            .await;
        }
        Err(anyhow!("Unexpected exit of start_trader."))
    }

    async fn send_tx_group(self: Arc<Self>, tx_group: TxGroup) -> Result<()> {
        if tx_group.is_empty() {
            anyhow::bail!("TxGroup can't be empty!");
        }
        let mut payloads = Vec::new();
        let mut shots = Vec::new();

        let r = self.running_txs.clone();
        let mut r = r.write().await;
        let p = self.pending_tx_count.clone();
        let mut p = p.lock().await;
        *p -= tx_group.len();

        for mut t in tx_group {
            t.state = TransactionState::Running;
            r.push(t.clone());
            shots.push(t.shot.unwrap());
            payloads.push(t.tx_payload.unwrap());
        }
        drop(r);
        drop(p);

        Ok(())
    }

    pub async fn send_to_queue(
        self: Arc<Self>,
        pid: u64,
        tx_payload: DynamicTxPayloadStatic,
        desc: String,
    ) -> Result<()> {
        let id = self.total_tx_count.clone();
        let mut id = id.lock().await;
        *id += 1;
        let (tx, rx) = oneshot::channel();
        let c = self.pending_tx_count.clone();
        let mut c = c.lock().await;
        *c += 1;
        let tx = Transaction::new(*id, pid, tx_payload, desc, tx);
        self.queue_tx.clone().send(tx)?;
        drop(c);
        drop(id);
        rx.await?
    }
}

impl TxManager {
    pub async fn register_worker(
        self: Arc<Self>,
        pid: u64,
        pruntime_info: Vec<u8>,
        attestation: Vec<u8>,
        v2: bool,
    ) -> Result<()> {
        let tx_payload = if v2 {
            dynamic::tx(
                "PhalaRegistry",
                "register_worker_v2",
                vec![
                    Value::from_bytes(&pruntime_info),
                    Value::from_bytes(&attestation),
                ],
            )
        } else {
            dynamic::tx(
                "PhalaRegistry",
                "register_worker",
                vec![
                    Value::from_bytes(&pruntime_info),
                    Value::from_bytes(&attestation),
                ],
            )
        };
        let desc = format!("Register worker for pool #{pid}");
        self.clone().send_to_queue(pid, tx_payload, desc).await
    }
}
