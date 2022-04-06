use anyhow::Result;
use pink_sidevm_host_runtime::WasmRun;
use std::time::Duration;

#[tokio::test]
async fn test_timer() -> Result<()> {
    let wasm_bytes = include_bytes!("res/sidevm_timer.wasm");
    let (run, env) = WasmRun::run(wasm_bytes, 100)?;
    std::thread::spawn(move || loop {
        loop {
            std::thread::sleep(Duration::from_secs(2));
            println!("push message...");
            env.blocking_push_message(b"foo".to_vec())
                .expect("push message failed");
        }
    });
    println!("waiting...");
    let rv = run.await?;
    println!("result: {}", rv);
    Ok(())
}
