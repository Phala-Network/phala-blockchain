use pink_sidevm_host_runtime::service::service;
use pink_sidevm_host_runtime::{instrument, CacheOps, DynCacheOps, OcallError};

use clap::{AppSettings, Parser};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

mod web_api;

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
    /// Don't instrument the program.
    #[clap(long)]
    no_instrument: bool,
    #[clap(long, default_value_t = 1)]
    workers: usize,
    /// The WASM program to run
    program: String,
}

fn simple_cache() -> DynCacheOps {
    static CACHE: Lazy<RwLock<HashMap<Vec<u8>, Vec<u8>>>> = Lazy::new(Default::default);
    struct Ops;
    type OpResult<T> = Result<T, OcallError>;
    impl CacheOps for Ops {
        fn get(&self, _contract: &[u8], key: &[u8]) -> OpResult<Option<Vec<u8>>> {
            let cache = CACHE.read().unwrap();
            let value = cache.get(key).cloned();
            Ok(value)
        }

        fn set(&self, _contract: &[u8], key: &[u8], value: &[u8]) -> OpResult<()> {
            let mut cache = CACHE.write().unwrap();
            cache.insert(key.to_vec(), value.to_vec());
            Ok(())
        }

        fn set_expiration(
            &self,
            _contract: &[u8],
            _key: &[u8],
            _expire_after_secs: u64,
        ) -> OpResult<()> {
            Ok(())
        }

        fn remove(&self, _contract: &[u8], key: &[u8]) -> OpResult<Option<Vec<u8>>> {
            let mut cache = CACHE.write().unwrap();
            let value = cache.remove(key);
            Ok(value)
        }
    }
    &Ops
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::init();

    let (run, spawner) = service(args.workers);
    std::thread::spawn(move || {
        run.blocking_run(|evt| {
            println!("event: {:?}", evt);
            std::process::exit(0);
        });
    });

    println!("Reading {}...", args.program);
    let mut wasm_bytes = std::fs::read(&args.program)?;
    if !args.no_instrument {
        println!("Instrumenting...");
        wasm_bytes = instrument::instrument(&wasm_bytes)?;
    }
    println!("VM running...");
    let (sender, handle) = spawner
        .start(
            &wasm_bytes,
            1024,
            Default::default(),
            args.gas,
            args.gas_per_breath,
            simple_cache(),
        )
        .unwrap();
    tokio::spawn(handle);
    web_api::serve(sender).await.unwrap();
    Ok(())
}
