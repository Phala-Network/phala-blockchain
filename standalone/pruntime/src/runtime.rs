use crate::pal_gramine::GraminePlatform;

use anyhow::Result;
use core::sync::atomic::{AtomicU32, Ordering};
use phactory::{benchmark, Phactory, RpcService};
use rocket::http::Status;
use sidevm_host_runtime::rocket_stream::{connect, RequestInfo, StreamResponse};
use std::path::PathBuf;
use tracing::info;

lazy_static::lazy_static! {
    static ref APPLICATION: RpcService<GraminePlatform> = RpcService::new(GraminePlatform);
}

pub fn ecall_handle(req_id: u64, action: u8, input: &[u8]) -> Result<Vec<u8>> {
    let allow_rcu = action == phactory_api::actions::ACTION_GET_INFO;
    let mut factory = APPLICATION.lock_phactory(allow_rcu, true)?;
    Ok(factory.handle_scale_api(req_id, action, input))
}

pub fn ecall_getinfo() -> String {
    let Ok(guard) = APPLICATION.lock_phactory(true, true) else {
        return r#"{"error": "Failed to lock Phactory""#.into();
    };
    let info = guard.get_info();
    serde_json::to_string_pretty(&info).unwrap_or_default()
}

pub fn ecall_sign_http_response(data: &[u8]) -> Option<String> {
    APPLICATION
        .lock_phactory(true, true)
        .ok()?
        .sign_http_response(data)
}

pub fn ecall_init(args: phactory_api::ecall_args::InitArgs) -> Result<()> {
    static INITIALIZED: AtomicU32 = AtomicU32::new(0);
    if INITIALIZED.fetch_add(1, Ordering::SeqCst) != 0 {
        anyhow::bail!("Enclave already initialized.");
    }

    if args.enable_checkpoint {
        match Phactory::restore_from_checkpoint(&GraminePlatform, &args) {
            Ok(Some(factory)) => {
                info!("Loaded checkpoint");
                **APPLICATION
                    .lock_phactory(true, true)
                    .expect("Failed to lock Phactory") = factory;
                return Ok(());
            }
            Err(err) => {
                anyhow::bail!("Failed to load checkpoint: {:?}", err);
            }
            Ok(None) => {
                info!("No checkpoint found");
            }
        }
    } else {
        info!("Checkpoint disabled.");
    }

    APPLICATION.lock_phactory(true, true).unwrap().init(args);

    info!("Enclave init OK");
    Ok(())
}

pub fn ecall_bench_run(index: u32) {
    if !benchmark::paused() {
        info!(index, "Benchmark thread started");
        benchmark::run();
    }
}

pub async fn ecall_prpc_request(
    req_id: u64,
    path: String,
    data: &[u8],
    json: bool,
) -> (u16, Vec<u8>) {
    info!(%path, json, "Handling pRPC request");
    let (code, data) = APPLICATION.dispatch_request(req_id, path, data, json).await;
    info!(code, size = data.len(), "pRPC returned");
    (code, data)
}

pub(crate) async fn ecall_connect_sidevm(
    head: RequestInfo,
    id: String,
    path: PathBuf,
    body: Option<rocket::Data<'_>>,
) -> Result<StreamResponse, (Status, String)> {
    let contract_id = hex::decode(&id.trim_start_matches("0x"))
        .map_err(|err| (Status::BadRequest, err.to_string()))?;
    let Some(command_tx) = APPLICATION
        .lock_phactory(false, false)
        .map_err(|err| (Status::InternalServerError, err.to_string()))?
        .sidevm_command_sender(&contract_id)
    else {
        return Err((Status::NotFound, Default::default()));
    };
    let path = path
        .to_str()
        .ok_or((Status::BadRequest, "Invalid path".to_string()))?;
    let result = connect(head, path, body, command_tx).await;
    match result {
        Ok(response) => Ok(response),
        Err(err) => Err((Status::InternalServerError, err.to_string())),
    }
}
