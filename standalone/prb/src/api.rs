use crate::api::ApiError::{LifecycleManagerNotInitialized, WorkerNotFound};
use crate::cli::{ConfigCommands, WorkerManagerCliArgs};
use crate::configurator::api_handler;
use crate::db::Worker;
use crate::tx::Transaction;
use crate::wm::WorkerManagerMessage::ShouldResetLifecycleManager;
use crate::wm::{send_to_main_channel, WrappedWorkerManagerContext};
use crate::worker::{WorkerLifecycleCommand, WorkerLifecycleState, WrappedWorkerContext};
use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::*;
use axum::{Json, Router};
use futures::future::try_join_all;
use log::info;
use phactory_api::prpc::PhactoryInfo;
use phala_pallets::pallet_computation::SessionInfo;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

type AppContext = State<WrappedWorkerManagerContext>;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Server error")]
    ServerError(anyhow::Error),

    #[error("lifecycle manager not initialized yet")]
    LifecycleManagerNotInitialized,

    #[error("worker not found: {0}")]
    WorkerNotFound(String),

    #[error("pool not found: {0}")]
    PoolNotFound(u64),

    #[error("db write failed")]
    WriteFailed,
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkerStatus {
    pub worker: Worker,
    pub state: WorkerLifecycleState,
    pub phactory_info: Option<PhactoryInfo>,
    pub last_message: String,
    pub session_info: Option<SessionInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkerStatusResponse {
    workers: Vec<WorkerStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxStatusResponse {
    pub tx_count: usize,
    pub running_txs: Vec<Transaction>,
    pub pending_txs: Vec<Transaction>,
    pub past_txs: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OkResponse {
    pub ok: bool,
}
impl Default for OkResponse {
    fn default() -> Self {
        Self { ok: true }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdsRequest {
    pub ids: Vec<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::ServerError(e) => {
                let backtrace = e.backtrace().to_string();
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": true,
                        "code": "ServerError",
                        "message": format!("{}", &e),
                        "backtrace": backtrace
                    })),
                )
            }
            .into_response(),
            _ => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": true,
                    "code": format!("{:?}", &self),
                    "message": format!("{self}"),
                })),
            )
                .into_response(),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::ServerError(err)
    }
}

pub type WorkerContexts = Vec<WrappedWorkerContext>;
pub type WrappedWorkerContexts = Arc<Mutex<WorkerContexts>>;

pub async fn start_api_server(
    ctx: WrappedWorkerManagerContext,
    args: WorkerManagerCliArgs,
) -> anyhow::Result<()> {
    // todo: mdns

    let app = Router::new()
        .route("/", get(handle_get_root))
        .route("/wm/status", get(handle_get_wm_status))
        .route("/wm/restart", put(handle_restart_wm))
        .route("/wm/config", post(handle_config_wm))
        .route("/workers/status", get(handle_get_worker_status))
        .route("/workers/restart", put(handle_restart_specific_workers))
        .route("/tx/status", get(handle_get_tx_status))
        .fallback(handle_get_root)
        .with_state(ctx);

    let fut_vec = args
        .mgmt_listen_addresses
        .into_iter()
        .map(|addr| {
            info!("Listening on {} for management interface.", &addr);
            let addr = SocketAddr::parse_ascii(addr.as_bytes()).unwrap();
            axum::Server::bind(&addr).serve(app.clone().into_make_service())
        })
        .collect::<Vec<_>>();

    try_join_all(fut_vec).await?;
    Ok(())
}

async fn handle_get_root() -> (StatusCode, ()) {
    (StatusCode::IM_A_TEAPOT, ())
}

async fn handle_get_wm_status() {
    todo!()
}

async fn handle_restart_wm(State(ctx): AppContext) -> ApiResult<(StatusCode, Json<OkResponse>)> {
    let tx = ctx.current_lifecycle_tx.clone();
    let tx = tx.lock().await;
    let tx_move = tx.as_ref().ok_or(LifecycleManagerNotInitialized)?.clone();
    drop(tx);
    send_to_main_channel(tx_move, ShouldResetLifecycleManager).await?;
    Ok((StatusCode::OK, Json(OkResponse::default())))
}

async fn handle_get_worker_status(
    State(ctx): AppContext,
) -> ApiResult<(StatusCode, Json<WorkerStatusResponse>)> {
    let w = ctx.workers.clone();
    let w = w.lock().await;
    let mut workers = Vec::new();
    for w in w.iter() {
        let w = w.clone();
        let w = w.read().await;
        workers.push(WorkerStatus {
            worker: w.worker.clone(),
            state: w.state.clone(),
            phactory_info: w.info.clone(),
            last_message: w.last_message.clone(),
            session_info: w.session_info.clone(),
        })
    }
    Ok((StatusCode::OK, Json(WorkerStatusResponse { workers })))
}

async fn handle_restart_specific_workers(
    State(ctx): State<WrappedWorkerManagerContext>,
    Json(payload): Json<IdsRequest>,
) -> ApiResult<(StatusCode, Json<OkResponse>)> {
    let worker_map = ctx.worker_map.clone();
    let worker_map = worker_map.lock().await;
    let mut c: Vec<WrappedWorkerContext> = vec![];

    for id in payload.ids {
        c.push(
            worker_map
                .get(id.as_str())
                .ok_or(WorkerNotFound(id))?
                .clone(),
        )
    }
    for c in c {
        let c = c.read().await;
        match &c.state {
            WorkerLifecycleState::Restarting => drop(c),
            _ => {
                let tx = c.tx.clone();
                drop(c);
                tx.send(WorkerLifecycleCommand::ShouldRestart)
                    .map_err(|e| anyhow!(e.to_string()))?;
            }
        }
    }

    Ok((StatusCode::OK, Json(OkResponse::default())))
}

async fn handle_get_tx_status(
    State(ctx): AppContext,
) -> ApiResult<(StatusCode, Json<TxStatusResponse>)> {
    let txm = ctx.txm.clone();
    Ok((StatusCode::OK, Json(txm.dump().await?)))
}

async fn handle_config_wm(
    State(ctx): State<WrappedWorkerManagerContext>,
    Json(payload): Json<ConfigCommands>,
) -> ApiResult<String> {
    let po_db = ctx.txm.db.clone();
    let inv_db = ctx.inv_db.clone();
    let ret = api_handler(inv_db, po_db, payload).await?;
    Ok(ret)
}
