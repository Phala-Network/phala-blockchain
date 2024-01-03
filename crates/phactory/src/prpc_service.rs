use std::borrow::Cow;
use std::future::Future;
use std::io::Read;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::benchmark::Flags;
use crate::system::{System, MAX_SUPPORTED_CONSENSUS_VERSION};
use crate::types::BaseBlockInfo;
use crate::{hex, try_decode_hex};

use super::*;
use ::pink::types::{AccountId, ExecSideEffects, ExecutionMode};
use parity_scale_codec::Encode;
use pb::{
    phactory_api_server::{PhactoryApi, PhactoryApiServer},
    server::Error as RpcError,
};
use phactory_api::blocks::StorageState;
use phactory_api::{blocks, crypto, endpoints::EndpointType, prpc as pb};
use phala_crypto::ecdh::{self, EcdhPublicKey};
use phala_crypto::{
    key_share,
    sr25519::{Persistence, KDF},
};
use phala_mq::MessageOrigin;
use phala_pallets::utils::attestation::{validate as validate_attestation_report, IasFields};
use phala_types::contract::contract_id_preimage;
use phala_types::{
    contract, messaging::EncryptedKey, wrap_content_to_sign, AttestationReport,
    ChallengeHandlerInfo, EncryptedWorkerKey, HandoverChallenge, SignedContentType,
    VersionedWorkerEndpoints, WorkerEndpointPayload, WorkerPublicKey, WorkerRegistrationInfoV2,
};
use sp_application_crypto::UncheckedFrom;
use sp_core::hashing::blake2_256;
use tracing::{error, info};

type RpcResult<T> = Result<T, RpcError>;

fn from_display(e: impl core::fmt::Display) -> RpcError {
    RpcError::AppError(e.to_string())
}

fn from_debug(e: impl core::fmt::Debug) -> RpcError {
    RpcError::AppError(format!("{e:?}"))
}

fn now() -> u64 {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    now.as_secs()
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

    pub(crate) fn current_block(&mut self) -> RpcResult<(BlockNumber, u64)> {
        let now_ms = self.runtime_state()?.chain_storage.timestamp_now();
        let block = self
            .runtime_state()?
            .storage_synchronizer
            .counters()
            .next_block_number
            .saturating_sub(1);
        Ok((block, now_ms))
    }

    pub fn get_info(&self) -> pb::PhactoryInfo {
        let initialized = self.runtime_state.is_some();
        let state = self.runtime_state.as_ref();
        let genesis_block_hash = state.map(|state| hex::encode(state.genesis_block_hash));
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

        let system_info = self.system.as_ref().map(|s| s.get_info());
        let score = benchmark::score();

        // Deprecated fields
        let registered;
        let public_key;
        let ecdh_public_key;
        let gatekeeper;
        match &system_info {
            None => {
                registered = false;
                public_key = None;
                ecdh_public_key = None;
                gatekeeper = Some(pb::GatekeeperStatus {
                    role: pb::GatekeeperRole::None.into(),
                    master_public_key: Default::default(),
                });
            }
            Some(sys) => {
                registered = sys.registered;
                public_key = Some(sys.public_key.clone());
                ecdh_public_key = Some(sys.ecdh_public_key.clone());
                gatekeeper = sys.gatekeeper.clone();
            }
        };

        let current_block_time = match self.args.safe_mode_level {
            0 => self
                .system
                .as_ref()
                .map(|sys| sys.now_ms)
                .unwrap_or_default(),
            1 => self
                .runtime_state
                .as_ref()
                .map(|state| state.chain_storage.timestamp_now())
                .unwrap_or_default(),
            _ => 0,
        };

        pb::PhactoryInfo {
            initialized,
            registered,
            public_key,
            ecdh_public_key,
            gatekeeper,
            genesis_block_hash,
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
            memory_usage: Some(self.platform.memory_usage()),
            system: system_info,
            can_load_chain_state: self.can_load_chain_state,
            safe_mode_level: self.args.safe_mode_level as _,
            current_block_time,
            max_supported_pink_runtime_version: {
                let (major, minor) = ::pink::runtimes::max_supported_version();
                format!("{major}.{minor}")
            },
            supported_attestation_methods: self.platform.supported_attestation_methods(),
            live_sidevm_instances: sidevm::vm_count() as u32,
        }
    }

    pub(crate) fn sync_header(
        &mut self,
        headers: Vec<blocks::HeaderToSync>,
        authority_set_change: Option<blocks::AuthoritySetChange>,
    ) -> RpcResult<pb::SyncedTo> {
        info!(
            range=?(
                headers.first().map(|h| h.header.number),
                headers.last().map(|h| h.header.number)
            ),
            auth_chg=authority_set_change.is_some(),
            "sync_header",
        );
        self.can_load_chain_state = false;
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
            range=?(
                headers.first().map(|h| h.number),
                headers.last().map(|h| h.number)
            ),
            "sync_para_header",
        );

        let state = self.runtime_state()?;

        let storage_key = light_validation::utils::storage_map_prefix_twox_64_concat(
            b"Paras",
            b"Heads",
            &state.para_id,
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

    pub(crate) fn dispatch_blocks(
        &mut self,
        req_id: u64,
        mut blocks: Vec<blocks::BlockHeaderWithChanges>,
    ) -> RpcResult<pb::SyncedTo> {
        info!(
            range=?(
                blocks.first().map(|h| h.block_header.number),
                blocks.last().map(|h| h.block_header.number)
            ),
            "dispatch_block",
        );
        let counters = self.runtime_state()?.storage_synchronizer.counters();
        blocks.retain(|b| b.block_header.number >= counters.next_block_number);

        let last_block = blocks
            .last()
            .map(|b| b.block_header.number)
            .unwrap_or(counters.next_block_number - 1);

        let safe_mode_level = self.args.safe_mode_level;

        for block in blocks.into_iter() {
            info!(block = block.block_header.number, "Dispatching");
            let state = self.runtime_state()?;
            let drop_proofs = safe_mode_level > 1;
            state
                .storage_synchronizer
                .feed_block(&block, state.chain_storage.inner_mut(), drop_proofs)
                .map_err(from_display)?;
            if safe_mode_level > 0 {
                continue;
            }
            info!("State synced");
            state.purge_mq();
            let now_ms = state.chain_storage.timestamp_now();
            let chain_storage = state.chain_storage.snapshot();
            let block_number = block.block_header.number;
            let contracts = self.system()?.contracts.clone();
            let mut context = contracts::pink::context::ContractExecContext::new(
                ExecutionMode::Transaction,
                now_ms,
                block_number,
                self.system()?.identity_key.clone(),
                chain_storage,
                req_id,
                contracts,
                self.sidevm_spawner.event_tx(),
            );
            self.check_requirements();
            contracts::pink::context::using(&mut context, || {
                self.handle_inbound_messages(block_number)
            })?;

            self.maybe_apply_cluster_state();

            if let Err(e) = self.maybe_take_checkpoint() {
                error!("Failed to take checkpoint: {:?}", e);
            }
        }
        Ok(pb::SyncedTo {
            synced_to: last_block,
        })
    }

    fn maybe_take_checkpoint(&mut self) -> anyhow::Result<()> {
        if !self.args.enable_checkpoint {
            return Ok(());
        }
        if self.last_checkpoint.elapsed().as_secs() < self.args.checkpoint_interval {
            return Ok(());
        }
        self.take_checkpoint()?;
        Ok(())
    }

    fn maybe_apply_cluster_state(&mut self) {
        let (Some(system), Some(runtime_state)) = (&mut self.system, &mut self.runtime_state)
        else {
            // unreachable
            return;
        };
        let Some(cluster_state) = self.cluster_state_to_apply.take() else {
            return;
        };
        if system.gatekeeper.is_some() {
            info!("It is not allowed to run contracts on a gatekeeper, dropping the state");
            return;
        }
        if system.contract_cluster.is_some() {
            info!("An cluster has already been deployed on this worker, dropping the state");
            return;
        }
        if cluster_state.block_number != system.block_number {
            info!(
                target_block = cluster_state.block_number,
                "Pending cluster state found, but not ready to apply"
            );
            self.cluster_state_to_apply = Some(cluster_state);
            return;
        }
        info!("Applying cluster state");
        system.contract_cluster = Some(cluster_state.cluster.into_owned());
        system.contracts = cluster_state.contracts.into_owned();

        for (sender, messages) in cluster_state.pending_messages {
            runtime_state.send_mq.load_state(&sender, messages);
        }
        if let Err(e) = self.take_checkpoint() {
            error!("Failed to take checkpoint: {:?}", e);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn init_runtime(
        &mut self,
        is_parachain: bool,
        genesis: blocks::GenesisBlockInfo,
        genesis_state: blocks::StorageState,
        operator: Option<chain::AccountId>,
        debug_set_key: ::core::option::Option<Vec<u8>>,
        attestation_provider: ::core::option::Option<AttestationProvider>,
    ) -> RpcResult<pb::InitRuntimeResponse> {
        if self.system.is_some() {
            return Err(from_display("Runtime already initialized"));
        }

        info!("Initializing runtime");
        info!("is_parachain  : {is_parachain}");
        info!("operator      : {operator:?}");
        info!("ra_provider   : {attestation_provider:?}");
        info!("debug_set_key : {debug_set_key:?}");

        // load chain genesis
        let genesis_block_hash = genesis.block_header.hash();
        let chain_storage = ChainStorage::from_pairs(genesis_state.into_iter());
        let para_id = chain_storage.para_id();
        info!(
            "Genesis state loaded: root={:?}, para_id={para_id}",
            chain_storage.root()
        );

        // load identity
        let rt_data = if let Some(raw_key) = debug_set_key {
            let priv_key = sr25519::Pair::from_seed_slice(&raw_key).map_err(from_debug)?;
            self.init_runtime_data(genesis_block_hash, para_id, Some(priv_key))
                .map_err(from_debug)?
        } else {
            self.init_runtime_data(genesis_block_hash, para_id, None)
                .map_err(from_debug)?
        };
        self.dev_mode = rt_data.dev_mode;
        self.trusted_sk = rt_data.trusted_sk;

        self.attestation_provider = attestation_provider;
        info!("attestation_provider: {:?}", self.attestation_provider);

        if self.dev_mode && self.attestation_provider.is_some() {
            return Err(from_display(
                "RA is disallowed when debug_set_key is enabled",
            ));
        }

        self.platform
            .quote_test(self.attestation_provider)
            .map_err(from_debug)?;

        let (identity_key, ecdh_key) = rt_data.decode_keys();

        let ecdsa_pk = identity_key.public();
        let ecdsa_hex_pk = hex::encode(ecdsa_pk);
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

        // Initialize bridge
        let next_headernum = genesis.block_header.number + 1;
        let mut light_client = LightValidation::new();
        let main_bridge = light_client
            .initialize_bridge(
                genesis.block_header.clone(),
                genesis.authority_set,
                genesis.proof,
            )
            .expect("Bridge initialize failed");

        let storage_synchronizer = if is_parachain {
            Synchronizer::new_parachain(light_client, main_bridge, next_headernum)
        } else {
            Synchronizer::new_solochain(light_client, main_bridge)
        };

        let send_mq = MessageSendQueue::default();
        let recv_mq = MessageDispatcher::default();

        let mut runtime_state = RuntimeState {
            send_mq,
            recv_mq,
            storage_synchronizer,
            chain_storage,
            genesis_block_hash,
            para_id,
        };

        // In parachain mode the state root is stored in parachain header which isn't passed in here.
        // The storage root would be checked at the time each block being synced in(where the storage
        // being patched) and pRuntime doesn't read any data from the chain storage until the first
        // block being synced in. So it's OK to skip the check here.
        if !is_parachain && *runtime_state.chain_storage.root() != genesis.block_header.state_root {
            error!(
                "Genesis state root mismatch, required in header: {:?}, actual: {:?}",
                genesis.block_header.state_root,
                runtime_state.chain_storage.root(),
            );
            return Err(from_display("state root mismatch"));
        }

        let system = system::System::new(
            self.platform.clone(),
            self.dev_mode,
            self.args.sealing_path.clone(),
            self.args.storage_path.clone(),
            identity_key,
            ecdh_key,
            &runtime_state.send_mq,
            &mut runtime_state.recv_mq,
        );

        // Build WorkerRegistrationInfoV2
        let runtime_info = WorkerRegistrationInfoV2::<chain::AccountId> {
            version: Self::compat_app_version(),
            machine_id: self.machine_id.clone(),
            pubkey: ecdsa_pk,
            ecdh_pubkey,
            genesis_block_hash,
            features: vec![cpu_core_num, cpu_feature_level],
            operator,
            para_id,
            max_consensus_version: MAX_SUPPORTED_CONSENSUS_VERSION,
        };

        let resp = pb::InitRuntimeResponse::new(
            runtime_info,
            genesis_block_hash,
            ecdsa_pk,
            ecdh_pubkey,
            None,
        );
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
        // The IdentityKey is considered valid in two situations:
        //
        // 1. It's generated by pRuntime thus is safe;
        // 2. It's handovered, but we find out that it was successfully registered as a worker on-chain;
        let validated_identity_key = self.trusted_sk || self.system()?.registered();
        let validated_state = self.runtime_state()?.storage_synchronizer.state_validated();

        let reset_operator = operator.is_some();
        if reset_operator {
            self.update_runtime_info(move |info| {
                info.operator = operator;
            });
        }

        let cached_resp = self
            .runtime_info
            .as_mut()
            .ok_or_else(|| from_display("Uninitiated runtime info"))?;

        if let Some(cached_attestation) = &cached_resp.attestation {
            const MAX_ATTESTATION_AGE: u64 = 60 * 60;
            if refresh_ra
                || reset_operator
                || now() > cached_attestation.timestamp + MAX_ATTESTATION_AGE
            {
                cached_resp.attestation = None;
            }
        }

        let allow_attestation =
            validated_state && (validated_identity_key || self.attestation_provider.is_none());
        info!("validated_identity_key :{validated_identity_key}");
        info!("validated_state        :{validated_state}");
        info!("refresh_ra             :{refresh_ra}");
        info!("reset_operator         :{reset_operator}");
        info!("attestation_provider   :{:?}", self.attestation_provider);
        info!("allow_attestation      :{allow_attestation}");
        // Never generate RA report for a potentially injected identity key
        // else he is able to fake a Secure Worker
        if allow_attestation && cached_resp.attestation.is_none() {
            // We hash the encoded bytes directly
            let runtime_info_hash = blake2_256(&cached_resp.encoded_runtime_info);
            info!("Encoded runtime info");
            info!("{:?}", hex::encode(&cached_resp.encoded_runtime_info));

            let report = create_attestation_report_on(
                &self.platform,
                self.attestation_provider,
                &runtime_info_hash,
                self.args.ra_timeout,
                self.args.ra_max_retries,
            )?;
            cached_resp.attestation = Some(report);
        }

        Ok(cached_resp.clone())
    }

    fn get_egress_messages(&mut self) -> RpcResult<pb::EgressMessages> {
        let messages: Vec<_> = self
            .runtime_state
            .as_ref()
            .map(|state| state.send_mq.all_messages_grouped().into_iter().collect())
            .unwrap_or_default();
        Ok(messages)
    }

    fn contract_query(
        &mut self,
        req_id: u64,
        request: pb::ContractQueryRequest,
    ) -> RpcResult<
        impl Future<Output = RpcResult<(pb::ContractQueryResponse, Option<ExecSideEffects>)>>,
    > {
        // Validate signature
        let origin = if let Some(sig) = &request.signature {
            let current_block = self.get_info().blocknum - 1;
            // At most two level cert chain supported
            match sig.verify(
                &request.encoded_encrypted_data,
                crypto::MessageType::ContractQuery,
                current_block,
                2,
            ) {
                Ok(key_chain) => match &key_chain[..] {
                    [root_pubkey, ..] => Some(root_pubkey.clone()),
                    _ => {
                        return Err(from_display("BUG: verify ok but no key?"));
                    }
                },
                Err(err) => {
                    return Err(from_display(format!("Verifying signature failed: {err:?}")));
                }
            }
        } else {
            info!("No query signature");
            None
        };

        let ecdh_key = self.system()?.ecdh_key.clone();

        // Decrypt data
        let encrypted_req = request.decode_encrypted_data()?;
        let data = encrypted_req.decrypt(&ecdh_key).map_err(from_debug)?;

        // Decode head
        let mut data_cursor = &data[..];
        let head = contract::ContractQueryHead::decode(&mut data_cursor)?;
        let rest = data_cursor.len();

        let query_scheduler = self.query_scheduler.clone();
        // Dispatch
        let query_future = self
            .system
            .as_mut()
            .expect("system always exists here")
            .make_query(
                req_id,
                &AccountId::unchecked_from(head.id),
                origin.as_ref(),
                data[data.len() - rest..].to_vec(),
                query_scheduler,
                &self
                    .runtime_state
                    .as_ref()
                    .expect("runtime state always exists here")
                    .chain_storage,
                self.sidevm_spawner.event_tx(),
            )?;

        Ok(async move {
            let (_, response, effects) = query_future.await?;
            let response = contract::ContractQueryResponse {
                nonce: head.nonce,
                result: contract::Data(response.encode()),
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

            Ok((pb::ContractQueryResponse::new(encrypted_resp), effects))
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
        let messages = state.chain_storage.mq_messages();

        state.recv_mq.reset_local_index();

        let now_ms = state.chain_storage.timestamp_now();

        let mut block = BlockInfo {
            base: BaseBlockInfo {
                block_number,
                now_ms,
                storage: &state.chain_storage,
                send_mq: &state.send_mq,
                recv_mq: &mut state.recv_mq,
            },
            sidevm_spawner: &self.sidevm_spawner,
        };

        system.will_process_block(&mut block);

        for message in messages {
            use phala_types::messaging::SystemEvent;
            macro_rules! log_message {
                ($msg: expr, $t: ident) => {{
                    let event: Result<$t, _> =
                        parity_scale_codec::Decode::decode(&mut &$msg.payload[..]);
                    match event {
                        Ok(event) => {
                            debug!(target: "phala_mq",
                                "mq dispatching message: sender={} dest={:?} payload={:?}",
                                $msg.sender, $msg.destination, event
                            );
                        }
                        Err(_) => {
                            debug!(target: "phala_mq", "mq dispatching message (decode failed): {:?}", $msg);
                        }
                    }
                }};
            }
            // TODO.kevin: reuse codes in debug-cli
            if message.destination.path() == &SystemEvent::topic() {
                log_message!(message, SystemEvent);
            } else {
                debug!(target: "phala_mq",
                    "mq dispatching message: sender={}, dest={:?}",
                    message.sender, message.destination
                );
            }
            block.recv_mq.dispatch(message);

            system.process_messages(&mut block);
        }
        system.did_process_block(&mut block);

        let n_unhandled = block.recv_mq.clear();
        if n_unhandled > 0 {
            warn!("There are {} unhandled messages dropped", n_unhandled);
        }

        let block_time = now_ms / 1000;
        let sys_time = now();

        // When delta time reaches 3600s, there are about 3600 / 12 = 300 blocks rest.
        // It need about 30 more seconds to sync up to date.
        let syncing = block_time + 3600 <= sys_time;
        debug!(
            "block_time={}, sys_time={}, syncing={}",
            block_time, sys_time, syncing
        );
        benchmark::set_flag(Flags::SYNCING, syncing);

        Ok(())
    }

    fn add_endpoint(
        &mut self,
        endpoint_type: EndpointType,
        endpoint: String,
    ) -> RpcResult<pb::GetEndpointResponse> {
        self.endpoints.insert(endpoint_type, endpoint);
        self.sign_endpoints()
    }

    fn sign_endpoints(&mut self) -> RpcResult<pb::GetEndpointResponse> {
        let system = self.system()?;
        let block_time: u64 = system.now_ms;
        let public_key = system.identity_key.public();
        let versioned_endpoints =
            VersionedWorkerEndpoints::V1(self.endpoints.values().cloned().collect());
        let endpoint_payload = WorkerEndpointPayload {
            pubkey: public_key,
            versioned_endpoints,
            signing_time: block_time,
        };
        let signature = self.sign_endpoint_payload(&endpoint_payload)?;
        let resp = pb::GetEndpointResponse::new(Some(endpoint_payload.clone()), Some(signature));
        self.signed_endpoints = Some(resp.clone());
        Ok(resp)
    }

    fn get_endpoint_info(&mut self) -> RpcResult<pb::GetEndpointResponse> {
        if self.endpoints.is_empty() {
            info!("Endpoint not found");
            return Ok(pb::GetEndpointResponse::new(None, None));
        }
        match &self.signed_endpoints {
            Some(response) => Ok(response.clone()),
            None => self.sign_endpoints(),
        }
    }

    fn sign_endpoint_info(
        &mut self,
        versioned_endpoints: VersionedWorkerEndpoints,
    ) -> RpcResult<pb::GetEndpointResponse> {
        let system = self.system()?;
        let pubkey = system.identity_key.public();
        let signing_time = system.now_ms;
        let payload = WorkerEndpointPayload {
            pubkey,
            versioned_endpoints,
            signing_time,
        };
        let signature = self.sign_endpoint_payload(&payload)?;

        Ok(pb::GetEndpointResponse::new(Some(payload), Some(signature)))
    }

    fn sign_endpoint_payload(&mut self, payload: &WorkerEndpointPayload) -> RpcResult<Vec<u8>> {
        const MAX_PAYLOAD_SIZE: usize = 2048;
        let data_to_sign = payload.encode();
        if data_to_sign.len() > MAX_PAYLOAD_SIZE {
            return Err(from_display("Endpoints too large"));
        }
        let wrapped_data = wrap_content_to_sign(&data_to_sign, SignedContentType::EndpointInfo);
        let signature = self
            .system()?
            .identity_key
            .clone()
            .sign(&wrapped_data)
            .encode();
        Ok(signature)
    }

    pub fn get_contract_info(
        &mut self,
        contract_ids: &[String],
    ) -> RpcResult<pb::GetContractInfoResponse> {
        // TODO: use `let else`
        let system = match &self.system {
            None => return Ok(Default::default()),
            Some(system) => system,
        };
        let cluster = system
            .contract_cluster
            .as_ref()
            .ok_or_else(|| from_display("No cluster found"))?;
        let contracts = if contract_ids.is_empty() {
            // TODO.kevin: reject to query all for performance reason
            system
                .contracts
                .iter()
                .map(|(_, contract)| contract.info(cluster))
                .collect()
        } else {
            let mut contracts = vec![];
            for id in contract_ids.iter() {
                let raw: [u8; 32] = try_decode_hex(id)
                    .map_err(|_| from_display("Invalid contract id"))?
                    .try_into()
                    .map_err(|_| from_display("Invalid contract id"))?;
                let contract = system.contracts.get(&raw.into());
                // TODO: use `let else`.
                let contract = match contract {
                    None => continue,
                    Some(contract) => contract,
                };
                contracts.push(contract.info(cluster));
            }
            contracts
        };
        Ok(pb::GetContractInfoResponse { contracts })
    }

    pub fn get_cluster_info(&self) -> RpcResult<pb::GetClusterInfoResponse> {
        let Some(System {
            contract_cluster: Some(cluster),
            contracts,
            ..
        }) = &self.system
        else {
            return Ok(Default::default());
        };
        let ver = cluster.config.runtime_version;
        let runtime_version = format!("{}.{}", ver.0, ver.1);

        Ok(pb::GetClusterInfoResponse {
            info: Some(pb::ClusterInfo {
                id: hex(cluster.id),
                state_root: cluster.storage.root().map(hex).unwrap_or_default(),
                runtime_version,
                number_of_contracts: contracts.len() as _,
                system_contract: cluster.system_contract().map(hex).unwrap_or_default(),
                logger_contract: cluster
                    .config
                    .log_handler
                    .as_ref()
                    .map(hex)
                    .unwrap_or_default(),
                js_runtime: cluster
                    .config
                    .js_runtime
                    .as_ref()
                    .map(hex)
                    .unwrap_or_default(),
            }),
        })
    }

    pub fn upload_sidevm_code(&mut self, contract_id: AccountId, code: Vec<u8>) -> RpcResult<()> {
        let spawner = self.sidevm_spawner.clone();
        self.system()?
            .upload_sidevm_code(contract_id, code, &spawner)
            .map_err(from_debug)
    }

    pub fn load_chain_state(
        &mut self,
        block: chain::BlockNumber,
        storage: StorageState,
    ) -> anyhow::Result<()> {
        if !self.can_load_chain_state {
            anyhow::bail!("Can not load chain state");
        }
        if block == 0 {
            anyhow::bail!("Can not load chain state from block 0");
        }
        let Some(system) = &mut self.system else {
            anyhow::bail!("System is uninitialized");
        };
        let Some(state) = &mut self.runtime_state else {
            anyhow::bail!("Runtime is uninitialized");
        };
        let chain_storage = ChainStorage::from_pairs(storage.into_iter());
        let para_id = chain_storage.para_id();
        if para_id != state.para_id {
            anyhow::bail!(
                "Loading different parachain state, loading={para_id}, init={}",
                state.para_id
            );
        }
        if chain_storage.is_worker_registered(&system.identity_key.public()) {
            anyhow::bail!(
                "Failed to load state: the worker is already registered at block {block}",
            );
        }
        state
            .storage_synchronizer
            .assume_at_block(block)
            .context("Failed to set synchronizer state")?;
        state.chain_storage = chain_storage;
        system.genesis_block = block;
        self.can_load_chain_state = false;
        Ok(())
    }

    pub fn stop(&self, remove_checkpoints: bool) -> RpcResult<()> {
        info!("Requested to stop remove_checkpoints={remove_checkpoints}");
        if remove_checkpoints {
            crate::maybe_remove_checkpoints(&self.args.storage_path);
        }
        std::process::abort()
    }

    fn load_storage_proof(&mut self, proof: Vec<Vec<u8>>) -> RpcResult<()> {
        if self.args.safe_mode_level < 2 {
            return Err(from_display(
                "Can not load storage proof when safe_mode_level < 2",
            ));
        }
        self.runtime_state()?
            .chain_storage
            .inner_mut()
            .load_proof(proof);
        Ok(())
    }

    fn save_cluster_state(
        &self,
        min_block_number: BlockNumber,
        receiver: &[u8],
    ) -> RpcResult<pb::SaveClusterStateResponse> {
        let Ok(receiver) = receiver.try_into() else {
            return Err(from_display("Invalid receiver"));
        };
        let receiver = WorkerPublicKey(receiver);
        let Some(system) = &self.system else {
            return Err(from_display("System is uninitialized"));
        };
        let Some(runtime_state) = &self.runtime_state else {
            return Err(from_display("Runtime is uninitialized"));
        };
        let Some(cluster) = &system.contract_cluster else {
            return Err(from_display("No cluster found"));
        };
        let block_number = system.block_number;
        if block_number < min_block_number {
            return Err(from_display(format!(
                "Can not save cluster state at block {block_number} which earlier than \
                 min={min_block_number}",
            )));
        }
        if min_block_number + 128 < block_number {
            return Err(from_display(format!(
                "The destination worker state is too old, \
                min_block_number={min_block_number}, \
                current_block_number={block_number}",
            )));
        }
        let cluster_of_the_worker = runtime_state
            .chain_storage
            .get_worker_cluster(&receiver)
            .ok_or(from_display(
                "The destination worker is not attached to any cluster",
            ))?;
        if cluster_of_the_worker != cluster.id {
            return Err(from_display(format!(
                "The destination worker is attached to another cluster, its cluster={}, our_cluster={}",
                hex_fmt::HexFmt(&cluster_of_the_worker),
                hex_fmt::HexFmt(&cluster.id),
            )));
        }
        info!(receiver=%hex_fmt::HexFmt(&receiver), "Saving cluster state");
        let mq_sender = MessageOrigin::Cluster(cluster.id);
        let messages = runtime_state
            .send_mq
            .dump_state(&mq_sender)
            .ok_or(from_display("Failed to dump send mq state"))?;
        let cluster_state = ClusterState {
            block_number,
            pending_messages: vec![(mq_sender, messages)],
            contracts: Cow::Borrowed(&system.contracts),
            cluster: Cow::Borrowed(cluster),
        };
        let filename = format!(
            "cluster-{}-{}-{}.bin",
            hex::encode(cluster.id),
            hex::encode(receiver),
            block_number,
        );
        let key = ecdh::agree(&system.ecdh_key, &receiver).or(Err(from_display(
            "Failed to derive ECDH key for cluster state",
        )))?;
        let key128 = derive_key_for_cluster_state(&key);
        let nonce = rand::thread_rng().gen();
        let dir = public_data_dir(&self.args.storage_path);
        // Create directory if it does not exist
        std::fs::create_dir_all(&dir).map_err(from_debug)?;
        let fullname = dir.join(&filename);
        let mut file = File::create(fullname).map_err(from_debug)?;
        file.write_all(&system.ecdh_key.public())
            .map_err(from_debug)?;

        let mut enc_writer = aead::stream::new_aes128gcm_writer(key128, nonce, file);
        enc_writer
            .write_all(&CHECKPOINT_VERSION.to_be_bytes())
            .map_err(from_debug)?;

        serde_cbor::to_writer(enc_writer, &cluster_state).map_err(from_debug)?;
        info!(%filename, "Cluster state saved");
        Ok(pb::SaveClusterStateResponse {
            block_number,
            filename,
        })
    }

    fn load_cluster_state(&mut self, filename: &str) -> RpcResult<()> {
        let Some(system) = &self.system else {
            return Err(from_display("System is uninitialized"));
        };
        if system.contract_cluster.is_some() {
            return Err(from_display("Already in cluster"));
        }
        let Some(runtime_state) = &mut self.runtime_state else {
            return Err(from_display("Runtime is uninitialized"));
        };

        let fullname = PathBuf::from(&self.args.storage_path).join(filename);
        let mut reader = File::open(fullname).map_err(from_debug)?;
        let mut pubkey: EcdhPublicKey = Default::default();
        reader.read_exact(&mut pubkey).map_err(from_debug)?;

        // Make sure the pubkey is not fake
        if !runtime_state
            .chain_storage
            .is_worker_registered(&WorkerPublicKey(pubkey))
        {
            return Err(from_display(format!(
                "The pubkey {} is not registered",
                hex_fmt::HexFmt(&pubkey)
            )));
        }

        let key = ecdh::agree(&system.ecdh_key, &pubkey).or(Err(from_display(
            "Failed to derive ECDH key for cluster state",
        )))?;
        let key128 = derive_key_for_cluster_state(&key);
        let mut dec_reader = aead::stream::new_aes128gcm_reader(key128, &mut reader);

        let mut version = [0u8; 4];
        dec_reader.read_exact(&mut version).map_err(from_debug)?;
        let version = u32::from_be_bytes(version);
        info!(version, "Loading cluster state");
        if version != CHECKPOINT_VERSION {
            return Err(from_display(format!(
                "Incompatible cluster state version {version}, expected {CHECKPOINT_VERSION}",
            )));
        }
        let recv_mq = &mut runtime_state.recv_mq;
        let send_mq = &mut runtime_state.send_mq;
        // The recv_mq and send_mq are used to restore the rx/tx handles during the deserilization.
        let cluster_state: ClusterState =
            phala_mq::checkpoint_helper::using_dispatcher(recv_mq, move || {
                phala_mq::checkpoint_helper::using_send_mq(send_mq, move || {
                    serde_cbor::from_reader(dec_reader)
                })
            })
            .map_err(from_debug)?;
        if cluster_state.block_number <= system.block_number {
            return Err(from_display(format!(
                "Staled cluster state, current_block_number={}, state_block_number={}",
                system.block_number, cluster_state.block_number,
            )));
        }
        self.cluster_state_to_apply = Some(cluster_state);
        Ok(())
    }

    fn get_worker_key_challenge(
        &mut self,
        block_number: chain::BlockNumber,
        now: u64,
    ) -> HandoverChallenge<chain::BlockNumber> {
        let sgx_target_info = if self.dev_mode {
            vec![]
        } else {
            let my_target_info = sgx_api_lite::target_info().unwrap();
            sgx_api_lite::encode(&my_target_info).to_vec()
        };
        let challenge = HandoverChallenge {
            sgx_target_info,
            block_number,
            now,
            dev_mode: self.dev_mode,
            nonce: crate::generate_random_info(),
        };
        self.handover_last_challenge = Some(challenge.clone());
        challenge
    }

    pub fn verify_worker_key_challenge(
        &mut self,
        challenge: &HandoverChallenge<chain::BlockNumber>,
    ) -> bool {
        return self.handover_last_challenge.take().as_ref() == Some(challenge);
    }
}

pub struct RpcService<Platform> {
    req_id: u64,
    pub(crate) phactory: Arc<Mutex<Phactory<Platform>>>,
}

impl<Platform: pal::Platform> RpcService<Platform> {
    pub fn new(platform: Platform, args: InitArgs) -> RpcService<Platform> {
        RpcService {
            phactory: Arc::new_cyclic(|weak_self| {
                Mutex::new(Phactory::new(platform, args, weak_self.clone()))
            }),
            req_id: 0,
        }
    }

    pub fn with_id(&self, req_id: u64) -> RpcService<Platform> {
        RpcService {
            phactory: self.phactory.clone(),
            req_id,
        }
    }
}

impl<P> RpcService<P> {
    pub fn weak_phactory(&self) -> Weak<Mutex<Phactory<P>>> {
        Arc::downgrade(&self.phactory)
    }
}

impl<Platform> RpcService<Platform>
where
    Platform: pal::Platform + Serialize + DeserializeOwned,
{
    pub fn dispatch_request(
        &self,
        req_id: u64,
        path: String,
        data: &[u8],
        json: bool,
    ) -> impl Future<Output = (u16, Vec<u8>)> {
        use prpc::server::{Error, ProtoError};
        let data = data.to_vec();

        let mut server = PhactoryApiServer::new(self.with_id(req_id));

        async move {
            info!("Dispatching request: {}", path);

            let result = if json {
                server.dispatch_json_request(&path, data).await
            } else {
                server.dispatch_request(&path, data).await
            };

            let (code, data) = match result {
                Ok(data) => (200, data),
                Err(err) => {
                    error!("Rpc error: {:?}", err);
                    let (code, err) = match err {
                        Error::NotFound => (404, ProtoError::new("Method Not Found")),
                        Error::DecodeError(err) => {
                            (400, ProtoError::new(format!("DecodeError({err:?})")))
                        }
                        Error::AppError(msg) => (500, ProtoError::new(msg)),
                        Error::ContractQueryError(msg) => (500, ProtoError::new(msg)),
                    };
                    if json {
                        let error = format!("{err:?}");
                        let body =
                            serde_json::to_string_pretty(&serde_json::json!({ "error": error }))
                                .unwrap_or_else(|_| {
                                    r#"{"error": "Failed to encode the error"}"#.to_string()
                                })
                                .into_bytes();
                        (code, body)
                    } else {
                        (code, prpc::codec::encode_message_to_vec(&err))
                    }
                }
            };
            (code, data)
        }
    }
}

pub struct LogOnDrop<T> {
    inner: T,
    msg: &'static str,
}

impl<T> core::ops::Deref for LogOnDrop<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> core::ops::DerefMut for LogOnDrop<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Drop for LogOnDrop<T> {
    fn drop(&mut self) {
        debug!(target: "phactory::lock", "{}", self.msg);
    }
}

impl<Platform: pal::Platform> RpcService<Platform> {
    pub fn lock_phactory(
        &self,
        allow_rcu: bool,
        allow_safemode: bool,
    ) -> RpcResult<LogOnDrop<MutexGuard<'_, Phactory<Platform>>>> {
        debug!(target: "phactory::lock", "Locking phactory...");
        let guard = self.phactory.lock().unwrap();
        debug!(target: "phactory::lock", "Locked phactory");
        if !allow_rcu && guard.rcu_dispatching {
            return Err(from_display(
                "RCU in progress, please try the request again later",
            ));
        }
        if !allow_safemode && guard.args.safe_mode_level > 0 {
            return Err(from_display("This RPC is disabled in safe mode"));
        }
        Ok(LogOnDrop {
            inner: guard,
            msg: "Unlocked phactory",
        })
    }
}

fn create_attestation_report_on<Platform: pal::Platform>(
    platform: &Platform,
    attestation_provider: Option<AttestationProvider>,
    data: &[u8],
    timeout: Duration,
    max_retries: u32,
) -> RpcResult<pb::Attestation> {
    let mut tried = 0;
    let encoded_report = loop {
        break match platform.create_attestation_report(attestation_provider, data, timeout) {
            Ok(r) => r,
            Err(e) => {
                let message = format!("Failed to create attestation report: {e:?}");
                error!("{}", message);
                if tried >= max_retries {
                    return Err(from_display(message));
                }
                let sleep_secs = (1 << tried).min(8);
                info!("Retrying after {} seconds...", sleep_secs);
                std::thread::sleep(Duration::from_secs(sleep_secs));
                tried += 1;
                continue;
            }
        };
    };
    Ok(pb::Attestation {
        version: 1,
        provider: serde_json::to_string(&attestation_provider).unwrap(),
        payload: None,
        encoded_report,
        timestamp: now(),
    })
}

#[async_trait::async_trait]
/// A server that process all RPCs.
impl<Platform: pal::Platform + Serialize + DeserializeOwned> PhactoryApi for RpcService<Platform> {
    /// Get basic information about Phactory state.
    async fn get_info(&mut self, _request: ()) -> RpcResult<pb::PhactoryInfo> {
        let info = self.lock_phactory(true, true)?.get_info();
        #[cfg(target_env = "gnu")]
        info!("Got info: {:?} mallinfo: {:?}", info.debug_info(), unsafe {
            libc::mallinfo()
        });
        #[cfg(not(target_env = "gnu"))]
        info!("Got info: {:?}", info.debug_info());
        Ok(info)
    }

    /// Sync the parent chain header
    async fn sync_header(&mut self, request: pb::HeadersToSync) -> RpcResult<pb::SyncedTo> {
        let headers = request.decode_headers()?;
        let authority_set_change = request.decode_authority_set_change()?;
        self.lock_phactory(false, true)?
            .sync_header(headers, authority_set_change)
    }

    /// Sync the parachain header
    async fn sync_para_header(
        &mut self,
        request: pb::ParaHeadersToSync,
    ) -> RpcResult<pb::SyncedTo> {
        let headers = request.decode_headers()?;
        self.lock_phactory(false, true)?
            .sync_para_header(headers, request.proof)
    }

    async fn sync_combined_headers(
        &mut self,
        request: pb::CombinedHeadersToSync,
    ) -> Result<pb::HeadersSyncedTo, prpc::server::Error> {
        self.lock_phactory(false, true)?.sync_combined_headers(
            request.decode_relaychain_headers()?,
            request.decode_authority_set_change()?,
            request.decode_parachain_headers()?,
            request.proof,
        )
    }

    /// Dispatch blocks (Sync storage changes)"
    async fn dispatch_blocks(&mut self, request: pb::Blocks) -> RpcResult<pb::SyncedTo> {
        let blocks = request.decode_blocks()?;
        let mut phactory = {
            let mut phactory = self.lock_phactory(false, true)?;
            if phactory.args.no_rcu || benchmark::syncing() {
                // If RCU way is not suitable here, we do the traditional locked dispatch.
                return phactory.dispatch_blocks(self.req_id, blocks);
            }
            // Otherwise, we clone the phactory and execute/dispatch the events of the blocks with
            // a cloned phactory without locking(the lock would be released in the end of this scope)
            // the singleton phactory. This way, we can avoid blocking the RPC server.
            info!("Cloning Phactory to do RCU dispatch...");
            let cloned = phactory.clone();
            // We set rcu_dispatching = true to avoid a reentrant call to dispatch_blocks which
            // would cause a state inconsistency.
            phactory.rcu_dispatching = true;
            cloned
        };
        // Start to dispatch the blocks with the cloned phactory without locking the singleton phactory.
        info!("Unlocked Phactory, dispatching blocks...");
        let req_id = self.req_id;
        let span = tracing::Span::current();
        let (synced_to, phactory) = tokio::task::spawn_blocking(move || {
            let _guard = span.enter();
            let synced_to = phactory.dispatch_blocks(req_id, blocks);
            (synced_to, phactory)
        })
        .await
        .expect("Dispatch blocks failed");
        info!("Done, writing state back to Phactory...");
        let mut guard = self.lock_phactory(true, true).unwrap();
        let pending_effects = std::mem::take(&mut guard.pending_effects);
        // While putting the cloned phactory back, the rcu_dispatching flag is also overwritten with
        // its original value `false`.
        **guard = phactory;
        if !pending_effects.is_empty() {
            // Apply pending effects with the singleton phactory locked.
            tracing::info!(count = pending_effects.len(), "Applying pending effects");
            for effects in pending_effects {
                guard.apply_side_effects(effects);
            }
        }
        synced_to
    }

    async fn init_runtime(
        &mut self,
        request: pb::InitRuntimeRequest,
    ) -> RpcResult<pb::InitRuntimeResponse> {
        self.lock_phactory(false, false)?.init_runtime(
            request.is_parachain,
            request.decode_genesis_info()?,
            request.decode_genesis_state()?,
            request.decode_operator()?,
            request.debug_set_key.clone(),
            request.decode_attestation_provider()?,
        )
    }

    async fn get_runtime_info(
        &mut self,
        req: pb::GetRuntimeInfoRequest,
    ) -> RpcResult<pb::InitRuntimeResponse> {
        self.lock_phactory(true, false)?
            .get_runtime_info(req.force_refresh_ra, req.decode_operator()?)
    }

    async fn get_egress_messages(&mut self, _: ()) -> RpcResult<pb::GetEgressMessagesResponse> {
        self.lock_phactory(true, false)?
            .get_egress_messages()
            .map(pb::GetEgressMessagesResponse::new)
    }

    async fn contract_query(
        &mut self,
        request: pb::ContractQueryRequest,
    ) -> RpcResult<pb::ContractQueryResponse> {
        let query_fut = self
            .lock_phactory(true, false)?
            .contract_query(self.req_id, request)?;
        let (response, effects) = query_fut.await?;
        'apply_effects: {
            let Some(effects) = effects else {
                break 'apply_effects;
            };
            if effects.is_empty() {
                break 'apply_effects;
            }
            self.lock_phactory(true, false)?.apply_side_effects(effects);
        }
        Ok(response)
    }

    async fn get_worker_state(
        &mut self,
        request: pb::GetWorkerStateRequest,
    ) -> RpcResult<pb::WorkerState> {
        let mut phactory = self.lock_phactory(true, false)?;
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
            .computing_economics
            .worker_state(&pubkey)
            .ok_or_else(|| from_display("Worker not found"))?;
        Ok(state)
    }

    async fn add_endpoint(
        &mut self,
        request: pb::AddEndpointRequest,
    ) -> RpcResult<pb::GetEndpointResponse> {
        self.lock_phactory(false, false)?
            .add_endpoint(request.decode_endpoint_type()?, request.endpoint)
    }

    async fn refresh_endpoint_signing_time(&mut self, _: ()) -> RpcResult<pb::GetEndpointResponse> {
        self.lock_phactory(false, false)?.sign_endpoints()
    }

    async fn get_endpoint_info(&mut self, _: ()) -> RpcResult<pb::GetEndpointResponse> {
        self.lock_phactory(true, false)?.get_endpoint_info()
    }

    async fn sign_endpoint_info(
        &mut self,
        request: pb::SignEndpointsRequest,
    ) -> Result<pb::GetEndpointResponse, prpc::server::Error> {
        self.lock_phactory(true, false)?
            .sign_endpoint_info(VersionedWorkerEndpoints::V1(request.decode_endpoints()?))
    }

    async fn derive_phala_i2p_key(&mut self, _: ()) -> RpcResult<pb::DerivePhalaI2pKeyResponse> {
        let mut phactory = self.lock_phactory(true, false)?;
        let system = phactory.system()?;
        let derive_key = system
            .identity_key
            .derive_sr25519_pair(&[b"PhalaI2PKey"])
            .expect("should not fail with valid key");
        Ok(pb::DerivePhalaI2pKeyResponse {
            phala_i2p_key: derive_key.dump_secret_key().to_vec(),
        })
    }

    async fn echo(&mut self, request: pb::EchoMessage) -> RpcResult<pb::EchoMessage> {
        let echo_msg = request.echo_msg;
        Ok(pb::EchoMessage { echo_msg })
    }

    // WorkerKey Handover Server
    async fn handover_create_challenge(
        &mut self,
        _request: (),
    ) -> RpcResult<pb::HandoverChallenge> {
        let mut phactory = self.lock_phactory(false, true)?;
        let (block, ts) = phactory.current_block()?;
        let challenge = phactory.get_worker_key_challenge(block, ts);
        Ok(pb::HandoverChallenge::new(challenge))
    }

    async fn handover_start(
        &mut self,
        request: pb::HandoverChallengeResponse,
    ) -> RpcResult<pb::HandoverWorkerKey> {
        let mut phactory = self.lock_phactory(false, true)?;
        let attestation_provider = phactory.attestation_provider;
        let dev_mode = phactory.dev_mode;
        let in_sgx = attestation_provider == Some(AttestationProvider::Ias);
        let (block_number, now_ms) = phactory.current_block()?;

        // 1. verify client RA report to ensure it's in sgx
        // this also ensure the message integrity
        let challenge_handler = request.decode_challenge_handler().map_err(from_display)?;
        let block_sec = now_ms / 1000;
        let attestation = if !dev_mode && in_sgx {
            let payload_hash = blake2_256(&challenge_handler.encode());
            let raw_attestation = request
                .attestation
                .ok_or_else(|| from_display("Client attestation not found"))?;
            let attn_to_validate =
                Option::<AttestationReport>::decode(&mut &raw_attestation.encoded_report[..])
                    .map_err(|_| from_display("Decode client attestation failed"))?;
            // The time from attestation report is generated by IAS, thus trusted. By default, it's valid for **10h**.
            // By ensuring our system timestamp is within the valid period, we know that this pRuntime is not hold back by
            // malicious workers.
            validate_attestation_report(
                attn_to_validate.clone(),
                &payload_hash,
                block_sec,
                false,
                vec![],
                false,
            )
            .map_err(|_| from_display("Invalid client RA report"))?;
            attn_to_validate
        } else {
            info!("Skip client RA report check in dev mode");
            None
        };
        // 2. verify challenge validity to prevent replay attack
        let challenge = challenge_handler.challenge;
        if !phactory.verify_worker_key_challenge(&challenge) {
            return Err(from_display("Invalid challenge"));
        }
        // 3. verify sgx local attestation report to ensure the handover pRuntimes are on the same machine
        if !dev_mode && in_sgx {
            let recv_local_report = unsafe {
                sgx_api_lite::decode(&challenge_handler.sgx_local_report)
                    .map_err(|_| from_display("Invalid client LA report"))?
            };
            sgx_api_lite::verify(recv_local_report)
                .map_err(|_| from_display("No remote handover"))?;
        } else {
            info!("Skip client LA report check in dev mode");
        }
        // 4. verify challenge block height and report timestamp
        // only challenge within 150 blocks (30 minutes) is accepted
        let challenge_height = challenge.block_number;
        if !(challenge_height <= block_number && block_number - challenge_height <= 150) {
            return Err(from_display("Outdated challenge"));
        }
        // 5. verify pruntime launch date, never handover to old pruntime
        if !dev_mode && in_sgx {
            let my_la_report = {
                // target_info and reportdata not important, we just need the report metadata
                let target_info =
                    sgx_api_lite::target_info().expect("should not fail in SGX; qed.");
                sgx_api_lite::report(&target_info, &[0; 64])
                    .map_err(|_| from_display("Cannot read server pRuntime info"))?
            };
            let my_runtime_hash = {
                let ias_fields = IasFields {
                    mr_enclave: my_la_report.body.mr_enclave.m,
                    mr_signer: my_la_report.body.mr_signer.m,
                    isv_prod_id: my_la_report.body.isv_prod_id.to_ne_bytes(),
                    isv_svn: my_la_report.body.isv_svn.to_ne_bytes(),
                    report_data: [0; 64],
                    confidence_level: 0,
                };
                ias_fields.extend_mrenclave()
            };
            let runtime_state = phactory.runtime_state()?;
            let my_runtime_timestamp = runtime_state
                .chain_storage
                .get_pruntime_added_at(&my_runtime_hash)
                .ok_or_else(|| from_display("Server pRuntime not allowed on chain"))?;

            let attestation =
                attestation.ok_or_else(|| from_display("Client attestation not found"))?;
            let runtime_hash = match attestation {
                AttestationReport::SgxIas {
                    ra_report,
                    signature: _,
                    raw_signing_cert: _,
                } => {
                    let (ias_fields, _) = IasFields::from_ias_report(&ra_report)
                        .map_err(|_| from_display("Invalid client RA report"))?;
                    ias_fields.extend_mrenclave()
                }
                AttestationReport::SgxDcap {
                    quote: _,
                    collateral: _,
                } => todo!(),
            };
            let req_runtime_timestamp = runtime_state
                .chain_storage
                .get_pruntime_added_at(&runtime_hash)
                .ok_or_else(|| from_display("Client pRuntime not allowed on chain"))?;

            if my_runtime_timestamp >= req_runtime_timestamp {
                return Err(from_display("No handover for old pRuntime"));
            }
        } else {
            info!("Skip pRuntime timestamp check in dev mode");
        }

        // Share the key with attestation
        let ecdh_pubkey = challenge_handler.ecdh_pubkey;
        let iv = crate::generate_random_iv();
        let runtime_data = phactory.persistent_runtime_data().map_err(from_display)?;
        let (my_identity_key, _) = runtime_data.decode_keys();
        let (ecdh_pubkey, encrypted_key) = key_share::encrypt_secret_to(
            &my_identity_key,
            &[b"worker_key_handover"],
            &ecdh_pubkey.0,
            &my_identity_key.dump_secret_key(),
            &iv,
        )
        .map_err(from_debug)?;
        let encrypted_key = EncryptedKey {
            ecdh_pubkey: sr25519::Public(ecdh_pubkey),
            encrypted_key,
            iv,
        };
        let runtime_state = phactory.runtime_state()?;
        let genesis_block_hash = runtime_state.genesis_block_hash;
        let encrypted_worker_key = EncryptedWorkerKey {
            genesis_block_hash,
            para_id: runtime_state.para_id,
            dev_mode,
            encrypted_key,
        };

        let worker_key_hash = blake2_256(&encrypted_worker_key.encode());
        let attestation = if !dev_mode && in_sgx {
            Some(create_attestation_report_on(
                &phactory.platform,
                attestation_provider,
                &worker_key_hash,
                phactory.args.ra_timeout,
                phactory.args.ra_max_retries,
            )?)
        } else {
            info!("Omit RA report in workerkey response in dev mode");
            None
        };

        Ok(pb::HandoverWorkerKey::new(
            encrypted_worker_key,
            attestation,
        ))
    }

    // WorkerKey Handover Client

    async fn handover_accept_challenge(
        &mut self,
        request: pb::HandoverChallenge,
    ) -> RpcResult<pb::HandoverChallengeResponse> {
        let mut phactory = self.lock_phactory(false, true)?;

        // generate and save tmp key only for key handover encryption
        let handover_key = crate::new_sr25519_key();
        let handover_ecdh_key = handover_key
            .derive_ecdh_key()
            .expect("should never fail with valid key; qed.");
        let ecdh_pubkey = phala_types::EcdhPublicKey(handover_ecdh_key.public());
        phactory.handover_ecdh_key = Some(handover_ecdh_key);

        let challenge = request.decode_challenge().map_err(from_display)?;
        let dev_mode = challenge.dev_mode;
        // generate local attestation report to ensure the handover pRuntimes are on the same machine
        let sgx_local_report = if !dev_mode {
            let its_target_info = unsafe {
                sgx_api_lite::decode(&challenge.sgx_target_info)
                    .map_err(|_| from_display("Invalid client sgx target info"))?
            };
            // the report data does not matter since we only care about the origin
            let report = sgx_api_lite::report(its_target_info, &[0; 64])
                .map_err(|_| from_display("Failed to create client LA report"))?;
            sgx_api_lite::encode(&report).to_vec()
        } else {
            info!("Omit client LA report for dev mode challenge");
            vec![]
        };

        let challenge_handler = ChallengeHandlerInfo {
            challenge,
            sgx_local_report,
            ecdh_pubkey,
        };
        let handler_hash = blake2_256(&challenge_handler.encode());
        let attestation = if !dev_mode {
            Some(create_attestation_report_on(
                &phactory.platform,
                Some(AttestationProvider::Ias),
                &handler_hash,
                phactory.args.ra_timeout,
                phactory.args.ra_max_retries,
            )?)
        } else {
            info!("Omit client RA report for dev mode challenge");
            None
        };

        Ok(pb::HandoverChallengeResponse::new(
            challenge_handler,
            attestation,
        ))
    }

    async fn handover_receive(&mut self, request: pb::HandoverWorkerKey) -> RpcResult<()> {
        let mut phactory = self.lock_phactory(false, true)?;
        let encrypted_worker_key = request.decode_worker_key().map_err(from_display)?;

        let dev_mode = encrypted_worker_key.dev_mode;
        // verify RA report
        if !dev_mode {
            let worker_key_hash = blake2_256(&encrypted_worker_key.encode());
            let raw_attestation = request
                .attestation
                .ok_or_else(|| from_display("Server attestation not found"))?;
            let attn_to_validate =
                Option::<AttestationReport>::decode(&mut &raw_attestation.encoded_report[..])
                    .map_err(|_| from_display("Decode server attestation failed"))?;
            validate_attestation_report(
                attn_to_validate,
                &worker_key_hash,
                now(),
                false,
                vec![],
                false,
            )
            .map_err(|_| from_display("Invalid server RA report"))?;
        } else {
            info!("Skip server RA report check for dev mode key");
        }

        let encrypted_key = encrypted_worker_key.encrypted_key;
        let my_ecdh_key = phactory
            .handover_ecdh_key
            .as_ref()
            .ok_or_else(|| from_display("Handover ecdhkey not initialized"))?;
        let secret = key_share::decrypt_secret_from(
            my_ecdh_key,
            &encrypted_key.ecdh_pubkey.0,
            &encrypted_key.encrypted_key,
            &encrypted_key.iv,
        )
        .map_err(from_debug)?;

        // only seal if the key is successfully updated
        phactory
            .save_runtime_data(
                encrypted_worker_key.genesis_block_hash,
                encrypted_worker_key.para_id,
                sr25519::Pair::restore_from_secret_key(&secret),
                false, // we are not sure whether this key is injected
                dev_mode,
            )
            .map_err(from_display)?;

        // clear cached RA report and handover ecdh key to prevent replay
        phactory.runtime_info = None;
        phactory.handover_ecdh_key = None;
        Ok(())
    }

    async fn config_network(
        &mut self,
        request: super::NetworkConfig,
    ) -> Result<(), prpc::server::Error> {
        self.lock_phactory(false, false)?.set_netconfig(request);
        Ok(())
    }

    async fn http_fetch(
        &mut self,
        request: pb::HttpRequest,
    ) -> Result<pb::HttpResponse, prpc::server::Error> {
        use reqwest::{
            header::{HeaderMap, HeaderName, HeaderValue},
            Method,
        };
        use reqwest_env_proxy::EnvProxyBuilder;

        let url: reqwest::Url = request.url.parse().map_err(from_debug)?;

        let client = reqwest::Client::builder()
            .trust_dns(true)
            .env_proxy(url.host_str().unwrap_or_default())
            .build()
            .map_err(from_debug)?;

        let method: Method = FromStr::from_str(&request.method)
            .or(Err("Invalid HTTP method"))
            .map_err(from_display)?;
        let mut headers = HeaderMap::new();
        for header in &request.headers {
            let name = HeaderName::from_str(&header.name)
                .or(Err("Invalid HTTP header key"))
                .map_err(from_display)?;
            let value = HeaderValue::from_str(&header.value)
                .or(Err("Invalid HTTP header value"))
                .map_err(from_display)?;
            headers.insert(name, value);
        }

        let response = client
            .request(method, url)
            .headers(headers)
            .body(request.body)
            .send()
            .await
            .map_err(from_debug)?;

        let headers: Vec<_> = response
            .headers()
            .iter()
            .map(|(name, value)| {
                let name = name.to_string();
                let value = value.to_str().unwrap_or_default().into();
                pb::HttpHeader { name, value }
            })
            .collect();

        let status_code = response.status().as_u16() as _;
        let body = response
            .bytes()
            .await
            .or(Err("Failed to copy response body"))
            .map_err(from_display)?
            .to_vec();

        Ok(pb::HttpResponse {
            status_code,
            body,
            headers,
        })
    }

    async fn get_contract_info(
        &mut self,
        request: pb::GetContractInfoRequest,
    ) -> Result<pb::GetContractInfoResponse, prpc::server::Error> {
        self.lock_phactory(true, false)?
            .get_contract_info(&request.contracts)
    }

    async fn get_cluster_info(
        &mut self,
        _request: (),
    ) -> Result<pb::GetClusterInfoResponse, prpc::server::Error> {
        self.lock_phactory(true, false)?.get_cluster_info()
    }

    async fn upload_sidevm_code(
        &mut self,
        request: pb::SidevmCode,
    ) -> Result<(), prpc::server::Error> {
        let contract_id: [u8; 32] = request
            .contract
            .try_into()
            .map_err(|_| from_display("Invalid contract id"))?;
        self.lock_phactory(true, false)?
            .upload_sidevm_code(contract_id.into(), request.code)
    }

    async fn calculate_contract_id(
        &mut self,
        req: pb::ContractParameters,
    ) -> Result<pb::ContractId, prpc::server::Error> {
        let deployer =
            try_decode_hex(&req.deployer).map_err(|_| from_display("Invalid deployer"))?;
        let code_hash =
            try_decode_hex(&req.code_hash).map_err(|_| from_display("Invalid code hash"))?;
        let cluster_id =
            try_decode_hex(&req.cluster_id).map_err(|_| from_display("Invalid cluster id"))?;
        let salt = try_decode_hex(&req.salt).map_err(|_| from_display("Invalid code salt"))?;
        let buf = contract_id_preimage(&deployer, &code_hash, &cluster_id, &salt);
        let hash = sp_core::blake2_256(&buf);
        Ok(pb::ContractId { id: hex(hash) })
    }

    async fn get_network_config(
        &mut self,
        _request: (),
    ) -> Result<pb::NetworkConfigResponse, prpc::server::Error> {
        let phactory = self.lock_phactory(true, false)?;
        Ok(pb::NetworkConfigResponse {
            public_rpc_port: phactory.args.public_port.map(Into::into),
            config: phactory.netconfig.clone(),
        })
    }
    async fn load_chain_state(
        &mut self,
        request: pb::ChainState,
    ) -> Result<(), prpc::server::Error> {
        self.lock_phactory(false, false)?
            .load_chain_state(request.block_number, request.decode_state()?)
            .map_err(from_display)
    }
    async fn stop(&mut self, request: pb::StopOptions) -> Result<(), prpc::server::Error> {
        self.lock_phactory(true, true)?
            .stop(request.remove_checkpoints)
    }
    async fn load_storage_proof(
        &mut self,
        req: phactory_api::prpc::StorageProof,
    ) -> Result<(), prpc::server::Error> {
        self.lock_phactory(false, true)?
            .load_storage_proof(req.proof)?;
        Ok(())
    }
    async fn take_checkpoint(&mut self, _req: ()) -> Result<pb::SyncedTo, prpc::server::Error> {
        let synced_to = self
            .lock_phactory(false, false)?
            .take_checkpoint()
            .map_err(from_debug)?;
        Ok(pb::SyncedTo { synced_to })
    }
    async fn statistics(
        &mut self,
        request: pb::StatisticsReqeust,
    ) -> Result<pb::StatisticsResponse, prpc::server::Error> {
        self.lock_phactory(true, false)?
            .statistics(request)
            .map_err(from_debug)
    }
    async fn generate_cluster_state_request(
        &mut self,
        _: (),
    ) -> Result<pb::SaveClusterStateArguments, prpc::server::Error> {
        let mut phactory = self.lock_phactory(true, false)?;
        let system = phactory.system()?;
        if system.contract_cluster.is_some() {
            return Err(from_display("Already in cluster"));
        }
        let block_number = system.block_number;
        let min_block_number = block_number + 1;
        let sig = system.identity_key.sign(&wrap_content_to_sign(
            &min_block_number.to_be_bytes(),
            SignedContentType::ClusterStateRequest,
        ));
        Ok(pb::SaveClusterStateArguments {
            receiver: hex(system.identity_key.public()),
            min_block_number,
            signature: hex(sig),
        })
    }
    async fn save_cluster_state(
        &mut self,
        req: pb::SaveClusterStateArguments,
    ) -> Result<pb::SaveClusterStateResponse, prpc::server::Error> {
        let pb::SaveClusterStateArguments {
            receiver,
            min_block_number,
            signature,
        } = req;
        let receiver = try_decode_hex(&receiver).map_err(|_| from_display("Invalid receiver"))?;
        let signature =
            try_decode_hex(&signature).map_err(|_| from_display("Invalid signature"))?;

        // Check the validity of the state request with the remote public key
        if !crypto::verify::<sr25519::Pair>(
            &receiver,
            &signature,
            &wrap_content_to_sign(
                &min_block_number.to_be_bytes(),
                SignedContentType::ClusterStateRequest,
            ),
        ) {
            return Err(from_display("Invalid signature"));
        }
        // RCU in progress is not allowed because we need to save the mq egress messages.
        // If there is an in pregress block execution, we'll get an inconsistent state.
        self.lock_phactory(false, false)?
            .save_cluster_state(min_block_number, &receiver)
    }
    async fn load_cluster_state(
        &mut self,
        req: pb::SaveClusterStateResponse,
    ) -> Result<(), prpc::server::Error> {
        self.lock_phactory(false, false)?
            .load_cluster_state(&req.filename)
            .map_err(from_debug)
    }
    async fn try_upgrade_pink_runtime(
        &mut self,
        req: pb::PinkRuntimeVersion,
    ) -> Result<(), prpc::server::Error> {
        let version = (req.major, req.minor);
        let block_number;
        let mut cluster = {
            let mut phactory = self.lock_phactory(true, false)?;
            block_number = phactory.current_block()?.0;
            let system = phactory.system()?;
            let cluster = system
                .contract_cluster
                .as_ref()
                .ok_or_else(|| from_display("Worker not in cluster"))?;
            cluster.clone()
        };
        cluster.upgrade_runtime(version);
        cluster.on_idle(block_number);
        Ok(())
    }
}
