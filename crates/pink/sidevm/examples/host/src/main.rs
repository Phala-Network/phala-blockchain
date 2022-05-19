use pink_sidevm_host_runtime::service::service;
use pink_sidevm_host_runtime::{instrument, CacheOps};

use clap::{AppSettings, Parser};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Parser)]
#[clap(about = "Demo sidevm host app", version, author)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Args {
    /// The gas limit for the program to consume.
    #[clap(long, default_value_t = u128::MAX)]
    gas: u128,
    /// The gas limit for each poll.
    #[clap(long, default_value_t = 1000_000_000_000_u128)]
    gas_per_breath: u128,
    /// The WASM program to run
    program: String,
}

fn simple_cache() -> CacheOps {
    static CACHE: Lazy<RwLock<HashMap<Vec<u8>, Vec<u8>>>> = Lazy::new(Default::default);
    CacheOps {
        get: |_, key| {
            let cache = CACHE.read().unwrap();
            let value = cache.get(key).cloned();
            Ok(value)
        },
        set: |_, key, value| {
            let mut cache = CACHE.write().unwrap();
            cache.insert(key.to_vec(), value.to_vec());
            Ok(())
        },
        set_expiration: |_, _, _| Ok(()),
        remove: |_, key| {
            let mut cache = CACHE.write().unwrap();
            let value = cache.remove(key);
            Ok(value)
        },
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::init();

    let (run, spawner) = service();
    std::thread::spawn(move || {
        run.blocking_run(|evt| {
            println!("event: {:?}", evt);
            std::process::exit(0);
        });
    });

    println!("Reading {}...", args.program);
    let wasm_bytes = std::fs::read(&args.program)?;
    println!("Instrumenting...");
    let wasm_bytes = instrument::instrument(&wasm_bytes)?;
    println!("VM running...");
    let (_sender, handle) = spawner
        .start(
            &wasm_bytes,
            1024,
            Default::default(),
            args.gas,
            args.gas_per_breath,
            simple_cache(),
        )
        .unwrap();
    handle.await?;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    println!("done");
    Ok(())
}
