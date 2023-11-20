use anyhow::Result;
use log::debug;
use phactory_api::prpc::client::{Error as ClientError, RequestClient};
use phactory_api::prpc::phactory_api_client::PhactoryApiClient;
use phactory_api::prpc::server::ProtoError as ServerError;
use phactory_api::prpc::Message;
use reqwest::Client;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub type PRuntimeClient = PhactoryApiClient<RpcRequest>;

pub struct RpcRequest {
    pub base_url: String,
    client: Client,
    semaphore: Arc<Semaphore>,
}

#[async_trait::async_trait]
pub trait PRuntimeClientWithSemaphore {
    async fn with_lock<'a, R>(&'a self, f: (impl Future<Output = R> + Send + 'a)) -> Result<R>;
}

#[async_trait::async_trait]
impl PRuntimeClientWithSemaphore for PRuntimeClient {
    // Lock the pruntime requester only when needed;
    // The additional with_lock method is to provide locking ability without breaking compatibility from functions in pherry crate.
    async fn with_lock<'a, R>(&'a self, f: (impl Future<Output = R> + Send + 'a)) -> Result<R> {
        let s = self.client.semaphore.clone();
        let s = s.acquire().await?;
        let ret = f.await;
        drop(s);
        Ok(ret)
    }
}

impl RpcRequest {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .tcp_keepalive(Some(core::time::Duration::from_secs(10)))
            .build()
            .expect("Should build reqwest client");
        Self {
            base_url,
            client,
            semaphore: Arc::new(Semaphore::new(1)),
        }
    }
}

pub fn create_client(base_url: String) -> PRuntimeClient {
    PhactoryApiClient::new(RpcRequest::new(base_url))
}

fn from_display(err: impl core::fmt::Display) -> ClientError {
    ClientError::RpcError(err.to_string())
}

#[async_trait::async_trait]
impl RequestClient for RpcRequest {
    async fn request(&self, path: &str, body: Vec<u8>) -> Result<Vec<u8>, ClientError> {
        let url = format!("{}/prpc/{path}", self.base_url);
        let res = self
            .client
            .post(url.clone())
            .body(body)
            .send()
            .await
            .map_err(from_display)?;

        debug!("{url}: {}", res.status());
        let status = res.status();
        let body = res.bytes().await.map_err(from_display)?;
        if status.is_success() {
            Ok(body.as_ref().to_vec())
        } else {
            let err: ServerError = Message::decode(body.as_ref())?;
            Err(ClientError::ServerError(err))
        }
    }
}
