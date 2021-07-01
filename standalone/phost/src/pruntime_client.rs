use anyhow::Result;
use codec::Encode;
use serde::Serialize;
use hyper::Client as HttpClient;
use hyper::{Body, Method, Request};
use log::info;
use bytes::buf::BufExt as _;
use sp_runtime::DeserializeOwned;

use crate::types::{RuntimeReq, Resp, SignedResp};

pub struct PRuntimeClient {
    base_url: String
}

impl PRuntimeClient {
    pub fn new(base_url: &str) -> Self {
        PRuntimeClient {
            base_url: base_url.to_string()
        }
    }

    async fn req<T>(&self, command: &str, param: &T) -> Result<SignedResp>  where T: Serialize {
        let client = HttpClient::new();
        let endpoint = format!("{}/{}", self.base_url, command);

        let body_json = serde_json::to_string(param)?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(endpoint)
            .header("content-type", "application/json")
            .body(Body::from(body_json))?;

        let res = client.request(req).await?;

        info!("Response: {}", res.status());

        let body = hyper::body::aggregate(res.into_body()).await?;
        let signed_resp: SignedResp = serde_json::from_reader(body.reader())?;

        // TODO: validate the response from pRuntime

        Ok(signed_resp)
    }

    pub async fn req_decode<Req>(&self, command: &str, request: Req) -> Result<Req::Resp>
        where Req: Serialize + Resp {
        let payload = RuntimeReq::new(request);
        let resp = self.req(command, &payload).await?;
        let result: Req::Resp = serde_json::from_str(&resp.payload)?;
        Ok(result)
    }

    async fn bin_req<T: Encode>(&self, command: &str, param: &T) -> Result<SignedResp> {
        let client = HttpClient::new();
        let endpoint = format!("{}/{}", self.base_url, command);
        let body = param.encode();

        let req = Request::builder()
            .method(Method::POST)
            .uri(endpoint)
            .body(Body::from(body))?;

        let res = client.request(req).await?;

        info!("Response: {}", res.status());

        let body = hyper::body::aggregate(res.into_body()).await?;
        let signed_resp: SignedResp = serde_json::from_reader(body.reader())?;

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
