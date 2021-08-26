use super::{TypedReceiver, WorkerState};
use phala_crypto::sr25519::{Persistence, KDF};
use phala_mq::MessageDispatcher;
use phala_types::{
    messaging::{
        GatekeeperEvent, MessageOrigin, MiningInfoUpdateEvent, MiningReportEvent, RandomNumber,
        RandomNumberEvent, SettleInfo, SystemEvent, WorkerEvent, WorkerEventWithKey,
    },
    WorkerPublicKey,
};
use sp_core::{hashing, sr25519};

use crate::{
    std::collections::{BTreeMap, VecDeque},
    types::BlockInfo,
};

use crate::std::vec::Vec;
use enclave_api::prpc as pb;
use log::debug;
use msg_trait::MessageChannel;
use tokenomic::{FixedPoint, TokenomicInfo};

/// Block interval to generate pseudo-random on chain
///
/// WARNING: this interval need to be large enough considering the latency of mq
const VRF_INTERVAL: u32 = 5;

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
    last_heartbeat_for_block: chain::BlockNumber,
    last_heartbeat_at_block: chain::BlockNumber,
    last_gk_responsive_event: i32,
    last_gk_responsive_event_at_block: chain::BlockNumber,
}

impl WorkerInfo {
    fn new(pubkey: WorkerPublicKey) -> Self {
        Self {
            state: WorkerState::new(pubkey),
            waiting_heartbeats: Default::default(),
            unresponsive: false,
            tokenomic: Default::default(),
            heartbeat_flag: false,
            last_heartbeat_for_block: 0,
            last_heartbeat_at_block: 0,
            last_gk_responsive_event: 0,
            last_gk_responsive_event_at_block: 0,
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
pub(crate) struct Gatekeeper<MsgChan> {
    master_key: sr25519::Pair,
    egress: MsgChan, // TODO.kevin: syncing the egress state while migrating.
    gatekeeper_events: TypedReceiver<GatekeeperEvent>,
    mining_events: TypedReceiver<MiningReportEvent>,
    system_events: TypedReceiver<SystemEvent>,
    workers: BTreeMap<WorkerPublicKey, WorkerInfo>,
    // Randomness
    last_random_number: RandomNumber,
    // Tokenomic
    tokenomic_params: tokenomic::Params,
}

impl<MsgChan> Gatekeeper<MsgChan>
where
    MsgChan: MessageChannel,
{
    pub fn new(
        master_key: sr25519::Pair,
        recv_mq: &mut MessageDispatcher,
        egress: MsgChan,
    ) -> Self {
        egress.set_dummy(true);

        Self {
            master_key,
            egress,
            gatekeeper_events: recv_mq.subscribe_bound(),
            mining_events: recv_mq.subscribe_bound(),
            system_events: recv_mq.subscribe_bound(),
            workers: Default::default(),
            last_random_number: [0_u8; 32],
            tokenomic_params: tokenomic::test_params(),
        }
    }

    pub fn register_on_chain(&mut self) {
        info!("Gatekeeper: register on chain");
        self.egress.set_dummy(false);
    }

    #[allow(dead_code)]
    pub fn unregister_on_chain(&mut self) {
        info!("Gatekeeper: unregister on chain");
        self.egress.set_dummy(true);
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
            self.egress.push_message(report);
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
        self.egress.push_message(GatekeeperEvent::new_random_number(
            block_number,
            random_number,
            self.last_random_number,
        ));
        self.last_random_number = random_number;
    }

    pub fn worker_state(&self, pubkey: &WorkerPublicKey) -> Option<pb::WorkerState> {
        let info = self.workers.get(pubkey)?;
        Some(pb::WorkerState {
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
            last_heartbeat_for_block: info.last_heartbeat_for_block,
            last_heartbeat_at_block: info.last_heartbeat_at_block,
            last_gk_responsive_event: info.last_gk_responsive_event,
            last_gk_responsive_event_at_block: info.last_gk_responsive_event_at_block,
        })
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
        debug!("Gatekeeper: processing block {}", self.block.block_number);
        self.prepare();
        loop {
            let ok = phala_mq::select! {
                message = self.state.mining_events => match message {
                    Ok((_, event, origin)) => {
                        debug!("Processing mining report: {:?}, origin: {}",  event, origin);
                        self.process_mining_report(origin, event);
                    }
                    Err(e) => {
                        error!("Read message failed: {:?}", e);
                    }
                },
                message = self.state.system_events => match message {
                    Ok((_, event, origin)) => {
                        debug!("Processing system event: {:?}, origin: {}",  event, origin);
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
        debug!("Gatekeeper: processed block {}", self.block.block_number);
    }

    fn prepare(&mut self) {
        for worker in self.state.workers.values_mut() {
            worker.heartbeat_flag = false;
        }
    }

    fn block_post_process(&mut self) {
        for worker_info in self.state.workers.values_mut() {
            debug!(
                "[{}] block_post_process",
                hex::encode(&worker_info.state.pubkey)
            );
            let mut tracker = WorkerSMTracker {
                waiting_heartbeats: &mut worker_info.waiting_heartbeats,
            };
            worker_info
                .state
                .on_block_processed(self.block, &mut tracker);

            if worker_info.state.mining_state.is_none() {
                debug!(
                    "[{}] Mining already stopped, do nothing.",
                    hex::encode(&worker_info.state.pubkey)
                );
                continue;
            }

            if worker_info.unresponsive {
                if worker_info.heartbeat_flag {
                    debug!(
                        "[{}] case5: Unresponsive, successful heartbeat.",
                        hex::encode(&worker_info.state.pubkey)
                    );
                    worker_info.unresponsive = false;
                    self.report
                        .recovered_to_online
                        .push(worker_info.state.pubkey);
                    worker_info.last_gk_responsive_event = pb::ResponsiveEvent::ExitUnresponsive as _;
                    worker_info.last_gk_responsive_event_at_block = self.block.block_number;
                }
            } else if let Some(&hb_sent_at) = worker_info.waiting_heartbeats.get(0) {
                if self.block.block_number - hb_sent_at
                    > self.state.tokenomic_params.heartbeat_window
                {
                    debug!(
                        "[{}] case3: Idle, heartbeat failed, current={} waiting for {}.",
                        hex::encode(&worker_info.state.pubkey),
                        self.block.block_number,
                        hb_sent_at
                    );
                    self.report.offline.push(worker_info.state.pubkey);
                    worker_info.unresponsive = true;
                    worker_info.last_gk_responsive_event = pb::ResponsiveEvent::EnterUnresponsive as _;
                    worker_info.last_gk_responsive_event_at_block = self.block.block_number;
                }
            }

            let params = &self.state.tokenomic_params;
            if worker_info.unresponsive {
                debug!(
                    "[{}] case3/case4: Idle, heartbeat failed or Unresponsive, no event",
                    hex::encode(&worker_info.state.pubkey)
                );
                worker_info.tokenomic.update_v_slash(&params);
            } else if !worker_info.heartbeat_flag {
                debug!(
                    "[{}] case1: Idle, no event",
                    hex::encode(&worker_info.state.pubkey)
                );
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
                worker_info.last_heartbeat_at_block = self.block.block_number;
                worker_info.last_heartbeat_for_block = challenge_block;

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
                    debug!(
                        "[{}] Mining already stopped, ignore the heartbeat.",
                        hex::encode(&worker_info.state.pubkey)
                    );
                    return;
                };

                if session_id != mining_state.session_id {
                    debug!(
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

                if worker_info.unresponsive {
                    debug!(
                        "[{}] heartbeat handling case5: Unresponsive, successful heartbeat.",
                        hex::encode(&worker_info.state.pubkey)
                    );
                } else {
                    debug!("[{}] heartbeat handling case2: Idle, successful heartbeat, report to pallet", hex::encode(&worker_info.state.pubkey));
                    let (payout, treasury) = worker_info.tokenomic.update_v_heartbeat(
                        &self.state.tokenomic_params,
                        self.sum_share,
                        self.block.now_ms,
                        self.block.block_number,
                    );

                    // NOTE: keep the reporting order (vs the one while mining stop).
                    self.report.settle.push(SettleInfo {
                        pubkey: worker_pubkey,
                        v: worker_info.tokenomic.v.to_bits(),
                        payout: payout.to_bits(),
                        treasury: treasury.to_bits(),
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
                .entry(*pubkey)
                .or_insert_with(|| WorkerInfo::new(*pubkey));
        }

        let log_on = log::log_enabled!(log::Level::Debug);
        // TODO.kevin: Avoid unnecessary iteration for WorkerEvents.
        for worker_info in self.state.workers.values_mut() {
            // Replay the event on worker state, and collect the egressed heartbeat into waiting_heartbeats.
            let mut tracker = WorkerSMTracker {
                waiting_heartbeats: &mut worker_info.waiting_heartbeats,
            };
            debug!("for worker {}", hex::encode(&worker_info.state.pubkey));
            worker_info
                .state
                .process_event(self.block, &event, &mut tracker, log_on);
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
                                v_last: v,
                                v_update_at: self.block.now_ms,
                                v_update_block: self.block.block_number,
                                iteration_last: 0,
                                challenge_time_last: self.block.now_ms,
                                p_bench: FixedPoint::from_num(*init_p),
                                p_instant: FixedPoint::from_num(*init_p),
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
                                pubkey: worker.state.pubkey,
                                v: worker.tokenomic.v.to_bits(),
                                payout: 0,
                                treasury: 0,
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

    /// Verify on-chain random number
    fn process_random_number_event(&mut self, origin: MessageOrigin, event: RandomNumberEvent) {
        if !origin.is_gatekeeper() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        };

        let expect_random = next_random_number(
            &self.state.master_key,
            event.block_number,
            event.last_random_number,
        );
        // instead of checking the origin, we directly verify the random to avoid access storage
        if expect_random != event.random_number {
            error!("Fatal error: Expect random number {:?}", expect_random);
            panic!("GK state poisoned");
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
        debug!("Worker should emit heartbeat for {}", challenge_block);
        self.waiting_heartbeats.push_back(challenge_block);
    }
}

mod tokenomic {
    pub use fixed::types::U64F64 as FixedPoint;
    use fixed_macro::types::U64F64 as fp;
    use fixed_sqrt::FixedSqrt as _;
    use phala_types::messaging::TokenomicParameters;

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

    #[derive(Default, Clone, Copy)]
    pub struct TokenomicInfo {
        pub v: FixedPoint,
        pub v_last: FixedPoint,
        pub v_update_at: u64,
        pub v_update_block: u32,
        pub iteration_last: u64,
        pub challenge_time_last: u64,
        pub p_bench: FixedPoint,
        pub p_instant: FixedPoint,
        pub confidence_level: u8,
    }

    #[derive(Debug)]
    pub struct Params {
        rho: FixedPoint,
        slash_rate: FixedPoint,
        budget_per_block: FixedPoint,
        v_max: FixedPoint,
        cost_k: FixedPoint,
        cost_b: FixedPoint,
        treasury_ration: FixedPoint,
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
            let v = self.v + perf_multiplier * ((params.rho - fp!(1)) * self.v + cost_idle);
            self.v = v.min(params.v_max);
        }

        /// case2: Idle, successful heartbeat
        /// return payout
        pub fn update_v_heartbeat(
            &mut self,
            params: &Params,
            sum_share: FixedPoint,
            now_ms: u64,
            block_number: u32,
        ) -> (FixedPoint, FixedPoint) {
            const NO_UPDATE: (FixedPoint, FixedPoint) = (fp!(0), fp!(0));
            if sum_share == fp!(0) {
                return NO_UPDATE;
            }
            if self.v < self.v_last {
                return NO_UPDATE;
            }
            if block_number <= self.v_update_block {
                // May receive more than one heartbeat for a single worker in a single block.
                return NO_UPDATE;
            }
            let dv = self.v - self.v_last;
            let blocks = FixedPoint::from_num(block_number - self.v_update_block);
            let budget = self.share() / sum_share * params.budget_per_block * blocks;
            let to_payout = budget * params.payout_ration;
            let to_treasury = budget * params.treasury_ration;

            let actual_payout = dv.max(fp!(0)).min(to_payout); // w
            let actual_treasury = (actual_payout / to_payout) * to_treasury;

            self.v -= actual_payout;
            self.v_last = self.v;
            self.v_update_at = now_ms;
            self.v_update_block = block_number;
            (actual_payout, actual_treasury)
        }

        pub fn update_v_slash(&mut self, params: &Params) {
            self.v -= self.v * params.slash_rate;
            self.v_last = self.v;
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

mod msg_trait {
    use parity_scale_codec::Encode;
    use phala_mq::{BindTopic, MessageSigner};

    pub trait MessageChannel {
        fn push_message<M: Encode + BindTopic>(&self, message: M);
        fn set_dummy(&self, dummy: bool);
    }

    impl<T: MessageSigner> MessageChannel for phala_mq::MessageChannel<T> {
        fn push_message<M: Encode + BindTopic>(&self, message: M) {
            self.send(&message);
        }

        fn set_dummy(&self, dummy: bool) {
            self.set_dummy(dummy);
        }
    }
}

#[cfg(feature = "tests")]
pub mod tests {
    use super::{msg_trait::MessageChannel, BlockInfo, FixedPoint, Gatekeeper};
    use crate::std::{cell::RefCell, vec::Vec};
    use fixed_macro::types::U64F64 as fp;
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
        fn push_message<M: Encode + BindTopic>(&self, message: M) {
            let message = Message {
                sender: MessageOrigin::Gatekeeper,
                destination: M::topic().into(),
                payload: message.encode(),
            };
            self.messages.borrow_mut().push(message);
        }

        fn set_dummy(&self, _dummy: bool) {}
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
        let mut mq = phala_mq::MessageDispatcher::new();
        let block = BlockInfo {
            block_number,
            now_ms: block_ts(block_number),
            storage: &storage,
            recv_mq: &mut mq,
        };
        call(&block);
    }

    fn block_ts(block_number: chain::BlockNumber) -> u64 {
        block_number as u64 * 12000
    }

    pub fn run_all_tests() {
        gk_should_be_able_to_observe_worker_states();
        gk_should_not_miss_any_heartbeats_cross_session();
        gk_should_reward_normal_workers_do_not_hit_the_seed_case1();
        gk_should_report_payout_for_normal_heartbeats_case2();
        gk_should_slash_and_report_offline_workers_case3();
        gk_should_slash_offline_workers_sliently_case4();
        gk_should_report_recovered_workers_case5();
        check_tokenomic_numerics();
        test_update_p_instant();
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
}
