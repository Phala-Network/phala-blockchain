use log::info;

use pink_sidevm as sidevm;
use sidevm::{logger::Logger, ocall};

#[sidevm::main]
async fn main() {
    Logger::with_max_level(log::LevelFilter::Trace).init();
    ocall::enable_ocall_trace(true).unwrap();
    info!("starting...");
    loop {
        sidevm::time::take_a_rest_if_needed().await;
    }
}
