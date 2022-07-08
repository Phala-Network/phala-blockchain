use super::{RotatedMasterKey, TransactionError, TypedReceiver, WorkerState};
use chain::pallet_fat::ClusterRegistryEvent;
use chain::pallet_registry::GatekeeperRegistryEvent;
use phala_crypto::{
    aead, key_share,
    sr25519::{Persistence, Sr25519SecretKey, KDF},
};
use phala_mq::{traits::MessageChannel, MessageDispatcher, Sr25519Signer};
use phala_serde_more as more;
use phala_types::{
    contract::{messaging::ClusterEvent, ContractClusterId},
    messaging::{
        BatchRotateMasterKeyEvent, ClusterOperation, DispatchMasterKeyHistoryEvent, EncryptedKey,
        GatekeeperEvent, KeyDistribution, MessageOrigin, MiningInfoUpdateEvent, MiningReportEvent,
        RandomNumber, RandomNumberEvent, RotateMasterKeyEvent, SettleInfo, SystemEvent,
        WorkerEvent, WorkerEventWithKey,
    },
    EcdhPublicKey, WorkerPublicKey,
};
use serde::{Deserialize, Serialize};
use sp_core::{hashing, sr25519, Pair};

use crate::types::BlockInfo;

use std::{
    collections::{BTreeMap, VecDeque},
    convert::TryInto,
};

use fixed_macro::types::U64F64 as fp;
use log::{debug, info, trace};
use phactory_api::prpc as pb;
pub use tokenomic::{FixedPoint, TokenomicInfo};

/// Block interval to generate pseudo-random on chain
///
/// WARNING: this interval need to be large enough considering the latency of mq
const VRF_INTERVAL: u32 = 5;

const MASTER_KEY_SHARING_SALT: &[u8] = b"master_key_sharing";

// pesudo_random_number = blake2_256(last_random_number, block_number, derived_master_key)
//
// NOTICE: we abandon the random number involving master key signature, since the malleability of sr25519 signature
// refer to: https://github.com/w3f/schnorrkel/blob/34cdb371c14a73cbe86dfd613ff67d61662b4434/old/README.md#a-note-on-signature-malleability
fn next_random_number(
    master_key: &sr25519::Pair,
    block_number: chain::BlockNumber,
    last_random_number: RandomNumber,
) -> RandomNumber {
    let derived_random_key = master_key
        .derive_sr25519_pair(&[b"random_number"])
        .expect("should not fail with valid info");

    let mut buf: Vec<u8> = last_random_number.to_vec();
    buf.extend(block_number.to_be_bytes().iter().copied());
    buf.extend(derived_random_key.dump_secret_key().iter().copied());

    hashing::blake2_256(buf.as_ref())
}

fn get_cluster_key(master_key: &sr25519::Pair, cluster: &ContractClusterId) -> sr25519::Pair {
    master_key
        .derive_sr25519_pair(&[b"cluster_key", cluster.as_bytes()])
        .expect("should not fail with valid info")
}

#[cfg(feature = "gk-stat")]
#[derive(Debug, Default, Serialize, Deserialize)]
struct WorkerStat {
    last_heartbeat_for_block: chain::BlockNumber,
    last_heartbeat_at_block: chain::BlockNumber,
    last_gk_responsive_event: i32,
    last_gk_responsive_event_at_block: chain::BlockNumber,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerInfo {
    state: WorkerState,
    waiting_heartbeats: VecDeque<chain::BlockNumber>,
    unresponsive: bool,
    tokenomic: TokenomicInfo,
    heartbeat_flag: bool,
    #[cfg(feature = "gk-stat")]
    stat: WorkerStat,
}

impl WorkerInfo {
    fn new(pubkey: WorkerPublicKey) -> Self {
        Self {
            state: WorkerState::new(pubkey),
            waiting_heartbeats: Default::default(),
            unresponsive: false,
            tokenomic: Default::default(),
            heartbeat_flag: false,
            #[cfg(feature = "gk-stat")]
            stat: Default::default(),
        }
    }

    pub fn tokenomic_info(&self) -> &TokenomicInfo {
        &self.tokenomic
    }

    pub fn pubkey(&self) -> &WorkerPublicKey {
        &self.state.pubkey
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Gatekeeper<MsgChan> {
    /// The current master key in use
    #[serde(with = "more::key_bytes")]
    master_key: sr25519::Pair,
    /// This will be switched once when the first master key is uploaded
    master_pubkey_on_chain: bool,
    /// Unregistered GK will sync all the GK messages silently
    registered_on_chain: bool,
    #[serde(with = "more::scale_bytes")]
    master_key_history: Vec<RotatedMasterKey>,
    egress: MsgChan, // TODO.kevin: syncing the egress state while migrating.
    gatekeeper_events: TypedReceiver<GatekeeperEvent>,
    cluster_events: TypedReceiver<ClusterEvent>,
    // Randomness
    last_random_number: RandomNumber,
    iv_seq: u64,
    pub(crate) mining_economics: MiningEconomics<MsgChan>,
}

impl<MsgChan> Gatekeeper<MsgChan>
where
    MsgChan: MessageChannel<Signer = Sr25519Signer> + Clone,
{
    pub fn new(
        master_key_history: Vec<RotatedMasterKey>,
        recv_mq: &mut MessageDispatcher,
        egress: MsgChan,
    ) -> Self {
        let master_key = sr25519::Pair::restore_from_secret_key(
            &master_key_history
                .first()
                .expect("empty master key history")
                .secret,
        );
        egress.set_dummy(true);

        Self {
            master_key,
            master_pubkey_on_chain: false,
            registered_on_chain: false,
            master_key_history,
            egress: egress.clone(),
            gatekeeper_events: recv_mq.subscribe_bound(),
            cluster_events: recv_mq.subscribe_bound(),
            last_random_number: [0_u8; 32],
            iv_seq: 0,
            mining_economics: MiningEconomics::new(recv_mq, egress),
        }
    }

    fn generate_iv(&mut self, block_number: chain::BlockNumber) -> aead::IV {
        let derived_key = self
            .master_key
            .derive_sr25519_pair(&[b"iv_generator"])
            .expect("should not fail with valid info");

        let mut buf: Vec<u8> = Vec::new();
        buf.extend(derived_key.dump_secret_key().iter().copied());
        buf.extend(block_number.to_be_bytes().iter().copied());
        buf.extend(self.iv_seq.to_be_bytes().iter().copied());
        self.iv_seq += 1;

        let hash = hashing::blake2_256(buf.as_ref());
        hash[0..12]
            .try_into()
            .expect("should never fail given correct length; qed.")
    }

    pub fn register_on_chain(&mut self) {
        info!("Gatekeeper: register on chain");
        self.egress.set_dummy(false);
        self.registered_on_chain = true;
    }

    pub fn unregister_on_chain(&mut self) {
        info!("Gatekeeper: unregister on chain");
        self.egress.set_dummy(true);
        self.registered_on_chain = false;
    }

    pub fn registered_on_chain(&self) -> bool {
        self.registered_on_chain
    }

    pub fn master_pubkey_uploaded(&mut self, master_pubkey: sr25519::Public) {
        assert!(
            self.master_key.public() == master_pubkey,
            "local and on-chain master key mismatch"
        );
        self.master_pubkey_on_chain = true;
    }

    pub fn master_pubkey(&self) -> sr25519::Public {
        self.master_key.public()
    }

    pub fn master_key_history(&self) -> &Vec<RotatedMasterKey> {
        &self.master_key_history
    }

    /// Return whether the history is really updated
    pub fn set_master_key_history(&mut self, master_key_history: &Vec<RotatedMasterKey>) -> bool {
        if master_key_history.len() <= self.master_key_history.len() {
            return false;
        }
        self.master_key_history = master_key_history.clone();
        true
    }

    /// Append the rotated key to Gatekeeper's master key history, return whether the history is really updated
    pub fn append_master_key(&mut self, rotated_master_key: RotatedMasterKey) -> bool {
        if !self.master_key_history.contains(&rotated_master_key) {
            // the rotation id must be in order
            assert!(
                rotated_master_key.rotation_id == self.master_key_history.len() as u64,
                "Gatekeeper Master key history corrupted"
            );
            self.master_key_history.push(rotated_master_key.clone());
            return true;
        }
        false
    }

    /// Update the master key and Gatekeeper mq
    pub fn switch_master_key(
        &mut self,
        rotation_id: u64,
        block_height: chain::BlockNumber,
    ) -> bool {
        let raw_key = self.master_key_history.get(rotation_id as usize);
        if raw_key.is_none() {
            return false;
        }

        let raw_key = raw_key.expect("checked; qed.");
        assert!(
            raw_key.rotation_id == rotation_id && raw_key.block_height == block_height,
            "Gatekeeper Master key history corrupted"
        );
        let new_master_key = sr25519::Pair::restore_from_secret_key(&raw_key.secret);
        // send the RotatedMasterPubkey event with old master key
        let master_pubkey = new_master_key.public();
        self.egress
            .push_message(&GatekeeperRegistryEvent::RotatedMasterPubkey {
                rotation_id: raw_key.rotation_id,
                master_pubkey,
            });

        self.master_key = new_master_key.clone();
        self.egress.set_signer(self.master_key.clone().into());
        true
    }

    pub fn share_master_key(
        &mut self,
        pubkey: &WorkerPublicKey,
        ecdh_pubkey: &EcdhPublicKey,
        block_number: chain::BlockNumber,
    ) {
        if self.master_key_history.len() > 1 {
            self.share_master_key_history(pubkey, ecdh_pubkey, block_number);
        } else {
            self.share_latest_master_key(pubkey, ecdh_pubkey, block_number);
        }
    }

    pub fn share_latest_master_key(
        &mut self,
        pubkey: &WorkerPublicKey,
        ecdh_pubkey: &EcdhPublicKey,
        block_number: chain::BlockNumber,
    ) {
        info!("Gatekeeper: try dispatch master key");
        let master_key = self.master_key.dump_secret_key();
        let encrypted_key = self.encrypt_key_to(
            &[MASTER_KEY_SHARING_SALT],
            ecdh_pubkey,
            &master_key,
            block_number,
        );
        self.egress.push_message(
            &KeyDistribution::<chain::BlockNumber>::master_key_distribution(
                *pubkey,
                encrypted_key.ecdh_pubkey,
                encrypted_key.encrypted_key,
                encrypted_key.iv,
            ),
        );
    }

    pub fn share_master_key_history(
        &mut self,
        pubkey: &WorkerPublicKey,
        ecdh_pubkey: &EcdhPublicKey,
        block_number: chain::BlockNumber,
    ) {
        info!("Gatekeeper: try dispatch all historical master keys");
        let encrypted_master_key_history = self
            .master_key_history
            .clone()
            .iter()
            .map(|key| {
                (
                    key.rotation_id,
                    key.block_height,
                    self.encrypt_key_to(
                        &[MASTER_KEY_SHARING_SALT],
                        ecdh_pubkey,
                        &key.secret,
                        block_number,
                    ),
                )
            })
            .collect();
        self.egress.push_message(&KeyDistribution::MasterKeyHistory(
            DispatchMasterKeyHistoryEvent {
                dest: pubkey.clone(),
                encrypted_master_key_history,
            },
        ));
    }

    pub fn process_master_key_rotation_request(
        &mut self,
        block: &BlockInfo,
        event: RotateMasterKeyEvent,
        identity_key: sr25519::Pair,
    ) {
        let new_master_key = crate::new_sr25519_key();
        let secret_key = new_master_key.dump_secret_key();
        let secret_keys: BTreeMap<_, _> = event
            .gk_identities
            .into_iter()
            .map(|gk_identity| {
                let encrypted_key = self.encrypt_key_to(
                    &[MASTER_KEY_SHARING_SALT],
                    &gk_identity.ecdh_pubkey,
                    &secret_key,
                    block.block_number,
                );
                (gk_identity.pubkey, encrypted_key)
            })
            .collect();
        let mut event = BatchRotateMasterKeyEvent {
            rotation_id: event.rotation_id,
            secret_keys,
            sender: identity_key.public(),
            sig: vec![],
        };
        let data_to_sign = event.data_be_signed();
        event.sig = identity_key.sign(&data_to_sign).0.to_vec();
        self.egress
            .push_message(&KeyDistribution::<chain::BlockNumber>::MasterKeyRotation(
                event,
            ));
    }

    pub fn process_messages(&mut self, block: &BlockInfo<'_>) {
        if !self.master_pubkey_on_chain {
            info!(
                "Gatekeeper: not handle the messages because Gatekeeper has not launched on chain"
            );
            return;
        }

        debug!("Gatekeeper: processing block {}", block.block_number);
        loop {
            let ok = phala_mq::select_ignore_errors! {
                (event, origin) = self.gatekeeper_events => {
                    self.process_gatekeeper_event(origin, event);
                },
                (event, origin) = self.cluster_events => {
                    if let Err(err) = self.process_cluster_event(block, origin, event) {
                        error!(
                            "Failed to process cluster event: {:?}",
                            err
                        );
                    };
                },
            };
            if ok.is_none() {
                // All messages processed
                break;
            }
        }

        self.mining_economics.process_messages(block);

        debug!("Gatekeeper: processed block {}", block.block_number);
    }

    fn process_gatekeeper_event(&mut self, origin: MessageOrigin, event: GatekeeperEvent) {
        debug!("Incoming gatekeeper event: {:?}", event);
        match event {
            GatekeeperEvent::NewRandomNumber(random_number_event) => {
                self.process_random_number_event(origin, random_number_event)
            }
            GatekeeperEvent::TokenomicParametersChanged(_params) => {
                // Handled by MiningEconomics
            }
            GatekeeperEvent::RepairV => {
                // Handled by MiningEconomics
            }
            GatekeeperEvent::PhalaLaunched => {
                // Handled by MiningEconomics
            }
            GatekeeperEvent::UnrespFix => {
                // Handled by MiningEconomics
            }
        }
    }

    /// Manually encrypt the secret key for sharing
    ///
    /// The encrypted key intends to be shared through public channel in a broadcast way,
    /// so it is possible to share one key to multiple parties in one message.
    ///
    /// For end-to-end secret sharing, use `SecretMessageChannel`.
    fn encrypt_key_to(
        &mut self,
        key_derive_info: &[&[u8]],
        ecdh_pubkey: &EcdhPublicKey,
        secret_key: &Sr25519SecretKey,
        block_number: chain::BlockNumber,
    ) -> EncryptedKey {
        let iv = self.generate_iv(block_number);
        let (ecdh_pubkey, encrypted_key) = key_share::encrypt_secret_to(
            &self.master_key,
            key_derive_info,
            &ecdh_pubkey.0,
            secret_key,
            &iv,
        )
        .expect("should never fail with valid master key; qed.");
        EncryptedKey {
            ecdh_pubkey: sr25519::Public(ecdh_pubkey),
            encrypted_key,
            iv,
        }
    }

    fn process_cluster_event(
        &mut self,
        block: &BlockInfo<'_>,
        origin: MessageOrigin,
        event: ClusterEvent,
    ) -> Result<(), TransactionError> {
        info!("Incoming cluster event: {:?}", event);
        match event {
            ClusterEvent::DeployCluster { cluster, workers } => {
                if !origin.is_pallet() {
                    error!("Attempt to deploy cluster from bad origin");
                    return Err(TransactionError::BadOrigin);
                }

                // first, update the on-chain cluster pubkey
                let cluster_key = get_cluster_key(&self.master_key, &cluster);
                let cluster_pubkey = cluster_key.public();
                self.egress
                    .push_message(&ClusterRegistryEvent::PubkeyAvailable {
                        cluster,
                        pubkey: cluster_pubkey,
                    });
                // then distribute cluster key to all workers in one event
                // the on-chain deployment state should be updated by assigned workers
                // TODO.shelven: set up expiration
                let secret_key = cluster_key.dump_secret_key();
                let secret_keys: BTreeMap<_, _> = workers
                    .into_iter()
                    .map(|worker| {
                        let encrypted_key = self.encrypt_key_to(
                            &[b"cluster_key_sharing"],
                            &worker.ecdh_pubkey,
                            &secret_key,
                            block.block_number,
                        );
                        (worker.pubkey, encrypted_key)
                    })
                    .collect();
                self.egress
                    .push_message(&ClusterOperation::batch_distribution(
                        secret_keys,
                        cluster,
                        0,
                    ));
                Ok(())
            }
        }
    }

    /// Verify on-chain random number
    fn process_random_number_event(&mut self, origin: MessageOrigin, event: RandomNumberEvent) {
        if !origin.is_gatekeeper() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        };

        // determine which master key to use
        // the random number may be generated with the latest master key or last key that was just rotated
        let master_key = if self
            .master_key_history
            .last()
            .expect("at least one key in gk; qed")
            .block_height
            > event.block_number
        {
            let len = self.master_key_history.len();
            assert!(len >= 2, "no proper key for random generation");
            let secret = self.master_key_history[len - 2].secret;
            sr25519::Pair::restore_from_secret_key(&secret)
        } else {
            self.master_key.clone()
        };

        let expect_random =
            next_random_number(&master_key, event.block_number, event.last_random_number);
        // instead of checking the origin, we directly verify the random to avoid access storage
        if expect_random != event.random_number {
            error!("Fatal error: Expect random number {:?}", expect_random);
            #[cfg(not(feature = "shadow-gk"))]
            panic!("GK state poisoned");
        }
    }

    pub fn emit_random_number(&mut self, block_number: chain::BlockNumber) {
        if block_number % VRF_INTERVAL != 0 {
            return;
        }

        let random_number =
            next_random_number(&self.master_key, block_number, self.last_random_number);
        info!(
            "Gatekeeper: emit random number {} in block {}",
            hex::encode(&random_number),
            block_number
        );
        self.egress
            .push_message(&GatekeeperEvent::new_random_number(
                block_number,
                random_number,
                self.last_random_number,
            ));
        self.last_random_number = random_number;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinanceEvent {
    MiningStart,
    MiningStop,
    HeartbeatChallenge,
    Heartbeat { payout: FixedPoint },
    EnterUnresponsive,
    ExitUnresponsive,
    RecoverV,
}

impl FinanceEvent {
    pub fn event_string(&self) -> &'static str {
        match self {
            FinanceEvent::MiningStart => "mining_start",
            FinanceEvent::MiningStop => "mining_stop",
            FinanceEvent::HeartbeatChallenge => "heartbeat_challenge",
            FinanceEvent::Heartbeat { .. } => "heartbeat",
            FinanceEvent::EnterUnresponsive => "enter_unresponsive",
            FinanceEvent::ExitUnresponsive => "exit_unresponsive",
            FinanceEvent::RecoverV => "recover_v",
        }
    }

    pub fn payout(&self) -> FixedPoint {
        if let Self::Heartbeat { payout } = self {
            *payout
        } else {
            fp!(0)
        }
    }
}

pub trait FinanceEventListener {
    fn on_finance_event(&mut self, event: FinanceEvent, state: &WorkerInfo);
}

impl FinanceEventListener for () {
    fn on_finance_event(&mut self, _event: FinanceEvent, _state: &WorkerInfo) {}
}

impl<F: FnMut(FinanceEvent, &WorkerInfo)> FinanceEventListener for F {
    fn on_finance_event(&mut self, event: FinanceEvent, state: &WorkerInfo) {
        (self)(event, state);
    }
}

#[derive(Serialize, Deserialize)]
pub struct MiningEconomics<MsgChan> {
    egress: MsgChan, // TODO.kevin: syncing the egress state while migrating.
    mining_events: TypedReceiver<MiningReportEvent>,
    system_events: TypedReceiver<SystemEvent>,
    gatekeeper_events: TypedReceiver<GatekeeperEvent>,
    workers: BTreeMap<WorkerPublicKey, WorkerInfo>,
    tokenomic_params: tokenomic::Params,
    /// Indicates a set of update is enabled on-chain
    /// - Remove payout delta V limitation
    ///   (https://github.com/Phala-Network/phala-blockchain/issues/693)
    /// - Fix issue 676 (https://github.com/Phala-Network/phala-blockchain/issues/676)
    #[serde(default)]
    phala_launched: bool,
    /// Indicates if the payout duration problem in unresponsive state if fixed
    #[serde(default)]
    unresp_fix: bool,
}

#[test]
fn test_restore_phala_launched() {
    #[derive(Serialize, Deserialize)]
    struct MiningEconomics0 {
        tokenomic_params: u32,
    }

    #[derive(Serialize, Deserialize)]
    struct MiningEconomics1 {
        tokenomic_params: u32,
        #[serde(default)]
        phala_launched: bool,
    }

    let checkpoint = serde_cbor::to_vec(&MiningEconomics0 {
        tokenomic_params: 1,
    })
    .unwrap();

    let state: MiningEconomics1 = serde_cbor::from_slice(&checkpoint).unwrap();
    assert!(!state.phala_launched);
}

#[cfg(feature = "gk-stat")]
impl From<&WorkerStat> for pb::WorkerStat {
    fn from(stat: &WorkerStat) -> Self {
        Self {
            last_heartbeat_for_block: stat.last_heartbeat_for_block,
            last_heartbeat_at_block: stat.last_heartbeat_at_block,
            last_gk_responsive_event: stat.last_gk_responsive_event,
            last_gk_responsive_event_at_block: stat.last_gk_responsive_event_at_block,
        }
    }
}

impl From<&WorkerInfo> for pb::WorkerState {
    fn from(info: &WorkerInfo) -> Self {
        pb::WorkerState {
            registered: info.state.registered,
            unresponsive: info.unresponsive,
            bench_state: info.state.bench_state.as_ref().map(|state| pb::BenchState {
                start_block: state.start_block,
                start_time: state.start_time,
                duration: state.duration,
            }),
            mining_state: info
                .state
                .mining_state
                .as_ref()
                .map(|state| pb::MiningState {
                    session_id: state.session_id,
                    paused: matches!(state.state, super::MiningState::Paused),
                    start_time: state.start_time,
                }),
            waiting_heartbeats: info.waiting_heartbeats.iter().copied().collect(),
            #[cfg(feature = "gk-stat")]
            stat: Some((&info.stat).into()),
            #[cfg(not(feature = "gk-stat"))]
            stat: None,
            tokenomic_info: Some(info.tokenomic.clone().into()),
        }
    }
}

impl<MsgChan: MessageChannel<Signer = Sr25519Signer>> MiningEconomics<MsgChan> {
    pub fn new(recv_mq: &mut MessageDispatcher, egress: MsgChan) -> Self {
        MiningEconomics {
            egress,
            mining_events: recv_mq.subscribe_bound(),
            system_events: recv_mq.subscribe_bound(),
            gatekeeper_events: recv_mq.subscribe_bound(),
            workers: Default::default(),
            tokenomic_params: tokenomic::test_params(),
            phala_launched: false,
            unresp_fix: false,
        }
    }

    pub fn dump_workers_state(&self) -> Vec<(WorkerPublicKey, pb::WorkerState)> {
        self.workers
            .values()
            .map(|info| (info.state.pubkey, info.into()))
            .collect()
    }

    pub fn worker_state(&self, pubkey: &WorkerPublicKey) -> Option<pb::WorkerState> {
        self.workers.get(pubkey).map(Into::into)
    }

    pub fn process_messages(&mut self, block: &BlockInfo<'_>) {
        self.process_messages_with_event_listener(block, &mut ());
    }

    pub fn process_messages_with_event_listener(
        &mut self,
        block: &BlockInfo<'_>,
        event_listener: &mut impl FinanceEventListener,
    ) {
        let sum_share = self.sum_share();

        let mut processor = MiningMessageProcessor {
            state: self,
            block,
            report: MiningInfoUpdateEvent::new(block.block_number, block.now_ms),
            event_listener,
            sum_share,
        };

        processor.process();

        let report = processor.report;

        if !report.is_empty() {
            debug!(target: "mining", "Report: {:?}", report);
            self.egress.push_message(&report);
        }
    }

    pub fn sum_share(&self) -> FixedPoint {
        self.workers
            .values()
            .filter(|info| {
                if self.phala_launched {
                    !info.unresponsive && info.state.mining_state.is_some()
                } else {
                    !info.unresponsive
                }
            })
            .map(|info| info.tokenomic.share())
            .sum()
    }
}

struct MiningMessageProcessor<'a, MsgChan> {
    state: &'a mut MiningEconomics<MsgChan>,
    block: &'a BlockInfo<'a>,
    report: MiningInfoUpdateEvent<chain::BlockNumber>,
    event_listener: &'a mut dyn FinanceEventListener,
    sum_share: FixedPoint,
}

impl<MsgChan> MiningMessageProcessor<'_, MsgChan>
where
    MsgChan: MessageChannel<Signer = Sr25519Signer>,
{
    fn process(&mut self) {
        self.prepare();
        loop {
            let ok = phala_mq::select! {
                message = self.state.mining_events => match message {
                    Ok((_, event, origin)) => {
                        trace!(target: "mining", "Processing mining report: {:?}, origin: {}",  event, origin);
                        self.process_mining_report(origin, event);
                    }
                    Err(e) => {
                        error!(target: "mining", "Read message failed: {:?}", e);
                    }
                },
                message = self.state.system_events => match message {
                    Ok((_, event, origin)) => {
                        trace!(target: "mining", "Processing system event: {:?}, origin: {}",  event, origin);
                        self.process_system_event(origin, event);
                    }
                    Err(e) => {
                        error!(target: "mining", "Read message failed: {:?}", e);
                    }
                },
                message = self.state.gatekeeper_events => match message {
                    Ok((_, event, origin)) => {
                        self.process_gatekeeper_event(origin, event);
                    }
                    Err(e) => {
                        error!(target: "mining", "Read message failed: {:?}", e);
                    }
                },
            };
            if ok.is_none() {
                // All messages processed
                break;
            }
        }
        self.block_post_process();
    }

    fn prepare(&mut self) {
        for worker in self.state.workers.values_mut() {
            worker.heartbeat_flag = false;
        }
    }

    fn block_post_process(&mut self) {
        for worker_info in self.state.workers.values_mut() {
            trace!(target: "mining",
                "[{}] block_post_process",
                hex::encode(&worker_info.state.pubkey)
            );
            let mut tracker = WorkerSMTracker::new(&mut worker_info.waiting_heartbeats);
            worker_info
                .state
                .on_block_processed(self.block, &mut tracker);

            if worker_info.state.mining_state.is_none() {
                trace!(
                    target: "mining",
                    "[{}] Mining already stopped, do nothing.",
                    hex::encode(&worker_info.state.pubkey)
                );
                continue;
            }

            if worker_info.unresponsive {
                if worker_info.heartbeat_flag {
                    trace!(
                        target: "mining",
                        "[{}] case5: Unresponsive, successful heartbeat.",
                        hex::encode(&worker_info.state.pubkey)
                    );
                    worker_info.unresponsive = false;
                    self.report
                        .recovered_to_online
                        .push(worker_info.state.pubkey);
                    #[cfg(feature = "gk-stat")]
                    {
                        worker_info.stat.last_gk_responsive_event =
                            pb::ResponsiveEvent::ExitUnresponsive as _;
                        worker_info.stat.last_gk_responsive_event_at_block =
                            self.block.block_number;
                    }
                    self.event_listener
                        .on_finance_event(FinanceEvent::ExitUnresponsive, worker_info);
                }
            } else if let Some(&hb_sent_at) = worker_info.waiting_heartbeats.get(0) {
                if self.block.block_number - hb_sent_at
                    > self.state.tokenomic_params.heartbeat_window
                {
                    trace!(
                        target: "mining",
                        "[{}] case3: Idle, heartbeat failed, current={} waiting for {}.",
                        hex::encode(&worker_info.state.pubkey),
                        self.block.block_number,
                        hb_sent_at
                    );
                    self.report.offline.push(worker_info.state.pubkey);
                    worker_info.unresponsive = true;

                    #[cfg(feature = "gk-stat")]
                    {
                        worker_info.stat.last_gk_responsive_event =
                            pb::ResponsiveEvent::EnterUnresponsive as _;
                        worker_info.stat.last_gk_responsive_event_at_block =
                            self.block.block_number;
                    }
                    self.event_listener
                        .on_finance_event(FinanceEvent::EnterUnresponsive, worker_info);
                }
            }

            let params = &self.state.tokenomic_params;
            if worker_info.unresponsive {
                trace!(
                    target: "mining",
                    "[{}] case3/case4: Idle, heartbeat failed or Unresponsive, no event",
                    hex::encode(&worker_info.state.pubkey)
                );
                worker_info
                    .tokenomic
                    .update_v_slash(params, self.block.block_number);
            } else if !worker_info.heartbeat_flag {
                trace!(
                    target: "mining",
                    "[{}] case1: Idle, no event",
                    hex::encode(&worker_info.state.pubkey)
                );
                worker_info.tokenomic.update_v_idle(params);
            }
        }
    }

    fn process_mining_report(&mut self, origin: MessageOrigin, event: MiningReportEvent) {
        let worker_pubkey = if let MessageOrigin::Worker(pubkey) = origin {
            pubkey
        } else {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        };
        match event {
            MiningReportEvent::Heartbeat {
                session_id,
                challenge_block,
                challenge_time,
                iterations,
            } => {
                let worker_info = match self.state.workers.get_mut(&worker_pubkey) {
                    Some(info) => info,
                    None => {
                        error!(
                            target: "mining",
                            "Unknown worker {} sent a {:?}",
                            hex::encode(worker_pubkey),
                            event
                        );
                        return;
                    }
                };

                #[cfg(feature = "gk-stat")]
                {
                    worker_info.stat.last_heartbeat_at_block = self.block.block_number;
                    worker_info.stat.last_heartbeat_for_block = challenge_block;
                }

                if Some(&challenge_block) != worker_info.waiting_heartbeats.get(0) {
                    error!(target: "mining", "Fatal error: Unexpected heartbeat {:?}", event);
                    error!(target: "mining", "Sent from worker {}", hex::encode(worker_pubkey));
                    error!(target: "mining", "Waiting heartbeats {:#?}", worker_info.waiting_heartbeats);
                    // The state has been poisoned. Make no sence to keep moving on.
                    panic!("GK or Worker state poisoned");
                }

                // The oldest one comfirmed.
                let _ = worker_info.waiting_heartbeats.pop_front();

                let mining_state = if let Some(state) = &worker_info.state.mining_state {
                    state
                } else {
                    trace!(
                        target: "mining",
                        "[{}] Mining already stopped, ignore the heartbeat.",
                        hex::encode(&worker_info.state.pubkey)
                    );
                    return;
                };

                if session_id != mining_state.session_id {
                    trace!(
                        target: "mining",
                        "[{}] Heartbeat response to previous mining sessions, ignore it.",
                        hex::encode(&worker_info.state.pubkey)
                    );
                    return;
                }

                worker_info.heartbeat_flag = true;

                let tokenomic = &mut worker_info.tokenomic;
                tokenomic.update_p_instant(self.block.now_ms, iterations);
                tokenomic.challenge_time_last = challenge_time;
                tokenomic.iteration_last = iterations;

                let payout = if worker_info.unresponsive {
                    trace!(
                        target: "mining",
                        "[{}] heartbeat handling case5: Unresponsive, successful heartbeat.",
                        hex::encode(&worker_info.state.pubkey)
                    );
                    if self.state.unresp_fix {
                        worker_info
                            .tokenomic
                            .update_v_recover(self.block.now_ms, self.block.block_number);
                    }
                    fp!(0)
                } else {
                    trace!(
                        target: "mining",
                        "[{}] heartbeat handling case2: Idle, successful heartbeat, report to pallet",
                        hex::encode(&worker_info.state.pubkey)
                    );
                    let (payout, treasury) = worker_info.tokenomic.update_v_heartbeat(
                        &self.state.tokenomic_params,
                        self.sum_share,
                        self.block.now_ms,
                        self.block.block_number,
                        self.state.phala_launched,
                    );

                    // NOTE: keep the reporting order (vs the one while mining stop).
                    self.report.settle.push(SettleInfo {
                        pubkey: worker_pubkey,
                        v: worker_info.tokenomic.v.to_bits(),
                        payout: payout.to_bits(),
                        treasury: treasury.to_bits(),
                    });
                    payout
                };
                self.event_listener
                    .on_finance_event(FinanceEvent::Heartbeat { payout }, worker_info);
            }
        }
    }

    fn process_system_event(&mut self, origin: MessageOrigin, event: SystemEvent) {
        if !origin.is_pallet() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        }

        // Create the worker info on it's first time registered
        if let SystemEvent::WorkerEvent(WorkerEventWithKey {
            pubkey,
            event: WorkerEvent::Registered(_),
        }) = &event
        {
            let _ = self
                .state
                .workers
                .entry(*pubkey)
                .or_insert_with(|| WorkerInfo::new(*pubkey));
        }

        let log_on = log::log_enabled!(log::Level::Debug);
        // TODO.kevin: Avoid unnecessary iteration for WorkerEvents.
        for worker_info in self.state.workers.values_mut() {
            // Replay the event on worker state, and collect the egressed heartbeat into waiting_heartbeats.
            let mut tracker = WorkerSMTracker::new(&mut worker_info.waiting_heartbeats);
            worker_info
                .state
                .process_event(self.block, &event, &mut tracker, log_on);
            if tracker.challenge_received {
                self.event_listener
                    .on_finance_event(FinanceEvent::HeartbeatChallenge, worker_info);
            }
        }

        match &event {
            SystemEvent::WorkerEvent(e) => {
                if let Some(worker) = self.state.workers.get_mut(&e.pubkey) {
                    match &e.event {
                        WorkerEvent::Registered(info) => {
                            worker.tokenomic.confidence_level = info.confidence_level;
                        }
                        WorkerEvent::BenchStart { .. } => {}
                        WorkerEvent::BenchScore(_) => {}
                        WorkerEvent::MiningStart {
                            session_id: _, // Aready recorded by the state machine.
                            init_v,
                            init_p,
                        } => {
                            let v = FixedPoint::from_bits(*init_v);
                            let prev = worker.tokenomic;
                            // NOTE.kevin: To track the heartbeats by global timeline, don't clear the waiting_heartbeats.
                            // worker.waiting_heartbeats.clear();
                            worker.unresponsive = false;
                            worker.tokenomic = TokenomicInfo {
                                v,
                                v_init: v,
                                v_deductible: fp!(0),
                                v_update_at: self.block.now_ms,
                                v_update_block: self.block.block_number,
                                iteration_last: 0,
                                challenge_time_last: self.block.now_ms,
                                p_bench: FixedPoint::from_num(*init_p),
                                p_instant: FixedPoint::from_num(*init_p),
                                confidence_level: prev.confidence_level,

                                #[cfg(feature = "gk-stat")]
                                stat: Default::default(),
                            };
                            self.event_listener
                                .on_finance_event(FinanceEvent::MiningStart, worker);
                        }
                        WorkerEvent::MiningStop => {
                            // TODO.kevin: report the final V?
                            // We may need to report a Stop event in worker.
                            // Then GK report the final V to pallet, when observed the Stop event from worker.
                            // The pallet wait for the final V report in CoolingDown state.
                            // Pallet  ---------(Stop)--------> Worker
                            // Worker  ----(Rest Heartbeats)--> *
                            // Worker  --------(Stopped)------> *
                            // GK      --------(Final V)------> Pallet

                            // Just report the final V ATM.
                            // NOTE: keep the reporting order (vs the one while heartbeat).
                            self.report.settle.push(SettleInfo {
                                pubkey: worker.state.pubkey,
                                v: worker.tokenomic.v.to_bits(),
                                payout: 0,
                                treasury: 0,
                            });
                            self.event_listener
                                .on_finance_event(FinanceEvent::MiningStop, worker);
                        }
                        WorkerEvent::MiningEnterUnresponsive => {}
                        WorkerEvent::MiningExitUnresponsive => {}
                    }
                }
            }
            SystemEvent::HeartbeatChallenge(_) => {}
        }
    }

    fn process_gatekeeper_event(&mut self, origin: MessageOrigin, event: GatekeeperEvent) {
        match event {
            GatekeeperEvent::NewRandomNumber(_random_number_event) => {
                // Handled by Gatekeeper.
            }
            GatekeeperEvent::TokenomicParametersChanged(params) => {
                if origin.is_pallet() {
                    self.state.tokenomic_params = params.into();
                    info!(
                        target: "mining",
                        "Tokenomic parameter updated: {:#?}",
                        &self.state.tokenomic_params
                    );
                }
            }
            GatekeeperEvent::RepairV => {
                if origin.is_pallet() {
                    info!(target: "mining", "Repairing V");
                    // Fixup the V for those workers that have been slashed due to the initial tokenomic parameters
                    // not being applied.
                    //
                    // See below links for more detail:
                    // https://github.com/Phala-Network/phala-blockchain/issues/489
                    // https://github.com/Phala-Network/phala-blockchain/issues/495
                    // https://forum.phala.network/t/topic/2753#timeline
                    // https://forum.phala.network/t/topic/2909
                    for w in self.state.workers.values_mut() {
                        if w.state.mining_state.is_some() && w.tokenomic.v < w.tokenomic.v_init {
                            w.tokenomic.v = w.tokenomic.v_init;
                            self.event_listener
                                .on_finance_event(FinanceEvent::RecoverV, w)
                        }
                    }
                }
            }
            GatekeeperEvent::PhalaLaunched => {
                if origin.is_pallet() {
                    // Fixes:
                    // - https://github.com/Phala-Network/phala-blockchain/issues/693
                    // - https://github.com/Phala-Network/phala-blockchain/issues/676
                    self.state.phala_launched = true;
                }
            }
            GatekeeperEvent::UnrespFix => {
                if origin.is_pallet() {
                    self.state.unresp_fix = true;
                }
            }
        }
    }
}

struct WorkerSMTracker<'a> {
    waiting_heartbeats: &'a mut VecDeque<chain::BlockNumber>,
    challenge_received: bool,
}

impl<'a> WorkerSMTracker<'a> {
    fn new(waiting_heartbeats: &'a mut VecDeque<chain::BlockNumber>) -> Self {
        Self {
            waiting_heartbeats,
            challenge_received: false,
        }
    }
}

impl super::WorkerStateMachineCallback for WorkerSMTracker<'_> {
    fn heartbeat(
        &mut self,
        _session_id: u32,
        challenge_block: runtime::BlockNumber,
        _challenge_time: u64,
        _iterations: u64,
    ) {
        trace!(target: "mining", "Worker should emit heartbeat for {}", challenge_block);
        self.waiting_heartbeats.push_back(challenge_block);
        self.challenge_received = true;
    }
}

mod tokenomic {
    use super::serde_fp;
    pub use fixed::types::U64F64 as FixedPoint;
    use fixed_macro::types::U64F64 as fp;
    use fixed_sqrt::FixedSqrt as _;
    use phala_types::messaging::TokenomicParameters;
    use serde::{Deserialize, Serialize};

    fn square(v: FixedPoint) -> FixedPoint {
        v * v
    }

    fn conf_score(level: u8) -> FixedPoint {
        match level {
            1 | 2 | 3 | 128 => fp!(1),
            4 => fp!(0.8),
            5 => fp!(0.7),
            _ => fp!(0),
        }
    }

    #[cfg(feature = "gk-stat")]
    #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct TokenomicStat {
        #[serde(with = "serde_fp")]
        pub last_payout: FixedPoint,
        pub last_payout_at_block: chain::BlockNumber,
        #[serde(with = "serde_fp")]
        pub total_payout: FixedPoint,
        pub total_payout_count: chain::BlockNumber,
        #[serde(with = "serde_fp")]
        pub last_slash: FixedPoint,
        pub last_slash_at_block: chain::BlockNumber,
        #[serde(with = "serde_fp")]
        pub total_slash: FixedPoint,
        pub total_slash_count: chain::BlockNumber,
    }

    #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct TokenomicInfo {
        #[serde(with = "serde_fp")]
        pub v: FixedPoint,
        #[serde(with = "serde_fp")]
        pub v_init: FixedPoint,
        #[serde(with = "serde_fp", alias = "payable")]
        pub v_deductible: FixedPoint,
        pub v_update_at: u64,
        pub v_update_block: u32,
        pub iteration_last: u64,
        pub challenge_time_last: u64,
        #[serde(with = "serde_fp")]
        pub p_bench: FixedPoint,
        #[serde(with = "serde_fp")]
        pub p_instant: FixedPoint,
        pub confidence_level: u8,

        #[cfg(feature = "gk-stat")]
        pub stat: TokenomicStat,
    }

    #[cfg(feature = "gk-stat")]
    impl From<TokenomicStat> for super::pb::TokenomicStat {
        fn from(stat: TokenomicStat) -> Self {
            Self {
                last_payout: stat.last_payout.to_string(),
                last_payout_at_block: stat.last_payout_at_block,
                last_slash: stat.last_slash.to_string(),
                last_slash_at_block: stat.last_slash_at_block,
                total_payout: stat.total_payout.to_string(),
                total_payout_count: stat.total_payout_count,
                total_slash: stat.total_slash.to_string(),
                total_slash_count: stat.total_slash_count,
            }
        }
    }

    impl From<TokenomicInfo> for super::pb::TokenomicInfo {
        fn from(info: TokenomicInfo) -> Self {
            Self {
                v: info.v.to_string(),
                v_init: info.v_init.to_string(),
                v_deductible: info.v_deductible.to_string(),
                share: info.share().to_string(),
                v_update_at: info.v_update_at,
                v_update_block: info.v_update_block,
                iteration_last: info.iteration_last,
                challenge_time_last: info.challenge_time_last,
                p_bench: info.p_bench.to_string(),
                p_instant: info.p_instant.to_string(),
                confidence_level: info.confidence_level as _,
                #[cfg(feature = "gk-stat")]
                stat: Some(info.stat.into()),
                #[cfg(not(feature = "gk-stat"))]
                stat: None,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        #[serde(with = "serde_fp")]
        rho: FixedPoint,
        #[serde(with = "serde_fp")]
        slash_rate: FixedPoint,
        #[serde(with = "serde_fp")]
        budget_per_block: FixedPoint,
        #[serde(with = "serde_fp")]
        v_max: FixedPoint,
        #[serde(with = "serde_fp")]
        cost_k: FixedPoint,
        #[serde(with = "serde_fp")]
        cost_b: FixedPoint,
        #[serde(with = "serde_fp")]
        treasury_ration: FixedPoint,
        #[serde(with = "serde_fp")]
        payout_ration: FixedPoint,
        pub heartbeat_window: u32,
    }

    impl From<TokenomicParameters> for Params {
        fn from(params: TokenomicParameters) -> Self {
            let treasury_ration = FixedPoint::from_bits(params.treasury_ratio);
            let payout_ration = fp!(1) - treasury_ration;
            Params {
                rho: FixedPoint::from_bits(params.rho),
                slash_rate: FixedPoint::from_bits(params.slash_rate),
                budget_per_block: FixedPoint::from_bits(params.budget_per_block),
                v_max: FixedPoint::from_bits(params.v_max),
                cost_k: FixedPoint::from_bits(params.cost_k),
                cost_b: FixedPoint::from_bits(params.cost_b),
                treasury_ration,
                payout_ration,
                heartbeat_window: params.heartbeat_window,
            }
        }
    }

    pub fn test_params() -> Params {
        Params {
            rho: fp!(1.000000666600231),
            slash_rate: fp!(0.0000033333333333333240063),
            budget_per_block: fp!(100),
            v_max: fp!(30000),
            cost_k: fp!(0.000000015815258751856933056),
            cost_b: fp!(0.000033711472602739674283),
            treasury_ration: fp!(0.2),
            payout_ration: fp!(0.8),
            heartbeat_window: 10,
        }
    }

    impl TokenomicInfo {
        /// case1: Idle, no event
        pub fn update_v_idle(&mut self, params: &Params) {
            let cost_idle = params.cost_k * self.p_bench + params.cost_b;
            let perf_multiplier = if self.p_bench == fp!(0) {
                fp!(1)
            } else {
                self.p_instant / self.p_bench
            };
            let delta_v = perf_multiplier * ((params.rho - fp!(1)) * self.v + cost_idle);
            let v = self.v + delta_v;
            self.v = v.min(params.v_max);
            self.v_deductible += delta_v;
        }

        /// case2: Idle, successful heartbeat
        /// return payout
        pub fn update_v_heartbeat(
            &mut self,
            params: &Params,
            sum_share: FixedPoint,
            now_ms: u64,
            block_number: u32,
            full_payout: bool,
        ) -> (FixedPoint, FixedPoint) {
            const NO_UPDATE: (FixedPoint, FixedPoint) = (fp!(0), fp!(0));
            if sum_share == fp!(0) {
                return NO_UPDATE;
            }
            if self.v_deductible == fp!(0) {
                return NO_UPDATE;
            }
            if block_number <= self.v_update_block {
                // May receive more than one heartbeat for a single worker in a single block.
                return NO_UPDATE;
            }
            let share = self.share();
            if share == fp!(0) {
                return NO_UPDATE;
            }
            let blocks = FixedPoint::from_num(block_number - self.v_update_block);
            let budget = share / sum_share * params.budget_per_block * blocks;
            let to_payout = budget * params.payout_ration;
            let to_treasury = budget * params.treasury_ration;

            let actual_payout;
            let actual_treasury;
            if full_payout {
                // With `full_payout`, the miner gets paid with its share directly. However, v can
                // only be deducted up to the v increment since the last payout.
                // (Current behavior)
                actual_payout = to_payout; // w
                actual_treasury = to_treasury;
                let actual_v_deduct = self.v_deductible.max(fp!(0)).min(actual_payout);
                self.v -= actual_v_deduct;
            } else {
                // Without `full_payout`, the miner gets paid up to the v increment to ensure v
                // will not decrease over the time by payout.
                // (Legacy behavior)
                actual_payout = self.v_deductible.max(fp!(0)).min(to_payout); // w
                actual_treasury = (actual_payout / to_payout) * to_treasury; // to_payout > 0
                self.v -= actual_payout;
            }

            self.v_deductible = fp!(0);
            self.v_update_at = now_ms;
            self.v_update_block = block_number;

            #[cfg(feature = "gk-stat")]
            {
                self.stat.last_payout = actual_payout;
                self.stat.last_payout_at_block = block_number;
                self.stat.total_payout += actual_payout;
                self.stat.total_payout_count += 1;
            }

            (actual_payout, actual_treasury)
        }

        pub fn update_v_recover(&mut self, now_ms: u64, block_number: chain::BlockNumber) {
            self.v_deductible = fp!(0);
            self.v_update_at = now_ms;
            self.v_update_block = block_number;

            #[cfg(feature = "gk-stat")]
            {
                self.stat.last_payout = fp!(0);
                self.stat.last_payout_at_block = block_number;
            }
        }

        pub fn update_v_slash(&mut self, params: &Params, block_number: chain::BlockNumber) {
            let slash = self.v * params.slash_rate;
            self.v -= slash;
            self.v_deductible = fp!(0);

            #[cfg(not(feature = "gk-stat"))]
            let _ = block_number;
            #[cfg(feature = "gk-stat")]
            {
                self.stat.last_slash = slash;
                self.stat.last_slash_at_block = block_number;
                self.stat.total_slash += slash;
                self.stat.total_slash_count += 1;
            }
        }

        pub fn share(&self) -> FixedPoint {
            (square(self.v) + square(fp!(2) * self.p_instant * conf_score(self.confidence_level)))
                .sqrt()
        }

        pub fn update_p_instant(&mut self, now: u64, iterations: u64) {
            if now <= self.challenge_time_last {
                return;
            }
            if iterations < self.iteration_last {
                self.iteration_last = iterations;
            }
            let dt = FixedPoint::from_num(now - self.challenge_time_last) / 1000;
            let p = FixedPoint::from_num(iterations - self.iteration_last) / dt * 6; // 6s iterations
            self.p_instant = p.min(self.p_bench * fp!(1.2));
        }
    }
}

mod serde_fp {
    use super::FixedPoint;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &FixedPoint, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bits: u128 = value.to_bits();
        // u128 is not supported by messagepack, so we encode it via bytes
        bits.to_be_bytes().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<FixedPoint, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Deserialize::deserialize(deserializer)?;
        let bits = u128::from_be_bytes(bytes);
        Ok(FixedPoint::from_bits(bits))
    }
}

#[cfg(test)]
pub mod tests {
    use super::{BlockInfo, FixedPoint, MessageChannel, MiningEconomics};
    use fixed_macro::types::U64F64 as fp;
    use parity_scale_codec::{Decode, Encode};
    use phala_mq::{BindTopic, Message, MessageDispatcher, MessageOrigin, Path, Sr25519Signer};
    use phala_types::{messaging as msg, WorkerPublicKey};
    use std::cell::RefCell;

    type MiningInfoUpdateEvent = super::MiningInfoUpdateEvent<chain::BlockNumber>;

    trait DispatcherExt {
        fn dispatch_bound<M: Encode + BindTopic>(&mut self, sender: &MessageOrigin, msg: M);
    }

    impl DispatcherExt for MessageDispatcher {
        fn dispatch_bound<M: Encode + BindTopic>(&mut self, sender: &MessageOrigin, msg: M) {
            let _ = self.dispatch(mk_msg(sender, msg));
        }
    }

    fn mk_msg<M: Encode + BindTopic>(sender: &MessageOrigin, msg: M) -> Message {
        Message {
            sender: sender.clone(),
            destination: M::topic().into(),
            payload: msg.encode(),
        }
    }

    #[derive(Default)]
    struct CollectChannel {
        messages: RefCell<Vec<Message>>,
    }

    impl CollectChannel {
        fn drain(&self) -> Vec<Message> {
            self.messages.borrow_mut().drain(..).collect()
        }

        fn drain_decode<M: Decode + BindTopic>(&self) -> Vec<M> {
            self.drain()
                .into_iter()
                .filter_map(|m| {
                    if &m.destination.path()[..] == &M::topic() {
                        Decode::decode(&mut &m.payload[..]).ok()
                    } else {
                        None
                    }
                })
                .collect()
        }

        fn drain_mining_info_update_event(&self) -> Vec<MiningInfoUpdateEvent> {
            self.drain_decode()
        }

        fn clear(&self) {
            self.messages.borrow_mut().clear();
        }
    }

    impl MessageChannel for CollectChannel {
        type Signer = Sr25519Signer;

        fn push_data(&self, data: Vec<u8>, to: impl Into<Path>) {
            let message = Message {
                sender: MessageOrigin::Gatekeeper,
                destination: to.into().into(),
                payload: data,
            };
            self.messages.borrow_mut().push(message);
        }
    }

    struct Roles {
        mq: MessageDispatcher,
        gk: MiningEconomics<CollectChannel>,
        workers: [WorkerPublicKey; 2],
    }

    impl Roles {
        fn test_roles() -> Roles {
            let mut mq = MessageDispatcher::new();
            let egress = CollectChannel::default();
            let gk = MiningEconomics::new(&mut mq, egress);
            Roles {
                mq,
                gk,
                workers: [
                    WorkerPublicKey::from_raw([0x01u8; 32]),
                    WorkerPublicKey::from_raw([0x02u8; 32]),
                ],
            }
        }

        fn for_worker(&mut self, n: usize) -> ForWorker {
            ForWorker {
                mq: &mut self.mq,
                pubkey: &self.workers[n],
            }
        }

        fn get_worker(&self, n: usize) -> &super::WorkerInfo {
            &self.gk.workers[&self.workers[n]]
        }

        fn get_worker_mut(&mut self, n: usize) -> &mut super::WorkerInfo {
            self.gk.workers.get_mut(&self.workers[n]).unwrap()
        }
    }

    struct ForWorker<'a> {
        mq: &'a mut MessageDispatcher,
        pubkey: &'a WorkerPublicKey,
    }

    impl ForWorker<'_> {
        fn pallet_say(&mut self, event: msg::WorkerEvent) {
            let sender = MessageOrigin::Pallet(b"Pallet".to_vec());
            let message = msg::SystemEvent::new_worker_event(self.pubkey.clone(), event);
            self.mq.dispatch_bound(&sender, message);
        }

        fn say<M: Encode + BindTopic>(&mut self, message: M) {
            let sender = MessageOrigin::Worker(self.pubkey.clone());
            self.mq.dispatch_bound(&sender, message);
        }

        fn challenge(&mut self) {
            use sp_core::U256;

            let sender = MessageOrigin::Pallet(b"Pallet".to_vec());
            // Use the same hash algrithm as the worker to produce the seed, so that only this worker will
            // respond to the challenge
            let pkh = sp_core::blake2_256(self.pubkey.as_ref());
            let hashed_id: U256 = pkh.into();
            let challenge = msg::HeartbeatChallenge {
                seed: hashed_id,
                online_target: U256::zero(),
            };
            let message = msg::SystemEvent::HeartbeatChallenge(challenge);
            self.mq.dispatch_bound(&sender, message);
        }

        fn heartbeat(&mut self, session_id: u32, block: chain::BlockNumber, iterations: u64) {
            let message = msg::MiningReportEvent::Heartbeat {
                session_id,
                challenge_block: block,
                challenge_time: block_ts(block),
                iterations,
            };
            self.say(message)
        }
    }

    fn with_block(block_number: chain::BlockNumber, call: impl FnOnce(&BlockInfo)) {
        // GK never use the storage ATM.
        let storage = crate::Storage::default();
        let mut recv_mq = phala_mq::MessageDispatcher::new();
        let mut send_mq = phala_mq::MessageSendQueue::new();
        let block = BlockInfo {
            block_number,
            now_ms: block_ts(block_number),
            storage: &storage,
            recv_mq: &mut recv_mq,
            send_mq: &mut send_mq,
            side_task_man: &mut Default::default(),
        };
        call(&block);
    }

    fn block_ts(block_number: chain::BlockNumber) -> u64 {
        block_number as u64 * 12000
    }

    #[test]
    fn gk_should_be_able_to_observe_worker_states() {
        let mut r = Roles::test_roles();

        with_block(1, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                confidence_level: 2,
            }));
            r.gk.process_messages(block);
        });

        assert_eq!(r.gk.workers.len(), 1);

        assert!(r.get_worker(0).state.registered);

        with_block(2, |block| {
            let mut worker1 = r.for_worker(1);
            worker1.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 1,
                init_v: 1,
                init_p: 100,
            });
            r.gk.process_messages(block);
        });

        assert_eq!(
            r.gk.workers.len(),
            1,
            "Unregistered worker should not start mining."
        );
    }

    #[test]
    fn gk_should_not_miss_any_heartbeats_cross_session() {
        let mut r = Roles::test_roles();

        with_block(1, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                confidence_level: 2,
            }));
            r.gk.process_messages(block);
        });

        assert_eq!(r.gk.workers.len(), 1);

        assert!(r.get_worker(0).state.registered);

        with_block(2, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 1,
                init_v: 1,
                init_p: 100,
            });
            worker0.challenge();
            r.gk.process_messages(block);
        });

        // Stop mining before the heartbeat response.
        with_block(3, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::MiningStop);
            r.gk.process_messages(block);
        });

        with_block(4, |block| {
            r.gk.process_messages(block);
        });

        with_block(5, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 2,
                init_v: 1,
                init_p: 100,
            });
            worker0.challenge();
            r.gk.process_messages(block);
        });

        // Force enter unresponsive
        with_block(100, |block| {
            r.gk.process_messages(block);
        });

        assert_eq!(
            r.get_worker(0).waiting_heartbeats.len(),
            2,
            "There should be 2 waiting HBs"
        );

        assert!(
            r.get_worker(0).unresponsive,
            "The worker should be unresponsive now"
        );

        with_block(101, |block| {
            let mut worker = r.for_worker(0);
            // Response the first challenge.
            worker.heartbeat(1, 2, 10000000);
            r.gk.process_messages(block);
        });
        assert_eq!(
            r.get_worker(0).waiting_heartbeats.len(),
            1,
            "There should be only one waiting HBs"
        );

        assert!(
            r.get_worker(0).unresponsive,
            "The worker should still be unresponsive now"
        );

        with_block(102, |block| {
            let mut worker = r.for_worker(0);
            // Response the second challenge.
            worker.heartbeat(2, 5, 10000000);
            r.gk.process_messages(block);
        });

        assert!(
            !r.get_worker(0).unresponsive,
            "The worker should be mining idle now"
        );
    }

    #[test]
    fn gk_should_reward_normal_workers_do_not_hit_the_seed_case1() {
        let mut r = Roles::test_roles();
        let mut block_number = 1;

        // Register worker
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                confidence_level: 2,
            }));
            r.gk.process_messages(block);
        });

        // Start mining & send heartbeat challenge
        block_number += 1;
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 1,
                init_v: fp!(1).to_bits(),
                init_p: 100,
            });
            r.gk.process_messages(block);
        });

        block_number += 1;

        // Normal Idle state, no event
        let v_snap = r.get_worker(0).tokenomic.v;
        r.gk.egress.clear();
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        assert!(!r.get_worker(0).unresponsive, "Worker should be online");
        assert_eq!(
            r.gk.egress.drain_mining_info_update_event().len(),
            0,
            "Should not report any event"
        );
        assert!(
            v_snap < r.get_worker(0).tokenomic.v,
            "Worker should be rewarded"
        );

        // Once again.
        let v_snap = r.get_worker(0).tokenomic.v;
        r.gk.egress.clear();
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        assert!(!r.get_worker(0).unresponsive, "Worker should be online");
        assert_eq!(
            r.gk.egress.drain_mining_info_update_event().len(),
            0,
            "Should not report any event"
        );
        assert!(
            v_snap < r.get_worker(0).tokenomic.v,
            "Worker should be rewarded"
        );
    }

    #[test]
    fn gk_should_report_payout_for_normal_heartbeats_case2() {
        let mut r = Roles::test_roles();
        let mut block_number = 1;

        // Register worker
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                confidence_level: 2,
            }));
            r.gk.process_messages(block);
        });

        // Start mining & send heartbeat challenge
        block_number += 1;
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 1,
                init_v: fp!(1).to_bits(),
                init_p: 100,
            });
            worker0.challenge();
            r.gk.process_messages(block);
        });
        let challenge_block = block_number;

        block_number += r.gk.tokenomic_params.heartbeat_window;

        // About to timeout then A heartbeat received, report payout event.
        let v_snap = r.get_worker(0).tokenomic.v;
        r.gk.egress.clear();
        with_block(block_number, |block| {
            let mut worker = r.for_worker(0);
            worker.heartbeat(1, challenge_block, 10000000);
            r.gk.process_messages(block);
        });

        assert!(!r.get_worker(0).unresponsive, "Worker should be online");
        assert!(
            v_snap > r.get_worker(0).tokenomic.v,
            "Worker should be paid out"
        );

        {
            let messages = r.gk.egress.drain_mining_info_update_event();
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].offline.len(), 0);
            assert_eq!(messages[0].recovered_to_online.len(), 0);
            assert_eq!(messages[0].settle.len(), 1);
        }
    }

    #[test]
    fn gk_should_slash_and_report_offline_workers_case3() {
        let mut r = Roles::test_roles();
        let mut block_number = 1;

        // Register worker
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                confidence_level: 2,
            }));
            r.gk.process_messages(block);
        });

        // Start mining & send heartbeat challenge
        block_number += 1;
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 1,
                init_v: fp!(1).to_bits(),
                init_p: 100,
            });
            worker0.challenge();
            r.gk.process_messages(block);
        });

        assert!(r.get_worker(0).state.mining_state.is_some());

        block_number += r.gk.tokenomic_params.heartbeat_window;
        // About to timeout
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        assert!(!r.get_worker(0).unresponsive);

        let v_snap = r.get_worker(0).tokenomic.v;

        block_number += 1;
        // Heartbeat timed out
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        assert!(r.get_worker(0).unresponsive);
        {
            let offline = [r.workers[0].clone()].to_vec();
            let expected_message = MiningInfoUpdateEvent {
                block_number,
                timestamp_ms: block_ts(block_number),
                offline,
                recovered_to_online: Vec::new(),
                settle: Vec::new(),
            };
            let messages = r.gk.egress.drain_mining_info_update_event();
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0], expected_message);
        }
        assert!(
            v_snap > r.get_worker(0).tokenomic.v,
            "Worker should be slashed"
        );

        r.gk.egress.clear();

        let v_snap = r.get_worker(0).tokenomic.v;
        block_number += 1;
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        assert_eq!(
            r.gk.egress.drain_mining_info_update_event().len(),
            0,
            "Should not report offline workers"
        );
        assert!(
            v_snap > r.get_worker(0).tokenomic.v,
            "Worker should be slashed again"
        );
    }

    #[test]
    fn gk_should_slash_offline_workers_sliently_case4() {
        let mut r = Roles::test_roles();
        let mut block_number = 1;

        // Register worker
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                confidence_level: 2,
            }));
            r.gk.process_messages(block);
        });

        // Start mining & send heartbeat challenge
        block_number += 1;
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 1,
                init_v: fp!(1).to_bits(),
                init_p: 100,
            });
            worker0.challenge();
            r.gk.process_messages(block);
        });

        block_number += r.gk.tokenomic_params.heartbeat_window;
        // About to timeout
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        block_number += 1;
        // Heartbeat timed out
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        r.gk.egress.clear();

        // Worker already offline, don't report again until one more heartbeat received.
        let v_snap = r.get_worker(0).tokenomic.v;
        block_number += 1;
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        assert_eq!(
            r.gk.egress.drain_mining_info_update_event().len(),
            0,
            "Should not report offline workers"
        );
        assert!(
            v_snap > r.get_worker(0).tokenomic.v,
            "Worker should be slashed"
        );

        let v_snap = r.get_worker(0).tokenomic.v;
        block_number += 1;
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        assert_eq!(
            r.gk.egress.drain_mining_info_update_event().len(),
            0,
            "Should not report offline workers"
        );
        assert!(
            v_snap > r.get_worker(0).tokenomic.v,
            "Worker should be slashed again"
        );
    }

    #[test]
    fn gk_should_report_recovered_workers_case5() {
        let mut r = Roles::test_roles();
        let mut block_number = 1;

        // Register worker
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                confidence_level: 2,
            }));
            r.gk.process_messages(block);
        });

        // Start mining & send heartbeat challenge
        block_number += 1;
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 1,
                init_v: fp!(1).to_bits(),
                init_p: 100,
            });
            worker0.challenge();
            r.gk.process_messages(block);
        });
        let challenge_block = block_number;

        block_number += r.gk.tokenomic_params.heartbeat_window;
        // About to timeout
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        block_number += 1;
        // Heartbeat timed out
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        r.gk.egress.clear();

        // Worker offline, report recover event on the next heartbeat received.
        let v_snap = r.get_worker(0).tokenomic.v;
        block_number += 1;
        with_block(block_number, |block| {
            let mut worker = r.for_worker(0);
            worker.heartbeat(1, challenge_block, 10000000);
            r.gk.process_messages(block);
        });
        assert_eq!(
            v_snap,
            r.get_worker(0).tokenomic.v,
            "Worker should not be slashed or rewarded"
        );
        {
            let recovered_to_online = [r.workers[0].clone()].to_vec();
            let expected_message = MiningInfoUpdateEvent {
                block_number,
                timestamp_ms: block_ts(block_number),
                offline: Vec::new(),
                recovered_to_online,
                settle: Vec::new(),
            };
            let messages = r.gk.egress.drain_mining_info_update_event();
            assert_eq!(messages.len(), 1, "Should report recover event");
            assert_eq!(messages[0], expected_message);
        }
    }

    #[test]
    fn check_tokenomic_numerics() {
        let mut r = Roles::test_roles();
        let mut block_number = 1;

        // Register worker
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                confidence_level: 2,
            }));
            r.gk.process_messages(block);
        });

        // Start mining & send heartbeat challenge
        block_number += 1;
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::BenchScore(3000));
            worker0.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 1,
                init_v: fp!(3000).to_bits(),
                init_p: 100,
            });
            r.gk.process_messages(block);
        });
        assert!(r.get_worker(0).state.mining_state.is_some());
        assert_eq!(r.get_worker(0).tokenomic.p_bench, fp!(100));
        assert_eq!(r.get_worker(0).tokenomic.v, fp!(3000.00203509369147797934));

        // V increment for one day
        for _ in 0..3600 * 24 / 12 {
            block_number += 1;
            with_block(block_number, |block| {
                r.gk.process_messages(block);
            });
        }
        assert_eq!(r.get_worker(0).tokenomic.v, fp!(3014.6899337932040476463));

        // Payout
        block_number += 1;
        r.for_worker(0).challenge();
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        // Check heartbeat updates
        assert_eq!(r.get_worker(0).tokenomic.challenge_time_last, 24000);
        assert_eq!(r.get_worker(0).tokenomic.iteration_last, 0);
        r.for_worker(0)
            .heartbeat(1, block_number, (110 * 7200 * 12 / 6) as u64);
        block_number += 1;
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        assert_eq!(r.get_worker(0).tokenomic.v, fp!(3000));
        assert_eq!(
            r.get_worker(0).tokenomic.p_instant,
            fp!(109.96945292974173840575)
        );
        // Payout settlement has correct treasury split
        let report = r.gk.egress.drain_mining_info_update_event();
        assert_eq!(
            FixedPoint::from_bits(report[0].settle[0].payout),
            fp!(14.69197867920878555043)
        );
        assert_eq!(
            FixedPoint::from_bits(report[0].settle[0].treasury),
            fp!(3.6729946698021946595)
        );

        // Slash 0.1% (1hr + 10 blocks challenge window)
        let _ = r.gk.egress.drain_mining_info_update_event();
        r.for_worker(0).challenge();
        for _ in 0..=3600 / 12 + 10 {
            block_number += 1;
            with_block(block_number, |block| {
                r.gk.process_messages(block);
            });
        }
        assert!(r.get_worker(0).unresponsive);
        let report = r.gk.egress.drain_mining_info_update_event();
        assert_eq!(report[0].offline, vec![r.workers[0].clone()]);
        assert_eq!(r.get_worker(0).tokenomic.v, fp!(2997.0260877851113935014));

        // TODO(hangyin): also check miner reconnection and V recovery
    }

    #[test]
    fn should_payout_at_v_max() {
        let mut r = Roles::test_roles();
        let mut block_number = 1;

        // Register worker
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                confidence_level: 2,
            }));
            r.gk.process_messages(block);
        });

        // Start mining & send heartbeat challenge
        block_number += 1;
        with_block(block_number, |block| {
            let mut worker0 = r.for_worker(0);
            worker0.pallet_say(msg::WorkerEvent::BenchScore(3000));
            worker0.pallet_say(msg::WorkerEvent::MiningStart {
                session_id: 1,
                init_v: fp!(30000).to_bits(),
                init_p: 3000,
            });
            r.gk.process_messages(block);
        });
        // Mine for 24h
        for _ in 0..7200 {
            block_number += 1;
            with_block(block_number, |block| {
                r.gk.process_messages(block);
            });
        }
        // Trigger payout
        block_number += 1;
        with_block(block_number, |block| {
            r.for_worker(0).challenge();
            r.gk.process_messages(block);
        });
        r.for_worker(0).heartbeat(1, block_number, 1000000 as u64);
        block_number += 1;
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        // Check payout
        assert_eq!(r.get_worker(0).tokenomic.v, fp!(29855.38985958385856094607));
        assert_eq!(r.get_worker(0).tokenomic.v_deductible, fp!(0));
        let report = r.gk.egress.drain_mining_info_update_event();
        assert_eq!(
            FixedPoint::from_bits(report[0].settle[0].payout),
            fp!(144.61014041614143905393)
        );
    }

    #[test]
    fn test_update_p_instant() {
        let mut info = super::TokenomicInfo {
            p_bench: fp!(100),
            ..Default::default()
        };

        // Normal
        info.update_p_instant(100_000, 1000);
        info.challenge_time_last = 90_000;
        info.iteration_last = 1000;
        assert_eq!(info.p_instant, fp!(60));

        // Reset
        info.update_p_instant(200_000, 999);
        assert_eq!(info.p_instant, fp!(0));
    }

    #[test]
    fn test_repair_v() {
        let mut r = Roles::test_roles();
        let mut block_number = 1;

        // Register worker
        with_block(block_number, |block| {
            for i in 0..=1 {
                let mut worker = r.for_worker(i);
                worker.pallet_say(msg::WorkerEvent::Registered(msg::WorkerInfo {
                    confidence_level: 2,
                }));
            }
            r.gk.process_messages(block);
        });

        // Start mining & send heartbeat challenge
        block_number += 1;
        with_block(block_number, |block| {
            for i in 0..=1 {
                let mut worker = r.for_worker(i);
                worker.pallet_say(msg::WorkerEvent::BenchScore(3000));
                worker.pallet_say(msg::WorkerEvent::MiningStart {
                    session_id: 1,
                    init_v: fp!(30000).to_bits(),
                    init_p: 3000,
                });
            }
            r.gk.process_messages(block);
        });

        for i in 0..=1 {
            let worker = r.get_worker_mut(i);

            worker.tokenomic.v = fp!(100);
            worker.tokenomic.v_init = fp!(200);

            assert!(worker.tokenomic.v < worker.tokenomic.v_init);
        }

        assert_eq!(r.get_worker(0).tokenomic.v, fp!(100));
        assert_eq!(r.get_worker(1).tokenomic.v, fp!(100));

        block_number += 1;
        with_block(block_number, |block| {
            let sender = MessageOrigin::Pallet(b"Pallet".to_vec());
            r.mq.dispatch_bound(&sender, msg::GatekeeperEvent::RepairV);
            r.gk.process_messages(block);
        });

        // Should repaired and rewarded
        assert_eq!(r.get_worker(0).tokenomic.v, fp!(200.00021447729505831407));
        assert_eq!(r.get_worker(1).tokenomic.v, fp!(200.00021447729505831407));
    }

    #[test]
    fn serde_fp_works_for_msgpack() {
        use serde::{Deserialize, Serialize};
        #[derive(Serialize, Deserialize)]
        struct Wrapper(#[serde(with = "super::serde_fp")] FixedPoint);

        let fp = Wrapper(fp!(1.23456789));
        let fp_serde = serde_json::to_string(&fp).unwrap();
        let fp_de: Wrapper = serde_json::from_str(&fp_serde).unwrap();
        assert_eq!(fp.0, fp_de.0);

        let fp_serde = rmp_serde::to_vec(&fp).unwrap();
        let fp_de: Wrapper = rmp_serde::from_slice(&fp_serde).unwrap();
        assert_eq!(fp.0, fp_de.0);
    }

    #[test]
    fn serde_fp_works_for_cbor() {
        use serde::{Deserialize, Serialize};
        #[derive(Serialize, Deserialize)]
        struct Wrapper(#[serde(with = "super::serde_fp")] FixedPoint);

        let mut buf = Vec::new();
        let fp = Wrapper(fp!(1.23456789));
        ciborium::ser::into_writer(&fp, &mut buf).unwrap();
        let fp_de: Wrapper = ciborium::de::from_reader(&*buf).unwrap();
        assert_eq!(fp.0, fp_de.0);
    }
}
