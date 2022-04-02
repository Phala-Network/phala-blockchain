use anyhow::Result;

use pink_sidevm_runtime::WasmRun;

#[tokio::test]
async fn test_timer() -> Result<()> {
    let wasm_bytes = include_bytes!("res/sidevm_timer.wasm");
    let run = WasmRun::run(wasm_bytes, 100)?;
    println!("waiting...");
    let rv = run.await?;
    println!("result: {}", rv);
    assert_eq!(rv, 1);
    Ok(())
}
