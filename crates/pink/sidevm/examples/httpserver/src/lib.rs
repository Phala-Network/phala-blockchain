use log::info;
use pink_sidevm as sidevm;

use futures::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[sidevm::main]
async fn main() {
    sidevm::logger::Logger::with_max_level(log::Level::Trace).init();
    sidevm::ocall::enable_ocall_trace(true).unwrap();

    let address = "127.0.0.1:9999";

    info!("Listening on {}", address);

    let listener = sidevm::net::TcpListener::listen(address).await.unwrap();

    loop {
        info!("Waiting for imcomming connection...");
        let mut stream = BufReader::new(listener.accept().await.unwrap());

        info!("New imcomming connection");
        // Spawn a new task to handle the new connection concurrently
        sidevm::spawn(async move {
            info!("=====================");
            loop {
                let mut line = String::new();
                let _nbytes = stream.read_line(&mut line).await.unwrap();
                let line = line.trim_end();
                info!("> {}", &line);
                if line.is_empty() {
                    info!("---------------------");
                    stream
                        .write_all(b"HTTP/1.0 200 OK\r\n\r\nHello, world!\n")
                        .await
                        .unwrap();
                    break;
                }
            }
        });
    }
}
