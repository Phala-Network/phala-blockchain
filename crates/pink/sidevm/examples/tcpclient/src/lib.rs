use log::info;
use pink_sidevm as sidevm;

use futures::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[sidevm::main]
async fn main() {
    sidevm::logger::Logger::with_max_level(log::Level::Trace).init();
    sidevm::ocall::enable_ocall_trace(true).unwrap();

    let host = "example.com";
    // let port = 80;
    let port = 443;

    info!("Connecting to {host}:{port}");

    let stream = sidevm::net::TcpStream::connect(host, port, port == 443).await.unwrap();
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
