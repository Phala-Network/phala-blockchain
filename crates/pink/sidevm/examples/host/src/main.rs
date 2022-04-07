use std::env::args;

use pink_sidevm_host_runtime::WasmRun;

#[tokio::main]
async fn main() {
    env_logger::init();

    let wasm_bytes = std::fs::read(args().nth(1).unwrap()).unwrap();
    println!("VM running...");
    let (run, _) = WasmRun::run(&wasm_bytes, 100).expect("Failed to run the VM");
    let rv = run.await.expect("VM exited with error");
    println!("VM exited: {}", rv);
}
