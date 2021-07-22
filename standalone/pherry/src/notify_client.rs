use anyhow::Result;

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

    pub async fn notify(&self, param: &NotifyReq) -> Result<()> {
        if self.base_url.is_empty() {
            return Ok(())
        }

        let client = reqwest::Client::new();

        let body_json = serde_json::to_string(param).unwrap();

        let res = client.post(&self.base_url)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::Error::msg(res.status()))
        }
    }
}
