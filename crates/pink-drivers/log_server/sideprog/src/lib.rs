use std::cell::RefCell;
use std::rc::Rc;

use log::{error, info};
use scale::Decode;

use sidevm::env::ocall_funcs_guest::local_cache_get;

use buffer::Buffer;
mod buffer;

#[derive(Clone)]
struct AppState {
    log_buffer: Rc<RefCell<Buffer>>,
}

impl AppState {
    fn new(buffer_size: usize) -> Self {
        Self {
            log_buffer: Rc::new(RefCell::new(Buffer::new(buffer_size))),
        }
    }
}

fn log_buffer_size() -> u32 {
    let buf = local_cache_get(b"LOG_BUFFER_SIZE")
        .unwrap_or_default()
        .unwrap_or_default();
    u32::decode(&mut &buf[..]).unwrap_or(1024 * 1024 * 8)
}

async fn query_serve(app: AppState) {
    #[derive(serde::Deserialize)]
    #[serde(tag="action")]
    enum Query {
        GetLog {
            #[serde(default)]
            contract: String,
            #[serde(default)]
            from: u64,
            #[serde(default)]
            count: u64,
        }
    }

    loop {
        let query = sidevm::channel::incoming_queries().next().await;
        if let Some(query) = query {
            // todo: use `let else`
            let Query::GetLog {
                contract,
                from,
                count,
            } = match serde_json::from_slice(&query.payload) {
                Err(_) => {
                    info!("Invalid input");
                    _ = query.reply_tx.send(b"{\"error\": \"Invalid input\"}");
                    continue;
                }
                Ok(query) => query,
            };
            let reply = app.log_buffer
                .borrow_mut()
                .get_records(&contract, from, count);
            let _ = query.reply_tx.send(reply.as_bytes());
        } else {
            info!("Query channel closed");
            break;
        }
    }
}

#[sidevm::main]
async fn main() {
    sidevm::logger::Logger::with_max_level(log::LevelFilter::Info).init();
    info!("Starting log server");

    let app = AppState::new(log_buffer_size() as _);

    sidevm::spawn(query_serve(app.clone()));

    loop {
        let message = sidevm::channel::incoming_system_messages().next().await;
        // todo: use `let else`
        let message = match message {
            None => {
                info!("Input message channel closed");
                break;
            }
            Some(Ok(message)) => message,
            Some(Err(e)) => {
                error!("Decode system message failed: {}", e);
                continue;
            }
        };
        app.log_buffer.borrow_mut().push(message);
    }
}

mod allocator {
    use phala_allocator::StatSizeAllocator;
    use std::alloc::System;

    #[global_allocator]
    pub static ALLOC: StatSizeAllocator<System> = StatSizeAllocator::new(System);

    pub fn mem_usage() -> usize {
        ALLOC.stats().current
    }
}
