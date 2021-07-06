use crate::{benchmark, std::prelude::v1::*};
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
    messaging::{BlockRewardInfo, SystemEvent, WorkerEvent, WorkerReportEvent},
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
    machine_id: Vec<u8>,
    // Transaction
    receipts: BTreeMap<CommandIndex, TransactionReceipt>,
    // Messageing
    egress: EcdsaMessageChannel,
    ingress: TypedReceiver<Event>,

    registered: bool,
    bench_state: Option<BenchState>,
    mining_state: Option<MiningInfo>,
}

impl System {
    pub fn new(
        machine_id: Vec<u8>,
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
            machine_id,
            receipts: Default::default(),
            egress: send_mq.channel(sender, pair.clone()),
            ingress: recv_mq.subscribe_bound(),
            registered: false,
            bench_state: None,
            mining_state: None,
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

    pub fn process_events(&mut self, block_number: chain::BlockNumber) -> anyhow::Result<()> {
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
        Ok(())
    }

    fn process_event(&mut self, block_number: chain::BlockNumber, event: &Event) -> Result<()> {
        match event {
            Event::WorkerEvent(evt) => {
                if evt.pubkey != self.pubkey {
                    return Ok(());
                }

                use WorkerEvent::*;
                use MiningState::*;
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
        let compute_hit = x <= reward_info.compute_target;

        // Push queue when necessary
        if online_hit || compute_hit {
            info!(
                "System::handle_reward_seed: x={}, online={}, compute={}",
                x, reward_info.online_target, reward_info.compute_target,
            );
            self.egress.send(&WorkerReportEvent::Heartbeat {
                machine_id: self.machine_id.clone(),
                block_num: blocknum as u32,
                claim_online: online_hit,
                claim_compute: compute_hit,
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
