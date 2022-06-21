use std::convert::TryFrom;
use std::sync::{Mutex, MutexGuard};

use crate::system::{chain_state, System};

use super::*;
use chain::pallet_registry::{extract_mrenclave, Attestation, AttestationValidator, IasValidator};
use parity_scale_codec::Encode;
use pb::{
    phactory_api_server::{PhactoryApi, PhactoryApiServer},
    server::Error as RpcError,
};
use phactory_api::{blocks, crypto, prpc as pb};
use phala_crypto::{
    aead, ecdh, key_share,
    sr25519::{Persistence, KDF},
};
use phala_types::worker_endpoint_v1::{EndpointInfo, WorkerEndpoint};
use phala_types::{
    contract, messaging::EncryptedKey, EndpointType, VersionedWorkerEndpoints,
    WorkerEndpointPayload, WorkerPublicKey,
};

type RpcResult<T> = Result<T, RpcError>;

fn from_display(e: impl core::fmt::Display) -> RpcError {
    RpcError::AppError(e.to_string())
}

fn from_debug(e: impl core::fmt::Debug) -> RpcError {
    RpcError::AppError(format!("{:?}", e))
}

pub const VERSION: u32 = 1;

fn now() -> u64 {
    use std::time::SystemTime;
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

impl<Platform: pal::Platform + Serialize + DeserializeOwned> Phactory<Platform> {
    fn runtime_state(&mut self) -> RpcResult<&mut RuntimeState> {
        self.runtime_state
            .as_mut()
            .ok_or_else(|| from_display("Runtime not initialized"))
    }

    fn system(&mut self) -> RpcResult<&mut System<Platform>> {
        self.system
            .as_mut()
            .ok_or_else(|| from_display("Runtime not initialized"))
    }

    pub fn get_info(&self) -> pb::PhactoryInfo {
        let initialized = self.system.is_some();
        let state = self.runtime_state.as_ref();
        let system = self.system.as_ref();
        let genesis_block_hash = state.map(|state| hex::encode(&state.genesis_block_hash));
        let public_key = system.map(|state| hex::encode(state.identity_key.public()));
        let ecdh_public_key = system.map(|state| hex::encode(&state.ecdh_key.public()));
        let dev_mode = self.dev_mode;

        let (state_root, pending_messages, counters) = match state.as_ref() {
            Some(state) => {
                let state_root = hex::encode(state.chain_storage.root());
                let pending_messages = state.send_mq.count_messages();
                let counters = state.storage_synchronizer.counters();
                (state_root, pending_messages, counters)
            }
            None => Default::default(),
        };

        let registered;
        let gatekeeper_status;
        let number_of_clusters;
        let number_of_contracts;

        match system {
            Some(system) => {
                registered = system.is_registered();
                gatekeeper_status = system.gatekeeper_status();
                number_of_clusters = system.contract_clusters.len();
                number_of_contracts = system.contracts.len();
            }
            None => {
                registered = false;
                gatekeeper_status = pb::GatekeeperStatus {
                    role: pb::GatekeeperRole::None.into(),
                    master_public_key: Default::default(),
                    share_master_key_history: Default::default(),
                };
                number_of_clusters = 0;
                number_of_contracts = 0;
            }
        }

        let score = benchmark::score();
        let m_usage = self.platform.memory_usage();

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
            waiting_for_paraheaders: counters.waiting_for_paraheaders,
            state_root,
            dev_mode,
            pending_messages: pending_messages as _,
            score,
            version: self.args.version.clone(),
            git_revision: self.args.git_revision.clone(),
            running_side_tasks: self.side_task_man.tasks_count() as _,
            memory_usage: Some(pb::MemoryUsage {
                rust_used: m_usage.rust_used as _,
                rust_peak_used: m_usage.rust_peak_used as _,
                total_peak_used: m_usage.total_peak_used as _,
            }),
            number_of_clusters: number_of_clusters as _,
            number_of_contracts: number_of_contracts as _,
        }
    }

    pub(crate) fn sync_header(
        &mut self,
        headers: Vec<blocks::HeaderToSync>,
        authority_set_change: Option<blocks::AuthoritySetChange>,
    ) -> RpcResult<pb::SyncedTo> {
        info!(
            "sync_header from={:?} to={:?}",
            headers.first().map(|h| h.header.number),
            headers.last().map(|h| h.header.number)
        );
        let last_header = self
            .runtime_state()?
            .storage_synchronizer
            .sync_header(headers, authority_set_change)
            .map_err(from_display)?;

        Ok(pb::SyncedTo {
            synced_to: last_header,
        })
    }

    pub(crate) fn sync_para_header(
        &mut self,
        headers: blocks::Headers,
        proof: blocks::StorageProof,
    ) -> RpcResult<pb::SyncedTo> {
        info!(
            "sync_para_header from={:?} to={:?}",
            headers.first().map(|h| h.number),
            headers.last().map(|h| h.number)
        );

        let state = self.runtime_state()?;

        let para_id = state
            .chain_storage
            .para_id()
            .ok_or_else(|| from_display("No para_id"))?;

        let storage_key = light_validation::utils::storage_map_prefix_twox_64_concat(
            b"Paras", b"Heads", &para_id,
        );

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
    pub(crate) fn sync_combined_headers(
        &mut self,
        relaychain_headers: Vec<blocks::HeaderToSync>,
        authority_set_change: Option<blocks::AuthoritySetChange>,
        parachain_headers: blocks::Headers,
        proof: blocks::StorageProof,
    ) -> RpcResult<pb::HeadersSyncedTo> {
        let relaychain_synced_to = self
            .sync_header(relaychain_headers, authority_set_change)?
            .synced_to;
        let parachain_synced_to = if parachain_headers.is_empty() {
            self.runtime_state()?
                .storage_synchronizer
                .counters()
                .next_para_header_number
                - 1
        } else {
            self.sync_para_header(parachain_headers, proof)?.synced_to
        };
        Ok(pb::HeadersSyncedTo {
            relaychain_synced_to,
            parachain_synced_to,
        })
    }

    pub(crate) fn dispatch_block(
        &mut self,
        mut blocks: Vec<blocks::BlockHeaderWithChanges>,
    ) -> RpcResult<pb::SyncedTo> {
        info!(
            "dispatch_block from={:?} to={:?}",
            blocks.first().map(|h| h.block_header.number),
            blocks.last().map(|h| h.block_header.number)
        );
        let counters = self.runtime_state()?.storage_synchronizer.counters();
        blocks.retain(|b| b.block_header.number >= counters.next_block_number);

        let mut last_block = counters.next_block_number - 1;
        for block in blocks.into_iter() {
            info!("Dispatching block: {}", block.block_header.number);
            let state = self.runtime_state()?;
            state
                .storage_synchronizer
                .feed_block(&block, &mut state.chain_storage)
                .map_err(from_display)?;
            info!("State synced");
            state.purge_mq();
            self.handle_inbound_messages(block.block_header.number)?;
            self.poll_side_tasks(block.block_header.number)?;
            if block
                .block_header
                .number
                .saturating_sub(self.last_storage_purge_at)
                >= self.args.gc_interval
            {
                self.last_storage_purge_at = block.block_header.number;
                info!("Purging database");
                self.runtime_state()?.chain_storage.purge();
            }
            last_block = block.block_header.number;

            if let Err(e) = self.maybe_take_checkpoint(last_block) {
                error!("Failed to take checkpoint: {:?}", e);
            }
        }

        Ok(pb::SyncedTo {
            synced_to: last_block,
        })
    }

    fn maybe_take_checkpoint(&mut self, current_block: chain::BlockNumber) -> anyhow::Result<()> {
        if !self.args.enable_checkpoint {
            return Ok(());
        }
        if self.last_checkpoint.elapsed().as_secs() < self.args.checkpoint_interval {
            return Ok(());
        }
        self.take_checkpoint(current_block)
    }

    fn init_runtime(
        &mut self,
        skip_ra: bool,
        is_parachain: bool,
        genesis: blocks::GenesisBlockInfo,
        genesis_state: blocks::StorageState,
        operator: Option<chain::AccountId>,
        debug_set_key: ::core::option::Option<Vec<u8>>,
    ) -> RpcResult<pb::InitRuntimeResponse> {
        if self.system.is_some() {
            return Err(from_display("Runtime already initialized"));
        }

        // load chain genesis
        let genesis_block_hash = genesis.block_header.hash();

        // load identity
        let rt_data = if let Some(raw_key) = debug_set_key {
            if !skip_ra {
                return Err(from_display(
                    "RA is disallowed when debug_set_key is enabled",
                ));
            }
            let priv_key = sr25519::Pair::from_seed_slice(&raw_key).map_err(from_debug)?;
            self.init_runtime_data(genesis_block_hash, Some(priv_key))
                .map_err(from_debug)?
        } else {
            self.init_runtime_data(genesis_block_hash, None)
                .map_err(from_debug)?
        };
        self.dev_mode = rt_data.dev_mode;
        self.skip_ra = skip_ra;

        if !skip_ra && self.dev_mode {
            return Err(from_display(
                "RA is disallowed when debug_set_key is enabled",
            ));
        }
        if !skip_ra {
            self.platform.quote_test().map_err(from_debug)?;
        }

        let (identity_key, ecdh_key) = rt_data.decode_keys();

        let ecdsa_pk = identity_key.public();
        let ecdsa_hex_pk = hex::encode(&ecdsa_pk);
        info!("Identity pubkey: {:?}", ecdsa_hex_pk);

        // derive ecdh key
        let ecdh_pubkey = phala_types::EcdhPublicKey(ecdh_key.public());
        let ecdh_hex_pk = hex::encode(ecdh_pubkey.0.as_ref());
        info!("ECDH pubkey: {:?}", ecdh_hex_pk);

        // Measure machine score
        let cpu_core_num: u32 = self.platform.cpu_core_num();
        info!("CPU cores: {}", cpu_core_num);

        let cpu_feature_level: u32 = self.platform.cpu_feature_level();
        info!("CPU feature level: {}", cpu_feature_level);

        // Build WorkerRegistrationInfo
        let runtime_info = WorkerRegistrationInfo::<chain::AccountId> {
            version: VERSION,
            machine_id: self.machine_id.clone(),
            pubkey: ecdsa_pk,
            ecdh_pubkey: ecdh_pubkey.clone(),
            genesis_block_hash,
            features: vec![cpu_core_num, cpu_feature_level],
            operator,
        };

        // Initialize bridge
        let next_headernum = genesis.block_header.number + 1;
        let mut light_client = LightValidation::new();
        let main_bridge = light_client
            .initialize_bridge(genesis.block_header, genesis.authority_set, genesis.proof)
            .expect("Bridge initialize failed");

        let storage_synchronizer = if is_parachain {
            Synchronizer::new_parachain(light_client, main_bridge, next_headernum)
        } else {
            Synchronizer::new_solochain(light_client, main_bridge)
        };

        let send_mq = MessageSendQueue::default();
        let recv_mq = MessageDispatcher::default();

        let contracts = contracts::ContractsKeeper::default();

        let mut runtime_state = RuntimeState {
            send_mq,
            recv_mq,
            storage_synchronizer,
            chain_storage: Default::default(),
            genesis_block_hash,
        };

        // Initialize other states
        runtime_state.chain_storage.load(genesis_state.into_iter());

        info!(
            "Genesis state loaded: {:?}",
            runtime_state.chain_storage.root()
        );

        let system = system::System::new(
            self.platform.clone(),
            self.args.sealing_path.clone(),
            self.args.storage_path.clone(),
            false,
            self.args.geoip_city_db.clone(),
            identity_key,
            ecdh_key,
            &runtime_state.send_mq,
            &mut runtime_state.recv_mq,
            contracts,
        );

        let resp = pb::InitRuntimeResponse::new(
            runtime_info,
            genesis_block_hash,
            ecdsa_pk,
            ecdh_pubkey,
            None,
        );
        self.skip_ra = skip_ra;
        self.runtime_info = Some(resp.clone());
        self.runtime_state = Some(runtime_state);
        self.system = Some(system);
        Ok(resp)
    }

    fn get_runtime_info(
        &mut self,
        refresh_ra: bool,
        operator: Option<chain::AccountId>,
    ) -> RpcResult<pb::InitRuntimeResponse> {
        let skip_ra = self.skip_ra;

        let mut cached_resp = self
            .runtime_info
            .as_mut()
            .ok_or_else(|| from_display("Uninitiated runtime info"))?;

        let reset_operator = operator.is_some();
        if reset_operator {
            let mut runtime_info = cached_resp
                .decode_runtime_info()
                .expect("Decode runtime_info failed");
            runtime_info.operator = operator;
            cached_resp.encoded_runtime_info = runtime_info.encode();
        }

        if !skip_ra {
            if let Some(cached_attestation) = &cached_resp.attestation {
                const MAX_ATTESTATION_AGE: u64 = 60 * 60;
                if refresh_ra
                    || reset_operator
                    || now() > cached_attestation.timestamp + MAX_ATTESTATION_AGE
                {
                    cached_resp.attestation = None;
                }
            }
            if cached_resp.attestation.is_none() {
                // We hash the encoded bytes directly
                let runtime_info_hash =
                    sp_core::hashing::blake2_256(&cached_resp.encoded_runtime_info);
                info!("Encoded runtime info");
                info!("{:?}", hex::encode(&cached_resp.encoded_runtime_info));

                let (attn_report, sig, cert) =
                    match self.platform.create_attestation_report(&runtime_info_hash) {
                        Ok(r) => r,
                        Err(e) => {
                            let message = format!("Failed to create attestation report: {:?}", e);
                            error!("{}", message);
                            return Err(from_display(message));
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

    fn get_egress_messages(&mut self, output_buf_len: usize) -> RpcResult<pb::EgressMessages> {
        let messages: Vec<_> = self
            .runtime_state
            .as_ref()
            .map(|state| state.send_mq.all_messages_grouped().into_iter().collect())
            .unwrap_or_default();
        // Prune messages if needed to avoid the OUTPUT BUFFER overflow.
        Ok(fit_size(messages, output_buf_len))
    }

    fn contract_query(
        &mut self,
        request: pb::ContractQueryRequest,
    ) -> RpcResult<impl FnOnce() -> RpcResult<pb::ContractQueryResponse>> {
        // Validate signature
        let origin = if let Some(sig) = &request.signature {
            let current_block = self.get_info().blocknum - 1;
            // At most two level cert chain supported
            match sig.verify(&request.encoded_encrypted_data, current_block, 2) {
                Ok(key_chain) => match &key_chain[..] {
                    [root_pubkey, ..] => Some(root_pubkey.clone()),
                    _ => {
                        return Err(from_display("BUG: verify ok but no key?"));
                    }
                },
                Err(err) => {
                    return Err(from_display(format!(
                        "Verifying signature failed: {:?}",
                        err
                    )));
                }
            }
        } else {
            info!("No query signature");
            None
        };

        info!("Verifying signature passed! origin={:?}", origin);

        let ecdh_key = self.system()?.ecdh_key.clone();

        // Decrypt data
        let encrypted_req = request.decode_encrypted_data()?;
        let data = encrypted_req.decrypt(&ecdh_key).map_err(from_debug)?;

        // Decode head
        let mut data_cursor = &data[..];
        let head = contract::ContractQueryHead::decode(&mut data_cursor)?;
        let rest = data_cursor.len();

        // Origin
        let accid_origin = match origin {
            Some(origin) => {
                let accid = chain::AccountId::try_from(origin.as_slice())
                    .map_err(|_| from_display("Bad account id"))?;
                Some(accid)
            }
            None => None,
        };

        // Dispatch
        let call = self.system()?.make_query(&head.id)?;

        Ok(move || {
            // Encode response
            let response = contract::ContractQueryResponse {
                nonce: head.nonce,
                result: contract::Data(call(accid_origin.as_ref(), &data[data.len() - rest..])?),
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
        })
    }

    fn handle_inbound_messages(&mut self, block_number: chain::BlockNumber) -> RpcResult<()> {
        let state = self
            .runtime_state
            .as_mut()
            .ok_or_else(|| from_display("Runtime not initialized"))?;
        let system = self
            .system
            .as_mut()
            .ok_or_else(|| from_display("Runtime not initialized"))?;

        // Dispatch events
        let messages = state
            .chain_storage
            .mq_messages()
            .map_err(|_| from_display("Can not get mq messages from storage"))?;

        state.recv_mq.reset_local_index();

        for message in messages {
            use phala_types::messaging::SystemEvent;
            macro_rules! log_message {
                ($msg: expr, $t: ident) => {{
                    let event: Result<$t, _> =
                        parity_scale_codec::Decode::decode(&mut &$msg.payload[..]);
                    match event {
                        Ok(event) => {
                            debug!(target: "mq",
                                "mq dispatching message: sender={} dest={:?} payload={:?}",
                                $msg.sender, $msg.destination, event
                            );
                        }
                        Err(_) => {
                            debug!(target: "mq", "mq dispatching message (decode failed): {:?}", $msg);
                        }
                    }
                }};
            }
            // TODO.kevin: reuse codes in debug-cli
            if message.destination.path() == &SystemEvent::topic() {
                log_message!(message, SystemEvent);
            } else {
                debug!(target: "mq",
                    "mq dispatching message: sender={}, dest={:?}",
                    message.sender, message.destination
                );
            }
            state.recv_mq.dispatch(message);
        }

        let mut guard = scopeguard::guard(&mut state.recv_mq, |mq| {
            let n_unhandled = mq.clear();
            if n_unhandled > 0 {
                warn!("There are {} unhandled messages dropped", n_unhandled);
            }
        });

        let now_ms = state
            .chain_storage
            .timestamp_now()
            .ok_or_else(|| from_display("No timestamp found in block"))?;

        let block_time = now_ms / 1000;
        let sys_time = now();

        // When delta time reaches 3600s, there are about 3600 / 12 = 300 blocks rest.
        // It need about 30 more seconds to sync up to date.
        let ready = block_time + 3600 > sys_time;
        debug!(
            "block_time={}, sys_time={}, ready={}",
            block_time, sys_time, ready
        );
        benchmark::set_ready(ready);

        let storage = &state.chain_storage;
        let side_task_man = &mut self.side_task_man;
        let recv_mq = &mut *guard;
        let mut block = BlockInfo {
            block_number,
            now_ms,
            storage,
            send_mq: &state.send_mq,
            recv_mq,
            side_task_man,
        };

        system.process_messages(&mut block);
        Ok(())
    }

    fn poll_side_tasks(&mut self, block_number: chain::BlockNumber) -> RpcResult<()> {
        let state = self
            .runtime_state
            .as_ref()
            .ok_or_else(|| from_display("Runtime not initialized"))?;
        let context = side_task::PollContext {
            block_number,
            send_mq: &state.send_mq,
            storage: &state.chain_storage,
        };
        self.side_task_man.poll(&context);
        Ok(())
    }

    fn add_endpoint(
        &mut self,
        endpoint_type: EndpointType,
        endpoint: Vec<u8>,
    ) -> RpcResult<pb::GetEndpointResponse> {
        let endpoints = if self.endpoint_cache.is_some() {
            // update the endpoints instead of inserting
            let mut endpoint_cache = self
                .endpoint_cache
                .clone()
                .expect("SignedEndpointCache should be initialized");

            endpoint_cache.endpoints.insert(endpoint_type, endpoint);

            endpoint_cache.endpoints
        } else {
            let mut endpoints = HashMap::new();
            endpoints.insert(endpoint_type, endpoint);
            endpoints
        };

        self.refresh_endpoint_signing_time(endpoints)
    }

    fn refresh_endpoint_signing_time(
        &mut self,
        endpoints: HashMap<EndpointType, Vec<u8>>,
    ) -> RpcResult<pb::GetEndpointResponse> {
        let state = self
            .runtime_state
            .as_mut()
            .ok_or_else(|| from_display("Runtime not initialized"))?;
        let block_time: u64 = state
            .chain_storage
            .timestamp_now()
            .ok_or_else(|| from_display("No timestamp found in block"))?;
        let public_key = self
            .system
            .as_ref()
            .map(|state| state.identity_key.public())
            .expect("public_key should be set");
        let mut endpoints_info = Vec::<EndpointInfo>::new();

        for (endpoint_type, endpoint) in endpoints.iter() {
            let endpoint_info = match endpoint_type {
                EndpointType::I2P => EndpointInfo {
                    endpoint: WorkerEndpoint::I2P(endpoint.clone()),
                },
                EndpointType::Http => EndpointInfo {
                    endpoint: WorkerEndpoint::Http(endpoint.clone()),
                },
            };
            endpoints_info.push(endpoint_info);
        }

        let versioned_endpoints = VersionedWorkerEndpoints::V1(endpoints_info);
        let endpoint_payload = WorkerEndpointPayload {
            pubkey: public_key,
            versioned_endpoints,
            signing_time: block_time,
        };
        let resp = pb::GetEndpointResponse::new(endpoint_payload.clone(), None);

        self.endpoint_cache = Some(SignedEndpointCache {
            endpoints,
            endpoint_payload,
            signature: None,
        });

        Ok(resp)
    }

    fn get_endpoint_info(&mut self) -> RpcResult<pb::GetEndpointResponse> {
        if self.system.is_none() {
            return Err(from_display("Runtime not initialized"));
        }

        if self.endpoint_cache.is_none() {
            return Err(from_display("No endpoint"));
        }

        let endpoint_cache = self
            .endpoint_cache
            .clone()
            .expect("SignedEndpointInfo should be initialized");

        let signature = if endpoint_cache.signature.is_none() {
            let signature = self
                .system
                .as_ref()
                .expect("Runtime should be initialized")
                .identity_key
                .clone()
                .sign(&endpoint_cache.endpoint_payload.encode())
                .encode();
            self.endpoint_cache = Some(SignedEndpointCache {
                endpoints: endpoint_cache.endpoints.clone(),
                endpoint_payload: endpoint_cache.endpoint_payload.clone(),
                signature: Some(signature.clone()),
            });

            signature
        } else {
            endpoint_cache
                .signature
                .as_ref()
                .expect("Signature should be existed")
                .clone()
        };

        let resp =
            pb::GetEndpointResponse::new(endpoint_cache.endpoint_payload.clone(), Some(signature));

        Ok(resp)
    }

    fn get_worker_key_challenge(&mut self) -> RpcResult<pb::WorkerKeyChallenge> {
        let system = self
            .system
            .as_mut()
            .ok_or_else(|| from_display("Runtime not initialized"))?;
        let challenge = system.get_worker_key_challenge();
        Ok(pb::WorkerKeyChallenge::new(challenge))
    }
}

pub fn dispatch_prpc_request<Platform>(
    path: &[u8],
    data: &[u8],
    output_buf_len: usize,
    phactory: &Mutex<Phactory<Platform>>,
) -> (u16, Vec<u8>)
where
    Platform: pal::Platform + Serialize + DeserializeOwned,
{
    use prpc::server::{Error, ProtoError};

    let path = match std::str::from_utf8(path) {
        Ok(path) => path,
        Err(e) => {
            error!("prpc_request: invalid path: {}", e);
            return (400, b"Invalid path".to_vec());
        }
    };
    info!("Dispatching request: {}", path);

    let mut server = PhactoryApiServer::new(RpcService {
        output_buf_len,
        phactory,
    });
    let (code, data) = match server.dispatch_request(path, data.to_vec()) {
        Ok(data) => (200, data),
        Err(err) => {
            error!("Rpc error: {:?}", err);
            let (code, err) = match err {
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

pub struct RpcService<'a, Platform> {
    output_buf_len: usize,
    phactory: &'a Mutex<Phactory<Platform>>,
}

impl<Platform> RpcService<'_, Platform> {
    fn lock_phactory(&self) -> MutexGuard<'_, Phactory<Platform>> {
        self.phactory.lock().unwrap()
    }
}

/// A server that process all RPCs.
impl<Platform: pal::Platform + Serialize + DeserializeOwned> PhactoryApi
    for RpcService<'_, Platform>
{
    /// Get basic information about Phactory state.
    fn get_info(&mut self, _request: ()) -> RpcResult<pb::PhactoryInfo> {
        Ok(self.lock_phactory().get_info())
    }

    /// Sync the parent chain header
    fn sync_header(&mut self, request: pb::HeadersToSync) -> RpcResult<pb::SyncedTo> {
        let headers = request.decode_headers()?;
        let authority_set_change = request.decode_authority_set_change()?;
        self.lock_phactory()
            .sync_header(headers, authority_set_change)
    }

    /// Sync the parachain header
    fn sync_para_header(&mut self, request: pb::ParaHeadersToSync) -> RpcResult<pb::SyncedTo> {
        let headers = request.decode_headers()?;
        self.lock_phactory()
            .sync_para_header(headers, request.proof)
    }

    fn sync_combined_headers(
        &mut self,
        request: pb::CombinedHeadersToSync,
    ) -> Result<pb::HeadersSyncedTo, prpc::server::Error> {
        self.lock_phactory().sync_combined_headers(
            request.decode_relaychain_headers()?,
            request.decode_authority_set_change()?,
            request.decode_parachain_headers()?,
            request.proof,
        )
    }

    /// Dispatch blocks (Sync storage changes)"
    fn dispatch_blocks(&mut self, request: pb::Blocks) -> RpcResult<pb::SyncedTo> {
        let blocks = request.decode_blocks()?;
        self.lock_phactory().dispatch_block(blocks)
    }

    fn init_runtime(
        &mut self,
        request: pb::InitRuntimeRequest,
    ) -> RpcResult<pb::InitRuntimeResponse> {
        self.lock_phactory().init_runtime(
            request.skip_ra,
            request.is_parachain,
            request.decode_genesis_info()?,
            request.decode_genesis_state()?,
            request.decode_operator()?,
            request.debug_set_key,
        )
    }

    fn get_runtime_info(
        &mut self,
        req: pb::GetRuntimeInfoRequest,
    ) -> RpcResult<pb::InitRuntimeResponse> {
        self.lock_phactory()
            .get_runtime_info(req.force_refresh_ra, req.decode_operator()?)
    }

    fn get_egress_messages(&mut self, _: ()) -> RpcResult<pb::GetEgressMessagesResponse> {
        // The ENCLAVE OUTPUT BUFFER is a fixed size big buffer.
        assert!(self.output_buf_len >= 1024);
        self.lock_phactory()
            .get_egress_messages(self.output_buf_len - 1024)
            .map(pb::GetEgressMessagesResponse::new)
    }

    fn contract_query(
        &mut self,
        request: pb::ContractQueryRequest,
    ) -> RpcResult<pb::ContractQueryResponse> {
        let do_query = self.lock_phactory().contract_query(request)?;
        do_query()
    }

    fn get_worker_state(
        &mut self,
        request: pb::GetWorkerStateRequest,
    ) -> RpcResult<pb::WorkerState> {
        let mut phactory = self.lock_phactory();
        let system = phactory.system()?;
        let gk = system
            .gatekeeper
            .as_ref()
            .ok_or_else(|| from_display("Not a gatekeeper"))?;
        let pubkey: WorkerPublicKey = request
            .public_key
            .as_slice()
            .try_into()
            .map_err(|_| from_display("Bad public key"))?;
        let state = gk
            .mining_economics
            .worker_state(&pubkey)
            .ok_or_else(|| from_display("Worker not found"))?;
        Ok(state)
    }

    fn add_endpoint(
        &mut self,
        request: pb::AddEndpointRequest,
    ) -> RpcResult<pb::GetEndpointResponse> {
        self.lock_phactory()
            .add_endpoint(request.decode_endpoint_type()?, request.endpoint)
    }

    fn refresh_endpoint_signing_time(&mut self, _: ()) -> RpcResult<pb::GetEndpointResponse> {
        let mut phactory = self.lock_phactory();
        let endpoints = phactory
            .endpoint_cache
            .as_ref()
            .ok_or(from_display("Endpoint cache not initialized"))?
            .endpoints
            .clone();

        phactory.refresh_endpoint_signing_time(endpoints)
    }

    fn get_endpoint_info(&mut self, _: ()) -> RpcResult<pb::GetEndpointResponse> {
        self.lock_phactory().get_endpoint_info()
    }

    fn derive_phala_i2p_key(&mut self, _: ()) -> RpcResult<pb::DerivePhalaI2pKeyResponse> {
        let mut phactory = self.lock_phactory();
        let system = phactory.system()?;
        let derive_key = system
            .identity_key
            .derive_sr25519_pair(&[b"PhalaI2PKey"])
            .expect("should not fail with valid key");
        Ok(pb::DerivePhalaI2pKeyResponse {
            phala_i2p_key: derive_key.dump_secret_key().to_vec(),
        })
    }

    fn echo(&mut self, request: pb::EchoMessage) -> RpcResult<pb::EchoMessage> {
        let echo_msg = request.echo_msg;
        Ok(pb::EchoMessage { echo_msg })
    }

    // WorkerKey Handover Server

    fn get_worker_key_challenge(&mut self, _request: ()) -> RpcResult<pb::WorkerKeyChallenge> {
        let mut phactory = self.lock_phactory();
        let dev_mode = phactory.dev_mode;
        let system = phactory.system()?;
        let challenge = system.get_worker_key_challenge(dev_mode);
        Ok(pb::WorkerKeyChallenge::new(challenge))
    }

    fn get_worker_key(
        &mut self,
        request: pb::WorkerKeyChallengeResponse,
    ) -> RpcResult<pb::GetWorkerKeyResponse> {
        let mut phactory = self.lock_phactory();
        let dev_mode = phactory.dev_mode;
        let system = phactory.system()?;
        let my_identity_key = system.identity_key.clone();

        // 1. verify RA report
        let challenge_client = request
            .payload
            .ok_or_else(|| from_display("Challenge response not found"))?;
        let payload_hash = sp_core::hashing::blake2_256(&challenge_client.encode());
        let now = system.now_ms;
        let block_number = system.block_number;
        let mut attestation = None;
        if !dev_mode {
            let raw_attestation = request
                .attestation
                .ok_or_else(|| from_display("Attestation not found"))?;
            let payload = raw_attestation
                .payload
                .ok_or_else(|| from_display("Missing attestation payload"))?;
            let attn_to_validate = Attestation::SgxIas {
                ra_report: payload.report.as_bytes().to_vec(),
                signature: payload.signature,
                raw_signing_cert: payload.signing_cert,
            };
            // The time from attestation report is generated by IAS, thus trusted. By default, it's valid for 10h.
            // By ensuring our system timestamp is within the valid period, we know that this pRuntime is not hold back by
            // malicious workers.
            IasValidator::validate(&attn_to_validate, &payload_hash, now, false, vec![])
                .map_err(|_| from_display("Invalid RA report"))?;
            attestation = Some(attn_to_validate);
        } else {
            info!("Skip RA report check in dev mode");
        }
        // 2. verify challenge validity to prevent replay attack
        let challenge = challenge_client.decode_challenge().map_err(from_display)?;
        if !system.verify_worker_key_challenge(&challenge) {
            return Err(from_display("Invalid challenge"));
        }
        // 3. verify challenge block height and report timestamp
        // only challenge within 150 blocks (30 minutes) is accepted
        let challenge_height = challenge.payload.block_number;
        if !(challenge_height <= block_number && block_number - challenge_height <= 150) {
            return Err(from_display("Outdated challenge"));
        }
        // 4. verify runtime timestamp
        if !dev_mode {
            let runtime_info = phactory
                .runtime_info
                .as_ref()
                .ok_or_else(|| from_display("Runtime not initialized"))?;
            let my_attn = runtime_info
                .attestation
                .as_ref()
                .ok_or_else(|| from_display("My attestation not found"))?;
            let my_attn_report = my_attn
                .payload
                .as_ref()
                .ok_or_else(|| from_display("My RA report not found"))?;
            let my_mrenclave = extract_mrenclave(&my_attn_report.report.as_bytes().to_vec())
                .map_err(|_| from_display("Invalid RA report"))?;
            let runtime_state = phactory.runtime_state()?;
            let my_runtime_timestamp =
                chain_state::get_pruntime_timestamp(&runtime_state.chain_storage, &my_mrenclave)
                    .ok_or_else(|| from_display("Key handover not supported in this pRuntime"))?;

            let attestation = attestation.ok_or_else(|| from_display("Attestation not found"))?;
            let mrenclave = match attestation {
                Attestation::SgxIas {
                    ra_report,
                    signature: _,
                    raw_signing_cert: _,
                } => extract_mrenclave(&ra_report),
            }
            .map_err(|_| from_display("Invalid received RA report"))?;
            let req_runtime_timestamp =
                chain_state::get_pruntime_timestamp(&runtime_state.chain_storage, &mrenclave)
                    .ok_or_else(|| from_display("Invalid req pRuntime"))?;
            if my_runtime_timestamp > req_runtime_timestamp {
                return Err(from_display("No handover for old pRuntime"));
            }
        } else {
            info!("Skip pRuntime timestamp check in dev mode");
        }

        // TODO.shelven: refactor
        // Share the key
        let ecdh_pubkey = challenge_client
            .decode_ecdh_public_key()
            .map_err(from_display)?;
        let derived_key = my_identity_key
            .derive_sr25519_pair(&[b"worker_key_handover"])
            .expect("should not fail with valid info; qed.");
        let my_ecdh_key = derived_key
            .derive_ecdh_key()
            .expect("ecdh key derivation should never failed with valid key; qed.");
        let secret = ecdh::agree(&my_ecdh_key, &ecdh_pubkey.0)
            .expect("should never fail with valid ecdh key; qed.");
        let iv = crate::generate_random_iv();
        let mut data = my_identity_key.dump_secret_key().to_vec();
        aead::encrypt(&iv, &secret, &mut data).expect("Failed to encrypt master key");
        let encrypted_key = EncryptedKey {
            ecdh_pubkey: sr25519::Public(my_ecdh_key.public()),
            encrypted_key: data,
            iv,
        };

        let runtime_state = phactory.runtime_state()?;
        let genesis_block_hash = runtime_state.genesis_block_hash;
        Ok(pb::GetWorkerKeyResponse::new(
            genesis_block_hash,
            Some(encrypted_key),
        ))
    }

    // WorkerKey Handover Client

    fn handle_worker_key_challenge(
        &mut self,
        request: pb::WorkerKeyChallenge,
    ) -> RpcResult<pb::WorkerKeyChallengeResponse> {
        let mut phactory = self.lock_phactory();

        // generate and save tmp key only for key handover encryption
        let tmp_key = crate::new_sr25519_key();
        let tmp_ecdh_key = tmp_key
            .derive_ecdh_key()
            .expect("should never fail with valid key; qed");
        let ecdh_public_key = phala_types::EcdhPublicKey(tmp_ecdh_key.public());
        phactory.tmp_ecdh_key = Some(tmp_ecdh_key);

        let challenge = request.decode_challenge().map_err(from_display)?;
        let payload = pb::ChallengeClient::new(challenge.clone(), ecdh_public_key);
        let payload_hash = sp_core::hashing::blake2_256(&payload.encode());

        let mut attestation = None;
        if !challenge.payload.dev_mode {
            let (attn_report, sig, cert) =
                match phactory.platform.create_attestation_report(&payload_hash) {
                    Ok(r) => r,
                    Err(e) => {
                        let message = format!("Failed to create attestation report: {:?}", e);
                        error!("{}", message);
                        return Err(from_display(message));
                    }
                };
            attestation = Some(pb::Attestation {
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
        } else {
            info!("Omit RA report in challenge response in dev mode");
        }

        Ok(pb::WorkerKeyChallengeResponse {
            payload: Some(payload),
            attestation,
        })
    }

    fn receive_worker_key(&mut self, request: pb::GetWorkerKeyResponse) -> RpcResult<bool> {
        let phactory = self.lock_phactory();

        let encrypted_key = request.decode_encrypted_key().map_err(from_display)?;
        if encrypted_key.is_none() {
            info!("Receive empty key handover");
            return Ok(false);
        }

        let encrypted_key = encrypted_key.expect("checked; qed");
        let my_ecdh_key = phactory
            .tmp_ecdh_key
            .as_ref()
            .ok_or_else(|| from_display("Ecdh key not initialized"))?;
        // TODO.shelven: fix DoS here
        let secret = key_share::decrypt_key_from(
            my_ecdh_key,
            &encrypted_key.ecdh_pubkey.0,
            &encrypted_key.encrypted_key,
            &encrypted_key.iv,
        )
        .map_err(from_debug)?;

        // only seal if the key is successfully updated
        let genesis_block_hash = request.decode_genesis_block_hash().map_err(from_display)?;
        phactory
            .save_runtime_data(genesis_block_hash, secret, phactory.dev_mode)
            .map_err(from_display)?;

        Ok(true)
    }
}
