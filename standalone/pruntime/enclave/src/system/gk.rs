use super::{Storage, TypedReceiver, WorkerState};
use phala_mq::{EcdsaMessageChannel, MessageDispatcher};
use phala_types::{
    messaging::{
        MessageOrigin, MiningInfoUpdateEvent, MiningReportEvent, SettleInfo, SystemEvent,
        WorkerEvent, WorkerEventWithKey,
    },
    WorkerPublicKey,
};

use crate::{std::collections::{BTreeMap, VecDeque}, types::BlockInfo};

const HEARTBEAT_TOLERANCE_WINDOW: u32 = 10;

struct WorkerInfo {
    state: WorkerState,
    waiting_heartbeats: VecDeque<chain::BlockNumber>,
    offline: bool,
    v: u64,
}

impl WorkerInfo {
    fn new(pubkey: WorkerPublicKey) -> Self {
        Self {
            state: WorkerState::new(pubkey),
            waiting_heartbeats: Default::default(),
            offline: false,
            v: 0, // TODO.kevin: initialize it according to the benchmark
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
        let mut processor = GKMessageProcesser {
            state: self,
            block,
            report: MiningInfoUpdateEvent::new(block.now),
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
}

impl GKMessageProcesser<'_> {
    fn process(&mut self) {
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

    fn block_post_process(&mut self) {
        for worker_info in self.state.workers.values_mut() {
            let mut tracker = WorkerSMTracker {
                waiting_heartbeats: &mut worker_info.waiting_heartbeats,
                v: &mut worker_info.v,
            };
            worker_info
                .state
                .on_block_processed(self.block, &mut tracker);

            // Only report once for each late heartbeat.
            if worker_info.offline {
                continue;
            }

            if let Some(&hb_sent_at) = worker_info.waiting_heartbeats.get(0) {
                if self.block.block_number - hb_sent_at > HEARTBEAT_TOLERANCE_WINDOW {
                    self.report.offline.push(worker_info.state.pubkey.clone());
                    worker_info.offline = true;
                }
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
                block_num,
                mining_start_time: _,
                iterations: _,
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

                if Some(&block_num) != worker_info.waiting_heartbeats.get(0) {
                    error!("Fatal error: Unexpected heartbeat {:?}", event);
                    error!("Sent from worker {}", hex::encode(worker_pubkey));
                    error!("Waiting heartbeats {:#?}", worker_info.waiting_heartbeats);
                    // The state has been poisoned. Make no sence to keep moving on.
                    panic!("GK or Worker state poisoned");
                }

                // The oldest one comfirmed.
                let _ = worker_info.waiting_heartbeats.pop_front();
                worker_info.offline = false;

                // TODO.kevin: Calculate the V according to the tokenomic design.
                worker_info.v += 1;

                self.report.settle.push(SettleInfo {
                    pubkey: worker_info.state.pubkey.clone(),
                    v: worker_info.v,
                    payout: 0,
                })
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
            event: WorkerEvent::Registered,
        }) = &event
        {
            let _ = self
                .state
                .workers
                .entry(pubkey.clone())
                .or_insert(WorkerInfo::new(pubkey.clone()));
        }

        for worker_info in self.state.workers.values_mut() {
            // Replay the event on worker state, and collect the egressed heartbeat into waiting_heartbeats.
            let mut tracker = WorkerSMTracker {
                waiting_heartbeats: &mut worker_info.waiting_heartbeats,
                v: &mut worker_info.v,
            };
            worker_info
                .state
                .process_event(self.block, &event, &mut tracker, false);
        }
    }
}

struct WorkerSMTracker<'a> {
    waiting_heartbeats: &'a mut VecDeque<chain::BlockNumber>,
    v: &'a mut u64,
}

impl super::WorkerStateMachineCallback for WorkerSMTracker<'_> {
    fn heartbeat(
        &mut self,
        block_num: runtime::BlockNumber,
        _mining_start_time: u64,
        _iterations: u64,
    ) {
        self.waiting_heartbeats.push_back(block_num);
    }

    fn init_v(&mut self, init_v: u64) {
        *self.v = init_v;
    }
}
