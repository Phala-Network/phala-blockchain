use log::info;
use pink_sidevm as sidevm;

use futures::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[sidevm::main]
async fn main() {
    sidevm::logger::Logger::with_max_level(log::LevelFilter::Trace).init();
    sidevm::ocall::enable_ocall_trace(true).unwrap();

    let address = "127.0.0.1:18080";

    info!("Listening on {}", address);

    let listener = sidevm::net::TcpListener::bind(address).await.unwrap();

    loop {
        info!("Waiting for incomming connection or message...");
        tokio::select! {
            message = sidevm::channel::input_messages().next() => {
                if let Some(message) = message {
                    let text_message = String::from_utf8_lossy(&message);
                    info!("Received message: {}", text_message);
                    let number = sidevm::ocall::local_cache_get(b"block_number");
                    info!("Current block number: {:?}", number);
                    if text_message == "stop" {
                        info!("Received 'stop', exiting...");
                        break;
                    }
                } else {
                    info!("Input message channel closed");
                    break;
                }
            }
            query = sidevm::channel::incoming_queries().next() => {
                if let Some(query) = query {
                    info!("Received query from: {:?}", query.origin);
                    let _ = query.reply_tx.send(b"Hello, world!");
                } else {
                    info!("Query channel closed");
                    break;
                }
            }
            stream = listener.accept() => {
                let mut stream = BufReader::new(stream.unwrap().0);

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
    }
}
