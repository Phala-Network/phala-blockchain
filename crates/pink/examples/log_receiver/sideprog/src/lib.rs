use std::cell::RefCell;
use std::convert::Infallible;
use std::fmt::Write;
use std::rc::Rc;

use chrono::{TimeZone, Utc};
use hyper::{Body, Request, Response};
use log::{error, info};

use pink_sidevm as sidevm;
use sidevm::env::messages::SystemMessage;

#[derive(Clone)]
struct AppState {
    log_buffer: Rc<RefCell<log_buffer::LogBuffer<Vec<u8>>>>,
}

impl AppState {
    fn new(buffer_size: usize) -> Self {
        Self {
            log_buffer: Rc::new(RefCell::new(log_buffer::LogBuffer::new(vec![
                0;
                buffer_size
            ]))),
        }
    }
}

async fn http_serve(state: AppState) {
    let address = "0.0.0.0:18080";

    info!("Listening http on {}", address);
    let listener = sidevm::net::TcpListener::bind(address).await.unwrap();

    let make_svc = hyper::service::make_service_fn(move |_conn| {
        let state = state.clone();
        async {
            Ok::<_, Infallible>(hyper::service::service_fn(move |request: Request<Body>| {
                let state = state.clone();
                async move {
                    match request.uri().path() {
                        "/log" => {
                            let body = state.log_buffer.borrow_mut().extract().to_string().into();
                            Ok::<_, Infallible>(Response::<Body>::new(body))
                        }
                        _ => {
                            let mut response = Response::new("Not Found".into());
                            *response.status_mut() = hyper::StatusCode::NOT_FOUND;
                            Ok::<_, Infallible>(response)
                        }
                    }
                }
            }))
        }
    });

    let server = hyper::Server::builder(listener)
        .executor(sidevm::exec::HyperExecutor)
        .serve(make_svc);

    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}

async fn query_serve(app: AppState) {
    loop {
        let query = sidevm::channel::incoming_queries().next().await;
        if let Some(query) = query {
            let _ = query
                .reply_tx
                .send(app.log_buffer.borrow_mut().extract().as_bytes());
        } else {
            info!("Query channel closed");
            break;
        }
    }
}

#[sidevm::main]
async fn main() {
    sidevm::logger::Logger::with_max_level(log::LevelFilter::Trace).init();
    sidevm::ocall::enable_ocall_trace(true).unwrap();

    let app = AppState::new(1024 * 256);

    sidevm::spawn(http_serve(app.clone()));
    sidevm::spawn(query_serve(app.clone()));

    loop {
        let message = sidevm::channel::incoming_system_messages().next().await;
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
        match message {
            SystemMessage::PinkLog {
                block_number,
                contract,
                in_query,
                timestamp_ms,
                level,
                message,
            } => {
                let mode = if in_query { "query" } else { "command" };
                let ts = Utc.timestamp_millis(timestamp_ms as _);
                let ts = ts.format("%Y-%m-%dT%H:%M:%S.%.3f");
                let contract_id = hex_fmt::HexFmt(contract);
                writeln!(
                    app.log_buffer.borrow_mut(),
                    "[{ts}][{block_number}][cid={contract_id}][{mode}][lvl={level}] {message}",
                )
                .expect("write log to buffer failed");
            }
            _ => {}
        }
    }
}
