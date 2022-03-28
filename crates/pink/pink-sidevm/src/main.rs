use anyhow::Result;

use pink_sidevm::WasmRun;

#[tokio::main]
async fn main() -> Result<()> {
    let wasm_bytes = include_bytes!("/tmp/sidevm_timer.wasm");
    let run = WasmRun::run(wasm_bytes, 100)?;
    println!("waiting...");
    let rv = run.await?;
    println!("result: {}", rv);
    Ok(())
}
