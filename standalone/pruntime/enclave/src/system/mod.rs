use crate::{benchmark, std::prelude::v1::*, Storage};
use anyhow::Result;
use core::fmt;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use chain::pallet_registry::RegistryEvent;
use phala_mq::{
    EcdsaMessageChannel, MessageDispatcher, MessageOrigin, MessageSendQueue, TypedReceiveError,
    TypedReceiver,
};
use phala_types::{
    messaging::{BlockRewardInfo, MiningReportEvent, SystemEvent, WorkerEvent},
    WorkerPublicKey,
};
use sp_core::{ecdsa, hashing::blake2_256, U256};

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
}

#[derive(Debug)]
enum MiningState {
    Mining,
    EnteringPause,
    Paused,
}

#[derive(Debug)]
struct MiningInfo {
    state: MiningState,
    start_time: u64,
    start_iter: u64,
}

pub struct System {
    // Keys and identity
    pubkey: WorkerPublicKey,
    hashed_id: U256,
    // Transaction
    receipts: BTreeMap<CommandIndex, TransactionReceipt>,
    // Messageing
    egress: EcdsaMessageChannel,
    ingress: TypedReceiver<Event>,

    registered: bool,
    bench_state: Option<BenchState>,
    mining_state: Option<MiningInfo>,

    gatekeeper_state: gk::GatekeeperState,
}

impl System {
    pub fn new(
        pair: &ecdsa::Pair,
        send_mq: &MessageSendQueue,
        recv_mq: &mut MessageDispatcher,
    ) -> Self {
        let pubkey = ecdsa::Public::from(pair.clone());
        let raw_pubkey: &[u8] = pubkey.as_ref();
        let pkh = blake2_256(raw_pubkey);
        let hashed_id: U256 = pkh.into();
        info!("System::set_id: hashed identity key: {:?}", hashed_id);
        let sender = MessageOrigin::Worker(pubkey.clone());
        System {
            pubkey,
            hashed_id,
            receipts: Default::default(),
            egress: send_mq.channel(sender, pair.clone()),
            ingress: recv_mq.subscribe_bound(),
            registered: false,
            bench_state: None,
            mining_state: None,
            gatekeeper_state: gk::GatekeeperState::new(recv_mq),
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

    pub fn process_messages(
        &mut self,
        block_number: chain::BlockNumber,
        storage: &Storage,
    ) -> anyhow::Result<()> {
        loop {
            match self.ingress.try_next() {
                Ok(Some((_, event, sender))) => {
                    if !sender.is_pallet() {
                        error!("Invalid SystemEvent sender: {:?}", sender);
                        continue;
                    }
                    self.process_event(block_number, &event)?;
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
        if let Some(BenchState {
            start_block,
            start_time,
            start_iter,
        }) = self.bench_state
        {
            const BENCH_DURATION: u32 = 8;
            if block_number - start_block >= BENCH_DURATION {
                let report = RegistryEvent::BenchReport {
                    start_time,
                    iterations: benchmark::iteration_counter() - start_iter,
                };
                info!("Reporting benchmark: {:?}", report);
                self.egress.send(&report);
                self.bench_state = None;
                self.pause_bench_if_needed();
            }
        }

        if crate::identity::is_gatekeeper(&self.pubkey, storage) {
            self.gatekeeper_state.process_messages(block_number, storage, &self.egress);
        }
        Ok(())
    }

    fn process_event(&mut self, block_number: chain::BlockNumber, event: &Event) -> Result<()> {
        match event {
            Event::WorkerEvent(evt) => {
                if evt.pubkey != self.pubkey {
                    return Ok(());
                }

                use MiningState::*;
                use WorkerEvent::*;
                info!("System::handle_event: {:?}", evt.event);
                match evt.event {
                    Registered => {
                        self.registered = true;
                    }
                    BenchStart { start_time } => {
                        self.bench_state = Some(BenchState {
                            start_block: block_number,
                            start_time: start_time,
                            start_iter: benchmark::iteration_counter(),
                        });
                        benchmark::resume();
                    }
                    MiningStart { start_time } => {
                        self.mining_state = Some(MiningInfo {
                            state: Mining,
                            start_time: start_time,
                            start_iter: benchmark::iteration_counter(),
                        });
                        benchmark::resume();
                    }
                    MiningStop => {
                        self.mining_state = None;
                        self.pause_bench_if_needed();
                    }
                    MiningEnterUnresponsive => {
                        if let Some(info) = &mut self.mining_state {
                            if let Mining = info.state {
                                info.state = EnteringPause;
                                return Ok(());
                            }
                        }
                        error!(
                            "Unexpected event received: {:?}, mining_state= {:?}",
                            evt.event, self.mining_state
                        );
                    }
                    MiningExitUnresponsive => {
                        if let Some(info) = &mut self.mining_state {
                            if let Paused | EnteringPause = info.state {
                                info.state = Mining;
                                return Ok(());
                            }
                        }
                        error!(
                            "Unexpected event received: {:?}, mining_state= {:?}",
                            evt.event, self.mining_state
                        );
                    }
                }
            }
            Event::RewardSeed(reward_info) => {
                self.handle_reward_seed(block_number, &reward_info);
            }
        };
        Ok(())
    }

    fn handle_reward_seed(&mut self, blocknum: chain::BlockNumber, reward_info: &BlockRewardInfo) {
        info!(
            "System::handle_reward_seed({}, {:?}), registered={:?}, mining_state={:?}",
            blocknum, reward_info, self.registered, self.mining_state
        );

        // Response heartbeat

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

        // Miner state swiched to Unresponsitive, we report one more heartbeat.
        if matches!(mining_state.state, MiningState::EnteringPause) {
            mining_state.state = MiningState::Paused;
        }

        let x = self.hashed_id ^ reward_info.seed;
        let online_hit = x <= reward_info.online_target;

        // Push queue when necessary
        if online_hit {
            info!(
                "System::handle_reward_seed: x={}, online={}",
                x, reward_info.online_target,
            );
            self.egress.send(&MiningReportEvent::Heartbeat {
                block_num: blocknum as u32,
                mining_start_time: mining_state.start_time,
                iterations: benchmark::iteration_counter() - mining_state.start_iter,
            });
        }
    }

    fn pause_bench_if_needed(&self) {
        if self.bench_state.is_none() && self.mining_state.is_none() {
            benchmark::pause()
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

mod gk {
    use crate::light_validation::utils::storage_prefix;

    use super::*;

    // Example GatekeeperState
    pub(super) struct GatekeeperState {
        // TODO: Define message channels here on demond.
        sample_event0: TypedReceiver<MiningReportEvent>,
        sample_event1: TypedReceiver<MiningReportEvent>,
    }

    impl GatekeeperState {
        pub fn new(recv_mq: &mut MessageDispatcher) -> Self {
            Self {
                sample_event0: recv_mq.subscribe_bound(),
                sample_event1: recv_mq.subscribe_bound(),
            }
        }

        pub fn process_messages(
            &mut self,
            block_number: chain::BlockNumber,
            storage: &Storage,
            egress: &EcdsaMessageChannel,
        ) {
            // Reach here once per block to process mq messages.
            loop {
                let ok = phala_mq::select! {
                    event0 = self.sample_event0 => match event0 {
                        Ok(Some((_, msg, origin))) => {
                            match msg {
                                MiningReportEvent::Heartbeat {
                                    block_num,
                                    mining_start_time,
                                    iterations,
                                } => {
                                    // TODO: process the heartbeat messages here.

                                    // If some storage value is needed, fetch it from storage:
                                    let workers_key = storage_prefix("Phala", "OnlineWorkers");
                                    let _workers = storage.get(workers_key);

                                    // Send message out with mq.
                                    let message: MiningReportEvent = todo!(); // MiningReportEvent for example
                                    egress.send(&message);
                                }
                            }
                        }
                        Ok(None) => {
                            // No more message in this channel.
                        }
                        Err(e) => {
                            error!("Read message failed: {:?}", e);
                        }
                    },
                    event1 = self.sample_event1 => todo!(),
                };
                if ok.is_none() {
                    // All messages processed
                    break;
                }
            }
        }
    }
}
