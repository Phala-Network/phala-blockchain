use super::*;

use crate::std::collections::VecDeque;

struct WorkerInfo {
    state: WorkerState,
    waiting_heartbeats: VecDeque<chain::BlockNumber>,
}

impl WorkerInfo {
    fn new(pubkey: WorkerPublicKey) -> Self {
        Self {
            state: WorkerState::new(pubkey),
            waiting_heartbeats: Default::default(),
        }
    }
}

// Example GatekeeperState
pub(super) struct GatekeeperState {
    mining_events: TypedReceiver<MiningReportEvent>,
    system_events: TypedReceiver<SystemEvent>,
    workers: BTreeMap<WorkerPublicKey, WorkerInfo>,
}

impl GatekeeperState {
    pub fn new(recv_mq: &mut MessageDispatcher) -> Self {
        Self {
            mining_events: recv_mq.subscribe_bound(),
            system_events: recv_mq.subscribe_bound(),
            workers: Default::default(),
        }
    }

    pub fn process_messages(
        &mut self,
        block_number: chain::BlockNumber,
        storage: &Storage,
        egress: &EcdsaMessageChannel,
    ) {
        GKMessageProcesser {
            state: self,
            block_number,
            storage,
            egress,
        }
        .process()
    }
}

struct GKMessageProcesser<'a> {
    state: &'a mut GatekeeperState,
    block_number: chain::BlockNumber,
    storage: &'a Storage,
    egress: &'a EcdsaMessageChannel,
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
            };
            worker_info
                .state
                .on_block_processed(self.block_number, &mut tracker);
        }

        // TODO: check offline workers
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
                mining_start_time,
                iterations,
            } => {
                let worker_info = match self.state.workers.get_mut(&worker_pubkey) {
                    Some(info) => info,
                    None => {
                        error!("Unknown worker sent a {:?}", event);
                        return;
                    }
                };

                if Some(&block_num) != worker_info.waiting_heartbeats.get(0) {
                    error!("Fatal error: Unexpected heartbeat {:?}", event);
                    error!("Waiting heartbeats {:?}", worker_info.waiting_heartbeats);
                    // The state has been poisoned. Make no sence to keep move on.
                    panic!("GK or Worker state poisoned");
                }

                // The oldest one comfirmed.
                let _ = worker_info.waiting_heartbeats.pop_front();

                // TODO: update V promise and interact with pallet-mining.
            }
        }
    }

    fn process_system_event(&mut self, origin: MessageOrigin, event: SystemEvent) {
        if !origin.is_pallet() {
            error!("Invalid origin {:?} sent a {:?}", origin, event);
            return;
        }
        for worker_info in self.state.workers.values_mut() {
            // Replay the event on worker state, and collect the egressed heartbeat into waiting_heartbeats.
            let mut tracker = WorkerSMTracker {
                waiting_heartbeats: &mut worker_info.waiting_heartbeats,
            };
            worker_info
                .state
                .process_event(self.block_number, &event, &mut tracker, false);
        }
    }
}

struct WorkerSMTracker<'a> {
    waiting_heartbeats: &'a mut VecDeque<chain::BlockNumber>,
}

impl super::WorkerStateMachineCallback for WorkerSMTracker<'_> {
    fn bench_iterations(&self) -> u64 {
        0
    }

    fn bench_resume(&mut self) {}

    fn bench_pause(&mut self) {}

    fn bench_report(&mut self, _start_time: u64, _iterations: u64) {}

    fn heartbeat(
        &mut self,
        block_num: runtime::BlockNumber,
        _mining_start_time: u64,
        _iterations: u64,
    ) {
        self.waiting_heartbeats.push_back(block_num);
    }
}
