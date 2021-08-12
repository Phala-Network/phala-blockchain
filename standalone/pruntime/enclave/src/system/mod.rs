mod gk;
mod master_key;

use crate::std::convert::TryInto;
use crate::{benchmark, std::prelude::v1::*, types::BlockInfo};
use anyhow::Result;
use core::fmt;
use log::info;

use chain::pallet_registry::RegistryEvent;
pub use enclave_api::prpc::{GatekeeperRole, GatekeeperStatus};
use parity_scale_codec::{Decode, Encode};
use phala_crypto::{
    aead, ecdh,
    sr25519::{Persistence, KDF},
};
use phala_mq::{
    BadOrigin, MessageDispatcher, MessageOrigin, MessageSendQueue, Sr25519MessageChannel,
    TypedReceiveError, TypedReceiver,
};
use phala_types::{
    messaging::{
        DispatchMasterKeyEvent, HeartbeatChallenge, MasterKeyEvent, MiningReportEvent,
        NewGatekeeperEvent, SystemEvent, WorkerEvent,
    },
    MasterPublicKey, WorkerPublicKey,
};
use sp_core::{hashing::blake2_256, sr25519, Pair, U256};

pub type TransactionResult = Result<(), TransactionError>;

#[derive(Encode, Decode, Debug, Clone)]
pub enum TransactionError {
    BadInput,
    BadOrigin,
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

#[derive(Debug)]
struct BenchState {
    start_block: chain::BlockNumber,
    start_time: u64,
    start_iter: u64,
    duration: u32,
}

#[derive(Debug)]
enum MiningState {
    Mining,
    Paused,
}

#[derive(Debug)]
struct MiningInfo {
    session_id: u32,
    state: MiningState,
    start_time: u64,
    start_iter: u64,
}

// Minimum worker state machine can be reused to replay in GK.
// TODO: shrink size
struct WorkerState {
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
                self.handle_heartbeat_challenge(block, &seed_info, callback, log_on);
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
            info!(
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

struct WorkerSMDelegate<'a>(&'a Sr25519MessageChannel);

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
        self.0.send(&report);
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
        self.0.send(&event);
    }
}

pub struct System {
    // Configuration
    sealing_path: String,
    // Messageing
    send_mq: MessageSendQueue,
    egress: Sr25519MessageChannel,
    system_events: TypedReceiver<SystemEvent>,
    master_key_events: TypedReceiver<MasterKeyEvent>,
    // Worker
    identity_key: sr25519::Pair,
    worker_state: WorkerState,
    // Gatekeeper
    registered_on_chain: bool,
    master_key: Option<sr25519::Pair>,
    gatekeeper: Option<gk::Gatekeeper<Sr25519MessageChannel>>,
}

impl System {
    pub fn new(
        sealing_path: String,
        identity_key: &sr25519::Pair,
        send_mq: &MessageSendQueue,
        recv_mq: &mut MessageDispatcher,
    ) -> Self {
        let pubkey = identity_key.clone().public();
        let sender = MessageOrigin::Worker(pubkey);
        let master_key = master_key::try_unseal(sealing_path.clone(), identity_key);

        System {
            sealing_path,
            send_mq: send_mq.clone(),
            egress: send_mq.channel(sender, identity_key.clone()),
            system_events: recv_mq.subscribe_bound(),
            master_key_events: recv_mq.subscribe_bound(),
            identity_key: identity_key.clone(),
            worker_state: WorkerState::new(pubkey),
            registered_on_chain: false,
            master_key,
            gatekeeper: None,
        }
    }

    pub fn handle_query(
        &mut self,
        _accid_origin: Option<&chain::AccountId>,
        _req: Request,
    ) -> Response {
        Response::Error("Unreachable!".to_string())
    }

    pub fn process_messages(&mut self, block: &mut BlockInfo) -> anyhow::Result<()> {
        loop {
            let ok = phala_mq::select! {
                message = self.system_events => match message {
                    Ok((_, event, origin)) => {
                        if !origin.is_pallet() {
                            error!("Invalid SystemEvent sender: {:?}", origin);
                            continue;
                        }
                        self.process_system_event(block, &event)?;
                    }
                    Err(e) => match e {
                        TypedReceiveError::CodecError(e) => {
                            error!("Decode system event failed: {:?}", e);
                            continue;
                        }
                        TypedReceiveError::SenderGone => {
                            return Err(anyhow::anyhow!("System message channel broken"));
                        }
                    }
                },
                message = self.master_key_events => match message {
                    Ok((_, event, origin)) => {
                        self.process_master_key_event(block, origin, event);
                    }
                    Err(e) => {
                        error!("Read message failed: {:?}", e);
                    }
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

        Ok(())
    }

    fn process_system_event(&mut self, block: &BlockInfo, event: &SystemEvent) -> Result<()> {
        self.worker_state
            .process_event(block, event, &mut WorkerSMDelegate(&self.egress), true);
        Ok(())
    }

    fn set_master_key(&mut self, master_key: sr25519::Pair, need_restart: bool) {
        if self.master_key.is_none() {
            master_key::seal(self.sealing_path.clone(), &master_key, &self.identity_key);
            self.master_key = Some(master_key);

            if need_restart {
                panic!("Received master key, please restart pRuntime and pherry");
            }
        } else if let Some(my_master_key) = &self.master_key {
            // TODO.shelven: remove this assertion after we enable master key rotation
            assert_eq!(my_master_key.to_raw_vec(), master_key.to_raw_vec());
        }
    }

    pub fn process_master_key_event(
        &mut self,
        block: &mut BlockInfo,
        origin: MessageOrigin,
        event: MasterKeyEvent,
    ) {
        info!("Incoming master key event: {:?}", event);
        match event {
            MasterKeyEvent::GatekeeperRegistered(new_gatekeeper_event) => {
                self.process_new_gatekeeper_event(block, origin, new_gatekeeper_event)
            }
            MasterKeyEvent::DispatchMasterKey(dispatch_master_key_event) => {
                self.process_dispatch_master_key_event(block, origin, dispatch_master_key_event)
            }
            MasterKeyEvent::MasterPubkeyOnChain(master_pubkey_event) => {
                if let Some(master_key) = &self.master_key {
                    assert!(
                        master_key.public() == master_pubkey_event.master_pubkey,
                        "Master key mismatches"
                    );
                    assert!(
                        self.gatekeeper.is_none(),
                        "Duplicated gatekeeper initialization"
                    );

                    info!("Init gatekeeper");
                    let mut gatekeeper = gk::Gatekeeper::new(
                        master_key.clone(),
                        &mut block.recv_mq,
                        self.send_mq
                            .channel(MessageOrigin::Gatekeeper, master_key.clone()),
                    );
                    if self.registered_on_chain {
                        gatekeeper.register_on_chain();
                    }
                    self.gatekeeper = Some(gatekeeper);
                }
            }
        }
    }

    /// Monitor the getakeeper registeration event to:
    ///
    /// 1. Generate the master key if this is the first gatekeeper;
    /// 2. Dispatch the master key to newly-registered gatekeepers;
    fn process_new_gatekeeper_event(
        &mut self,
        block: &BlockInfo,
        origin: MessageOrigin,
        event: NewGatekeeperEvent,
    ) {
        if !origin.is_pallet() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        }

        // double check the new gatekeeper is valid on chain
        if !chain_state::is_gatekeeper(&event.pubkey, block.storage) {
            error!("Fatal error: Invalid gatekeeper registration {:?}", event);
            panic!("System state poisoned");
        }

        let my_pubkey = self.identity_key.public();
        // gatekeeper genesis period
        if event.gatekeeper_count == 1 {
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
                self.egress.send(&master_pubkey);
            }
        } else {
            // TODO(shelven): move this logic to GK, and send the message on behalf of GK

            // dispatch the master key to the newly-registered gatekeeper using master key
            // if this pRuntime is the newly-registered gatekeeper himself,
            // its egress is still in dummy mode since we will tick the state later
            if let Some(master_key) = &self.master_key {
                info!(
                    "Gatekeeper: try dispatch master key {}",
                    hex::encode(master_key.public())
                );
                let derived_key = master_key
                    .derive_sr25519_pair(&[&crate::generate_random_info()])
                    .expect("should not fail with valid info");
                let my_ecdh_key = derived_key
                    .derive_ecdh_key()
                    .expect("ecdh key derivation should never failed with valid master key");
                let secret = ecdh::agree(&my_ecdh_key, &event.ecdh_pubkey.0)
                    .expect("should never fail with valid ecdh key");
                let iv = crate::generate_random_iv();
                let mut data = master_key.dump_secret_key().to_vec();

                aead::encrypt(&iv, &secret, &mut data).expect("Failed to encrypt master key");
                self.egress.send(&MasterKeyEvent::dispatch_master_key_event(
                    event.pubkey,
                    my_ecdh_key
                        .public()
                        .as_ref()
                        .try_into()
                        .expect("should never fail given pubkey with correct length"),
                    data,
                    iv,
                ));
            }
        }

        if my_pubkey == event.pubkey {
            self.registered_on_chain = true;
            if let Some(gatekeeper) = &mut self.gatekeeper {
                gatekeeper.register_on_chain();
            }
        }
    }

    /// Process encrypted master key from mq
    fn process_dispatch_master_key_event(
        &mut self,
        block: &BlockInfo,
        origin: MessageOrigin,
        event: DispatchMasterKeyEvent,
    ) {
        let worker_pubkey = if let MessageOrigin::Worker(pubkey) = origin {
            pubkey
        } else {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        };

        // ensure the message source is a gatekeeper
        if !chain_state::is_gatekeeper(&worker_pubkey, block.storage) {
            warn!("Warning: Fake master key dispatch {:?}", event);
            return;
        }

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

            let master_pair = sr25519::Pair::from_seed_slice(&master_key)
                .expect("Master key seed must be correct; qed.");
            info!("Gatekeeper: successfully decrypt received master key");
            self.set_master_key(master_pair, true);
        }
    }

    pub fn is_registered(&self) -> bool {
        self.worker_state.registered
    }

    pub fn gatekeeper_status(&self) -> GatekeeperStatus {
        let active = self.registered_on_chain;
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

#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {}

#[derive(Encode, Decode, Debug)]
pub enum Response {
    Error(String),
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

#[cfg(feature = "tests")]
pub fn run_all_tests() {
    gk::tests::run_all_tests();
}
