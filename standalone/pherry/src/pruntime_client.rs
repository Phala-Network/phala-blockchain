use anyhow::Context;
use anyhow::Result;
use codec::Encode;
use log::info;
use serde::Serialize;
use sp_runtime::DeserializeOwned;

use crate::types::{Resp, RuntimeReq, SignedResp};
use enclave_api::prpc::{
    client::{Error as ClientError, RequestClient},
    phactory_api_client::PhactoryApiClient,
    server::ProtoError as ServerError,
    Message,
};

pub struct PRuntimeClient {
    base_url: String,
    pub prpc: PhactoryApiClient<RpcRequest>,
}

impl PRuntimeClient {
    pub fn new(base_url: &str) -> Self {
        PRuntimeClient {
            base_url: base_url.to_string(),
            prpc: PhactoryApiClient::new(RpcRequest::new(base_url.to_string())),
        }
    }

    async fn req<T>(&self, command: &str, param: &T) -> Result<SignedResp>
    where
        T: Serialize,
    {
        let client = reqwest::Client::new();
        let endpoint = format!("{}/{}", self.base_url, command);

        let body_json = serde_json::to_string(param)?;
        let res = client
            .post(endpoint)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .await?;

        info!("Response: {}", res.status());

        let body = res.bytes().await?;
        let opaque_value: serde_json::Value = serde_json::from_slice(body.as_ref())?;
        let signed_resp: SignedResp = serde_json::from_value(opaque_value.clone())
            .with_context(|| format!("Cannot convert json to SignedResp ({})", opaque_value))?;

        // TODO: validate the response from pRuntime

        Ok(signed_resp)
    }

    pub async fn req_decode<Req>(&self, command: &str, request: Req) -> Result<Req::Resp>
    where
        Req: Serialize + Resp,
    {
        let payload = RuntimeReq::new(request);
        let resp = self.req(command, &payload).await?;
        let result: Req::Resp = serde_json::from_str(&resp.payload)?;
        Ok(result)
    }

    async fn bin_req<T: Encode>(&self, command: &str, param: &T) -> Result<SignedResp> {
        let client = reqwest::Client::new();
        let endpoint = format!("{}/{}", self.base_url, command);
        let body = param.encode();

        let res = client.post(endpoint).body(body).send().await?;

        info!("Response: {}", res.status());

        let body = res.bytes().await?;
        let signed_resp: SignedResp = serde_json::from_slice(body.as_ref())?;

        Ok(signed_resp)
    }

    pub async fn bin_req_decode<Req: Encode, Resp: DeserializeOwned>(
        &self,
        command: &str,
        request: Req,
    ) -> Result<Resp> {
        let resp = self.bin_req(command, &request).await?;
        let result: Resp = serde_json::from_str(&resp.payload)?;
        Ok(result)
    }
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
        fn display_err(err: impl std::fmt::Display) -> ClientError {
            ClientError::RpcError(err.to_string())
        }

        let url = format!("{}/prpc/{}", self.base_url, path);
        let res = self
            .client
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(display_err)?;

        info!("Response: {}", res.status());
        let status = res.status();
        let body = res.bytes().await.map_err(display_err)?;
        if status.is_success() {
            Ok(body.to_vec())
        } else {
            let err: ServerError = Message::decode(body.as_ref())?;
            Err(ClientError::ServerError(err))
        }
    }
}
