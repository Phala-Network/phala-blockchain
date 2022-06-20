use std::convert::Infallible;
use log::{error, info};

use hyper::{Body, Request, Response};
use sidevm::env::tls::TlsServerConfig;

const CERT: &str = "-----BEGIN CERTIFICATE-----
MIIBZzCCAQ2gAwIBAgIIbELHFTzkfHAwCgYIKoZIzj0EAwIwITEfMB0GA1UEAwwW
cmNnZW4gc2VsZiBzaWduZWQgY2VydDAgFw03NTAxMDEwMDAwMDBaGA80MDk2MDEw
MTAwMDAwMFowITEfMB0GA1UEAwwWcmNnZW4gc2VsZiBzaWduZWQgY2VydDBZMBMG
ByqGSM49AgEGCCqGSM49AwEHA0IABOoRzdEagFDZf/im79Z5JUyeXP96Yww6nH8X
ROvXOESnE0yFtlVjdj0NTNXT2m+PWzuxsjvPVBWR/tpDldjTW8CjLTArMCkGA1Ud
EQQiMCCCE2hlbGxvLndvcmxkLmV4YW1wbGWCCWxvY2FsaG9zdDAKBggqhkjOPQQD
AgNIADBFAiEAsuZKsdksPsrnJFdV9JTZ1P782IlqjqNL9aAURvrF3UkCIDDpTvE5
EyZ5zRflnB+ZwomjXNhTAnasRjQTDqXFrQbP
-----END CERTIFICATE-----";

const KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgH1VlVX/3DI37UR5g
tGzUOSAaOmjQbZMJQ2Z9eBnzh3+hRANCAATqEc3RGoBQ2X/4pu/WeSVMnlz/emMM
Opx/F0Tr1zhEpxNMhbZVY3Y9DUzV09pvj1s7sbI7z1QVkf7aQ5XY01vA
-----END PRIVATE KEY-----";

async fn handle(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    info!("Incoming request: {}", request.uri().path());
    Ok(Response::new("Hello, World!\n".into()))
}

#[sidevm::main]
async fn main() {
    sidevm::logger::Logger::with_max_level(log::Level::Trace).init();
    sidevm::ocall::enable_ocall_trace(true).unwrap();

    let make_svc = hyper::service::make_service_fn(|_conn| async {
        Ok::<_, Infallible>(hyper::service::service_fn(handle))
    });

    let address = "127.0.0.1:1999";
    info!("Listening on {}", address);

    let listener = sidevm::net::TcpListener::bind_tls(address, TlsServerConfig::V0 {
        cert: CERT.to_string(),
        key: KEY.to_string(),
    }).await.unwrap();

    let server = hyper::Server::builder(listener)
        .executor(sidevm::exec::HyperExecutor)
        .serve(make_svc);
    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}
