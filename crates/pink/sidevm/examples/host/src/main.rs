use std::env::args;

use pink_sidevm_host_runtime::service::service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (run, spawner) = service();
    std::thread::spawn(move || {
        run.blocking_run(|evt| {
            println!("event: {:?}", evt);
            std::process::exit(0);
        });
    });

    let wasm_bytes = std::fs::read(args().nth(1).unwrap()).unwrap();
    println!("VM running...");
    let (_sender, handle) = spawner.start(&wasm_bytes, 100, Default::default()).unwrap();
    handle.await?;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    println!("done");
    Ok(())
}
