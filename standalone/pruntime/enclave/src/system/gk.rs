use super::{Storage, TypedReceiver, WorkerState};
use phala_crypto::{
    aead, ecdh,
    secp256k1::{Signing, KDF},
};
use phala_mq::{EcdsaMessageChannel, MessageDispatcher};
use phala_types::{
    messaging::{
        DispatchMasterKeyEvent, MessageOrigin, MiningInfoUpdateEvent, MiningReportEvent,
        NewGatekeeperEvent, RandomNumber, RandomNumberEvent, SettleInfo, SystemEvent, WorkerEvent,
        WorkerEventWithKey,
    },
    WorkerPublicKey,
};
use sp_core::{ecdsa, hashing, Pair};

use crate::std::collections::{BTreeMap, VecDeque};
use crate::std::vec::Vec;
use crate::std::convert::TryInto;

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

    let sig = master_key.sign_data(buf.as_ref());
    hashing::blake2_256(&sig.0)
}

struct WorkerInfo {
    state: WorkerState,
    waiting_heartbeats: VecDeque<chain::BlockNumber>,
    offline: bool,
    v: u32,
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
    identity_key: ecdsa::Pair,
    egress: EcdsaMessageChannel, // TODO.kevin: syncing the egress state while migrating.
    mining_events: TypedReceiver<MiningReportEvent>,
    system_events: TypedReceiver<SystemEvent>,
    workers: BTreeMap<WorkerPublicKey, WorkerInfo>,
    // Master key
    new_gatekeeper_events: TypedReceiver<NewGatekeeperEvent>,
    dispatch_master_key_events: TypedReceiver<DispatchMasterKeyEvent>,
    master_key: Option<ecdsa::Pair>,
    // Randomness
    random_events: TypedReceiver<RandomNumberEvent>,
    last_random_number: RandomNumber,
    last_random_block: chain::BlockNumber,
}

impl Gatekeeper {
    pub fn new(
        identity_key: ecdsa::Pair,
        recv_mq: &mut MessageDispatcher,
        egress: EcdsaMessageChannel,
    ) -> Self {
        Self {
            identity_key,
            egress,
            mining_events: recv_mq.subscribe_bound(),
            system_events: recv_mq.subscribe_bound(),
            workers: Default::default(),
            new_gatekeeper_events: recv_mq.subscribe_bound(),
            dispatch_master_key_events: recv_mq.subscribe_bound(),
            master_key: None,
            random_events: recv_mq.subscribe_bound(),
            last_random_number: [0_u8; 32],
            last_random_block: 0,
        }
    }

    pub fn process_messages(&mut self, block_number: chain::BlockNumber, storage: &Storage) {
        let mut processor = GKMessageProcesser {
            state: self,
            block_number,
            storage,
            report: Default::default(),
        };

        processor.process();

        let report = processor.report;

        if !report.is_empty() {
            self.egress.send(&report);
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
            self.egress.send(&RandomNumberEvent {
                block_number: block_number,
                random_number: next_random_number(
                    master_key,
                    block_number,
                    self.last_random_number,
                ),
                last_random_number: self.last_random_number,
            })
        }
    }
}

struct GKMessageProcesser<'a> {
    state: &'a mut Gatekeeper,
    block_number: chain::BlockNumber,
    storage: &'a Storage,
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
                message = self.state.new_gatekeeper_events => match message {
                    Ok((_, event, origin)) => {
                        self.process_new_gatekeeper_event(origin, event);
                    }
                    Err(e) => {
                        error!("Read message failed: {:?}", e);
                    }
                },
                message = self.state.dispatch_master_key_events => match message {
                    Ok((_, event, origin)) => {
                        self.process_dispatch_master_key_event(origin, event);
                    }
                    Err(e) => {
                        error!("Read message failed: {:?}", e);
                    }
                },
                message = self.state.random_events => match message {
                    Ok((_, event, origin)) => {
                        self.process_random_number_event(origin, event);
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

            // Only report once for each late heartbeat.
            if worker_info.offline {
                continue;
            }

            if let Some(&hb_sent_at) = worker_info.waiting_heartbeats.get(0) {
                if self.block_number - hb_sent_at > HEARTBEAT_TOLERANCE_WINDOW {
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
            };
            worker_info
                .state
                .process_event(self.block_number, &event, &mut tracker, false);
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
        if !crate::identity::is_gatekeeper(&event.pubkey, self.storage) {
            error!("Fatal error: Invalid gatekeeper registration {:?}", event);
            panic!("GK state poisoned");
        }
        let my_pubkey = ecdsa::Public::from(self.state.identity_key.clone());
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
                let secret = ecdh::agree(&my_ecdh_key, &event.ecdh_pubkey);
                let iv = aead::generate_iv();
                let mut data = master_key.clone().to_raw_vec();
                aead::encrypt(&iv, &secret, &mut data);

                self.state.egress.send(&DispatchMasterKeyEvent {
                    dest: event.pubkey,
                    ecdh_pubkey: my_ecdh_key
                        .public()
                        .as_ref()
                        .try_into()
                        .expect("ecdh pubkey with incorrect length"),
                    encrypted_master_key: data,
                    iv: iv,
                })
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
        if !crate::identity::is_gatekeeper(&src_pubkey, self.storage) {
            error!(
                "Fatal error: Master key from invalid gatekeeper {:?}",
                event
            );
            panic!("GK state poisoned");
        }

        let my_pubkey = ecdsa::Public::from(self.state.identity_key.clone());
        if my_pubkey == event.dest {
            if self.state.master_key.is_none() {
                let my_ecdh_key = self
                    .state
                    .identity_key
                    .derive_ecdh_key()
                    .expect("ecdh key derivation should never failed with valid identity key");
                let secret = ecdh::agree(&my_ecdh_key, &event.ecdh_pubkey);
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
