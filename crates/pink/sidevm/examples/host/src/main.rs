use pink_sidevm_host_runtime::service::service;
use pink_sidevm_host_runtime::instrument;

use clap::{Parser, AppSettings};

#[derive(Parser)]
#[clap(about = "Cache server for relaychain headers", version, author)]
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
    let (_sender, handle) = spawner.start(
        &wasm_bytes,
        100,
        Default::default(),
        args.gas,
        args.gas_per_breath,
    ).unwrap();
    handle.await?;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    println!("done");
    Ok(())
}
