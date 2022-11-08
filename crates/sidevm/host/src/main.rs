use sidevm_host_runtime::{CacheOps, DynCacheOps, OcallError};

use clap::Parser;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

mod web_api;

#[derive(Parser)]
#[clap(about = "Demo sidevm host app", version, author)]
pub struct Args {
    /// The gas limit for each poll.
    #[arg(long, default_value_t = 50_000_000_000_u64)]
    gas_per_breath: u64,
    #[arg(long, default_value_t = 1)]
    workers: usize,
    /// The WASM program to run
    program: Option<String>,
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
    env_logger::init();
    if std::env::var("ROCKET_PORT").is_err() {
        std::env::set_var("ROCKET_PORT", "8003");
    }
    web_api::serve(Args::parse()).await.unwrap();
    Ok(())
}
