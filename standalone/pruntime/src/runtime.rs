use crate::pal_gramine::GraminePlatform;

use anyhow::Result;
use core::sync::atomic::{AtomicU32, Ordering};
use log::info;
use phactory::{benchmark, Phactory, RpcService};

lazy_static::lazy_static! {
    static ref APPLICATION: RpcService<GraminePlatform> = RpcService::new(GraminePlatform);
}

pub fn ecall_handle(action: u8, input: &[u8]) -> Result<Vec<u8>> {
    let mut factory = APPLICATION.lock_phactory();
    Ok(factory.handle_scale_api(action, input))
}

pub fn ecall_getinfo() -> String {
    let info = APPLICATION.lock_phactory().get_info();
    serde_json::to_string_pretty(&info).unwrap_or_default()
}

pub fn ecall_get_contract_info(id: &str) -> String {
    let result = APPLICATION.lock_phactory().get_contract_info(id);
    match result {
        Ok(info) => serde_json::to_string_pretty(&info.contracts).unwrap_or_default(),
        Err(err) => {
            let error = format!("{:?}", err);
            serde_json::to_string_pretty(&serde_json::json!({
                "error": error
            }))
        }
        .unwrap_or_default(),
    }
}

pub fn ecall_sign_http_response(data: &[u8]) -> Option<String> {
    APPLICATION.lock_phactory().sign_http_response(data)
}

pub fn ecall_init(args: phactory_api::ecall_args::InitArgs) -> Result<()> {
    static INITIALIZED: AtomicU32 = AtomicU32::new(0);
    if INITIALIZED.fetch_add(1, Ordering::SeqCst) != 0 {
        anyhow::bail!("Enclave already initialized.");
    }

    if args.enable_checkpoint {
        match Phactory::restore_from_checkpoint(
            &GraminePlatform,
            &args.sealing_path,
            &args.storage_path,
            args.remove_corrupted_checkpoint,
            args.cores as _,
        ) {
            Ok(Some(mut factory)) => {
                info!("Loaded checkpoint");
                factory.set_args(args.clone());
                *APPLICATION.lock_phactory() = factory;
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

    APPLICATION.lock_phactory().init(args);

    info!("Enclave init OK");
    Ok(())
}

pub fn ecall_bench_run(index: u32) {
    if !benchmark::paused() {
        info!("[{}] Benchmark thread started", index);
        benchmark::run();
    }
}

pub async fn ecall_prpc_request(path: String, data: &[u8]) -> (u16, Vec<u8>) {
    let (code, data) = APPLICATION.dispatch_request(path, data).await;
    info!("pRPC status code: {}, data len: {}", code, data.len());
    (code, data)
}
