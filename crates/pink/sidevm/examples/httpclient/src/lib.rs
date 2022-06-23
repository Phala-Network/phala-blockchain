use std::io::Read;

use sidevm::net::HttpConnector;
use hyper::body::Buf;
use log::info;

#[sidevm::main]
async fn main() {
    sidevm::logger::Logger::with_max_level(log::Level::Trace).init();
    sidevm::ocall::enable_ocall_trace(true).unwrap();

    let url = "https://example.com/";
    info!("Connecting to {}", url);
    let connector = HttpConnector::new();

    let client = hyper::Client::builder()
        .executor(sidevm::exec::HyperExecutor)
        .build::<_, String>(connector);

    let response = client
        .get(url.parse().expect("Bad url"))
        .await
        .expect("Failed to send request");
    info!("response status: {}", response.status());

    let mut buf = vec![];
    hyper::body::aggregate(response)
        .await
        .expect("Failed to read response body")
        .reader()
        .read_to_end(&mut buf)
        .expect("Failed to read body");
    info!("body: {}", String::from_utf8_lossy(&buf));
}
