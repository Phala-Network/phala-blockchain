use std::time::Duration;

use log::info;

use pink_sidevm as sidevm;
use sidevm::{logger::Logger, net, ocall};

#[sidevm::main]
async fn main() {
    Logger::with_max_level(log::Level::Trace).init();
    ocall::enable_ocall_trace(true).unwrap();

    let address = "127.0.0.1:9999";

    info!("Listening on {}", address);
    let listener = net::TcpListener::listen(address).await.unwrap();
    loop {
        info!("Waiting next connection...");
        let _stream = listener.accept().await.unwrap();
    }
}
