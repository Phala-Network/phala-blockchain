use core::convert::Infallible;

use log::{error, info};
use pink_sidevm as sidevm;

use hyper::{Body, Request, Response};

async fn handle(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    info!("Incoming request: {}", request.uri().path());
    Ok(Response::new("Hello, World!".into()))
}

#[sidevm::main]
async fn main() {
    sidevm::logger::Logger::with_max_level(log::Level::Trace).init();
    sidevm::ocall::enable_ocall_trace(true).unwrap();

    let address = "127.0.0.1:9999";

    info!("Listening on {}", address);

    let listener = sidevm::net::TcpListener::listen(address).await.unwrap();

    let make_svc = hyper::service::make_service_fn(|_conn| async {
        Ok::<_, Infallible>(hyper::service::service_fn(handle))
    });

    let server = hyper::Server::builder(listener).serve(make_svc);
    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}
