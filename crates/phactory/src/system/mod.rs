pub mod gk;
mod master_key;
mod side_tasks;

use crate::{
    benchmark,
    contracts::{
        pink::{
            group::Group,
            messaging::{WorkerPinkReport, WorkerPinkRequest},
        },
        ContractsKeeper, ExecuteEnv, NativeContract,
    },
    pink::{group::GroupKeeper, Pink},
    secret_channel::{ecdh_serde, SecretReceiver},
    types::{BlockInfo, OpaqueError, OpaqueQuery, OpaqueReply},
};
use anyhow::Result;
use core::fmt;
use log::info;
use pink::runtime::ExecSideEffects;
use runtime::BlockNumber;

use crate::contracts;
use crate::pal;
use chain::pallet_registry::RegistryEvent;
use parity_scale_codec::{Decode, Encode};
pub use phactory_api::prpc::{GatekeeperRole, GatekeeperStatus};
use phala_crypto::{
    aead,
    ecdh::{self, EcdhKey},
    sr25519::{Persistence, KDF},
};
use phala_mq::{
    traits::MessageChannel, BadOrigin, BindTopic, ContractId, MessageDispatcher, MessageOrigin,
    MessageSendQueue, SignedMessageChannel, TypedReceiver,
};
use phala_serde_more as more;
use phala_types::{
    contract,
    messaging::{
        DispatchMasterKeyEvent, GatekeeperChange, GatekeeperLaunch, HeartbeatChallenge,
        KeyDistribution, MiningReportEvent, NewGatekeeperEvent, SystemEvent, WorkerEvent,
    },
    EcdhPublicKey, MasterPublicKey, WorkerPublicKey,
};
use serde::{Deserialize, Serialize};
use side_tasks::geo_probe;
use sp_core::{hashing::blake2_256, sr25519, Pair, U256};

pub type TransactionResult = Result<pink::runtime::ExecSideEffects, TransactionError>;

#[derive(Encode, Decode, Debug, Clone)]
pub enum TransactionError {
    BadInput,
    BadOrigin,
    Other(String),
    // general
    InsufficientBalance,
    NoBalance,
    UnknownError,
    BadContractId,
    BadCommand,
    SymbolExist,
    AssetIdNotFound,
    NotAssetOwner,
    BadSecret,
    BadMachineId,
    FailedToSign,
    BadDecimal,
    DestroyNotAllowed,
    ChannelError,
    // for pdiem
    BadAccountInfo,
    BadLedgerInfo,
    BadTrustedStateData,
    BadEpochChangedProofData,
    BadTrustedState,
    InvalidAccount,
    BadTransactionWithProof,
    FailedToVerify,
    FailedToGetTransaction,
    FailedToCalculateBalance,
    BadChainId,
    TransferringNotAllowed,
}

impl From<BadOrigin> for TransactionError {
    fn from(_: BadOrigin) -> TransactionError {
        TransactionError::BadOrigin
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchState {
    start_block: chain::BlockNumber,
    start_time: u64,
    start_iter: u64,
    duration: u32,
}

#[derive(Debug, Serialize, Deserialize)]
enum MiningState {
    Mining,
    Paused,
}

#[derive(Debug, Serialize, Deserialize)]
struct MiningInfo {
    session_id: u32,
    state: MiningState,
    start_time: u64,
    start_iter: u64,
}

// Minimum worker state machine can be reused to replay in GK.
#[derive(Debug, Serialize, Deserialize)]
struct WorkerState {
    #[serde(with = "more::pubkey_bytes")]
    pubkey: WorkerPublicKey,
    hashed_id: U256,
    registered: bool,
    bench_state: Option<BenchState>,
    mining_state: Option<MiningInfo>,
}

impl WorkerState {
    pub fn new(pubkey: WorkerPublicKey) -> Self {
        let raw_pubkey: &[u8] = pubkey.as_ref();
        let pkh = blake2_256(raw_pubkey);
        let hashed_id: U256 = pkh.into();
        Self {
            pubkey,
            hashed_id,
            registered: false,
            bench_state: None,
            mining_state: None,
        }
    }

    pub fn process_event(
        &mut self,
        block: &BlockInfo,
        event: &SystemEvent,
        callback: &mut impl WorkerStateMachineCallback,
        log_on: bool,
    ) {
        match event {
            SystemEvent::WorkerEvent(evt) => {
                if evt.pubkey != self.pubkey {
                    return;
                }

                use MiningState::*;
                use WorkerEvent::*;
                if log_on {
                    info!("System::handle_event: {:?}", evt.event);
                }
                match evt.event {
                    Registered(_) => {
                        self.registered = true;
                    }
                    BenchStart { duration } => {
                        self.bench_state = Some(BenchState {
                            start_block: block.block_number,
                            start_time: block.now_ms,
                            start_iter: callback.bench_iterations(),
                            duration,
                        });
                        callback.bench_resume();
                    }
                    BenchScore(score) => {
                        if log_on {
                            info!("My benchmark score is {}", score);
                        }
                    }
                    MiningStart { session_id, .. } => {
                        self.mining_state = Some(MiningInfo {
                            session_id,
                            state: Mining,
                            start_time: block.now_ms,
                            start_iter: callback.bench_iterations(),
                        });
                        callback.bench_resume();
                    }
                    MiningStop => {
                        self.mining_state = None;
                        if self.need_pause() {
                            callback.bench_pause();
                        }
                    }
                    MiningEnterUnresponsive => {
                        if let Some(info) = &mut self.mining_state {
                            if let Mining = info.state {
                                if log_on {
                                    info!("Enter paused");
                                }
                                info.state = Paused;
                                return;
                            }
                        }
                        if log_on {
                            error!(
                                "Unexpected event received: {:?}, mining_state= {:?}",
                                evt.event, self.mining_state
                            );
                        }
                    }
                    MiningExitUnresponsive => {
                        if let Some(info) = &mut self.mining_state {
                            if let Paused = info.state {
                                if log_on {
                                    info!("Exit paused");
                                }
                                info.state = Mining;
                                return;
                            }
                        }
                        if log_on {
                            error!(
                                "Unexpected event received: {:?}, mining_state= {:?}",
                                evt.event, self.mining_state
                            );
                        }
                    }
                }
            }
            SystemEvent::HeartbeatChallenge(seed_info) => {
                self.handle_heartbeat_challenge(block, seed_info, callback, log_on);
            }
        };
    }

    fn handle_heartbeat_challenge(
        &mut self,
        block: &BlockInfo,
        seed_info: &HeartbeatChallenge,
        callback: &mut impl WorkerStateMachineCallback,
        log_on: bool,
    ) {
        if log_on {
            debug!(
                "System::handle_heartbeat_challenge({}, {:?}), registered={:?}, mining_state={:?}",
                block.block_number, seed_info, self.registered, self.mining_state
            );
        }

        if !self.registered {
            return;
        }

        let mining_state = if let Some(state) = &mut self.mining_state {
            state
        } else {
            return;
        };

        if matches!(mining_state.state, MiningState::Paused) {
            return;
        }

        let x = self.hashed_id ^ seed_info.seed;
        let online_hit = x <= seed_info.online_target;

        // Push queue when necessary
        if online_hit {
            let iterations = callback.bench_iterations() - mining_state.start_iter;
            callback.heartbeat(
                mining_state.session_id,
                block.block_number,
                block.now_ms,
                iterations,
            );
        }
    }

    fn need_pause(&self) -> bool {
        self.bench_state.is_none() && self.mining_state.is_none()
    }

    fn on_block_processed(
        &mut self,
        block: &BlockInfo,
        callback: &mut impl WorkerStateMachineCallback,
    ) {
        // Handle registering benchmark report
        if let Some(BenchState {
            start_block,
            start_time,
            start_iter,
            duration,
        }) = self.bench_state
        {
            if block.block_number - start_block >= duration {
                self.bench_state = None;
                let iterations = callback.bench_iterations() - start_iter;
                callback.bench_report(start_time, iterations);
                if self.need_pause() {
                    callback.bench_pause();
                }
            }
        }
    }
}

trait WorkerStateMachineCallback {
    fn bench_iterations(&self) -> u64 {
        0
    }
    fn bench_resume(&mut self) {}
    fn bench_pause(&mut self) {}
    fn bench_report(&mut self, _start_time: u64, _iterations: u64) {}
    fn heartbeat(
        &mut self,
        _session_id: u32,
        _block_num: chain::BlockNumber,
        _block_time: u64,
        _iterations: u64,
    ) {
    }
}

struct WorkerSMDelegate<'a>(&'a SignedMessageChannel);

impl WorkerStateMachineCallback for WorkerSMDelegate<'_> {
    fn bench_iterations(&self) -> u64 {
        benchmark::iteration_counter()
    }
    fn bench_resume(&mut self) {
        benchmark::resume();
    }
    fn bench_pause(&mut self) {
        benchmark::pause();
    }
    fn bench_report(&mut self, start_time: u64, iterations: u64) {
        let report = RegistryEvent::BenchReport {
            start_time,
            iterations,
        };
        info!("Reporting benchmark: {:?}", report);
        self.0.push_message(&report);
    }
    fn heartbeat(
        &mut self,
        session_id: u32,
        challenge_block: chain::BlockNumber,
        challenge_time: u64,
        iterations: u64,
    ) {
        let event = MiningReportEvent::Heartbeat {
            session_id,
            challenge_block,
            challenge_time,
            iterations,
        };
        info!("System: sending {:?}", event);
        self.0.push_message(&event);
    }
}

#[derive(
    Serialize, Deserialize, Clone, derive_more::Deref, derive_more::DerefMut, derive_more::From,
)]
#[serde(transparent)]
pub(crate) struct WorkerIdentityKey(#[serde(with = "more::key_bytes")] sr25519::Pair);

// By mocking the public key of the identity key pair, we can pretend to be the first Gatekeeper on Khala
// for "shadow-gk" simulation.
#[cfg(feature = "shadow-gk")]
impl WorkerIdentityKey {
    pub(crate) fn public(&self) -> sr25519::Public {
        // The pubkey of the first GK on khala
        sr25519::Public(hex_literal::hex!(
            "60067697c486c809737e50d30a67480c5f0cede44be181b96f7d59bc2116a850"
        ))
    }
}

#[derive(Serialize, Deserialize)]
pub struct System<Platform> {
    platform: Platform,
    // Configuration
    pub(crate) sealing_path: String,
    enable_geoprobing: bool,
    pub(crate) geoip_city_db: String,
    // Messageing
    egress: SignedMessageChannel,
    system_events: TypedReceiver<SystemEvent>,
    gatekeeper_launch_events: TypedReceiver<GatekeeperLaunch>,
    gatekeeper_change_events: TypedReceiver<GatekeeperChange>,
    key_distribution_events: TypedReceiver<KeyDistribution>,
    pink_events: SecretReceiver<WorkerPinkRequest>,
    // Worker
    pub(crate) identity_key: WorkerIdentityKey,
    #[serde(with = "ecdh_serde")]
    pub(crate) ecdh_key: EcdhKey,
    worker_state: WorkerState,
    // Gatekeeper
    #[serde(with = "more::option_key_bytes")]
    master_key: Option<sr25519::Pair>,
    pub(crate) gatekeeper: Option<gk::Gatekeeper<SignedMessageChannel>>,

    pub(crate) contracts: ContractsKeeper,
    contract_groups: GroupKeeper,

    // Cached for query
    block_number: BlockNumber,
    now_ms: u64,
}

impl<Platform: pal::Platform> System<Platform> {
    pub fn new(
        platform: Platform,
        sealing_path: String,
        enable_geoprobing: bool,
        geoip_city_db: String,
        identity_key: sr25519::Pair,
        ecdh_key: EcdhKey,
        send_mq: &MessageSendQueue,
        recv_mq: &mut MessageDispatcher,
        contracts: ContractsKeeper,
    ) -> Self {
        let identity_key = WorkerIdentityKey(identity_key);
        let pubkey = identity_key.public();
        let sender = MessageOrigin::Worker(pubkey);
        let master_key = master_key::try_unseal(sealing_path.clone(), &identity_key.0, &platform);

        System {
            platform,
            sealing_path,
            enable_geoprobing,
            geoip_city_db,
            egress: send_mq.channel(sender, identity_key.clone().0.into()),
            system_events: recv_mq.subscribe_bound(),
            gatekeeper_launch_events: recv_mq.subscribe_bound(),
            gatekeeper_change_events: recv_mq.subscribe_bound(),
            key_distribution_events: recv_mq.subscribe_bound(),
            pink_events: SecretReceiver::new_secret(
                recv_mq.subscribe(WorkerPinkRequest::topic()).into(),
                ecdh_key.clone(),
            ),
            identity_key,
            ecdh_key,
            worker_state: WorkerState::new(pubkey),
            master_key,
            gatekeeper: None,
            contracts,
            contract_groups: Default::default(),
            block_number: 0,
            now_ms: 0,
        }
    }

    pub fn handle_query(
        &mut self,
        origin: Option<&chain::AccountId>,
        contract_id: &ContractId,
        req: OpaqueQuery,
    ) -> Result<OpaqueReply, OpaqueError> {
        let contract = self
            .contracts
            .get_mut(contract_id)
            .ok_or(OpaqueError::ContractNotFound)?;
        let mut context = contracts::QueryContext {
            block_number: self.block_number,
            now_ms: self.now_ms,
            contract_groups: &mut self.contract_groups,
        };
        contract.handle_query(origin, req, &mut context)
    }

    pub fn process_messages(&mut self, block: &mut BlockInfo) -> anyhow::Result<()> {
        self.block_number = block.block_number;
        self.now_ms = block.now_ms;

        if self.enable_geoprobing {
            geo_probe::process_block(
                block.block_number,
                &self.egress,
                block.side_task_man,
                &self.identity_key,
                self.geoip_city_db.clone(),
            );
        }
        loop {
            let ok = phala_mq::select_ignore_errors! {
                (event, origin) = self.system_events => {
                    if !origin.is_pallet() {
                        error!("Invalid SystemEvent sender: {:?}", origin);
                        continue;
                    }
                    self.process_system_event(block, &event)?;
                },
                (event, origin) = self.gatekeeper_launch_events => {
                    self.process_gatekeeper_launch_event(block, origin, event);
                },
                (event, origin) = self.gatekeeper_change_events => {
                    self.process_gatekeeper_change_event(block, origin, event);
                },
                (event, origin) = self.key_distribution_events => {
                    self.process_key_distribution_event(block, origin, event);
                },
                (event, origin) = self.pink_events => {
                    self.process_pink_event(block, origin, event);
                },
            };
            if ok.is_none() {
                // All messages processed
                break;
            }
        }
        self.worker_state
            .on_block_processed(block, &mut WorkerSMDelegate(&self.egress));

        if let Some(gatekeeper) = &mut self.gatekeeper {
            gatekeeper.process_messages(block);
            gatekeeper.emit_random_number(block.block_number);
        }

        // Iterate over all contracts to handle their incoming commands.
        //
        // Since the wasm contracts can instantiate new contracts, it means that it will mutate the `self.contracts`.
        // So we can not directly itrate over the self.contracts.values_mut() which would keep borrowing on `self.contracts`
        // in the scope of entire `for loop` body.
        let contract_ids: Vec<_> = self.contracts.keys().cloned().collect();
        'outer: for key in contract_ids {
            // Inner loop to handle commands. One command per iteration and apply the command side-effects to make it
            // availabe for next command.
            loop {
                let contract = match self.contracts.get_mut(&key) {
                    None => continue 'outer,
                    Some(v) => v,
                };
                let group_id = contract.group_id();
                let mut env = ExecuteEnv {
                    block: block,
                    contract_groups: &mut self.contract_groups,
                };
                let result = match contract.process_next_message(&mut env) {
                    Some(result) => result,
                    None => break,
                };
                handle_contract_command_result(
                    result,
                    group_id,
                    &mut self.contracts,
                    &mut self.contract_groups,
                    block,
                    &self.egress,
                );
            }
            let mut env = ExecuteEnv {
                block: block,
                contract_groups: &mut self.contract_groups,
            };
            let contract = match self.contracts.get_mut(&key) {
                None => continue 'outer,
                Some(v) => v,
            };
            let result = contract.on_block_end(&mut env);
            let group_id = contract.group_id();
            handle_contract_command_result(
                result,
                group_id,
                &mut self.contracts,
                &mut self.contract_groups,
                block,
                &self.egress,
            );
        }

        Ok(())
    }

    fn process_system_event(&mut self, block: &BlockInfo, event: &SystemEvent) -> Result<()> {
        self.worker_state
            .process_event(block, event, &mut WorkerSMDelegate(&self.egress), true);
        Ok(())
    }

    fn set_master_key(&mut self, master_key: sr25519::Pair, need_restart: bool) {
        if self.master_key.is_none() {
            master_key::seal(
                self.sealing_path.clone(),
                &master_key,
                &self.identity_key,
                &self.platform,
            );
            self.master_key = Some(master_key);

            if need_restart {
                crate::maybe_remove_checkpoints(&self.sealing_path);
                panic!("Received master key, please restart pRuntime and pherry");
            }
        } else if let Some(my_master_key) = &self.master_key {
            // TODO.shelven: remove this assertion after we enable master key rotation
            assert_eq!(my_master_key.to_raw_vec(), master_key.to_raw_vec());
        }
    }

    fn init_gatekeeper(&mut self, block: &mut BlockInfo) {
        assert!(
            self.master_key.is_some(),
            "Gatekeeper initialization without master key"
        );
        assert!(
            self.gatekeeper.is_none(),
            "Duplicated gatekeeper initialization"
        );

        let gatekeeper = gk::Gatekeeper::new(
            self.master_key
                .as_ref()
                .expect("checked master key above; qed.")
                .clone(),
            block.recv_mq,
            block.send_mq.channel(
                MessageOrigin::Gatekeeper,
                self.master_key
                    .as_ref()
                    .expect("checked master key above; qed.")
                    .clone()
                    .into(),
            ),
        );
        self.gatekeeper = Some(gatekeeper);
    }

    fn process_gatekeeper_launch_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: GatekeeperLaunch,
    ) {
        info!("Incoming gatekeeper launch event: {:?}", event);
        match event {
            GatekeeperLaunch::FirstGatekeeper(new_gatekeeper_event) => {
                self.process_first_gatekeeper_event(block, origin, new_gatekeeper_event)
            }
            GatekeeperLaunch::MasterPubkeyOnChain(_) => {
                info!(
                    "Gatekeeper launches on chain in block {}",
                    block.block_number
                );
                if let Some(gatekeeper) = &mut self.gatekeeper {
                    gatekeeper.master_pubkey_uploaded();
                }
            }
        }
    }

    /// Generate the master key if this is the first gatekeeper
    fn process_first_gatekeeper_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: NewGatekeeperEvent,
    ) {
        if !origin.is_pallet() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        }

        // double check the first gatekeeper is valid on chain
        if !chain_state::is_gatekeeper(&event.pubkey, block.storage) {
            error!(
                "Fatal error: Invalid first gatekeeper registration {:?}",
                event
            );
            panic!("System state poisoned");
        }

        let my_pubkey = self.identity_key.public();
        // if the first gatekeeper reboots, it will possess the master key,
        // and should not re-generate it
        if my_pubkey == event.pubkey && self.master_key.is_none() {
            info!("Gatekeeper: generate master key as the first gatekeeper");
            // generate master key as the first gatekeeper
            // no need to restart
            let master_key = crate::new_sr25519_key();
            self.set_master_key(master_key.clone(), false);
            // upload the master key on chain via worker egress
            info!(
                "Gatekeeper: upload master key {} on chain",
                hex::encode(master_key.public())
            );
            let master_pubkey = RegistryEvent::MasterPubkey {
                master_pubkey: master_key.public(),
            };
            self.egress.push_message(&master_pubkey);
        }

        if self.master_key.is_some() {
            info!("Init gatekeeper in block {}", block.block_number);
            self.init_gatekeeper(block);
        }

        if my_pubkey == event.pubkey {
            self.gatekeeper
                .as_mut()
                .expect("gatekeeper must be initializaed here; qed.")
                .register_on_chain();
        }
    }

    fn process_gatekeeper_change_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: GatekeeperChange,
    ) {
        info!("Incoming gatekeeper change event: {:?}", event);
        match event {
            GatekeeperChange::GatekeeperRegistered(new_gatekeeper_event) => {
                self.process_new_gatekeeper_event(block, origin, new_gatekeeper_event)
            }
        }
    }

    /// Share the master key to the newly-registered gatekeeper
    /// Tick the state if the registered gatekeeper is this worker
    fn process_new_gatekeeper_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: NewGatekeeperEvent,
    ) {
        if !origin.is_pallet() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        }

        // double check the registered gatekeeper is valid on chain
        if !chain_state::is_gatekeeper(&event.pubkey, block.storage) {
            error!(
                "Fatal error: Invalid first gatekeeper registration {:?}",
                event
            );
            panic!("System state poisoned");
        }

        if let Some(gatekeeper) = &mut self.gatekeeper {
            gatekeeper.share_master_key(&event.pubkey, &event.ecdh_pubkey, block.block_number);

            let my_pubkey = self.identity_key.public();
            if my_pubkey == event.pubkey {
                gatekeeper.register_on_chain();
            }
        }
    }

    fn process_key_distribution_event(
        &mut self,
        _block: &mut BlockInfo,
        origin: MessageOrigin,
        event: KeyDistribution,
    ) {
        info!("Incoming key distribution event: {:?}", event);
        match event {
            KeyDistribution::MasterKeyDistribution(dispatch_master_key_event) => {
                self.process_master_key_distribution(origin, dispatch_master_key_event)
            }
        }
    }

    fn process_pink_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: WorkerPinkRequest,
    ) {
        match event {
            WorkerPinkRequest::Instantiate {
                group_id,
                worker,
                nonce,
                owner,
                wasm_bin,
                input_data,
                salt,
                key,
            } => {
                if worker != self.worker_state.pubkey {
                    return;
                }

                info!(
                    "Incoming pink instantiate event: origin={}, onwer={}, nonce={}, contract_size={}",
                    origin,
                    owner,
                    hex::encode(&nonce),
                    wasm_bin.len(),
                );

                if !origin.is_gatekeeper() && !origin.is_pallet() {
                    error!("Attempt to instantiate a pink instance from out of GK!");
                    return;
                }

                let contract_key = sr25519::Pair::restore_from_secret_key(&key);
                let result = self.contract_groups.instantiate_contract(
                    group_id,
                    owner.clone(),
                    wasm_bin,
                    input_data,
                    salt,
                    &contract_key,
                    block.block_number,
                    block.now_ms,
                );
                match result {
                    Err(err) => {
                        error!(
                            "Instantiate contract error: {}, owner: {:?}, nonce: {}",
                            err,
                            owner,
                            hex::encode(&nonce)
                        );
                    }
                    Ok(effects) => {
                        let group = self
                            .contract_groups
                            .get_group_mut(&group_id)
                            .expect("Group must exist after instantiate");

                        apply_pink_side_effects(
                            effects,
                            &group_id,
                            &mut self.contracts,
                            group,
                            block,
                            &self.egress,
                        );
                    }
                }
            }
        }
    }

    /// Process encrypted master key from mq
    fn process_master_key_distribution(
        &mut self,
        origin: MessageOrigin,
        event: DispatchMasterKeyEvent,
    ) {
        if !origin.is_gatekeeper() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        };

        let my_pubkey = self.identity_key.public();
        if my_pubkey == event.dest {
            let my_ecdh_key = self
                .identity_key
                .derive_ecdh_key()
                .expect("Should never failed with valid identity key; qed.");
            let secret = ecdh::agree(&my_ecdh_key, &event.ecdh_pubkey.0)
                .expect("Should never failed with valid ecdh key; qed.");

            let mut master_key_buff = event.encrypted_master_key.clone();
            let master_key = aead::decrypt(&event.iv, &secret, &mut master_key_buff[..])
                .expect("Failed to decrypt dispatched master key");

            let master_pair = sr25519::Pair::from_seed_slice(master_key)
                .expect("Master key seed must be correct; qed.");
            info!("Gatekeeper: successfully decrypt received master key");
            self.set_master_key(master_pair, true);
        }
    }

    pub fn is_registered(&self) -> bool {
        self.worker_state.registered
    }

    pub fn gatekeeper_status(&self) -> GatekeeperStatus {
        let active = match &self.gatekeeper {
            Some(gk) => gk.registered_on_chain(),
            None => false,
        };
        let has_key = self.master_key.is_some();
        let role = match (has_key, active) {
            (true, true) => GatekeeperRole::Active,
            (true, false) => GatekeeperRole::Dummy,
            _ => GatekeeperRole::None,
        };
        let master_public_key = self
            .master_key
            .as_ref()
            .map(|k| hex::encode(&k.public()))
            .unwrap_or_default();
        GatekeeperStatus {
            role: role.into(),
            master_public_key,
        }
    }

    pub fn commit_changes(&mut self) -> anyhow::Result<()> {
        self.contract_groups.commit_changes()
    }
}

pub fn handle_contract_command_result(
    result: TransactionResult,
    group_id: Option<phala_mq::ContractGroupId>,
    contracts: &mut ContractsKeeper,
    groups: &mut GroupKeeper,
    block: &mut BlockInfo,
    egress: &SignedMessageChannel,
) {
    let effects = match result {
        Err(err) => {
            error!("Run contract command failed: {:?}", err);
            return;
        }
        Ok(effects) => effects,
    };
    if let Some(group_id) = group_id {
        let group = match groups.get_group_mut(&group_id) {
            None => {
                error!(
                    "BUG: pink group not found, it should always exsists, group_id={:?}",
                    group_id
                );
                return;
            }
            Some(group) => group,
        };
        apply_pink_side_effects(effects, &group_id, contracts, group, block, egress);
    }
}

pub fn apply_pink_side_effects(
    effects: ExecSideEffects,
    group_id: &phala_mq::ContractGroupId,
    contracts: &mut ContractsKeeper,
    group: &mut Group,
    block: &mut BlockInfo,
    egress: &SignedMessageChannel,
) {
    let contract_key = group.key().clone();
    let ecdh_key = contract_key
        .derive_ecdh_key()
        .expect("Derive ecdh_key should not fail");

    for (deployer, address) in effects.instantiated {
        let pink = Pink::from_address(address, group_id.clone());
        let id = pink.id();

        install_contract(
            contracts,
            pink,
            contract_key.clone(),
            ecdh_key.clone(),
            block,
        );

        group.add_contract(id.clone());

        let message = WorkerPinkReport::PinkInstantiated {
            id,
            group_id: group_id.clone(),
            owner: phala_types::messaging::AccountId(deployer.into()),
            pubkey: EcdhPublicKey(ecdh_key.public()),
        };

        info!("pink instantiate status: {:?}", message);
        egress.push_message(&message);
    }

    for (address, event) in effects.pink_events {
        let id = Pink::address_to_id(&address);
        let contract = match contracts.get_mut(&id) {
            Some(contract) => contract,
            None => {
                panic!(
                    "BUG: Unknown contract sending pink event, address={:?}",
                    address
                );
            }
        };
        use pink::runtime::PinkEvent;
        match event {
            PinkEvent::Message(message) => {
                contract.push_message(message.payload, message.topic);
            }
            PinkEvent::OspMessage(message) => {
                contract.push_osp_message(
                    message.message.payload,
                    message.message.topic,
                    message.remote_pubkey.as_ref(),
                );
            }
            PinkEvent::OnBlockEndSelector(selector) => {
                contract.set_on_block_end_selector(selector);
            }
        }
    }
}

pub fn install_contract<Contract>(
    contracts: &mut ContractsKeeper,
    contract: Contract,
    contract_key: sr25519::Pair,
    ecdh_key: EcdhKey,
    block: &mut BlockInfo,
) where
    Contract: NativeContract + Send + 'static,
    <Contract as NativeContract>::Cmd: Send,
    contracts::AnyContract: From<contracts::NativeCompatContract<Contract>>,
{
    let contract_id = contract.id();
    let sender = MessageOrigin::Contract(contract_id);
    let mq = block.send_mq.channel(sender, contract_key.into());
    let cmd_mq = SecretReceiver::new_secret(
        block
            .recv_mq
            .subscribe(contract::command_topic(contract_id))
            .into(),
        ecdh_key.clone(),
    );
    let wrapped = contracts::NativeCompatContract::new(contract, mq, cmd_mq, ecdh_key.clone());
    contracts.insert(wrapped);
}

#[derive(Encode, Decode, Debug)]
pub enum Error {
    NotAuthorized,
    TxHashNotFound,
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::TxHashNotFound => write!(f, "transaction hash not found"),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

pub mod chain_state {
    use super::*;
    use crate::light_validation::utils::storage_prefix;
    use crate::storage::Storage;
    use parity_scale_codec::Decode;

    pub fn is_gatekeeper(pubkey: &WorkerPublicKey, chain_storage: &Storage) -> bool {
        let key = storage_prefix("PhalaRegistry", "Gatekeeper");
        let gatekeepers = chain_storage
            .get(&key)
            .map(|v| {
                Vec::<WorkerPublicKey>::decode(&mut &v[..])
                    .expect("Decode value of Gatekeeper Failed. (This should not happen)")
            })
            .unwrap_or_default();

        gatekeepers.contains(pubkey)
    }

    #[allow(dead_code)]
    pub fn read_master_pubkey(chain_storage: &Storage) -> Option<MasterPublicKey> {
        let key = storage_prefix("PhalaRegistry", "GatekeeperMasterPubkey");
        chain_storage
            .get(&key)
            .map(|v| {
                Some(
                    MasterPublicKey::decode(&mut &v[..])
                        .expect("Decode value of MasterPubkey Failed. (This should not happen)"),
                )
            })
            .unwrap_or(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_block_end() {
        let contract_key = sp_core::Pair::from_seed(&Default::default());
        let mut contracts = ContractsKeeper::default();
        let mut groupkeeper = GroupKeeper::default();
        let wasm_bin = pink::load_test_wasm("hooks_test");
        let group_id = phala_mq::ContractGroupId(Default::default());
        let effects = groupkeeper
            .instantiate_contract(
                group_id,
                Default::default(),
                wasm_bin,
                vec![0xed, 0x4b, 0x9d, 0x1b],
                Default::default(),
                &contract_key,
                1,
                1,
            )
            .unwrap();
        insta::assert_debug_snapshot!(effects);

        let group = groupkeeper.get_group_mut(&group_id).unwrap();
        let mut builder = BlockInfo::builder().block_number(1).now_ms(1);
        let signer = sr25519::Pair::from_seed(&Default::default());
        let egress = builder
            .send_mq
            .channel(MessageOrigin::Worker(Default::default()), signer.into());
        let mut block_info = builder.build();

        apply_pink_side_effects(
            effects,
            &group_id,
            &mut contracts,
            group,
            &mut block_info,
            &egress,
        );

        insta::assert_display_snapshot!(contracts.len());

        let mut env = ExecuteEnv {
            block: &block_info,
            contract_groups: &mut &mut groupkeeper,
        };

        for contract in contracts.values_mut() {
            let effects = contract.on_block_end(&mut env).unwrap();
            insta::assert_debug_snapshot!(effects);
        }

        let messages: Vec<_> = builder
            .send_mq
            .all_messages()
            .into_iter()
            .map(|msg| (msg.sequence, msg.message))
            .collect();
        insta::assert_debug_snapshot!(messages);
    }
}
