mod gk;

use crate::{benchmark, std::prelude::v1::*, types::BlockInfo};
use anyhow::Result;
use core::fmt;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use chain::pallet_registry::RegistryEvent;
pub use enclave_api::prpc::{GatekeeperRole, GatekeeperStatus};
use phala_mq::{
    MessageDispatcher, MessageOrigin, MessageSendQueue, Sr25519MessageChannel, TypedReceiveError,
    TypedReceiver,
};
use phala_types::{
    messaging::{HeartbeatChallenge, MiningReportEvent, SystemEvent, WorkerEvent},
    MasterPublicKey, WorkerPublicKey,
};
use sp_core::{hashing::blake2_256, sr25519, Pair, U256};

pub type CommandIndex = u64;

type Event = SystemEvent;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionStatus {
    Ok,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionReceipt {
    #[serde(
        serialize_with = "crate::se_to_b64",
        deserialize_with = "crate::de_from_b64"
    )]
    pub account: MessageOrigin,
    pub block_num: chain::BlockNumber,
    pub contract_id: u32,
    pub status: TransactionStatus,
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
        event: &Event,
        callback: &mut impl WorkerStateMachineCallback,
        log_on: bool,
    ) {
        match event {
            Event::WorkerEvent(evt) => {
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
            Event::HeartbeatChallenge(seed_info) => {
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

/// Master key filepath
pub const MASTER_KEY_FILE: &str = "master_key.seal";

pub struct System {
    // Configuration
    sealing_path: String,
    // Transaction
    receipts: BTreeMap<CommandIndex, TransactionReceipt>,
    // Messageing
    send_mq: MessageSendQueue,
    egress: Sr25519MessageChannel,
    ingress: TypedReceiver<Event>,
    // Worker
    worker_state: WorkerState,
    // Gatekeeper
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
        let sender = MessageOrigin::Worker(pubkey.clone());

        System {
            sealing_path: sealing_path.clone(),
            receipts: Default::default(),
            send_mq: send_mq.clone(),
            egress: send_mq.channel(sender, identity_key.clone()),
            ingress: recv_mq.subscribe_bound(),
            worker_state: WorkerState::new(pubkey.clone()),
            master_key: None,
            gatekeeper: None,
        }
    }

    pub fn add_receipt(&mut self, command_index: CommandIndex, tr: TransactionReceipt) {
        self.receipts.insert(command_index, tr);
    }

    pub fn get_receipt(&self, command_index: CommandIndex) -> Option<&TransactionReceipt> {
        self.receipts.get(&command_index)
    }

    pub fn handle_query(
        &mut self,
        accid_origin: Option<&chain::AccountId>,
        req: Request,
    ) -> Response {
        let inner = || -> Result<Response> {
            match req {
                Request::QueryReceipt { command_index } => match self.get_receipt(command_index) {
                    Some(receipt) => {
                        let origin =
                            accid_origin.ok_or_else(|| anyhow::Error::msg(Error::NotAuthorized))?;
                        let origin: [u8; 32] = *origin.as_ref();
                        if receipt.account == MessageOrigin::AccountId(origin.into()) {
                            Ok(Response::QueryReceipt {
                                receipt: receipt.clone(),
                            })
                        } else {
                            Err(anyhow::Error::msg(Error::NotAuthorized))
                        }
                    }
                    None => Err(anyhow::Error::msg(Error::Other(String::from(
                        "Transaction hash not found",
                    )))),
                },
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp,
        }
    }

    pub fn set_master_key(&mut self, master_key: sr25519::Pair, need_restart: bool) {
        if self.master_key.is_none() {
            self.master_key = Some(master_key.clone());
            self.seal_master_key();

            if need_restart {
                panic!("Received master key, please restart pRuntime and pherry");
            }

            // init gatekeeper egress dynamically with master key
            if self.gk_egress.is_none() {
                let gk_egress =
                    MsgChan::create(&self.send_mq, MessageOrigin::Gatekeeper, master_key);
                // by default, gk_egress is set to dummy mode until it is registered on chain
                gk_egress.set_dummy(true);
                self.gk_egress = Some(gk_egress);
            }
        } else if let Some(my_master_key) = &self.master_key {
            // TODO.shelven: remove this assertion after we enable master key rotation
            assert_eq!(my_master_key.to_raw_vec(), master_key.to_raw_vec());
        }
    }

    pub fn master_key_file_path(&self) -> PathBuf {
        PathBuf::from(&self.sealing_path).join(MASTER_KEY_FILE)
    }

    /// Seal master key seed with signature to ensure integrity
    pub fn seal_master_key(&self) {
        if let Some(master_key) = &self.master_key {
            let secret = master_key.dump_secret_key();
            let sig = self.identity_key.sign_data(&secret);

            // TODO(shelven): use serialization rather than manual concat.
            let mut buf = Vec::new();
            buf.extend_from_slice(&secret);
            buf.extend_from_slice(sig.as_ref());

            let filepath = self.master_key_file_path();
            info!(
                "Gatekeeper: seal master key to {}",
                filepath.as_path().display()
            );
            let mut file = SgxFile::create(filepath)
                .unwrap_or_else(|e| panic!("Create master key file failed: {:?}", e));
            file.write_all(&buf)
                .unwrap_or_else(|e| panic!("Seal master key failed: {:?}", e));
        }
    }

    /// Unseal local master key seed and verify signature
    ///
    /// This function could panic a lot.
    pub fn try_unseal_master_key(&mut self) {
        let filepath = self.master_key_file_path();
        info!(
            "Gatekeeper: unseal master key from {}",
            filepath.as_path().display()
        );
        let mut file = match SgxFile::open(filepath) {
            Ok(file) => file,
            Err(e) => {
                error!("Open master key file failed: {:?}", e);
                return;
            }
        };

        let mut secret = [0_u8; SECRET_KEY_LENGTH];
        let mut sig = [0_u8; SIGNATURE_BYTES];

        let n = file
            .read(secret.as_mut())
            .unwrap_or_else(|e| panic!("Read master key failed: {:?}", e));
        if n < SECRET_KEY_LENGTH {
            panic!(
                "Unexpected sealed secret key length {}, expected {}",
                n, SECRET_KEY_LENGTH
            );
        }

        let n = file
            .read(sig.as_mut())
            .unwrap_or_else(|e| panic!("Read master key sig failed: {:?}", e));
        if n < SIGNATURE_BYTES {
            panic!(
                "Unexpected sealed seed sig length {}, expected {}",
                n, SIGNATURE_BYTES
            );
        }

        assert!(
            self.identity_key
                .verify_data(&phala_crypto::sr25519::Signature::from_raw(sig), &secret),
            "Broken sealed master key"
        );

        self.set_master_key(sr25519::Pair::restore_from_secret_key(&secret), false);
    }

    pub fn process_messages(&mut self, block: &BlockInfo) -> anyhow::Result<()> {
        loop {
            match self.ingress.try_next() {
                Ok(Some((_, event, sender))) => {
                    if !sender.is_pallet() {
                        error!("Invalid SystemEvent sender: {:?}", sender);
                        continue;
                    }
                    self.process_event(block, &event)?;
                }
                Ok(None) => break,
                Err(e) => match e {
                    TypedReceiveError::CodecError(e) => {
                        error!("Decode system event failed: {:?}", e);
                        continue;
                    }
                    TypedReceiveError::SenderGone => {
                        return Err(anyhow::anyhow!("System message channel broken"));
                    }
                },
            }
        }
        self.worker_state
            .on_block_processed(block, &mut WorkerSMDelegate(&self.egress));

        // allow to process gatekeeper messages silently
        // if pRuntime possesses master key but is not registered on chain
        // TODO.shelven: this does not hold after we enable master key rotation
        if self.gatekeeper.possess_master_key()
            || chain_state::is_gatekeeper(&self.worker_state.pubkey, block.storage)
        {
            self.gatekeeper.process_messages(block);

            self.gatekeeper.emit_random_number(block.block_number);
        }
        Ok(())
    }

    fn process_event(&mut self, block: &BlockInfo, event: &Event) -> Result<()> {
        self.worker_state
            .process_event(block, event, &mut WorkerSMDelegate(&self.egress), true);
        Ok(())
    }

    /// Monitor the getakeeper registeration event to:
    ///
    /// 1. Generate the master key if this is the first gatekeeper;
    /// 2. Dispatch the master key to newly-registered gatekeepers;
    fn process_new_gatekeeper_event(&mut self, origin: MessageOrigin, event: NewGatekeeperEvent) {
        if !origin.is_pallet() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        }

        // double check the new gatekeeper is valid on chain
        if !chain_state::is_gatekeeper(&event.pubkey, self.block.storage) {
            error!("Fatal error: Invalid gatekeeper registration {:?}", event);
            panic!("GK state poisoned");
        }

        let my_pubkey = self.state.identity_key.public();
        if event.gatekeeper_count == 1 {
            if my_pubkey == event.pubkey && self.state.master_key.is_none() {
                info!("Gatekeeper: generate master key as the first gatekeeper");
                // generate master key as the first gatekeeper
                // no need to restart
                self.state.set_master_key(crate::new_sr25519_key(), false);
            }

            // upload the master key on chain
            // noted that every gk should execute this to ensure the state consistency of egress seq
            if let Some(master_key) = &self.state.master_key {
                info!("Gatekeeper: upload master key on chain");
                let master_pubkey = RegistryEvent::MasterPubkey {
                    master_pubkey: master_key.public(),
                };
                // master key should be uploaded as worker
                self.state.worker_egress.send(&master_pubkey);
            }
        } else {
            // dispatch the master key to the newly-registered gatekeeper using master key
            // if this pRuntime is the newly-registered gatekeeper himself,
            // its egress is still in dummy mode since we will tick the state later
            if let Some(master_key) = &self.state.master_key {
                info!("Gatekeeper: try dispatch master key");
                let derived_key = master_key
                    .derive_sr25519_pair(&[&crate::generate_random_info()])
                    .expect("should not fail with valid info");
                let my_ecdh_key = derived_key
                    .derive_ecdh_key()
                    .expect("ecdh key derivation should never failed with valid master key");
                let secret = ecdh::agree(&my_ecdh_key, &event.ecdh_pubkey.0)
                    .expect("should never fail with valid ecdh key");
                let iv = aead::generate_iv(&self.state.last_random_number);
                let mut data = master_key.dump_secret_key().to_vec();

                match aead::encrypt(&iv, &secret, &mut data) {
                    Ok(_) => {
                        self.state.push_gatekeeper_message(
                            GatekeeperEvent::dispatch_master_key_event(
                                event.pubkey.clone(),
                                my_ecdh_key
                                    .public()
                                    .as_ref()
                                    .try_into()
                                    .expect("should never fail given pubkey with correct length"),
                                data,
                                iv,
                            ),
                        );
                    }
                    Err(e) => error!("Failed to encrypt master key: {:?}", e),
                }
            }
        }

        // tick the registration state and enable message sending
        // for the newly-registered gatekeeper, NewGatekeeperEvent comes before DispatchMasterKeyEvent
        // so it's necessary to check the existence of master key here
        if self.state.possess_master_key() && my_pubkey == event.pubkey {
            self.state.register_on_chain();
        }
    }

    /// Process encrypted master key from mq
    fn process_dispatch_master_key_event(
        &mut self,
        origin: MessageOrigin,
        event: DispatchMasterKeyEvent,
    ) {
        if !origin.is_gatekeeper() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        };

        let my_pubkey = self.state.identity_key.public();
        if my_pubkey == event.dest {
            let my_ecdh_key = self
                .state
                .identity_key
                .derive_ecdh_key()
                .expect("Should never failed with valid identity key; qed.");
            // info!("PUBKEY TO AGREE: {}", hex::encode(event.ecdh_pubkey.0));
            let secret = ecdh::agree(&my_ecdh_key, &event.ecdh_pubkey.0)
                .expect("Should never failed with valid ecdh key; qed.");

            let mut master_key_buff = event.encrypted_master_key.clone();
            let master_key = match aead::decrypt(&event.iv, &secret, &mut master_key_buff[..]) {
                Err(e) => panic!("Failed to decrypt dispatched master key: {:?}", e),
                Ok(k) => k,
            };

            let master_pair = sr25519::Pair::from_seed_slice(&master_key)
                .expect("Master key seed must be correct; qed.");
            info!("Gatekeeper: successfully decrypt received master key, prepare to reboot");
            self.state.set_master_key(master_pair, true);
        }
    }

    pub fn is_registered(&self) -> bool {
        self.worker_state.registered
    }

    pub fn gatekeeper_status(&self) -> GatekeeperStatus {
        let active = self.gatekeeper.is_registered_on_chain();
        let has_key = self.gatekeeper.possess_master_key();
        let role = match (has_key, active) {
            (true, true) => GatekeeperRole::Active,
            (true, false) => GatekeeperRole::Dummy,
            _ => GatekeeperRole::None,
        };
        let master_public_key = self
            .gatekeeper
            .master_public_key()
            .map(|k| hex::encode(&k))
            .unwrap_or_default();
        GatekeeperStatus {
            role: role.into(),
            master_public_key,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    QueryReceipt { command_index: CommandIndex },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    QueryReceipt {
        receipt: TransactionReceipt,
    },
    GetWorkerEgress {
        length: usize,
        encoded_egress_b64: String,
    },
    Error(#[serde(with = "serde_anyhow")] anyhow::Error),
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
            .unwrap_or(Vec::new());

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

pub mod serde_anyhow {
    use crate::std::string::{String, ToString};
    use anyhow::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Error, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = value.to_string();
        String::serialize(&s, serializer)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Error, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Error::msg(s))
    }
}

#[cfg(feature = "tests")]
pub fn run_all_tests() {
    gk::tests::run_all_tests();
}
