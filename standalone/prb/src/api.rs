use crate::cli::WorkerManagerCliArgs;
use crate::db::Worker;
use crate::wm::WrappedWorkerManagerContext;
use crate::worker::WorkerLifecycleState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::*;
use axum::{Json, Router};
use futures::future::try_join_all;
use log::info;
use phactory_api::prpc::PhactoryInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

type AppContext = State<WrappedWorkerManagerContext>;

pub async fn start_api_server(ctx: WrappedWorkerManagerContext, args: WorkerManagerCliArgs) {
    // todo: mdns

    let app = Router::new()
        .route("/", get(handle_get_root))
        .route("/wm/status", get(handle_get_wm_status))
        .route("/wm/restart", put(handle_restart_wm))
        .route("/workers/status", get(handle_get_worker_status))
        .route("/workers/restart", put(handle_restart_specific_workers))
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

    let _ = try_join_all(fut_vec).await;
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

pub type WorkerStatusMap = HashMap<String, WorkerStatus>;

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
