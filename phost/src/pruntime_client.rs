use serde::Serialize;
use hyper::Client as HttpClient;
use hyper::{Body, Method, Request};
use bytes::buf::BufExt as _;

use crate::error::Error;
use crate::types::{
    RuntimeReq, Resp, SignedResp
};

pub struct PRuntimeClient {
    base_url: String
}

impl PRuntimeClient {
    pub fn new(base_url: &str) -> Self {
        PRuntimeClient {
            base_url: base_url.to_string()
        }
    }

    async fn req<T>(&self, command: &str, param: &T) -> Result<SignedResp, Error>  where T: Serialize {
        let client = HttpClient::new();
        let endpoint = format!("{}/{}", self.base_url, command);

        let body_json = serde_json::to_string(param)?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(endpoint)
            .header("content-type", "application/json")
            .body(Body::from(body_json))?;

        let res = client.request(req).await?;

        println!("Response: {}", res.status());

        let body = hyper::body::aggregate(res.into_body()).await?;
        let signed_resp: SignedResp = serde_json::from_reader(body.reader())?;

        // TODO: validate the response from pRuntime

        Ok(signed_resp)
    }

    pub async fn req_decode<Req>(&self, command: &str, request: Req) -> Result<Req::Resp, Error>
        where Req: Serialize + Resp {
        let payload = RuntimeReq::new(request);
        let resp = self.req(command, &payload).await?;
        let result: Req::Resp = serde_json::from_str(&resp.payload)?;
        Ok(result)
    }

}
