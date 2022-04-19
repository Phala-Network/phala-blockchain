use crate::pal_gramine::GraminePlatform;

use anyhow::Result;
use core::sync::atomic::{AtomicU32, Ordering};
use log::info;
use phactory::{benchmark, Phactory};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref APPLICATION: Mutex<Phactory<GraminePlatform>> = Mutex::new(Phactory::new(GraminePlatform));
}

pub fn ecall_handle(action: u8, input: &[u8]) -> Result<Vec<u8>> {
    let mut factory = APPLICATION.lock().unwrap();
    Ok(factory.handle_scale_api(action, input))
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
            args.skip_corrupted_checkpoint,
        ) {
            Ok(Some(mut factory)) => {
                info!("Loaded checkpoint");
                factory.set_args(args.clone());
                *APPLICATION.lock().unwrap() = factory;
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
        info!("No checkpoint file specified.");
    }

    APPLICATION.lock().unwrap().init(args);

    info!("Enclave init OK");
    Ok(())
}

pub fn ecall_bench_run(index: u32) {
    if !benchmark::paused() {
        info!("[{}] Benchmark thread started", index);
        benchmark::run();
    }
}

pub fn ecall_prpc_request(path: &[u8], data: &[u8]) -> (u16, Vec<u8>) {
    let (code, data) = phactory::dispatch_prpc_request(path, data, usize::MAX, &APPLICATION);
    info!("pRPC status code: {}, data len: {}", code, data.len());
    (code, data)
}

pub fn ecall_dump_state<W: std::io::Write>(writer: W) -> Result<()> {
    APPLICATION
        .lock()
        .unwrap()
        .take_checkpoint_to_writer(writer)?;
    Ok(())
}

pub fn ecall_load_state<R: std::io::Read>(reader: R) -> Result<()> {
    let sealing_path = {
        let app = APPLICATION.lock().unwrap();
        app.args.sealing_path.clone()
    };
    let phactory =
        Phactory::restore_from_checkpoint_reader(&GraminePlatform, &sealing_path, reader)?;
    *APPLICATION.lock().unwrap() = phactory;
    Ok(())
}
