use anyhow::Result;
use serde::Serialize;
use hyper::Client as HttpClient;
use hyper::{Body, Method, Request};
use log::info;
use bytes::buf::BufExt as _;

use crate::types::{
    RuntimeReq, Resp, SignedResp, ReqData, QueryReq, QueryRespData, Query, Payload
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

    /// Sends a Query to a confidential contract in pRuntime
    ///
    /// It's possible to query with e2e encryption. However currently only Plain message is
    /// supported.
    pub async fn query(&self, contract_id: u32, request: ReqData)
    -> Result<QueryRespData> {
        // Encode the query within Payload::Plain
        let query = Query {
            contract_id,
            nonce: 0,
            request,
        };
        let query_value = serde_json::to_value(&query)?;
        let payload = Payload::Plain(query_value.to_string());
        let query_payload = serde_json::to_string(&payload)?;
        info!("Query contract: {}, payload: {}", contract_id, query_payload);
        // Send the query
        let resp = self.req_decode("query", QueryReq { query_payload }).await?;
        // Only accept Payload::Plain response
        let Payload::Plain(plain_json) = resp;
        info!("Query response: {:}", &plain_json);
        let resp_data: QueryRespData = serde_json::from_str(plain_json.as_str())
            .map_err(|_| crate::error::Error::FailedToDecode)?;
        return Ok(resp_data)
    }

}
