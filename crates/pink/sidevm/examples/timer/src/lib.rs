use log::info;
use std::time::Duration;

use pink_sidevm as sidevm;
use sidevm::{logger::Logger, ocall};

#[sidevm::main]
async fn main() {
    Logger::with_max_level(log::Level::Trace).init();
    ocall::enable_ocall_trace(true).unwrap();

    info!("starting...");

    for _ in 0..10 {
        info!("sleeping...");
        sidevm::time::sleep(Duration::from_millis(100)).await;
    }
    info!("done");
}
