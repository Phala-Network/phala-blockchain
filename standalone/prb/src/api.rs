use crate::api::ApiError::LifecycleManagerNotInitialized;
use crate::cli::WorkerManagerCliArgs;
use crate::db::Worker;
use crate::wm::WrappedWorkerManagerContext;
use crate::worker::WorkerLifecycleState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::*;
use axum::{Json, Router};
use futures::future::try_join_all;
use log::{info, warn};
use phactory_api::prpc::PhactoryInfo;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;

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

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        warn!("{}", &self);
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
                    "code": format!("{:?}", self),
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

pub type WorkerStatusMap = HashMap<String, WorkerStatus>;

macro_rules! use_current_lm {
    () => {{
        let cc = ctx.clone()
        let cc = cc.read().await;
        let lm = cc.current_lifecycle_manager.ok_or(ApiError::LifecycleManagerNotInitialized)?.clone();
        drop(lm);
        lm
    }};
}

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
        .route("/tx/status", get(handle_get_status))
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
) -> (StatusCode, Json<WorkerStatusResponse>) {
    let ctx = ctx.read().await;
    let workers = &ctx.state_map;
    let workers = workers.iter().map(|(_k, v)| v.clone()).collect::<Vec<_>>();
    (StatusCode::OK, Json(WorkerStatusResponse { workers }))
}

async fn handle_restart_specific_workers() {
    todo!()
}

async fn handle_get_status() -> ApiResult<(StatusCode, ())> {
    None.ok_or(LifecycleManagerNotInitialized)?
}
