use anyhow::{Context, Result};
use clap::Parser;
use pink_types::js::JsValue;
use scale::Decode;
use sidevm_host_runtime::{
    CacheOps, DynCacheOps, OcallError, OutgoingRequest, WasmEngine, WasmInstanceConfig,
};
use tracing::error;

#[derive(Parser, Debug)]
#[clap(about = "sidevm runner", version, author)]
pub struct Args {
    /// The gas limit for each poll.
    #[arg(long, default_value_t = 50_000_000_000_u64)]
    vital_capacity: u64,
    /// Max memory pages
    #[arg(long, default_value_t = 256)]
    max_memory_pages: u32,
    /// The WASM program to run
    program: String,
    /// The rest of the arguments are passed to the WASM program
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    args: Vec<String>,
}

pub async fn run(mut args: Args) -> Result<JsValue> {
    let code = tokio::fs::read(&args.program).await?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(1);
    let config = WasmInstanceConfig {
        max_memory_pages: args.max_memory_pages,
        gas_per_breath: args.vital_capacity,
        cache_ops: no_cache(),
        scheduler: None,
        weight: 0,
        id: Default::default(),
        event_tx,
        log_handler: None,
    };
    let engine = WasmEngine::new();
    let module = engine.compile(&code)?;
    args.args.insert(0, args.program);
    let vm_args = args
        .args
        .into_iter()
        .map(|s| -> Result<String> {
            if s.starts_with('@') {
                let path = &s[1..];
                let content = std::fs::read_to_string(path).context("Failed to read file")?;
                Ok(content)
            } else {
                Ok(s)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (mut wasm_run, _env) = module
        .run(vm_args, config)
        .context("Failed to start sidevm instance")?;
    let mut output = None;
    tokio::select! {
        rv = &mut wasm_run => {
            if let Err(err) = rv {
                error!(target: "sidevm", ?err, "Js runtime exited with error.");
            }
        }
        _ = async {
            while let Some((_vmid, event)) = event_rx.recv().await {
                if let OutgoingRequest::Output(output_bytes) = event {
                    output = Some(output_bytes);
                    break;
                }
            }
        } => {}
    }
    if output.is_none() {
        while let Ok((_vmid, event)) = event_rx.try_recv() {
            if let OutgoingRequest::Output(output_bytes) = event {
                output = Some(output_bytes);
                break;
            }
        }
    }
    match output {
        Some(output) => Ok(JsValue::decode(&mut &output[..])?),
        None => Err(anyhow::anyhow!("No output")),
    }
}

fn no_cache() -> DynCacheOps {
    struct Ops;
    type OpResult<T> = Result<T, OcallError>;
    impl CacheOps for Ops {
        fn get(&self, _contract: &[u8], _key: &[u8]) -> OpResult<Option<Vec<u8>>> {
            Ok(None)
        }
        fn set(&self, _contract: &[u8], _key: &[u8], _value: &[u8]) -> OpResult<()> {
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
        fn remove(&self, _contract: &[u8], _key: &[u8]) -> OpResult<Option<Vec<u8>>> {
            Ok(None)
        }
    }
    &Ops
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let output = run(args).await?;
    println!("Output: {:?}", output);
    Ok(())
}
