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

use tokenomic::TokenomicInfo;

const HEARTBEAT_TOLERANCE_WINDOW: u32 = 10;
const F2U_RATE: f64 = 1000000f64;

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
        let sum_share = self
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
    sum_share: f64,
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

            if worker_info.unresponsive {
                if worker_info.heartbeat_flag {
                    // Unresponsive, successful heartbeat
                    worker_info.unresponsive = false;
                }
            } else {
                if let Some(&hb_sent_at) = worker_info.waiting_heartbeats.get(0) {
                    if self.block.block_number - hb_sent_at > HEARTBEAT_TOLERANCE_WINDOW {
                        self.report.offline.push(worker_info.state.pubkey.clone());
                        worker_info.unresponsive = true;
                    }
                }
            }

            let params = &self.tokenomic_params;
            if worker_info.unresponsive {
                // Idle, heartbeat failed or
                // Unresponsive, no event
                let v = tokenomic::update_v_slash(&worker_info.tokenomic, &params);
                worker_info.tokenomic.v = v;
            } else if !worker_info.heartbeat_flag {
                // Idle, no event
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

                // TODO.kevin: ignore heartbeats from previous mining sessions.

                worker_info.heartbeat_flag = true;

                let tokenomic = &mut worker_info.tokenomic;
                tokenomic.p_instant = tokenomic::calc_p_instant(tokenomic, self.block.now_ms, iterations);
                tokenomic.challenge_time_last = challenge_time;
                tokenomic.iteration_last = iterations;

                if !worker_info.unresponsive {
                    // Idle, successful heartbeat, report to pallet
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
                        pubkey: worker_info.state.pubkey.clone(),
                        v: (v * F2U_RATE) as _,
                        payout: (payout * F2U_RATE) as _,
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
                        WorkerEvent::BenchStart => {}
                        WorkerEvent::BenchScore(score) => {
                            worker.tokenomic.p_bench = (*score) as f64;
                        }
                        WorkerEvent::MiningStart { init_v } => {
                            let v = (*init_v) as f64 / F2U_RATE;
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
        challenge_block: runtime::BlockNumber,
        _challenge_time: u64,
        _mining_start_time: u64,
        _iterations: u64,
    ) {
        self.waiting_heartbeats.push_back(challenge_block);
    }
}

mod tokenomic {
    const CONF_SCORE: [f64; 5] = [1.0, 1.0, 1.0, 0.8, 0.7];

    #[derive(Default, Clone, Copy)]
    pub struct TokenomicInfo {
        pub v: f64,
        pub v_last: f64,
        pub v_update_at: u64,
        pub iteration_last: u64,
        pub challenge_time_last: u64,
        pub p_bench: f64,
        pub p_instant: f64,
        pub confidence_level: u8,
    }

    pub struct Params {
        pha_rate: f64,
        rho: f64,
        slash_rate: f64,
        budget_per_sec: f64,
        v_max: f64,
    }

    pub fn test_params() -> Params {
        Params {
            pha_rate: 1.0,
            rho: 1.00020,
            slash_rate: 0.001,
            budget_per_sec: 10.0,
            v_max: 30000.0,
        }
    }

    pub fn update_v_idle(state: &TokenomicInfo, params: &Params) -> f64 {
        let cost_idle = (0.0287 * state.p_bench + 15.0) / params.pha_rate / 365.0;
        let perf_multiplier = if state.p_bench == 0.0 {
            0.0
        } else {
            state.p_instant / state.p_bench
        };
        let v = state.v + perf_multiplier * ((params.rho - 1.0) * state.v + cost_idle);
        v.min(params.v_max)
    }

    pub fn update_v_heartbeat(
        state: &TokenomicInfo,
        params: &Params,
        sum_share: f64,
        now_ms: u64,
    ) -> (f64, f64) {
        if sum_share == 0.0 {
            return (state.v, 0.0);
        }
        let dt = (now_ms - state.v_update_at) as f64 / 1000.0;
        let dv = state.v - state.v_last;
        let budget = params.budget_per_sec * dt;
        let share = calc_share(state);
        let w = dv.max(0.0).min(share / sum_share * budget);
        let v = state.v - w;
        (v, w)
    }

    pub fn update_v_slash(state: &TokenomicInfo, params: &Params) -> f64 {
        state.v - (state.v * params.slash_rate)
    }

    pub fn calc_share(state: &TokenomicInfo) -> f64 {
        let conf_score = *CONF_SCORE
            .get(state.confidence_level as usize - 1)
            .unwrap_or(&0.7);
        (state.v.powf(2.0) + (2.0 * state.p_instant * conf_score).powf(2.0)).sqrt()
    }

    pub fn calc_p_instant(state: &TokenomicInfo, now: u64, iterations: u64) -> f64 {
        let dt = (now - state.challenge_time_last) as f64 / 1000.0;
        let p = (iterations - state.iteration_last) as f64 / dt * 6.0; // 6s iterations
        p.min(state.p_bench * 120.0 / 100.0)
    }
}
