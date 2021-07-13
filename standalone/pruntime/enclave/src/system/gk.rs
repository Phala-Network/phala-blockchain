use super::{TypedReceiver, WorkerState};
use phala_crypto::{
    aead, ecdh,
    secp256k1::{Signing, KDF},
};
use phala_mq::MessageDispatcher;
use phala_types::{
    messaging::{
        DispatchMasterKeyEvent, GatekeeperEvent, MessageOrigin, MiningInfoUpdateEvent,
        MiningReportEvent, NewGatekeeperEvent, RandomNumber, RandomNumberEvent, SettleInfo,
        SystemEvent, WorkerEvent, WorkerEventWithKey,
    },
    WorkerPublicKey,
};
use sp_core::{ecdsa, hashing, Pair};

use crate::{
    std::collections::{BTreeMap, VecDeque},
    types::BlockInfo,
};

use crate::std::convert::TryInto;
use crate::std::vec::Vec;
use msg_trait::{EgressMessage, MessageChannel};
use tokenomic::{FixedPoint, TokenomicInfo};

// TODO: Read from blockchain
const HEARTBEAT_TOLERANCE_WINDOW: u32 = 10;

/// Block interval to generate pseudo-random on chain
const VRF_INTERVAL: u32 = 5;

// pesudo_random_number = blake2_256(master_key.sign(last_random_number, block_number))
fn next_random_number(
    master_key: &ecdsa::Pair,
    block_number: chain::BlockNumber,
    last_random_number: RandomNumber,
) -> RandomNumber {
    let mut buf: Vec<u8> = last_random_number.to_vec();
    buf.extend(block_number.to_be_bytes().iter().copied());
    // TODO.shelven: use a derived key instead of master_key
    let sig = master_key.sign_data(buf.as_ref());
    hashing::blake2_256(&sig.0)
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

pub(super) struct Gatekeeper<MsgChan> {
    identity_key: ecdsa::Pair,
    master_key: Option<ecdsa::Pair>,
    egress: MsgChan, // TODO.kevin: syncing the egress state while migrating.
    gatekeeper_events: TypedReceiver<GatekeeperEvent>,
    mining_events: TypedReceiver<MiningReportEvent>,
    system_events: TypedReceiver<SystemEvent>,
    workers: BTreeMap<WorkerPublicKey, WorkerInfo>,
    // Randomness
    last_random_number: RandomNumber,
    last_random_block: chain::BlockNumber,
}

impl<MsgChan> Gatekeeper<MsgChan>
where
    MsgChan: MessageChannel,
{
    pub fn new(
        identity_key: ecdsa::Pair,
        recv_mq: &mut MessageDispatcher,
        egress: MsgChan,
    ) -> Self {
        Self {
            identity_key,
            master_key: None,
            egress,
            gatekeeper_events: recv_mq.subscribe_bound(),
            mining_events: recv_mq.subscribe_bound(),
            system_events: recv_mq.subscribe_bound(),
            workers: Default::default(),
            last_random_number: [0_u8; 32],
            last_random_block: 0,
        }
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
            tokenomic_params: tokenomic::test_params(), // TODO.kevin: replace with real params
            sum_share,
        };

        processor.process();

        let report = processor.report;

        if !report.is_empty() {
            self.egress
                .push_message(EgressMessage::MiningInfoUpdate(report));
        }
    }

    pub fn vrf(&mut self, block_number: chain::BlockNumber) {
        if block_number % VRF_INTERVAL != 1 {
            return;
        }

        if block_number - self.last_random_block != VRF_INTERVAL {
            // wait for random number syncing
            return;
        }

        if let Some(master_key) = &self.master_key {
            self.egress.push_message(EgressMessage::Gatekeeper(
                GatekeeperEvent::new_random_number(
                    block_number,
                    next_random_number(master_key, block_number, self.last_random_number),
                    self.last_random_number,
                ),
            ))
        }
    }
}

struct GKMessageProcesser<'a, MsgChan> {
    state: &'a mut Gatekeeper<MsgChan>,
    block: &'a BlockInfo<'a>,
    report: MiningInfoUpdateEvent<chain::BlockNumber>,
    tokenomic_params: tokenomic::Params,
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
                    if self.block.block_number - hb_sent_at > HEARTBEAT_TOLERANCE_WINDOW {
                        // case3: Idle, heartbeat failed
                        self.report.offline.push(worker_info.state.pubkey.clone());
                        worker_info.unresponsive = true;
                    }
                }
            }

            let params = &self.tokenomic_params;
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
                        &self.tokenomic_params,
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

        // double check the new gatekeeper is valid
        if !crate::identity::is_gatekeeper(&event.pubkey, self.block.storage) {
            error!("Fatal error: Invalid gatekeeper registration {:?}", event);
            panic!("GK state poisoned");
        }
        let my_pubkey = self.state.identity_key.public();
        if my_pubkey == event.pubkey && event.gatekeeper_count == 1 {
            if self.state.master_key.is_none() {
                // generate master key as the first gatekeeper
                self.state.master_key =
                    Some(crate::new_ecdsa_key().expect(
                        "key generation should never fail since we give seed of correct sieze",
                    ))
            }
        } else if my_pubkey != event.pubkey {
            if let Some(master_key) = &self.state.master_key {
                let my_ecdh_key = self
                    .state
                    .identity_key
                    .derive_ecdh_key()
                    .expect("ecdh key derivation should never failed with valid identity key");
                let secret = ecdh::agree(&my_ecdh_key, &event.ecdh_pubkey.0)
                    .expect("should never failed with valid ecdh key");
                let iv = crate::cryptography::aead::generate_iv();
                let mut data = master_key.clone().to_raw_vec();
                aead::encrypt(&iv, &secret, &mut data);

                self.state.egress.push_message(EgressMessage::Gatekeeper(
                    GatekeeperEvent::dispatch_master_key_event(
                        event.pubkey,
                        my_ecdh_key
                            .public()
                            .as_ref()
                            .try_into()
                            .expect("ecdh pubkey with incorrect length"),
                        data,
                        iv,
                    ),
                ));
            }
        }
    }

    /// Process encrypted master key from mq
    fn process_dispatch_master_key_event(
        &mut self,
        origin: MessageOrigin,
        event: DispatchMasterKeyEvent,
    ) {
        let src_pubkey = if let MessageOrigin::Worker(pubkey) = origin {
            pubkey
        } else {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        };

        // double check the event is from a valid gatekeeper
        if !crate::identity::is_gatekeeper(&src_pubkey, self.block.storage) {
            error!(
                "Fatal error: Master key from invalid gatekeeper {:?}",
                event
            );
            panic!("GK state poisoned");
        }

        let my_pubkey = self.state.identity_key.public();
        if my_pubkey == event.dest {
            if self.state.master_key.is_none() {
                let my_ecdh_key = self
                    .state
                    .identity_key
                    .derive_ecdh_key()
                    .expect("ecdh key derivation should never failed with valid identity key");
                let secret = ecdh::agree(&my_ecdh_key, &event.ecdh_pubkey.0)
                    .expect("should never failed with valid ecdh key");
                let mut seed: Vec<u8> = Vec::new();
                aead::decrypt(&event.iv, &secret, &mut seed);

                self.state.master_key =
                    Some(ecdsa::Pair::from_seed_slice(&seed).expect("invalid master key seed"));
            }
        }
    }

    /// Sync on-chain random number for next random generation
    fn process_random_number_event(&mut self, origin: MessageOrigin, event: RandomNumberEvent) {
        if let Some(master_key) = &self.state.master_key {
            // instead of checking the origin, we directly verify the random to avoid access storage
            if next_random_number(master_key, event.block_number, event.last_random_number)
                != event.random_number
            {
                error!("Fatal error: Unexpected random number {:?}", event);
                panic!("GK state poisoned");
            }
            if event.block_number > self.state.last_random_block {
                self.state.last_random_block = event.block_number;
                self.state.last_random_number = event.random_number;
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

    pub fn fp(n: u64) -> FixedPoint {
        FixedPoint::from_num(n)
    }

    fn pow2(v: FixedPoint) -> FixedPoint {
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

    pub struct Params {
        pha_rate: FixedPoint,
        rho: FixedPoint,
        slash_rate: FixedPoint,
        budget_per_sec: FixedPoint,
        v_max: FixedPoint,
        alpha: FixedPoint,
    }

    pub fn test_params() -> Params {
        Params {
            pha_rate: fp(1),
            rho: fp(10002) / 10000,   // 1.00020
            slash_rate: fp(1) / 1000, // TODO: hourly rate: 0.001, convert to per-block rate
            budget_per_sec: fp(10),
            v_max: fp(30000),
            alpha: fp(287) / 10000, // 0.0287
        }
    }

    impl TokenomicInfo {
        /// case1: Idle, no event
        pub fn update_v_idle(&mut self, params: &Params) {
            let cost_idle = (params.alpha * self.p_bench + fp(15)) / params.pha_rate / fp(365);
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
            (pow2(self.v) + pow2(fp(2) * self.p_instant * conf_score(self.confidence_level))).sqrt()
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
    use phala_mq::MessageSigner;

    #[derive(PartialEq, Eq, Debug)]
    pub enum EgressMessage {
        MiningInfoUpdate(super::MiningInfoUpdateEvent<chain::BlockNumber>),
        Gatekeeper(super::GatekeeperEvent),
    }

    pub trait MessageChannel {
        fn push_message(&self, message: EgressMessage);
    }

    impl<T: MessageSigner> MessageChannel for phala_mq::MessageChannel<T> {
        fn push_message(&self, message: EgressMessage) {
            match message {
                EgressMessage::MiningInfoUpdate(message) => self.send(&message),
                EgressMessage::Gatekeeper(message) => self.send(&message),
            }
        }
    }
}

#[cfg(feature = "tests")]
pub mod tests {
    use super::Gatekeeper;

    use super::msg_trait::{EgressMessage, MessageChannel};
    use super::tokenomic::fp;
    use super::BlockInfo;
    use crate::std::cell::RefCell;
    use crate::std::vec::Vec;
    use parity_scale_codec::Encode;
    use phala_mq::{BindTopic, Message, MessageDispatcher, MessageOrigin};
    use phala_types::{messaging as msg, WorkerPublicKey};

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
        messages: RefCell<Vec<EgressMessage>>,
    }

    impl MessageChannel for CollectChannel {
        fn push_message(&self, message: EgressMessage) {
            self.messages.borrow_mut().push(message);
        }
    }

    struct Roles {
        mq: MessageDispatcher,
        gk: Gatekeeper<CollectChannel>,
        workers: [WorkerPublicKey; 2],
    }

    impl Roles {
        fn test_roles() -> Roles {
            let mut mq = MessageDispatcher::new();
            let egress = CollectChannel::default();
            let gk = Gatekeeper::new(&mut mq, egress);
            Roles {
                mq,
                gk,
                workers: [
                    WorkerPublicKey::from_raw([0x01u8; 33]),
                    WorkerPublicKey::from_raw([0x02u8; 33]),
                ],
            }
        }

        fn for_worker(&mut self, n: usize) -> ForWorker {
            ForWorker {
                mq: &mut self.mq,
                pubkey: &self.workers[n],
            }
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

        let worker0 = r.gk.workers.get(&r.workers[0]).unwrap();
        assert!(worker0.state.registered);

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

        let worker0 = r.gk.workers.get(&r.workers[0]).unwrap();
        assert!(worker0.state.registered);

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
            r.gk.workers[&r.workers[0]].waiting_heartbeats.len(),
            2,
            "There should be 2 waiting HBs"
        );

        assert!(
            r.gk.workers[&r.workers[0]].unresponsive,
            "The worker should be unresponsive now"
        );

        with_block(101, |block| {
            let mut worker = r.for_worker(0);
            // Response the first challenge.
            worker.heartbeat(1, 2, 10000000);
            r.gk.process_messages(block);
        });
        assert_eq!(
            r.gk.workers[&r.workers[0]].waiting_heartbeats.len(),
            1,
            "There should be only one waiting HBs"
        );

        assert!(
            r.gk.workers[&r.workers[0]].unresponsive,
            "The worker should still be unresponsive now"
        );

        with_block(102, |block| {
            let mut worker = r.for_worker(0);
            // Response the second challenge.
            worker.heartbeat(2, 5, 10000000);
            r.gk.process_messages(block);
        });

        assert!(
            !r.gk.workers[&r.workers[0]].unresponsive,
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
        let v_snap = r.gk.workers[&r.workers[0]].tokenomic.v;
        r.gk.egress.messages.borrow_mut().clear();
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        assert!(
            !r.gk.workers[&r.workers[0]].unresponsive,
            "Worker should be online"
        );
        assert_eq!(
            r.gk.egress.messages.borrow().len(),
            0,
            "Should not report any event"
        );
        assert!(
            v_snap < r.gk.workers[&r.workers[0]].tokenomic.v,
            "Worker should be rewarded"
        );

        // Once again.
        let v_snap = r.gk.workers[&r.workers[0]].tokenomic.v;
        r.gk.egress.messages.borrow_mut().clear();
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        assert!(
            !r.gk.workers[&r.workers[0]].unresponsive,
            "Worker should be online"
        );
        assert_eq!(
            r.gk.egress.messages.borrow().len(),
            0,
            "Should not report any event"
        );
        assert!(
            v_snap < r.gk.workers[&r.workers[0]].tokenomic.v,
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

        block_number += super::HEARTBEAT_TOLERANCE_WINDOW;

        // About to timeout then A heartbeat received, report payout event.
        let v_snap = r.gk.workers[&r.workers[0]].tokenomic.v;
        r.gk.egress.messages.borrow_mut().clear();
        with_block(block_number, |block| {
            let mut worker = r.for_worker(0);
            worker.heartbeat(1, challenge_block, 10000000);
            r.gk.process_messages(block);
        });

        assert!(
            !r.gk.workers[&r.workers[0]].unresponsive,
            "Worker should be online"
        );
        assert_eq!(
            r.gk.egress.messages.borrow().len(),
            1,
            "Should report recover event"
        );
        assert!(
            v_snap > r.gk.workers[&r.workers[0]].tokenomic.v,
            "Worker should be payed out"
        );

        {
            let settle = [msg::SettleInfo {
                pubkey: r.workers[0].clone(),
                v: 4096,
                payout: 168,
            }]
            .to_vec();

            let expected_message = EgressMessage::MiningInfoUpdate(super::MiningInfoUpdateEvent {
                block_number,
                timestamp_ms: block_ts(block_number),
                offline: Vec::new(),
                recovered_to_online: Vec::new(),
                settle,
            });
            let message = r.gk.egress.messages.borrow_mut().drain(..).nth(0).unwrap();
            assert_eq!(
                message, expected_message,
                "Should report settle for normal heartbeats"
            );
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

        // assert_eq!(r.gk.workers[&r.workers[0]].tokenomic.v, 1);
        assert!(r.gk.workers[&r.workers[0]].state.mining_state.is_some());

        block_number += super::HEARTBEAT_TOLERANCE_WINDOW;
        // About to timeout
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        assert!(!r.gk.workers[&r.workers[0]].unresponsive);

        let v_snap = r.gk.workers[&r.workers[0]].tokenomic.v;

        block_number += 1;
        // Heartbeat timed out
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        assert!(r.gk.workers[&r.workers[0]].unresponsive);
        assert_eq!(r.gk.egress.messages.borrow().len(), 1);
        {
            let offline = [r.workers[0].clone()].to_vec();
            let expected_message = EgressMessage::MiningInfoUpdate(super::MiningInfoUpdateEvent {
                block_number,
                timestamp_ms: block_ts(block_number),
                offline,
                recovered_to_online: Vec::new(),
                settle: Vec::new(),
            });
            let message = r.gk.egress.messages.borrow_mut().drain(..).nth(0).unwrap();
            assert_eq!(message, expected_message, "Should report recover to online");
        }
        assert!(
            v_snap > r.gk.workers[&r.workers[0]].tokenomic.v,
            "Worker should be slashed"
        );

        r.gk.egress.messages.borrow_mut().clear();

        let v_snap = r.gk.workers[&r.workers[0]].tokenomic.v;
        block_number += 1;
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        assert_eq!(
            r.gk.egress.messages.borrow().len(),
            0,
            "Should not report offline workers"
        );
        assert!(
            v_snap > r.gk.workers[&r.workers[0]].tokenomic.v,
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

        block_number += super::HEARTBEAT_TOLERANCE_WINDOW;
        // About to timeout
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        block_number += 1;
        // Heartbeat timed out
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        r.gk.egress.messages.borrow_mut().clear();

        // Worker already offline, don't report again until one more heartbeat received.
        let v_snap = r.gk.workers[&r.workers[0]].tokenomic.v;
        block_number += 1;
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        assert_eq!(
            r.gk.egress.messages.borrow().len(),
            0,
            "Should not report offline workers"
        );
        assert!(
            v_snap > r.gk.workers[&r.workers[0]].tokenomic.v,
            "Worker should be slashed"
        );

        let v_snap = r.gk.workers[&r.workers[0]].tokenomic.v;
        block_number += 1;
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });
        assert_eq!(
            r.gk.egress.messages.borrow().len(),
            0,
            "Should not report offline workers"
        );
        assert!(
            v_snap > r.gk.workers[&r.workers[0]].tokenomic.v,
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

        block_number += super::HEARTBEAT_TOLERANCE_WINDOW;
        // About to timeout
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        block_number += 1;
        // Heartbeat timed out
        with_block(block_number, |block| {
            r.gk.process_messages(block);
        });

        r.gk.egress.messages.borrow_mut().clear();

        // Worker offline, report recover event on the next heartbeat received.
        let v_snap = r.gk.workers[&r.workers[0]].tokenomic.v;
        block_number += 1;
        with_block(block_number, |block| {
            let mut worker = r.for_worker(0);
            worker.heartbeat(1, challenge_block, 10000000);
            r.gk.process_messages(block);
        });
        assert_eq!(
            r.gk.egress.messages.borrow().len(),
            1,
            "Should report recover event"
        );
        assert_eq!(
            v_snap, r.gk.workers[&r.workers[0]].tokenomic.v,
            "Worker should not be slashed or rewarded"
        );
        {
            let recovered_to_online = [r.workers[0].clone()].to_vec();
            let expected_message = EgressMessage::MiningInfoUpdate(super::MiningInfoUpdateEvent {
                block_number,
                timestamp_ms: block_ts(block_number),
                offline: Vec::new(),
                recovered_to_online,
                settle: Vec::new(),
            });
            let message = r.gk.egress.messages.borrow_mut().drain(..).nth(0).unwrap();
            assert_eq!(message, expected_message);
        }
    }
}
