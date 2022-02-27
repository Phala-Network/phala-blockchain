use std::str::FromStr;

use super::*;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use phactory_api::prpc as pb;
use subxt::sp_runtime::AccountId32;

struct AppState {
    factory: Arc<Mutex<ReplayFactory>>,
}

#[get("/meminfo")]
async fn meminfo(data: web::Data<AppState>) -> HttpResponse {
    let factory = data.factory.lock().await;
    let size = factory
        .storage
        .pairs(&[])
        .iter()
        .map(|(k, v)| k.len() + v.len())
        .sum::<usize>();
    log::info!("Storage size: {}", size);
    HttpResponse::Ok().json(serde_json::json!({
            "storage_size": size,
    }))
}

fn serialize_worker_state(state: &pb::WorkerState) -> serde_json::Value {
    serde_json::json!({
        "benchmarking": state.bench_state.is_some(),
        "mining": state.mining_state.is_some(),
        "unresponsive": state.unresponsive,
        "last_heartbeat_for_block": state.last_heartbeat_for_block,
        "last_heartbeat_at_block": state.last_heartbeat_at_block,
        "waiting_heartbeats": state.waiting_heartbeats,
        "v": state.tokenomic_info.as_ref().map(|info| info.v.clone()),
        "v_init": state.tokenomic_info.as_ref().map(|info| info.v_init.clone()),
        "p_instant": state.tokenomic_info.as_ref().map(|info| info.p_instant.clone()),
        "p_init": state.tokenomic_info.as_ref().map(|info| info.p_bench.clone()),
        "tokenomic_info": format!("{:#?}", state.tokenomic_info),
    })
}

#[get("/worker-state/{pubkey}")]
async fn get_worker_state(
    web::Path(pubkey): web::Path<String>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let factory = data.factory.lock().await;
    let pubkey = match AccountId32::from_str(pubkey.as_str()) {
        Ok(accid) => WorkerPublicKey(accid.into()),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid pubkey"
            }));
        }
    };

    let total_share = factory.gk.sum_share();
    match factory.gk.worker_state(&pubkey) {
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Worker not found"
        })),
        Some(state) => HttpResponse::Ok().json(serde_json::json!({
            "current_block": factory.current_block,
            "total_share": total_share.to_string(),
            "worker": serialize_worker_state(&state),
        })),
    }
}

#[get("/workers")]
async fn dump_workers(data: web::Data<AppState>) -> HttpResponse {
    let factory = data.factory.lock().await;

    let total_share = factory.gk.sum_share();
    let workers = factory.gk.dump_workers_state();
    let workers: std::collections::BTreeMap<_, _> = workers
        .iter()
        .map(|(k, v)| {
            (
                "0x".to_string() + &hex::encode(&k),
                serialize_worker_state(&v),
            )
        })
        .collect();
    HttpResponse::Ok().json(serde_json::json!({
        "current_block": factory.current_block,
        "total_share": total_share.to_string(),
        "workers": workers
    }))
}

pub async fn serve(bind_addr: String, factory: Arc<Mutex<ReplayFactory>>) {
    HttpServer::new(move || {
        let factory = factory.clone();
        App::new()
            .data(AppState { factory })
            .service(get_worker_state)
            .service(meminfo)
            .service(dump_workers)
    })
    .disable_signals()
    .bind(&bind_addr)
    .expect("Can not bind http server")
    .run()
    .await
    .expect("Http server failed");
}
