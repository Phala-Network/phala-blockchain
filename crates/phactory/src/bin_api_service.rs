use phala_types::{wrap_content_to_sign, SignedContentType};

use super::*;

// For bin_api
impl<Platform: pal::Platform + Serialize + DeserializeOwned> Phactory<Platform> {
    pub fn sign_http_response(&self, body: &[u8]) -> Option<String> {
        self.system.as_ref().map(|state| {
            let bytes = wrap_content_to_sign(body, SignedContentType::RpcResponse);
            let sig = state.identity_key.sign(&bytes).0;
            hex::encode(sig)
        })
    }

    fn get_info_json(&self) -> Result<Value, Value> {
        Ok(json!(self.get_info()))
    }

    fn bin_sync_header(&mut self, input: blocks::SyncHeaderReq) -> Result<Value, Value> {
        let resp = self
            .sync_header(input.headers, input.authority_set_change)
            .map_err(display)?;
        Ok(json!({ "synced_to": resp.synced_to }))
    }

    fn bin_sync_para_header(&mut self, input: SyncParachainHeaderReq) -> Result<Value, Value> {
        let resp = self
            .sync_para_header(input.headers, input.proof)
            .map_err(display)?;
        Ok(json!({ "synced_to": resp.synced_to }))
    }

    fn bin_sync_combined_headers(&mut self, input: SyncCombinedHeadersReq) -> Result<Value, Value> {
        let resp = self
            .sync_combined_headers(
                input.relaychain_headers,
                input.authority_set_change,
                input.parachain_headers,
                input.proof,
            )
            .map_err(display)?;
        Ok(json!({
            "relaychain_synced_to": resp.relaychain_synced_to,
            "parachain_synced_to": resp.parachain_synced_to,
        }))
    }

    fn bin_dispatch_block(
        &mut self,
        req_id: u64,
        input: blocks::DispatchBlockReq,
    ) -> Result<Value, Value> {
        let resp = self.dispatch_blocks(req_id, input.blocks).map_err(display)?;
        Ok(json!({ "dispatched_to": resp.synced_to }))
    }

    fn try_handle_scale_api(
        &mut self,
        req_id: u64,
        action: u8,
        input: &[u8],
    ) -> Result<Value, Value> {
        use phactory_api::actions::*;

        fn load_scale<T: Decode>(mut scale: &[u8]) -> Result<T, Value> {
            Decode::decode(&mut scale).map_err(|_| error_msg("Decode input parameter failed"))
        }

        match action {
            ACTION_GET_INFO => self.get_info_json(),
            BIN_ACTION_SYNC_HEADER => self.bin_sync_header(load_scale(input)?),
            BIN_ACTION_SYNC_PARA_HEADER => self.bin_sync_para_header(load_scale(input)?),
            BIN_ACTION_SYNC_COMBINED_HEADERS => self.bin_sync_combined_headers(load_scale(input)?),
            BIN_ACTION_DISPATCH_BLOCK => self.bin_dispatch_block(req_id, load_scale(input)?),
            _ => Err(error_msg("Action not found")),
        }
    }

    pub fn handle_scale_api(&mut self, req_id: u64, action: u8, input: &[u8]) -> Vec<u8> {
        let result = self.try_handle_scale_api(req_id, action, input);
        let (status, payload) = match result {
            Ok(payload) => ("ok", payload),
            Err(payload) => ("error", payload),
        };

        // Sign the output payload
        let str_payload = payload.to_string();
        let signature = self.sign_http_response(str_payload.as_bytes());
        let output_json = json!({
            "status": status,
            "payload": str_payload,
            "signature": signature,
        });
        info!("{}", output_json.to_string());
        serde_json::to_vec(&output_json).unwrap()
    }
}
