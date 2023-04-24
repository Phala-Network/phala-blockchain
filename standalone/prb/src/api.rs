use crate::cli::WorkerManagerCliArgs;
use crate::db::Worker;
use crate::tx::Transaction;
use crate::wm::WrappedWorkerManagerContext;
use crate::worker::{WorkerLifecycleState, WrappedWorkerContext};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::*;
use axum::{Json, Router};
use futures::future::try_join_all;
use log::info;
use phactory_api::prpc::PhactoryInfo;
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
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkerStatus {
    pub worker: Worker,
    pub state: WorkerLifecycleState,
    pub phactory_info: Option<PhactoryInfo>,
    pub last_message: String,
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

async fn handle_restart_wm() {
    todo!()
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
        })
    }
    Ok((StatusCode::OK, Json(WorkerStatusResponse { workers })))
}

async fn handle_restart_specific_workers() {
    todo!()
}

async fn handle_get_tx_status(
    State(ctx): AppContext,
) -> ApiResult<(StatusCode, Json<TxStatusResponse>)> {
    let txm = ctx.txm.clone();
    Ok((StatusCode::OK, Json(txm.dump().await?)))
}
