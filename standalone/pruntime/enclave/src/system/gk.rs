use super::{Storage, TypedReceiver, WorkerState};
use phala_mq::{EcdsaMessageChannel, MessageDispatcher};
use phala_types::{
    messaging::{
        MessageOrigin, MiningInfoUpdateEvent, MiningReportEvent, SettleInfo, SystemEvent,
        WorkerEvent, WorkerEventWithKey,
    },
    WorkerPublicKey,
};

use crate::{
    std::collections::{BTreeMap, VecDeque},
    types::BlockInfo,
};

use tokenomic::{FixedPoint, TokenomicInfo};

const HEARTBEAT_TOLERANCE_WINDOW: u32 = 10;

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

pub(super) struct Gatekeeper {
    egress: EcdsaMessageChannel, // TODO.kevin: syncing the egress state while migrating.
    mining_events: TypedReceiver<MiningReportEvent>,
    system_events: TypedReceiver<SystemEvent>,
    workers: BTreeMap<WorkerPublicKey, WorkerInfo>,
}

impl Gatekeeper {
    pub fn new(recv_mq: &mut MessageDispatcher, egress: EcdsaMessageChannel) -> Self {
        Self {
            egress,
            mining_events: recv_mq.subscribe_bound(),
            system_events: recv_mq.subscribe_bound(),
            workers: Default::default(),
        }
    }

    pub fn process_messages(&mut self, block: &BlockInfo<'_>, _storage: &Storage) {
        let sum_share: FixedPoint = self
            .workers
            .values()
            .map(|info| tokenomic::calc_share(&info.tokenomic))
            .sum();

        let mut processor = GKMessageProcesser {
            state: self,
            block,
            report: MiningInfoUpdateEvent::new(block.now_ms),
            tokenomic_params: tokenomic::test_params(), // TODO.kevin: replace with real params
            sum_share,
        };

        processor.process();

        let report = processor.report;

        if !report.is_empty() {
            self.egress.send(&report);
        }
    }
}

struct GKMessageProcesser<'a> {
    state: &'a mut Gatekeeper,
    block: &'a BlockInfo<'a>,
    report: MiningInfoUpdateEvent,
    tokenomic_params: tokenomic::Params,
    sum_share: FixedPoint,
}

impl GKMessageProcesser<'_> {
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
                let v = tokenomic::update_v_slash(&worker_info.tokenomic, &params);
                worker_info.tokenomic.v = v;
            } else if !worker_info.heartbeat_flag {
                // case1: Idle, no event
                let v = tokenomic::update_v_idle(&worker_info.tokenomic, &params);
                worker_info.tokenomic.v = v;
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
                tokenomic.p_instant =
                    tokenomic::calc_p_instant(tokenomic, self.block.now_ms, iterations);
                tokenomic.challenge_time_last = challenge_time;
                tokenomic.iteration_last = iterations;

                if worker_info.unresponsive {
                    // case5: Unresponsive, successful heartbeat.
                } else {
                    // case2: Idle, successful heartbeat, report to pallet
                    let (v, payout) = tokenomic::update_v_heartbeat(
                        &worker_info.tokenomic,
                        &self.tokenomic_params,
                        self.sum_share,
                        self.block.now_ms,
                    );

                    worker_info.tokenomic.v = v;
                    worker_info.tokenomic.v_last = v;
                    worker_info.tokenomic.v_update_at = self.block.now_ms;

                    self.report.settle.push(SettleInfo {
                        pubkey: worker_pubkey.clone(),
                        v: v.to_bits(),
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
            let worker = WorkerInfo::new(pubkey.clone());
            let _ = self.state.workers.entry(pubkey.clone()).or_insert(worker);
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
                            session_id: _,
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
                            // No, we may need to report a Stop event in worker.
                            // Then GK report the final V to pallet, when observed the Stop event from worker.
                            // The pallet wait for the final V report in CoolingDown state.
                            // Pallet  ---------(Stop)--------> Worker
                            // Worker  ----(Rest Heartbeats)--> *
                            // Worker  --------(Stopped)------> *
                            // GK      --------(Final V)------> Pallet
                        }
                        WorkerEvent::MiningEnterUnresponsive => {}
                        WorkerEvent::MiningExitUnresponsive => {}
                    }
                }
            }
            SystemEvent::HeartbeatChallenge(_) => {}
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
    pub use fixed::types::U116F12 as FixedPoint;
    use fixed_sqrt::FixedSqrt as _;

    fn fp(n: u64) -> FixedPoint {
        FixedPoint::from_num(n)
    }

    fn pow2(v: FixedPoint) -> FixedPoint {
        v * v
    }

    fn conf_score(level: u8) -> FixedPoint {
        match level {
            1 | 2 | 3 => fp(1),
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
            slash_rate: fp(1) / 1000, // 0.001
            budget_per_sec: fp(10),
            v_max: fp(30000),
            alpha: fp(287) / 10000, // 0.0287
        }
    }

    pub fn update_v_idle(state: &TokenomicInfo, params: &Params) -> FixedPoint {
        let cost_idle = (params.alpha * state.p_bench + fp(15)) / params.pha_rate / fp(365);
        let perf_multiplier = if state.p_bench == fp(0) {
            fp(0)
        } else {
            state.p_instant / state.p_bench
        };
        let v = state.v + perf_multiplier * ((params.rho - fp(1)) * state.v + cost_idle);
        v.min(params.v_max)
    }

    pub fn update_v_heartbeat(
        state: &TokenomicInfo,
        params: &Params,
        sum_share: FixedPoint,
        now_ms: u64,
    ) -> (FixedPoint, FixedPoint) {
        if sum_share == fp(0) {
            return (state.v, fp(0));
        }
        let dt = fp(now_ms - state.v_update_at) / 1000;
        let dv = state.v - state.v_last;
        let budget = params.budget_per_sec * dt;
        let share = calc_share(state);
        let w = dv.max(fp(0)).min(share / sum_share * budget);
        let v = state.v - w;
        (v, w)
    }

    pub fn update_v_slash(state: &TokenomicInfo, params: &Params) -> FixedPoint {
        state.v - (state.v * params.slash_rate)
    }

    pub fn calc_share(state: &TokenomicInfo) -> FixedPoint {
        (pow2(state.v) + pow2(fp(2) * state.p_instant * conf_score(state.confidence_level))).sqrt()
    }

    pub fn calc_p_instant(state: &TokenomicInfo, now: u64, iterations: u64) -> FixedPoint {
        let dt = fp(now - state.challenge_time_last) / 1000;
        let p = fp(iterations - state.iteration_last) / dt * 6; // 6s iterations
        p.min(state.p_bench * fp(12) / fp(10))
    }
}
