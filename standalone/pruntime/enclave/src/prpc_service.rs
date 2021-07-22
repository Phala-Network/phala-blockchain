use super::*;
use enclave_api::prpc::phactory_api_server::{PhactoryApi, PhactoryApiServer};
use enclave_api::prpc::PhactoryInfo;

#[no_mangle]
pub extern "C" fn ecall_prpc_request(
    path: *const uint8_t,
    path_len: usize,
    data: *const uint8_t,
    data_len: usize,
    status_code: *mut u16,
    output_ptr: *mut uint8_t,
    output_buf_len: usize,
    output_len_ptr: *mut usize,
) -> sgx_status_t {
    let (code, data) = prpc_request(path, path_len, data, data_len);
    let (code, data) = if data.len() > output_buf_len {
        error!("ecall_prpc_request: output buffer too short");
        (500, vec![])
    } else {
        (code, data)
    };
    info!("rpc code: {}, data len: {}", code, data.len());
    unsafe {
        *status_code = code;
        let len = output_buf_len.min(data.len());
        core::ptr::copy_nonoverlapping(data.as_ptr(), output_ptr, len);
        *output_len_ptr = len;
    }
    sgx_status_t::SGX_SUCCESS
}

fn prpc_request(
    path: *const uint8_t,
    path_len: usize,
    data: *const uint8_t,
    data_len: usize,
) -> (u16, Vec<u8>) {
    use prpc::server::{Error, ProtoError};

    let path = unsafe { std::slice::from_raw_parts(path, path_len) };
    let path = match std::str::from_utf8(path) {
        Ok(path) => path,
        Err(e) => {
            error!("prpc_request: invalid path: {}", e);
            return (500, vec![]);
        }
    };
    let server = PhactoryApiServer::new(Server);
    let data = unsafe { std::slice::from_raw_parts(data, data_len) };
    info!("Dispatching request: {}", path);
    let (code, data) = match server.dispatch_request(path, data.to_vec()) {
        Ok(data) => (200, data),
        Err(e) => {
            let (code, err) = match e {
                Error::NotFound => (404, ProtoError::new("Method Not Found")),
                Error::DecodeError(_) => (400, ProtoError::new("DecodeError")),
                Error::AppError(msg) => (500, ProtoError::new(msg)),
            };
            (code, prpc::codec::encode_message_to_vec(&err))
        }
    };

    (code, data)
}

pub fn get_info() -> PhactoryInfo {
    let local_state = LOCAL_STATE.lock().unwrap();

    let initialized = local_state.initialized;
    let genesis_block_hash = local_state
        .genesis_block_hash
        .as_ref()
        .map(|hash| hex::encode(hash));
    let public_key = local_state
        .identity_key
        .as_ref()
        .map(|pair| hex::encode(pair.public().as_ref()));
    let ecdh_public_key = local_state
        .ecdh_key
        .as_ref()
        .map(|pair| hex::encode(pair.public().as_ref()));
    let dev_mode = local_state.dev_mode;
    drop(local_state);

    let state = STATE.lock().unwrap();
    let (state_root, pending_messages, counters) = match state.as_ref() {
        Some(state) => {
            let state_root = hex::encode(state.chain_storage.root());
            let pending_messages = state.send_mq.count_messages();
            let counters = state.storage_synchronizer.counters();
            (state_root, pending_messages, counters)
        }
        None => Default::default(),
    };
    drop(state);

    let (registered, role) = {
        match SYSTEM_STATE.lock().unwrap().as_ref() {
            Some(system) => (system.is_registered(), system.gatekeeper_role()),
            None => (false, GatekeeperRole::None),
        }
    };
    let score = benchmark::score();

    PhactoryInfo {
        initialized,
        registered,
        gatekeeper_role: role.into(),
        genesis_block_hash,
        public_key,
        ecdh_public_key,
        headernum: counters.next_header_number,
        para_headernum: counters.next_para_header_number,
        blocknum: counters.next_block_number,
        state_root,
        dev_mode,
        pending_messages: pending_messages as _,
        score,
    }
}

struct Server;

/// A server that process all RPCs.
impl PhactoryApi for Server {
    /// Get basic information about Phactory state.
    fn get_info(&self, _request: ()) -> Result<PhactoryInfo, prpc::server::Error> {
        Ok(get_info())
    }
}
