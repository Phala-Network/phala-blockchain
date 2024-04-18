use crate::inv_db::{get_pool_by_pid, Worker};
use crate::pruntime::{PRuntimeClient, PRuntimeClientWithSemaphore};
use crate::pool_operator::PoolOperatorAccess;
use crate::utils::fetch_storage_bytes;
use crate::wm::{WorkerManagerMessage, WrappedWorkerManagerContext};
use crate::use_parachain_api;
use anyhow::{anyhow, Result};
use chrono::prelude::*;
use log::{debug, error, info, warn};

use phactory_api::prpc::{
    GetRuntimeInfoRequest, PhactoryInfo, SignEndpointsRequest,
};
use phala_pallets::pallet_computation::{SessionInfo, WorkerState};
use phala_pallets::registry::WorkerInfoV2;
use phala_types::AttestationProvider;
use pherry::chain_client::{mq_next_sequence, search_suitable_genesis_for_worker};

use pherry::attestation_to_report;
use serde::{Deserialize, Serialize};
use sp_core::sr25519::Public as Sr25519Public;
use sp_core::{ByteArray, Pair};

use std::sync::Arc;
use std::time::Duration;
use subxt::dynamic::{storage, Value};
use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex, RwLock};
use tokio::time::sleep;

pub type WorkerLifecycleCommandTx = mpsc::UnboundedSender<WorkerLifecycleCommand>;
pub type WorkerLifecycleCommandRx = mpsc::UnboundedReceiver<WorkerLifecycleCommand>;

pub enum WorkerLifecycleCommand {
    ShouldRestart,
    ShouldForceRegister,
    ShouldUpdateEndpoint(Vec<String>),
    ShouldTakeCheckpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerLifecycleState {
    Starting,
    Synchronizing,
    Preparing,
    Working,
    GatekeeperWorking,

    HasError(String),
    Restarting,
    Disabled,
}

pub type WrappedWorkerContext = Arc<RwLock<WorkerContext>>;
pub type WorkerLifecycleStateTx = mpsc::UnboundedSender<WorkerLifecycleState>;
pub type WorkerLifecycleStateRx = mpsc::UnboundedReceiver<WorkerLifecycleState>;

pub struct WorkerContext {
    pub id: String,
    pub self_ref: Option<WrappedWorkerContext>,
    pub sm_tx: Option<WorkerLifecycleStateTx>,
    pub worker: Worker,
    pub state: WorkerLifecycleState,
    pub tx: WorkerLifecycleCommandTx,
    pub rx: Arc<TokioMutex<WorkerLifecycleCommandRx>>,
    pub ctx: WrappedWorkerManagerContext,
    pub pr: Arc<PRuntimeClient>,
    pub info: Option<PhactoryInfo>,
    pub last_message: String,
    pub session_info: Option<SessionInfo>,
}
