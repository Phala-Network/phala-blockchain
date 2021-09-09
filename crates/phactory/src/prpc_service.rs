use crate::system::System;

use super::*;
use pb::{
    phactory_api_server::{PhactoryApi, PhactoryApiServer},
    server::Error as RpcError,
};
use phactory_api::{blocks, crypto, prpc as pb};
use phala_types::{contract, WorkerPublicKey};

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

impl<Platform: pal::Platform> Phactory<Platform> {
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
        let initialized = self.runtime_info.is_some();
        let state = self.runtime_state.as_ref();
        let genesis_block_hash = state.map(|state| hex::encode(&state.genesis_block_hash));
        let public_key = state.map(|state| hex::encode(state.identity_key.public()));
        let ecdh_public_key = state.map(|state| hex::encode(&state.ecdh_key.public()));
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

        let (registered, gatekeeper_status) = {
            match self.system.as_ref() {
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
        blocks: Vec<blocks::BlockHeaderWithChanges>,
    ) -> RpcResult<pb::SyncedTo> {
        info!(
            "dispatch_block from={:?} to={:?}",
            blocks.first().map(|h| h.block_header.number),
            blocks.last().map(|h| h.block_header.number)
        );

        let mut last_block = 0;
        for block in blocks.into_iter() {
            let state = self.runtime_state()?;
            state
                .storage_synchronizer
                .feed_block(&block, &mut state.chain_storage)
                .map_err(from_display)?;

            state.purge_mq();
            self.handle_inbound_messages(block.block_header.number)?;
            last_block = block.block_header.number;
        }

        Ok(pb::SyncedTo {
            synced_to: last_block,
        })
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
        if self.runtime_info.is_some() {
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
                .map_err(from_display)?
        } else {
            self.init_runtime_data(genesis_block_hash, None)
                .map_err(from_display)?
        };
        self.dev_mode = rt_data.dev_mode;
        self.skip_ra = skip_ra;

        if !skip_ra && self.dev_mode {
            return Err(from_display(
                "RA is disallowed when debug_set_key is enabled",
            ));
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
            Box::new(ParachainSynchronizer::new(
                light_client,
                main_bridge,
                next_headernum,
            )) as _
        } else {
            Box::new(SolochainSynchronizer::new(light_client, main_bridge)) as _
        };

        let id_pair = identity_key.clone();
        let send_mq = MessageSendQueue::default();
        let mut recv_mq = MessageDispatcher::default();

        let mut contracts: BTreeMap<ContractId, Box<dyn contracts::Contract + Send>> =
            Default::default();

        if self.dev_mode {
            // Install contracts when running in dev_mode.

            macro_rules! install_contract {
                ($id: expr, $inner: expr) => {{
                    let contract_id = contract::id256($id);
                    let sender = MessageOrigin::native_contract($id);
                    let mq = send_mq.channel(sender, id_pair.clone());
                    // TODO.kevin: use real contract key
                    let contract_key = ecdh_key.clone();
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
                        ecdh_key.clone(),
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
            // TODO.kevin: This is temporaryly disabled due to the dependency on CPUID which is not allowed in SGX.
            // install_contract!(
            //     contracts::WEB3_ANALYTICS,
            //     contracts::web3analytics::Web3Analytics::new()
            // );
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
            genesis_block_hash,
            identity_key,
            ecdh_key,
        };

        // Initialize other states
        runtime_state.chain_storage.load(genesis_state.into_iter());

        info!(
            "Genesis state loaded: {:?}",
            runtime_state.chain_storage.root()
        );

        let system = system::System::new(
            self.platform.clone(),
            self.sealing_path.clone(),
            &id_pair,
            &runtime_state.send_mq,
            &mut runtime_state.recv_mq,
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

    fn get_runtime_info(&mut self) -> RpcResult<pb::InitRuntimeResponse> {
        let skip_ra = self.skip_ra;

        let mut cached_resp = self
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
    ) -> RpcResult<pb::ContractQueryResponse> {
        // Validate signature
        if let Some(origin) = &request.signature {
            if !origin.verify(&request.encoded_encrypted_data) {
                return Err(from_display("Verifying signature failed"));
            }
            info!("Verifying signature passed!");
        }
        let ecdh_key = self.runtime_state()?.ecdh_key.clone();

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
            let system = self.system()?;
            let response = system.handle_query(ref_origin, types::deopaque_query(data_cursor)?);
            response.encode()
        } else {
            let state = self.runtime_state()?;
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

    #[allow(unused_unsafe)]
    pub unsafe fn dispatch_prpc_request(
        &mut self,
        path: *const u8,
        path_len: usize,
        data: *const u8,
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
        info!("Dispatching request: {}", path);

        let data = unsafe { std::slice::from_raw_parts(data, data_len) };
        let mut server = PhactoryApiServer::new(RpcService {
            output_buf_len,
            phactory: self,
        });
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
                            info!(
                                "mq dispatching message: sender={:?} dest={:?} payload={:?}",
                                $msg.sender, $msg.destination, event
                            );
                        }
                        Err(_) => {
                            info!("mq dispatching message (decode failed): {:?}", $msg);
                        }
                    }
                }};
            }
            // TODO.kevin: reuse codes in debug-cli
            if message.destination.path() == &SystemEvent::topic() {
                log_message!(message, SystemEvent);
            } else {
                info!("mq dispatching message: {:?}", message);
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

        let storage = &state.chain_storage;
        let recv_mq = &mut *guard;
        let mut block = BlockInfo {
            block_number,
            now_ms,
            storage,
            recv_mq,
        };

        if let Err(e) = system.process_messages(&mut block) {
            error!("System process events failed: {:?}", e);
            return Err(from_display("System process events failed"));
        }

        let mut env = ExecuteEnv { block: &block };

        for contract in state.contracts.values_mut() {
            contract.process_messages(&mut env);
        }

        Ok(())
    }
}

pub struct RpcService<'a, Platform> {
    output_buf_len: usize,
    phactory: &'a mut Phactory<Platform>,
}

/// A server that process all RPCs.
impl<Platform: pal::Platform> PhactoryApi for RpcService<'_, Platform> {
    /// Get basic information about Phactory state.
    fn get_info(&mut self, _request: ()) -> RpcResult<pb::PhactoryInfo> {
        Ok(self.phactory.get_info())
    }

    /// Sync the parent chain header
    fn sync_header(&mut self, request: pb::HeadersToSync) -> RpcResult<pb::SyncedTo> {
        let headers = request.decode_headers()?;
        let authority_set_change = request.decode_authority_set_change()?;
        self.phactory.sync_header(headers, authority_set_change)
    }

    /// Sync the parachain header
    fn sync_para_header(&mut self, request: pb::ParaHeadersToSync) -> RpcResult<pb::SyncedTo> {
        let headers = request.decode_headers()?;
        self.phactory.sync_para_header(headers, request.proof)
    }

    fn sync_combined_headers(
        &mut self,
        request: pb::CombinedHeadersToSync,
    ) -> Result<pb::HeadersSyncedTo, prpc::server::Error> {
        self.phactory.sync_combined_headers(
            request.decode_relaychain_headers()?,
            request.decode_authority_set_change()?,
            request.decode_parachain_headers()?,
            request.proof,
        )
    }

    /// Dispatch blocks (Sync storage changes)"
    fn dispatch_blocks(&mut self, request: pb::Blocks) -> RpcResult<pb::SyncedTo> {
        let blocks = request.decode_blocks()?;
        self.phactory.dispatch_block(blocks)
    }

    fn init_runtime(
        &mut self,
        request: pb::InitRuntimeRequest,
    ) -> RpcResult<pb::InitRuntimeResponse> {
        self.phactory.init_runtime(
            request.skip_ra,
            request.is_parachain,
            request.decode_genesis_info()?,
            request.decode_genesis_state()?,
            request.decode_operator()?,
            request.debug_set_key,
        )
    }

    fn get_runtime_info(&mut self, _: ()) -> RpcResult<pb::InitRuntimeResponse> {
        self.phactory.get_runtime_info()
    }

    fn get_egress_messages(&mut self, _: ()) -> RpcResult<pb::GetEgressMessagesResponse> {
        // The ENCLAVE OUTPUT BUFFER is a fixed size big buffer.
        assert!(self.output_buf_len >= 1024);
        self.phactory
            .get_egress_messages(self.output_buf_len - 1024)
            .map(pb::GetEgressMessagesResponse::new)
    }

    fn contract_query(
        &mut self,
        request: pb::ContractQueryRequest,
    ) -> RpcResult<pb::ContractQueryResponse> {
        self.phactory.contract_query(request)
    }

    fn get_worker_state(
        &mut self,
        request: pb::GetWorkerStateRequest,
    ) -> RpcResult<pb::WorkerState> {
        let system = self.phactory.system()?;
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
            .worker_state(&pubkey)
            .ok_or_else(|| from_display("Worker not found"))?;
        Ok(state)
    }
}
