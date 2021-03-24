use hyper::Client as HttpClient;
use hyper::{Body, Method, Request, Response};
use hyper::error::Result;

use crate::types::{
    NotifyReq
};

pub struct NotifyClient {
    base_url: String
}

impl NotifyClient {
    pub fn new(base_url: &str) -> Self {
        NotifyClient {
            base_url: base_url.to_string()
        }
    }

    pub async fn notify(&self, param: &NotifyReq) -> Result<Response<Body>> {
        if self.base_url.is_empty() {
            return Ok(Response::default());
        }

        let client = HttpClient::new();

        let body_json = serde_json::to_string(param).unwrap();

        let req = Request::builder()
            .method(Method::POST)
            .uri(&self.base_url)
            .header("content-type", "application/json")
            .body(Body::from(body_json)).unwrap();

        let res = client.request(req).await;

        res
    }
}
