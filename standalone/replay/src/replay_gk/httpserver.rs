use std::str::FromStr;

use super::*;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use subxt::sp_runtime::AccountId32;

struct AppState {
    factory: Arc<Mutex<ReplayFactory>>,
}

#[get("/meminfo")]
async fn meminfo(
    data: web::Data<AppState>,
) -> HttpResponse {
    let factory = data.factory.lock().await;
    let size = factory.storage.pairs(&[]).iter().map(|(k, v)| k.len() + v.len()).sum::<usize>();
    log::info!("Storage size: {}", size);
    HttpResponse::Ok().json(serde_json::json!({
            "storage_size": size,
    }))
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

    match factory.gk.worker_state(&pubkey) {
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Worker not found"
        })),
        Some(state) => HttpResponse::Ok().json(serde_json::json!({
            "current_block": factory.current_block,
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
        })),
    }
}

pub async fn serve(bind_addr: String, factory: Arc<Mutex<ReplayFactory>>) {
    HttpServer::new(move || {
        let factory = factory.clone();
        App::new()
            .data(AppState { factory })
            .service(get_worker_state)
            .service(meminfo)
    })
    .disable_signals()
    .bind(&bind_addr)
    .expect("Can not bind http server")
    .run()
    .await
    .expect("Http server failed");
}
