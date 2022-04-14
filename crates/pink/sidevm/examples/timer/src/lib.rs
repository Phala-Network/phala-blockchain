use log::info;
use std::time::Duration;

use pink_sidevm as sidevm;
use sidevm::{logger::Logger, ocall};

#[sidevm::main]
async fn main() {
    Logger::with_max_level(log::Level::Trace).init();
    ocall::enable_ocall_trace(true).unwrap();

    info!("starting...");

    let _ = sidevm::env::spawn(async {
        for _ in 0..10 {
            info!("Task 1 sleeping...");
            sidevm::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        info!("Task 1 done");
    });

    loop {
        info!("Timer 0 sleeping...");
        sidevm::time::sleep(Duration::from_millis(500)).await;
    }
}
