use super::*;
use enclave_api::{blocks, crypto, prpc as pb};
use pb::{
    phactory_api_server::{PhactoryApi, PhactoryApiServer},
    server::Error as RpcError,
};
use phala_types::contract;

type RpcResult<T> = Result<T, RpcError>;

fn from_display(e: impl core::fmt::Display) -> RpcError {
    RpcError::AppError(e.to_string())
}

fn from_debug(e: impl core::fmt::Debug) -> RpcError {
    RpcError::AppError(format!("{:?}", e))
}

// TODO: Implement a secure channel of the RPC service.
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
    let (code, data) = prpc_request(path, path_len, data, data_len, output_buf_len);
    let (code, data) = if data.len() > output_buf_len {
        error!("ecall_prpc_request: output buffer too short");
        (500, vec![])
    } else {
        (code, data)
    };
    info!("pRPC status code: {}, data len: {}", code, data.len());
    unsafe {
        *status_code = code;
        let len = data.len().min(output_buf_len);
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
    output_buf_len: usize,
) -> (u16, Vec<u8>) {
    use prpc::server::{Error, ProtoError};

    let path = unsafe { std::slice::from_raw_parts(path, path_len) };
    let path = match std::str::from_utf8(path) {
        Ok(path) => path,
        Err(e) => {
            error!("prpc_request: invalid path: {}", e);
            return (400, b"Invalid path".to_vec());
        }
    };
    let server = PhactoryApiServer::new(RpcService { output_buf_len });
    let data = unsafe { std::slice::from_raw_parts(data, data_len) };
    info!("Dispatching request: {}", path);
    let (code, data) = match server.dispatch_request(path, data.to_vec()) {
        Ok(data) => (200, data),
        Err(e) => {
            let (code, err) = match e {
                Error::NotFound => (404, ProtoError::new("Method Not Found")),
                Error::DecodeError(err) => {
                    (400, ProtoError::new(format!("DecodeError({:?})", err)))
                }
                Error::AppError(msg) => (500, ProtoError::new(msg)),
                Error::ContractQueryError(msg) => (500, ProtoError::new(msg)),
            };
            (code, prpc::codec::encode_message_to_vec(&err))
        }
    };

    (code, data)
}

pub fn get_info() -> pb::PhactoryInfo {
    let local_state = LOCAL_STATE.lock().unwrap();

    let initialized = local_state.initialized;
    let genesis_block_hash = local_state
        .genesis_block_hash
        .as_ref()
        .map(hex::encode);
    let public_key = local_state
        .identity_key
        .as_ref()
        .map(|pair| hex::encode(pair.public()));
    let ecdh_public_key = local_state
        .ecdh_key
        .as_ref()
        .map(|pair| hex::encode(pair.public()));
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

    let (registered, gatekeeper_status) = {
        match SYSTEM_STATE.lock().unwrap().as_ref() {
            Some(system) => (system.is_registered(), system.gatekeeper_status()),
            None => (
                false,
                pb::GatekeeperStatus {
                    role: pb::GatekeeperRole::None.into(),
                    master_public_key: Default::default(),
                },
            ),
        }
    };
    let score = benchmark::score();

    pb::PhactoryInfo {
        initialized,
        registered,
        gatekeeper: Some(gatekeeper_status),
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

pub fn sync_header(
    headers: Vec<blocks::HeaderToSync>,
    authority_set_change: Option<blocks::AuthoritySetChange>,
) -> RpcResult<pb::SyncedTo> {
    info!(
        "sync_header from={:?} to={:?}",
        headers.first().map(|h| h.header.number),
        headers.last().map(|h| h.header.number)
    );
    let last_header = STATE
        .lock()
        .unwrap()
        .as_mut()
        .ok_or_else(|| from_display("Runtime not initialized"))?
        .storage_synchronizer
        .sync_header(headers, authority_set_change)
        .map_err(from_display)?;

    Ok(pb::SyncedTo {
        synced_to: last_header,
    })
}

pub fn sync_para_header(
    headers: blocks::Headers,
    proof: blocks::StorageProof,
) -> RpcResult<pb::SyncedTo> {
    info!(
        "sync_para_header from={:?} to={:?}",
        headers.first().map(|h| h.number),
        headers.last().map(|h| h.number)
    );
    let mut guard = STATE.lock().unwrap();
    let state = guard
        .as_mut()
        .ok_or_else(|| from_display("Runtime not initialized"))?;

    let para_id = state
        .chain_storage
        .para_id()
        .ok_or_else(|| from_display("No para_id"))?;

    let storage_key =
        light_validation::utils::storage_map_prefix_twox_64_concat(b"Paras", b"Heads", &para_id);

    let last_header = state
        .storage_synchronizer
        .sync_parachain_header(headers, proof, &storage_key)
        .map_err(from_display)?;

    Ok(pb::SyncedTo {
        synced_to: last_header,
    })
}

/// Sync a combined batch of relaychain & parachain headers
/// NOTE:
///   - The two latest headers MUST be aligned with each other by the `Para.Heads` read from the relaychain storage.
///   - The operation is not guarenteed to be atomical. If the parachain header is rejected, the already synced relaychain
///     headers will keep it's progress.
pub fn sync_combined_headers(
    relaychain_headers: Vec<blocks::HeaderToSync>,
    authority_set_change: Option<blocks::AuthoritySetChange>,
    parachain_headers: blocks::Headers,
    proof: blocks::StorageProof,
) -> RpcResult<pb::HeadersSyncedTo> {
    let relaychain_synced_to = sync_header(relaychain_headers, authority_set_change)?.synced_to;
    let parachain_synced_to = if parachain_headers.is_empty() {
        STATE
            .lock()
            .unwrap()
            .as_ref()
            .ok_or_else(|| from_display("Runtime not initialized"))?
            .storage_synchronizer
            .counters()
            .next_para_header_number
            - 1
    } else {
        sync_para_header(parachain_headers, proof)?.synced_to
    };
    Ok(pb::HeadersSyncedTo {
        relaychain_synced_to,
        parachain_synced_to,
    })
}

pub fn dispatch_block(blocks: Vec<blocks::BlockHeaderWithChanges>) -> RpcResult<pb::SyncedTo> {
    info!(
        "dispatch_block from={:?} to={:?}",
        blocks.first().map(|h| h.block_header.number),
        blocks.last().map(|h| h.block_header.number)
    );

    let mut state = STATE.lock().unwrap();
    let state = state
        .as_mut()
        .ok_or_else(|| from_display("Runtime not initialized"))?;

    let mut last_block = 0;
    for block in blocks.into_iter() {
        state
            .storage_synchronizer
            .feed_block(&block, &mut state.chain_storage)
            .map_err(from_display)?;

        state.purge_mq();
        handle_inbound_messages(block.block_header.number, state).map_err(from_display)?;
        last_block = block.block_header.number;
    }

    Ok(pb::SyncedTo {
        synced_to: last_block,
    })
}

pub fn init_runtime(
    skip_ra: bool,
    is_parachain: bool,
    genesis: blocks::GenesisBlockInfo,
    genesis_state: blocks::StorageState,
    operator: Option<chain::AccountId>,
    debug_set_key: ::core::option::Option<Vec<u8>>,
) -> RpcResult<pb::InitRuntimeResponse> {
    let mut local_state = LOCAL_STATE.lock().unwrap();

    if local_state.initialized {
        return Err(from_display("Runtime already initialized"));
    }

    // load chain genesis
    let genesis_block_hash = genesis.block_header.hash();

    // load identity
    if let Some(raw_key) = debug_set_key {
        if !skip_ra {
            return Err(from_display(
                "RA is disallowed when debug_set_key is enabled",
            ));
        }
        let priv_key = sr25519::Pair::from_seed_slice(&raw_key).map_err(from_debug)?;
        init_secret_keys(&mut local_state, genesis_block_hash, Some(priv_key))
            .map_err(from_display)?;
    } else {
        init_secret_keys(&mut local_state, genesis_block_hash, None)
            .map_err(from_display)?;
    }

    if !skip_ra && local_state.dev_mode {
        return Err(from_display(
            "RA is disallowed when debug_set_key is enabled",
        ));
    }

    let ecdsa_pk = local_state
        .identity_key
        .as_ref()
        .expect("Identity key must be initialized; qed.")
        .public();
    let ecdsa_hex_pk = hex::encode(&ecdsa_pk);
    info!("Identity pubkey: {:?}", ecdsa_hex_pk);

    let local_ecdh_key = local_state
        .ecdh_key
        .as_ref()
        .cloned()
        .expect("ECDH key must be initialized; qed.");

    // derive ecdh key
    let ecdh_pubkey = phala_types::EcdhPublicKey(local_ecdh_key.public());
    let ecdh_hex_pk = hex::encode(ecdh_pubkey.0.as_ref());
    info!("ECDH pubkey: {:?}", ecdh_hex_pk);

    // Measure machine score
    let cpu_core_num: u32 = sgx_trts::enclave::rsgx_get_cpu_core_num();
    info!("CPU cores: {}", cpu_core_num);

    let mut cpu_feature_level: u32 = 1;
    // Atom doesn't support AVX
    if is_x86_feature_detected!("avx2") {
        info!("CPU Support AVX2");
        cpu_feature_level += 1;

        // Customer-level Core doesn't support AVX512
        if is_x86_feature_detected!("avx512f") {
            info!("CPU Support AVX512");
            cpu_feature_level += 1;
        }
    }

    // Build WorkerRegistrationInfo
    let runtime_info = WorkerRegistrationInfo::<chain::AccountId> {
        version: VERSION,
        machine_id: local_state.machine_id,
        pubkey: ecdsa_pk,
        ecdh_pubkey: ecdh_pubkey.clone(),
        genesis_block_hash,
        features: vec![cpu_core_num, cpu_feature_level],
        operator,
    };

    // Initialize bridge
    let next_headernum = genesis.block_header.number + 1;
    let mut state = STATE.lock().unwrap();
    let mut light_client = LightValidation::new();
    let main_bridge = light_client
        .initialize_bridge(
            genesis.block_header,
            genesis.validator_set,
            genesis.validator_set_proof,
        )
        .expect("Bridge initialize failed");

    let storage_synchronizer = if is_parachain {
        Box::new(ParachainSynchronizer::new(
            light_client,
            main_bridge,
            next_headernum,
        )) as _
    } else {
        Box::new(SolochainSynchronizer::new(light_client, main_bridge)) as _
    };

    let id_pair = local_state
        .identity_key
        .clone()
        .expect("Unexpected ecdsa key error in init_runtime");

    let send_mq = MessageSendQueue::default();
    let mut recv_mq = MessageDispatcher::default();

    // Re-init some contracts because they require the identity key
    let mut system_state = SYSTEM_STATE.lock().unwrap();
    *system_state = Some(system::System::new(
        local_state.sealing_path.clone(),
        &id_pair,
        &send_mq,
        &mut recv_mq,
    ));
    drop(system_state);

    let mut contracts: BTreeMap<ContractId, Box<dyn contracts::Contract + Send>> =
        Default::default();

    if local_state.dev_mode {
        // Install contracts when running in dev_mode.

        macro_rules! install_contract {
            ($id: expr, $inner: expr) => {{
                let contract_id = contract::id256($id);
                let sender = MessageOrigin::native_contract($id);
                let mq = send_mq.channel(sender, id_pair.clone());
                // TODO.kevin: use real contract key
                let contract_key = local_ecdh_key.clone();
                let cmd_mq = PeelingReceiver::new_secret(
                    recv_mq
                        .subscribe(contract::command_topic(contract_id))
                        .into(),
                    contract_key,
                );
                let wrapped = Box::new(contracts::NativeCompatContract::new(
                    $inner,
                    mq,
                    cmd_mq,
                    local_ecdh_key.clone(),
                ));
                contracts.insert(contract_id, wrapped);
            }};
        }

        install_contract!(contracts::BALANCES, contracts::balances::Balances::new());
        install_contract!(contracts::ASSETS, contracts::assets::Assets::new());
        // TODO.kevin:
        // install_contract!(contracts::DIEM, contracts::diem::Diem::new());
        install_contract!(
            contracts::SUBSTRATE_KITTIES,
            contracts::substrate_kitties::SubstrateKitties::new()
        );
        install_contract!(
            contracts::BTC_LOTTERY,
            contracts::btc_lottery::BtcLottery::new(Some(id_pair.clone()))
        );
        install_contract!(
            contracts::WEB3_ANALYTICS,
            contracts::web3analytics::Web3Analytics::new()
        );
        install_contract!(
            contracts::DATA_PLAZA,
            contracts::data_plaza::DataPlaza::new()
        );
    }

    let mut runtime_state = RuntimeState {
        contracts,
        send_mq,
        recv_mq,
        storage_synchronizer,
        chain_storage: Default::default(),
    };

    // Initialize other states
    runtime_state.chain_storage.load(genesis_state.into_iter());

    info!(
        "Genesis state loaded: {:?}",
        runtime_state.chain_storage.root()
    );

    *state = Some(runtime_state);

    let resp = pb::InitRuntimeResponse::new(
        runtime_info,
        genesis_block_hash,
        ecdsa_pk,
        ecdh_pubkey,
        None,
    );
    local_state.skip_ra = skip_ra;
    local_state.runtime_info = Some(resp.clone());
    local_state.initialized = true;
    Ok(resp)
}

pub fn get_runtime_info() -> RpcResult<pb::InitRuntimeResponse> {
    let mut state = LOCAL_STATE.lock().unwrap();

    let skip_ra = state.skip_ra;

    let mut cached_resp = state
        .runtime_info
        .as_mut()
        .ok_or_else(|| from_display("Uninitiated runtime info"))?;

    if !skip_ra {
        if let Some(cached_attestation) = &cached_resp.attestation {
            const MAX_ATTESTATION_AGE: u64 = 60 * 60;
            if now() > cached_attestation.timestamp + MAX_ATTESTATION_AGE {
                cached_resp.attestation = None;
            }
        }
        if cached_resp.attestation.is_none() {
            // We hash the encoded bytes directly
            let runtime_info_hash = sp_core::hashing::blake2_256(&cached_resp.encoded_runtime_info);
            info!("Encoded runtime info");
            info!("{:?}", hex::encode(&cached_resp.encoded_runtime_info));
            let (attn_report, sig, cert) = match create_attestation_report(
                &runtime_info_hash,
                sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE,
            ) {
                Ok(r) => r,
                Err(e) => {
                    error!("Error in create_attestation_report: {:?}", e);
                    return Err(from_display("Error while connecting to IAS"));
                }
            };

            cached_resp.attestation = Some(pb::Attestation {
                version: 1,
                provider: "SGX".to_string(),
                payload: Some(pb::AttestationReport {
                    report: attn_report,
                    signature: base64::decode(sig).map_err(from_display)?,
                    signing_cert: base64::decode_config(cert, base64::STANDARD)
                        .map_err(from_display)?,
                }),
                timestamp: now(),
            });
        }
    }
    Ok(cached_resp.clone())
}

fn now() -> u64 {
    use crate::std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    now.as_secs()
}

// Drop latest messages if needed to fit in size.
fn fit_size(mut messages: pb::EgressMessages, size: usize) -> pb::EgressMessages {
    while messages.encoded_size() > size {
        for (_, queue) in messages.iter_mut() {
            if queue.pop().is_some() {
                break;
            }
        }
        messages.retain(|(_, q)| !q.is_empty());
        if messages.is_empty() {
            break;
        }
    }
    messages
}

pub fn get_egress_messages(output_buf_len: usize) -> RpcResult<pb::EgressMessages> {
    let messages: Vec<_> = STATE
        .lock()
        .unwrap()
        .as_ref()
        .map(|state| state.send_mq.all_messages_grouped().into_iter().collect())
        .unwrap_or_default();
    // Prune messages if needed to avoid the OUTPUT BUFFER overflow.
    Ok(fit_size(messages, output_buf_len))
}

fn contract_query(request: pb::ContractQueryRequest) -> RpcResult<pb::ContractQueryResponse> {
    // Validate signature
    if let Some(origin) = &request.signature {
        if !origin.verify(&request.encoded_encrypted_data) {
            return Err(from_display("Verifying signature failed"));
        }
        info!("Verifying signature passed!");
    }
    let ecdh_key = LOCAL_STATE
        .lock()
        .unwrap()
        .ecdh_key
        .clone()
        .ok_or_else(|| from_display("No ECDH key"))?;

    // Decrypt data
    let encrypted_req = request.decode_encrypted_data()?;
    let data = encrypted_req.decrypt(&ecdh_key).map_err(from_debug)?;

    // Decode head
    let mut data_cursor = &data[..];
    let head = contract::ContractQueryHead::decode(&mut data_cursor)?;
    let data_cursor = data_cursor;

    // Origin
    let accid_origin = match request.signature.as_ref() {
        Some(sig) => {
            use core::convert::TryFrom;
            let accid = chain::AccountId::try_from(sig.origin.as_slice())
                .map_err(|_| from_display("Bad account id"))?;
            Some(accid)
        }
        None => None,
    };

    // Dispatch
    let ref_origin = accid_origin.as_ref();

    let res = if head.id == contract::id256(SYSTEM) {
        let mut guard = SYSTEM_STATE.lock().unwrap();
        let system_state = guard
            .as_mut()
            .ok_or_else(|| from_display("Runtime not initialized"))?;
        let response = system_state.handle_query(ref_origin, types::deopaque_query(data_cursor)?);
        response.encode()
    } else {
        let mut state = STATE.lock().unwrap();
        let state = state
            .as_mut()
            .ok_or_else(|| from_display("Runtime not initialized"))?;
        let contract = state
            .contracts
            .get_mut(&head.id)
            .ok_or_else(|| from_display("Contract not found"))?;
        contract.handle_query(ref_origin, data_cursor)?
    };

    // Encode response
    let response = contract::ContractQueryResponse {
        nonce: head.nonce,
        result: contract::Data(res),
    };
    let response_data = response.encode();

    // Encrypt
    let encrypted_resp = crypto::EncryptedData::encrypt(
        &ecdh_key,
        &encrypted_req.pubkey,
        crate::generate_random_iv(),
        &response_data,
    )
    .map_err(from_debug)?;

    Ok(pb::ContractQueryResponse::new(encrypted_resp))
}

pub struct RpcService {
    output_buf_len: usize,
}

/// A server that process all RPCs.
impl PhactoryApi for RpcService {
    /// Get basic information about Phactory state.
    fn get_info(&self, _request: ()) -> RpcResult<pb::PhactoryInfo> {
        Ok(get_info())
    }

    /// Sync the parent chain header
    fn sync_header(&self, request: pb::HeadersToSync) -> RpcResult<pb::SyncedTo> {
        let headers = request.decode_headers()?;
        let authority_set_change = request.decode_authority_set_change()?;
        sync_header(headers, authority_set_change)
    }

    /// Sync the parachain header
    fn sync_para_header(&self, request: pb::ParaHeadersToSync) -> RpcResult<pb::SyncedTo> {
        let headers = request.decode_headers()?;
        sync_para_header(headers, request.proof)
    }

    fn sync_combined_headers(
        &self,
        request: pb::CombinedHeadersToSync,
    ) -> Result<pb::HeadersSyncedTo, prpc::server::Error> {
        sync_combined_headers(
            request.decode_relaychain_headers()?,
            request.decode_authority_set_change()?,
            request.decode_parachain_headers()?,
            request.proof,
        )
    }

    /// Dispatch blocks (Sync storage changes)"
    fn dispatch_blocks(&self, request: pb::Blocks) -> RpcResult<pb::SyncedTo> {
        let blocks = request.decode_blocks()?;
        dispatch_block(blocks)
    }

    fn init_runtime(&self, request: pb::InitRuntimeRequest) -> RpcResult<pb::InitRuntimeResponse> {
        init_runtime(
            request.skip_ra,
            request.is_parachain,
            request.decode_genesis_info()?,
            request.decode_genesis_state()?,
            request.decode_operator()?,
            request.debug_set_key,
        )
    }

    fn get_runtime_info(&self, _: ()) -> RpcResult<pb::InitRuntimeResponse> {
        get_runtime_info()
    }

    fn get_egress_messages(&self, _: ()) -> RpcResult<pb::GetEgressMessagesResponse> {
        // The ENCLAVE OUTPUT BUFFER is a fixed size big buffer.
        assert!(self.output_buf_len >= 1024);
        get_egress_messages(self.output_buf_len - 1024).map(pb::GetEgressMessagesResponse::new)
    }

    fn contract_query(
        &self,
        request: pb::ContractQueryRequest,
    ) -> RpcResult<pb::ContractQueryResponse> {
        contract_query(request)
    }
}
