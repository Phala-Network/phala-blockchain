use anyhow::Result;
use log::info;

use enclave_api::prpc::{
    client::{Error as ClientError, RequestClient},
    phactory_api_client::PhactoryApiClient,
    server::ProtoError as ServerError,
    Message,
};

pub type PRuntimeClient = PhactoryApiClient<RpcRequest>;

pub fn new_pruntime_client(base_url: String) -> PhactoryApiClient<RpcRequest> {
    PhactoryApiClient::new(RpcRequest::new(base_url.to_string()))
}

pub struct RpcRequest {
    base_url: String,
    client: reqwest::Client,
}

impl RpcRequest {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl RequestClient for RpcRequest {
    async fn request(&self, path: &str, body: Vec<u8>) -> Result<Vec<u8>, ClientError> {
        fn from_display(err: impl std::fmt::Display) -> ClientError {
            ClientError::RpcError(err.to_string())
        }

        let url = format!("{}/prpc/{}", self.base_url, path);
        let res = self
            .client
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(from_display)?;

        info!("Response: {}", res.status());
        let status = res.status();
        let body = res.bytes().await.map_err(from_display)?;
        fn payload(body: &[u8]) -> &[u8] {
            if body.len() >= 66 {
                &body[66..]
            } else {
                &[]
            }
        }
        if status.is_success() {
            Ok(payload(body.as_ref()).to_vec())
        } else {
            let err: ServerError = Message::decode(payload(body.as_ref()))?;
            Err(ClientError::ServerError(err))
        }
    }
}
