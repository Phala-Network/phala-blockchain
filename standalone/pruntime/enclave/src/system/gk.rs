use super::{TypedReceiver, WorkerState};
use chain::pallet_registry::RegistryEvent;
use phala_crypto::{
    aead, ecdh,
    sr25519::{Persistence, Signing, KDF, SECRET_KEY_LENGTH, SIGNATURE_BYTES},
};
use phala_mq::{BindTopic, MessageDispatcher, MessageSendQueue, Sr25519MessageChannel};
use phala_types::{
    messaging::{
        DispatchMasterKeyEvent, GatekeeperEvent, MessageOrigin, MiningInfoUpdateEvent,
        MiningReportEvent, NewGatekeeperEvent, RandomNumber, RandomNumberEvent, SettleInfo,
        SystemEvent, WorkerEvent, WorkerEventWithKey,
    },
    WorkerPublicKey,
};
use sp_core::{hashing, sr25519, Pair};

use crate::{
    std::collections::{BTreeMap, VecDeque},
    types::BlockInfo,
};

use crate::std::convert::TryInto;
use crate::std::vec::Vec;
use sgx_tstd::io::{Read, Write};
use sgx_tstd::sgxfs::SgxFile;
use msg_trait::MessageChannel;
use parity_scale_codec::Encode;
use tokenomic::{FixedPoint, TokenomicInfo};

/// Block interval to generate pseudo-random on chain
///
/// WARNING: this interval need to be large enough considering the latency of mq
const VRF_INTERVAL: u32 = 5;

/// Master key filepath
pub const MASTER_KEY_FILEPATH: &str = "master_key.seal";

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

struct WorkerInfo {
    state: WorkerState,
    waiting_heartbeats: VecDeque<chain::BlockNumber>,
    unresponsive: bool,
    tokenomic: TokenomicInfo,
    heartbeat_flag: bool,
}

impl WorkerInfo {
    fn new(pubkey: WorkerPublicKey) -> Self {
        Self {
            state: WorkerState::new(pubkey),
            waiting_heartbeats: Default::default(),
            unresponsive: false,
            tokenomic: Default::default(),
            heartbeat_flag: false,
        }
    }
}

// The Gatekeeper's common internal state is consisted of:
// 1. possessed master key;
// 2. egress sequence number;
// 3. worker list;
// 4. tokenomic params;
// 5. last random number & last random block;
//
// We should ensure the consistency of all the variables above in each block.
//
// For simplicity, we also ensure that each gatekeeper responds (if registered on chain)
// to received messages in the same way.
pub(super) struct Gatekeeper<MsgChan> {
    identity_key: sr25519::Pair,
    master_key: Option<sr25519::Pair>,
    registered_on_chain: bool,
    send_mq: MessageSendQueue,
    egress: Option<MsgChan>, // TODO.kevin: syncing the egress state while migrating.
    worker_egress: Sr25519MessageChannel,
    gatekeeper_events: TypedReceiver<GatekeeperEvent>,
    mining_events: TypedReceiver<MiningReportEvent>,
    system_events: TypedReceiver<SystemEvent>,
    workers: BTreeMap<WorkerPublicKey, WorkerInfo>,
    // Randomness
    last_random_number: RandomNumber,
    last_random_block: chain::BlockNumber,

    tokenomic_params: tokenomic::Params,
}

impl<MsgChan> Gatekeeper<MsgChan>
where
    MsgChan: MessageChannel,
{
    pub fn new(
        identity_key: sr25519::Pair,
        recv_mq: &mut MessageDispatcher,
        send_mq: MessageSendQueue,
        worker_egress: Sr25519MessageChannel,
    ) -> Self {
        Self {
            identity_key,
            master_key: None,
            registered_on_chain: false,
            send_mq,
            egress: None,
            worker_egress,
            gatekeeper_events: recv_mq.subscribe_bound(),
            mining_events: recv_mq.subscribe_bound(),
            system_events: recv_mq.subscribe_bound(),
            workers: Default::default(),
            last_random_number: [0_u8; 32],
            last_random_block: 0,
            tokenomic_params: tokenomic::test_params(),
        }
    }

    pub fn is_registered_on_chain(&self) -> bool {
        self.registered_on_chain
    }

    pub fn set_gatekeeper_egress_dummy(&mut self, dummy: bool) {
        self.egress
            .as_mut()
            .expect("gk should work after acquiring master key; qed.")
            .set_dummy(dummy);
    }

    pub fn register_on_chain(&mut self) {
        info!("Gatekeeper: register on chain");
        self.registered_on_chain = true;
        self.set_gatekeeper_egress_dummy(false);
    }

    #[allow(dead_code)]
    pub fn unregister_on_chain(&mut self) {
        info!("Gatekeeper: unregister on chain");
        self.registered_on_chain = false;
        self.set_gatekeeper_egress_dummy(true);
    }

    pub fn possess_master_key(&self) -> bool {
        self.master_key.is_some()
    }

    pub fn set_master_key(&mut self, master_key: sr25519::Pair, need_restart: bool) {
        if self.master_key.is_none() {
            self.master_key = Some(master_key.clone());
            self.seal_master_key(MASTER_KEY_FILEPATH);

            if need_restart {
                panic!("Received master key, please restart pRuntime and pherry");
            }

            // init gatekeeper egress dynamically with master key
            if self.egress.is_none() {
                let gk_egress =
                    MsgChan::create(&self.send_mq, MessageOrigin::Gatekeeper, master_key);
                // by default, gk_egress is set to dummy mode until it is registered on chain
                gk_egress.set_dummy(true);
                self.egress = Some(gk_egress);
            }
        } else if let Some(my_master_key) = &self.master_key {
            // TODO.shelven: remove this assertion after we enable master key rotation
            assert_eq!(my_master_key.to_raw_vec(), master_key.to_raw_vec());
        }
    }

    /// Seal master key seed with signature to ensure integrity
    pub fn seal_master_key(&self, filepath: &str) {
        if let Some(master_key) = &self.master_key {
            let secret = master_key.dump_secret_key();
            let sig = self.identity_key.sign_data(&secret);

            // TODO(shelven): use serialization rather than manual concat.
            let mut buf = Vec::new();
            buf.extend_from_slice(&secret);
            buf.extend_from_slice(sig.as_ref());

            let mut file = SgxFile::create(filepath)
                .unwrap_or_else(|e| panic!("Create master key file failed: {:?}", e));
            file.write_all(&buf)
                .unwrap_or_else(|e| panic!("Seal master key failed: {:?}", e));
        }
    }

    /// Unseal local master key seed and verify signature
    ///
    /// This function could panic a lot.
    pub fn try_unseal_master_key(&mut self, filepath: &str) {
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

    pub fn push_gatekeeper_message(&self, message: impl Encode + BindTopic) {
        self.egress
            .as_ref()
            .expect("gk should work after acquiring master key; qed.")
            .push_message(message)
    }

    pub fn process_messages(&mut self, block: &BlockInfo<'_>) {
        let sum_share: FixedPoint = self
            .workers
            .values()
            .map(|info| info.tokenomic.share())
            .sum();

        let mut processor = GKMessageProcesser {
            state: self,
            block,
            report: MiningInfoUpdateEvent::new(block.block_number, block.now_ms),
            sum_share,
        };

        processor.process();

        let report = processor.report;

        if !report.is_empty() {
            self.push_gatekeeper_message(report);
        }
    }

    pub fn emit_random_number(&mut self, block_number: chain::BlockNumber) {
        if block_number % VRF_INTERVAL != 0 {
            return;
        }

        if block_number - self.last_random_block != VRF_INTERVAL {
            // wait for random number syncing
            return;
        }

        if let Some(master_key) = &self.master_key {
            let random_number =
                next_random_number(master_key, block_number, self.last_random_number);
            info!(
                "Gatekeeper: emit random number {} in block {}",
                hex::encode(&random_number),
                block_number
            );
            self.push_gatekeeper_message(GatekeeperEvent::new_random_number(
                block_number,
                random_number,
                self.last_random_number,
            ));

            self.last_random_block = block_number;
            self.last_random_number = random_number;
        }
    }
}

struct GKMessageProcesser<'a, MsgChan> {
    state: &'a mut Gatekeeper<MsgChan>,
    block: &'a BlockInfo<'a>,
    report: MiningInfoUpdateEvent<chain::BlockNumber>,
    sum_share: FixedPoint,
}

impl<MsgChan> GKMessageProcesser<'_, MsgChan>
where
    MsgChan: MessageChannel,
{
    fn process(&mut self) {
        self.prepare();
        loop {
            let ok = phala_mq::select! {
                message = self.state.mining_events => match message {
                    Ok((_, event, origin)) => {
                        self.process_mining_report(origin, event);
                    }
                    Err(e) => {
                        error!("Read message failed: {:?}", e);
                    }
                },
                message = self.state.system_events => match message {
                    Ok((_, event, origin)) => {
                        self.process_system_event(origin, event);
                    }
                    Err(e) => {
                        error!("Read message failed: {:?}", e);
                    }
                },
                message = self.state.gatekeeper_events => match message {
                    Ok((_, event, origin)) => {
                        self.process_gatekeeper_event(origin, event);
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
        self.block_post_process();
    }

    fn prepare(&mut self) {
        for worker in self.state.workers.values_mut() {
            worker.heartbeat_flag = false;
        }
    }

    fn block_post_process(&mut self) {
        for worker_info in self.state.workers.values_mut() {
            let mut tracker = WorkerSMTracker {
                waiting_heartbeats: &mut worker_info.waiting_heartbeats,
            };
            worker_info
                .state
                .on_block_processed(self.block, &mut tracker);

            if worker_info.state.mining_state.is_none() {
                // Mining already stopped, do nothing.
                continue;
            }

            if worker_info.unresponsive {
                if worker_info.heartbeat_flag {
                    // case5: Unresponsive, successful heartbeat
                    worker_info.unresponsive = false;
                    self.report
                        .recovered_to_online
                        .push(worker_info.state.pubkey.clone());
                }
            } else {
                if let Some(&hb_sent_at) = worker_info.waiting_heartbeats.get(0) {
                    if self.block.block_number - hb_sent_at
                        > self.state.tokenomic_params.heartbeat_window
                    {
                        // case3: Idle, heartbeat failed
                        self.report.offline.push(worker_info.state.pubkey.clone());
                        worker_info.unresponsive = true;
                    }
                }
            }

            let params = &self.state.tokenomic_params;
            if worker_info.unresponsive {
                // case3/case4:
                // Idle, heartbeat failed or
                // Unresponsive, no event
                worker_info.tokenomic.update_v_slash(&params);
            } else if !worker_info.heartbeat_flag {
                // case1: Idle, no event
                worker_info.tokenomic.update_v_idle(&params);
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
                            "Unknown worker {} sent a {:?}",
                            hex::encode(worker_pubkey),
                            event
                        );
                        return;
                    }
                };

                if Some(&challenge_block) != worker_info.waiting_heartbeats.get(0) {
                    error!("Fatal error: Unexpected heartbeat {:?}", event);
                    error!("Sent from worker {}", hex::encode(worker_pubkey));
                    error!("Waiting heartbeats {:#?}", worker_info.waiting_heartbeats);
                    // The state has been poisoned. Make no sence to keep moving on.
                    panic!("GK or Worker state poisoned");
                }

                // The oldest one comfirmed.
                let _ = worker_info.waiting_heartbeats.pop_front();

                let mining_state = if let Some(state) = &worker_info.state.mining_state {
                    state
                } else {
                    // Mining already stopped, ignore the heartbeat.
                    return;
                };

                if session_id != mining_state.session_id {
                    // Heartbeat response to previous mining sessions, ignore it.
                    return;
                }

                worker_info.heartbeat_flag = true;

                let tokenomic = &mut worker_info.tokenomic;
                tokenomic.update_p_instant(self.block.now_ms, iterations);
                tokenomic.challenge_time_last = challenge_time;
                tokenomic.iteration_last = iterations;

                if worker_info.unresponsive {
                    // case5: Unresponsive, successful heartbeat.
                } else {
                    // case2: Idle, successful heartbeat, report to pallet
                    let payout = worker_info.tokenomic.update_v_heartbeat(
                        &self.state.tokenomic_params,
                        self.sum_share,
                        self.block.now_ms,
                    );

                    // NOTE: keep the reporting order (vs the one while mining stop).
                    self.report.settle.push(SettleInfo {
                        pubkey: worker_pubkey.clone(),
                        v: worker_info.tokenomic.v.to_bits(),
                        payout: payout.to_bits(),
                    })
                }
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
                .entry(pubkey.clone())
                .or_insert_with(|| WorkerInfo::new(pubkey.clone()));
        }

        // TODO.kevin: Avoid unnecessary iteration for WorkerEvents.
        for worker_info in self.state.workers.values_mut() {
            // Replay the event on worker state, and collect the egressed heartbeat into waiting_heartbeats.
            let mut tracker = WorkerSMTracker {
                waiting_heartbeats: &mut worker_info.waiting_heartbeats,
            };
            worker_info
                .state
                .process_event(self.block, &event, &mut tracker, false);
        }

        match &event {
            SystemEvent::WorkerEvent(e) => {
                if let Some(worker) = self.state.workers.get_mut(&e.pubkey) {
                    match &e.event {
                        WorkerEvent::Registered(info) => {
                            worker.tokenomic.confidence_level = info.confidence_level;
                        }
                        WorkerEvent::BenchStart { .. } => {}
                        WorkerEvent::BenchScore(score) => {
                            worker.tokenomic.p_bench = FixedPoint::from_num(*score);
                        }
                        WorkerEvent::MiningStart {
                            session_id: _, // Aready recorded by the state machine.
                            init_v,
                        } => {
                            let v = FixedPoint::from_bits(*init_v);
                            let prev = worker.tokenomic;
                            // NOTE.kevin: To track the heartbeats by global timeline, don't clear the waiting_heartbeats.
                            // worker.waiting_heartbeats.clear();
                            worker.unresponsive = false;
                            worker.tokenomic = TokenomicInfo {
                                v,
                                v_last: v,
                                v_update_at: self.block.now_ms,
                                iteration_last: 0,
                                challenge_time_last: self.block.now_ms,
                                p_bench: prev.p_bench,
                                p_instant: prev.p_bench,
                                confidence_level: prev.confidence_level,
                            };
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
                                pubkey: worker.state.pubkey.clone(),
                                v: worker.tokenomic.v.to_bits(),
                                payout: 0,
                            })
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
        info!("Incoming gatekeeper event: {:?}", event);
        match event {
            GatekeeperEvent::Registered(new_gatekeeper_event) => {
                self.process_new_gatekeeper_event(origin, new_gatekeeper_event)
            }
            GatekeeperEvent::DispatchMasterKey(dispatch_master_key_event) => {
                self.process_dispatch_master_key_event(origin, dispatch_master_key_event)
            }
            GatekeeperEvent::NewRandomNumber(random_number_event) => {
                self.process_random_number_event(origin, random_number_event)
            }
            GatekeeperEvent::TokenomicParametersChanged(params) => {
                if origin.is_pallet() {
                    self.state.tokenomic_params = params.into();
                }
            }
        }
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
        if !crate::gatekeeper::is_gatekeeper(&event.pubkey, self.block.storage) {
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
                    Ok(_) => self.state.push_gatekeeper_message(
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
                    ),
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

    /// Verify on-chain random number
    fn process_random_number_event(&mut self, origin: MessageOrigin, event: RandomNumberEvent) {
        if !origin.is_gatekeeper() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        };

        if let Some(master_key) = &self.state.master_key {
            let expect_random =
                next_random_number(master_key, event.block_number, event.last_random_number);
            // instead of checking the origin, we directly verify the random to avoid access storage
            if expect_random != event.random_number {
                error!("Fatal error: Expect random number {:?}", expect_random);
                panic!("GK state poisoned");
            }
        }
    }
}

struct WorkerSMTracker<'a> {
    waiting_heartbeats: &'a mut VecDeque<chain::BlockNumber>,
}

impl super::WorkerStateMachineCallback for WorkerSMTracker<'_> {
    fn heartbeat(
        &mut self,
        _session_id: u32,
        challenge_block: runtime::BlockNumber,
        _challenge_time: u64,
        _iterations: u64,
    ) {
        self.waiting_heartbeats.push_back(challenge_block);
    }
}

mod tokenomic {
    pub use fixed::types::U64F64 as FixedPoint;
    use fixed_sqrt::FixedSqrt as _;
    use phala_types::messaging::TokenomicParameters;

    pub fn fp(n: u64) -> FixedPoint {
        FixedPoint::from_num(n)
    }

    fn square(v: FixedPoint) -> FixedPoint {
        v * v
    }

    fn conf_score(level: u8) -> FixedPoint {
        match level {
            1 | 2 | 3 | 128 => fp(1),
            4 => fp(8) / 10,
            5 => fp(7) / 10,
            _ => fp(0),
        }
    }

    #[derive(Default, Clone, Copy)]
    pub struct TokenomicInfo {
        pub v: FixedPoint,
        pub v_last: FixedPoint,
        pub v_update_at: u64,
        pub iteration_last: u64,
        pub challenge_time_last: u64,
        pub p_bench: FixedPoint,
        pub p_instant: FixedPoint,
        pub confidence_level: u8,
    }

    #[derive(Debug)]
    pub struct Params {
        pha_rate: FixedPoint,
        rho: FixedPoint,
        slash_rate: FixedPoint,
        budget_per_sec: FixedPoint,
        v_max: FixedPoint,
        cost_k: FixedPoint,
        cost_b: FixedPoint,
        pub heartbeat_window: u32,
    }

    impl From<TokenomicParameters> for Params {
        fn from(params: TokenomicParameters) -> Self {
            Params {
                pha_rate: FixedPoint::from_bits(params.pha_rate),
                rho: FixedPoint::from_bits(params.rho),
                slash_rate: FixedPoint::from_bits(params.slash_rate),
                budget_per_sec: FixedPoint::from_bits(params.budget_per_sec),
                v_max: FixedPoint::from_bits(params.v_max),
                cost_k: FixedPoint::from_bits(params.cost_k),
                cost_b: FixedPoint::from_bits(params.cost_b),
                heartbeat_window: params.heartbeat_window,
            }
        }
    }

    pub fn test_params() -> Params {
        Params {
            pha_rate: fp(1),
            rho: fp(100000099985) / 100000000000, // hourly: 1.00020, 1.0002 ** (1/300)
            slash_rate: fp(1) / 1000 / 300,       // hourly rate: 0.001, convert to per-block rate
            budget_per_sec: fp(1000),
            v_max: fp(30000),
            cost_k: fp(287) / 10000 / 300, // 0.0287
            cost_b: fp(15) / 300,
            heartbeat_window: 10, // 10 blocks
        }
    }

    impl TokenomicInfo {
        /// case1: Idle, no event
        pub fn update_v_idle(&mut self, params: &Params) {
            let cost_idle =
                (params.cost_k * self.p_bench + params.cost_b) / params.pha_rate / fp(365);
            let perf_multiplier = if self.p_bench == fp(0) {
                fp(1)
            } else {
                self.p_instant / self.p_bench
            };
            let v = self.v + perf_multiplier * ((params.rho - fp(1)) * self.v + cost_idle);
            self.v = v.min(params.v_max);
        }

        /// case2: Idle, successful heartbeat
        /// return payout
        pub fn update_v_heartbeat(
            &mut self,
            params: &Params,
            sum_share: FixedPoint,
            now_ms: u64,
        ) -> FixedPoint {
            if sum_share == fp(0) {
                return fp(0);
            }
            if self.v < self.v_last {
                return fp(0);
            }
            if now_ms <= self.v_update_at {
                // May receive more than one heartbeat for a single worker in a single block.
                return fp(0);
            }
            let dv = self.v - self.v_last;
            let dt = fp(now_ms - self.v_update_at) / 1000;
            let budget = params.budget_per_sec * dt;
            let w = dv.max(fp(0)).min(self.share() / sum_share * budget);
            self.v -= w;
            self.v_last = self.v;
            self.v_update_at = now_ms;
            w
        }

        pub fn update_v_slash(&mut self, params: &Params) {
            self.v -= self.v * params.slash_rate;
        }

        pub fn share(&self) -> FixedPoint {
            (square(self.v) + square(fp(2) * self.p_instant * conf_score(self.confidence_level)))
                .sqrt()
        }

        pub fn update_p_instant(&mut self, now: u64, iterations: u64) {
            if now <= self.challenge_time_last {
                return;
            }
            let dt = fp(now - self.challenge_time_last) / 1000;
            let p = fp(iterations - self.iteration_last) / dt * 6; // 6s iterations
            self.p_instant = p.min(self.p_bench * fp(12) / fp(10));
        }
    }
}

mod msg_trait {
    use parity_scale_codec::Encode;
    use phala_mq::{BindTopic, MessageSendQueue, SenderId};

    pub trait MessageChannel {
        fn push_message<M: Encode + BindTopic>(&self, message: M);
        fn set_dummy(&self, dummy: bool);
        fn create(
            send_mq: &MessageSendQueue,
            sender: SenderId,
            signer: sp_core::sr25519::Pair,
        ) -> Self;
    }

    impl MessageChannel for phala_mq::MessageChannel<sp_core::sr25519::Pair> {
        fn push_message<M: Encode + BindTopic>(&self, message: M) {
            self.send(&message);
        }

        fn set_dummy(&self, dummy: bool) {
            self.set_dummy(dummy);
        }

        fn create(
            send_mq: &MessageSendQueue,
            sender: SenderId,
            signer: sp_core::sr25519::Pair,
        ) -> Self {
            send_mq.channel(sender, signer)
        }
    }
}

#[cfg(feature = "tests")]
pub mod tests {
    use super::{msg_trait::MessageChannel, tokenomic::fp, BlockInfo, Gatekeeper};
    use crate::std::{cell::RefCell, vec::Vec};
    use parity_scale_codec::{Decode, Encode};
    use phala_mq::{BindTopic, Message, MessageDispatcher, MessageOrigin};
    use phala_types::{messaging as msg, WorkerPublicKey};

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
            destination: M::TOPIC.to_vec().into(),
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
                    if &m.destination.path()[..] == M::TOPIC {
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
        fn push_message<M: Encode + BindTopic>(&self, message: M) {
            let message = Message {
                sender: MessageOrigin::Gatekeeper,
                destination: M::TOPIC.to_vec().into(),
                payload: message.encode(),
            };
            self.messages.borrow_mut().push(message);
        }

        fn set_dummy(&self, _dummy: bool) {}

        fn create(
            send_mq: &phala_mq::MessageSendQueue,
            sender: phala_mq::SenderId,
            signer: sp_core::sr25519::Pair,
        ) -> Self {
            panic!("We don't need this")
        }
    }

    struct Roles {
        mq: MessageDispatcher,
        gk: Gatekeeper<CollectChannel>,
        workers: [WorkerPublicKey; 2],
    }

    impl Roles {
        fn test_roles() -> Roles {
            use sp_core::crypto::Pair;

            let mut mq = MessageDispatcher::new();
            let egress = CollectChannel::default();
            let key = sp_core::sr25519::Pair::from_seed(&[1u8; 32]);
            let gk = Gatekeeper::new(key, &mut mq, egress);
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
        let block = BlockInfo {
            block_number,
            now_ms: block_ts(block_number),
            storage: &storage,
        };
        call(&block);
    }

    fn block_ts(block_number: chain::BlockNumber) -> u64 {
        block_number as u64 * 6000
    }

    pub fn run_all_tests() {
        gk_should_be_able_to_observe_worker_states();
        gk_should_not_miss_any_heartbeats_cross_session();
        gk_should_reward_normal_workers_do_not_hit_the_seed_case1();
        gk_should_report_payout_for_normal_heartbeats_case2();
        gk_should_slash_and_report_offline_workers_case3();
        gk_should_slash_offline_workers_sliently_case4();
        gk_should_report_recovered_workers_case5();
        show_v_computing();
    }

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
            });
            r.gk.process_messages(block);
        });

        assert_eq!(
            r.gk.workers.len(),
            1,
            "Unregistered worker should not start mining."
        );
    }

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
                init_v: fp(1).to_bits(),
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
                init_v: fp(1).to_bits(),
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
            "Worker should be payed out"
        );

        {
            let messages = r.gk.egress.drain_mining_info_update_event();
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].offline.len(), 0);
            assert_eq!(messages[0].recovered_to_online.len(), 0);
            assert_eq!(messages[0].settle.len(), 1);
        }
    }

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
                init_v: fp(1).to_bits(),
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
                init_v: fp(1).to_bits(),
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
                init_v: fp(1).to_bits(),
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

    fn show_v_computing() {
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
                init_v: fp(3000).to_bits(),
            });
            r.gk.process_messages(block);
        });

        info!("init v = {}", r.get_worker(0).tokenomic.v);

        // Reward
        for _ in 0..3600 * 24 / 12 {
            block_number += 1;
            with_block(block_number, |block| {
                r.gk.process_messages(block);
            });
        }
        info!("mined v = {}", r.get_worker(0).tokenomic.v);

        // Pay out
        block_number += 1;
        r.for_worker(0).challenge();
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        r.for_worker(0).heartbeat(1, block_number, 100000);
        block_number += 1;
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        info!("after payed out v = {}", r.get_worker(0).tokenomic.v);

        // Slash
        r.for_worker(0).challenge();

        for _ in 0..3600 * 24 / 12 {
            block_number += 1;
            with_block(block_number, |block| {
                r.gk.process_messages(block);
            });
        }
        info!("slashed v = {}", r.get_worker(0).tokenomic.v);
    }
}
