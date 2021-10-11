use super::*;

// For bin_api
impl<Platform: pal::Platform> Phactory<Platform> {
    fn get_info_json(&self) -> Result<Value, Value> {
        let info = self.get_info();
        let machine_id = hex::encode(&self.machine_id);
        let gatekeeper = info.gatekeeper.unwrap();
        let meminfo = info.memory_usage.unwrap_or_default();
        Ok(json!({
            "initialized": info.initialized,
            "registered": info.registered,
            "gatekeeper": {
                "role": gatekeeper.role,
                "master_public_key": gatekeeper.master_public_key,
            },
            "genesis_block_hash": info.genesis_block_hash,
            "public_key": info.public_key,
            "ecdh_public_key": info.ecdh_public_key,
            "headernum": info.headernum,
            "para_headernum": info.para_headernum,
            "blocknum": info.blocknum,
            "state_root": info.state_root,
            "dev_mode": info.dev_mode,
            "pending_messages": info.pending_messages,
            "score": info.score,
            "machine_id": machine_id,
            "version": info.version,
            "git_revision": info.git_revision,
            "running_side_tasks": info.running_side_tasks,
            "memory_usage": {
                "total_peak_used": meminfo.total_peak_used,
                "rust_used": meminfo.rust_used,
                "rust_peak_used": meminfo.rust_peak_used,
            }
        }))
    }

    fn bin_sync_header(&mut self, input: blocks::SyncHeaderReq) -> Result<Value, Value> {
        let resp =
            self.sync_header(input.headers, input.authority_set_change).map_err(display)?;
        Ok(json!({ "synced_to": resp.synced_to }))
    }

    fn bin_sync_para_header(&mut self, input: SyncParachainHeaderReq) -> Result<Value, Value> {
        let resp = self.sync_para_header(input.headers, input.proof).map_err(display)?;
        Ok(json!({ "synced_to": resp.synced_to }))
    }

    fn bin_sync_combined_headers(&mut self, input: SyncCombinedHeadersReq) -> Result<Value, Value> {
        let resp = self.sync_combined_headers(
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

    fn bin_dispatch_block(&mut self, input: blocks::DispatchBlockReq) -> Result<Value, Value> {
        let resp = self.dispatch_block(input.blocks).map_err(display)?;
        Ok(json!({ "dispatched_to": resp.synced_to }))
    }

    fn try_handle_scale_api(&mut self, action: u8, input: &[u8]) -> Result<Value, Value> {
        use phactory_api::actions::*;

        fn load_scale<T: Decode>(mut scale: &[u8]) -> Result<T, Value> {
            Decode::decode(&mut scale).map_err(|_| error_msg("Decode input parameter failed"))
        }

        match action {
            ACTION_GET_INFO => self.get_info_json(),
            BIN_ACTION_SYNC_HEADER => self.bin_sync_header(load_scale(input)?),
            BIN_ACTION_SYNC_PARA_HEADER => self.bin_sync_para_header(load_scale(input)?),
            BIN_ACTION_SYNC_COMBINED_HEADERS => self.bin_sync_combined_headers(load_scale(input)?),
            BIN_ACTION_DISPATCH_BLOCK => self.bin_dispatch_block(load_scale(input)?),
            _ => Err(error_msg("Action not found")),
        }
    }

    pub fn handle_scale_api(&mut self, action: u8, input: &[u8]) -> Vec<u8> {
        let result = self.try_handle_scale_api(action, input);
        let (status, payload) = match result {
            Ok(payload) => ("ok", payload),
            Err(payload) => ("error", payload),
        };

        // Sign the output payload
        let str_payload = payload.to_string();
        let signature: Option<String> = self.system.as_ref().map(|state| {
            let bytes = str_payload.as_bytes();
            let sig = state.identity_key.sign(bytes).0;
            hex::encode(&sig)
        });
        let output_json = json!({
            "status": status,
            "payload": str_payload,
            "signature": signature,
        });
        info!("{}", output_json.to_string());
        serde_json::to_vec(&output_json).unwrap()
    }
}
