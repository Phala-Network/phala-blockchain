use log::info;
use pink_sidevm as sidevm;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[sidevm::main]
async fn main() {
    sidevm::logger::Logger::with_max_level(log::Level::Trace).init();
    sidevm::ocall::enable_ocall_trace(true).unwrap();

    let address = "example.com:80";

    info!("Connecting to {}", address);

    let stream = sidevm::net::TcpStream::connect(address).await.unwrap();
    let mut stream = BufReader::new(stream);
    info!("Sending request");
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    info!("Receiving response");
    loop {
        let mut line = String::new();
        let nbytes = stream.read_line(&mut line).await.unwrap();
        if nbytes == 0 {
            break;
        }
        let line = line.trim_end();
        info!("|{}", &line);
    }
}
