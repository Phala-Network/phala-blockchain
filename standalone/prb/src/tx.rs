use self::khala::runtime_types;
use self::khala::runtime_types::khala_parachain_runtime::RuntimeCall;
use self::khala::runtime_types::phala_pallets::utils::attestation_legacy;
use crate::datasource::WrappedDataSourceManager;
use crate::tx::TxManagerError::*;
use crate::use_parachain_api;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use futures::future::{join_all, BoxFuture};
use hex::ToHex;
use log::{error, info};
use parity_scale_codec::{Decode, Encode};
use rocksdb::{DBCommon, DBCompactionStyle, DBWithThreadMode, MultiThreaded, Options};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use subxt::dynamic::{self, Value};
use subxt::tx::{DynamicTxPayload, StaticTxPayload, TxPayload};
use subxt::utils::AccountId32;
use subxt::Metadata;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

#[subxt::subxt(runtime_metadata_url = "wss://khala.api.onfinality.io:443/public-ws")]
pub mod khala {}

static TX_QUEUE_CHUNK_SIZE: usize = 30;
static TX_QUEUE_CHUNK_TIMEOUT_IN_MS: u64 = 1000;

type DB = DBWithThreadMode<MultiThreaded>;
type TxGroupMap = HashMap<u64, TxGroup>;
type TxGroup = Vec<Transaction>;

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
    pub tx_payload: Option<RuntimeCall>,
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
        tx_payload: RuntimeCall,
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
    metadata: Metadata,
}

impl TxManager {
    pub fn new(
        path_base: &str,
        dsm: WrappedDataSourceManager,
        metadata: Metadata,
    ) -> Result<(Arc<Self>, BoxFuture<'static, Result<()>>)> {
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
            metadata,
        });
        let handle = Box::pin(txm.clone().start_trader(rx));

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
                    .into_iter()
                    .map(|(pid, v)| {
                        let self_move = self.clone();
                        async move {
                            if let Err(e) = self_move.clone().wrap_send_tx_group(pid, v).await {
                                error!("wrap_send_tx_group: {e}");
                                std::process::exit(255);
                            }
                        }
                    })
                    .collect::<Vec<_>>(),
            )
            .await;
        }
        Err(anyhow!("Unexpected exit of start_trader."))
    }
    async fn wrap_send_tx_group(self: Arc<Self>, pid: u64, tx_group: TxGroup) -> Result<()> {
        if tx_group.is_empty() {
            anyhow::bail!("TxGroup can't be empty!");
        }
        let mut shots = VecDeque::new();
        let mut payloads = Vec::new();

        let r = self.running_txs.clone();
        let mut r = r.write().await;
        let p = self.pending_tx_count.clone();
        let mut p = p.lock().await;
        *p -= tx_group.len();

        for mut t in tx_group {
            t.state = TransactionState::Running;
            r.push(t.clone());
            shots.push_back(t.shot.unwrap());
            payloads.push(t.tx_payload.unwrap());
        }
        drop(r);
        drop(p);

        match self.send_tx_group(pid, payloads).await {
            Ok(ret) => {
                for r in ret {
                    let tx = shots
                        .pop_front()
                        .ok_or(anyhow!("unexpected of absence of channel shot"))?;
                    if let Err(_) = tx.send(r) {
                        return Err(anyhow!("shot can't be sent"));
                    }
                }
            }
            Err(e) => {
                error!("send_tx_group: {}", &e);
                for tx in shots {
                    if let Err(_) = tx.send(Err(anyhow!(e.to_string()))) {
                        return Err(anyhow!("shot can't be sent"));
                    }
                }
            }
        }
        Ok(())
    }
    async fn send_tx_group(
        self: Arc<Self>,
        pid: u64,
        payloads: Vec<RuntimeCall>,
    ) -> Result<Vec<Result<()>>> {
        let api = use_parachain_api!(self.dsm, false).ok_or(NoValidSubstrateDataSource)?;
        let tx = khala::tx().utility().force_batch(payloads);
        let tt = api.tx().call_data(&tx)?.encode_hex::<String>();

        info!("{}", &tt);

        Ok(vec![Ok(())])
    }

    pub async fn send_to_queue(
        self: Arc<Self>,
        pid: u64,
        tx_payload: RuntimeCall,
        desc: String,
    ) -> Result<()> {
        let id = self.total_tx_count.clone();
        let mut id = id.lock().await;
        *id += 1;
        let (shot, rx) = oneshot::channel();
        tokio::pin!(rx);
        let c = self.pending_tx_count.clone();
        let mut c = c.lock().await;
        *c += 1;
        let tx = Transaction::new(*id, pid, tx_payload, desc, shot);
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
        mut pruntime_info: Vec<u8>,
        mut attestation: Vec<u8>,
        v2: bool,
    ) -> Result<()> {
        let tx_payload = if v2 {
            let mut pruntime_info = &pruntime_info[..];
            let pruntime_info =
                runtime_types::phala_types::WorkerRegistrationInfoV2::decode(&mut pruntime_info)?;
            let mut attestation = &attestation[..];
            let attestation = Option::decode(&mut attestation)?;
            RuntimeCall::PhalaRegistry(
                khala::runtime_types::phala_pallets::registry::pallet::Call::register_worker_v2 {
                    pruntime_info,
                    attestation,
                },
            )
        } else {
            let mut pruntime_info = &pruntime_info[..];
            let pruntime_info =
                runtime_types::phala_types::WorkerRegistrationInfo::decode(&mut pruntime_info)?;
            let mut attestation = &attestation[..];
            let attestation = attestation_legacy::Attestation::decode(&mut attestation)?;
            RuntimeCall::PhalaRegistry(
                khala::runtime_types::phala_pallets::registry::pallet::Call::register_worker {
                    pruntime_info,
                    attestation,
                },
            )
        };

        let desc = format!("Register worker");
        self.clone().send_to_queue(pid, tx_payload, desc).await
    }
    pub async fn update_worker_endpoint(
        self: Arc<Self>,
        pid: u64,
        mut endpoint_payload: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<()> {
        let mut endpoint_payload = &endpoint_payload[..];
        let endpoint_payload =
            runtime_types::phala_types::WorkerEndpointPayload::decode(&mut endpoint_payload)?;
        let tx_payload = RuntimeCall::PhalaRegistry(
            khala::runtime_types::phala_pallets::registry::pallet::Call::update_worker_endpoint {
                endpoint_payload,
                signature,
            },
        );
        let desc = format!("Update endpoint of worker.");
        self.clone().send_to_queue(pid, tx_payload, desc).await
    }
    pub async fn sync_offchain_message(
        self: Arc<Self>,
        pid: u64,
        signed_message: runtime_types::phala_mq::types::SignedMessage,
    ) -> Result<()> {
        let tx_payload = RuntimeCall::PhalaMq(
            khala::runtime_types::phala_pallets::mq::pallet::Call::sync_offchain_message {
                signed_message,
            },
        );
        let desc = format!("Sync offchain message to chain.");
        self.clone().send_to_queue(pid, tx_payload, desc).await
    }
}

#[derive(Clone)]
pub struct InnerPoolOperator {
    pub uid: u64,
}

impl Serialize for InnerPoolOperator {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for InnerPoolOperator {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PoolOperator {}

pub trait PoolOperatorAccess {
    fn get_all(&self) -> Vec<InnerPoolOperator>;
    fn get(&self, uid: u64) -> Result<Option<InnerPoolOperator>>;
    fn set(&self, uid: u64, po: InnerPoolOperator) -> Result<InnerPoolOperator>;
}

impl PoolOperatorAccess for DB {
    fn get_all(&self) -> Vec<InnerPoolOperator> {
        todo!()
    }

    fn get(&self, uid: u64) -> Result<Option<InnerPoolOperator>> {
        todo!()
    }

    fn set(&self, uid: u64, po: InnerPoolOperator) -> Result<InnerPoolOperator> {
        todo!()
    }
}
