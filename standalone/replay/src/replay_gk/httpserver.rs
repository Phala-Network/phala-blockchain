use std::str::FromStr;

use super::*;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use sp_runtime::AccountId32;

struct AppState {
    factory: Arc<Mutex<ReplayFactory>>,
}

#[get("/meminfo")]
async fn meminfo(data: web::Data<AppState>) -> HttpResponse {
    let factory = data.factory.lock().await;
    let size = factory
        .storage
        .inner()
        .pairs(&[])
        .iter()
        .map(|(k, v)| k.len() + v.len())
        .sum::<usize>();
    log::info!("Storage size: {}", size);
    HttpResponse::Ok().json(serde_json::json!({
            "storage_size": size,
    }))
}

#[get("/worker-state/{pubkey}")]
async fn get_worker_state(pubkey: web::Path<String>, data: web::Data<AppState>) -> HttpResponse {
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
            "worker": state,
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
        .map(|(k, v)| ("0x".to_string() + &hex::encode(k), v))
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
            .app_data(web::Data::new(AppState { factory }))
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
