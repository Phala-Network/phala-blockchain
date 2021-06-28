use crate::std::prelude::v1::*;
use anyhow::Result;
use core::fmt;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

use chain::pallet_mq::MessageOriginInfo;
use parity_scale_codec::{Decode, Error as DecodeError, FullCodec};
use phala_mq::{
    EcdsaMessageChannel, MessageDispatcher, MessageOrigin, MessageSendQueue, TypedReceiveError,
    TypedReceiver,
};
use phala_types::{
    messaging::{BlockRewardInfo, SystemEvent, WorkerReportEvent},
    pruntime::StorageKV,
    WorkerStateEnum,
};
use sp_core::{ecdsa, hashing::blake2_256, storage::StorageKey, U256};

use crate::contracts::AccountIdWrapper;
use crate::light_validation::utils::storage_prefix;
use crate::OnlineWorkerSnapshot;

mod comp_election;

pub type CommandIndex = u64;

type Event = SystemEvent<chain::AccountId>;

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
    pub account: AccountIdWrapper,
    pub block_num: chain::BlockNumber,
    pub contract_id: u32,
    pub status: TransactionStatus,
}

pub struct System {
    // Keys and identity
    id_pubkey: Vec<u8>,
    hashed_id: U256,
    machine_id: Vec<u8>,
    // Computation task electino
    comp_elected: bool,
    // Transaction
    receipts: BTreeMap<CommandIndex, TransactionReceipt>,
    // Messageing
    egress: EcdsaMessageChannel,
    ingress: TypedReceiver<Event>,
    // Registered received
    active: bool,
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
        let id_pubkey = raw_pubkey.to_vec();
        let hashed_id: U256 = pkh.into();
        info!("System::set_id: hashed identity key: {:?}", hashed_id);
        let sender = MessageOrigin::Worker(pubkey);
        System {
            id_pubkey,
            hashed_id,
            machine_id,
            comp_elected: false,
            receipts: Default::default(),
            egress: send_mq.channel(sender, pair.clone()),
            ingress: recv_mq.subscribe_bound(),
            active: false,
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
                        if receipt.account == AccountIdWrapper(origin.clone()) {
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

    fn feed_event(&mut self) -> EventHandler {
        EventHandler {
            system: self,
            seed: None,
            snapshot: None,
            new_round: false,
        }
    }

    pub fn process_events(
        &mut self,
        block_number: chain::BlockNumber,
        storage: &crate::Storage,
    ) -> anyhow::Result<()> {
        let mut event_handler = self.feed_event();
        loop {
            match event_handler.system.ingress.try_next() {
                Ok(Some((_, event, sender))) => {
                    if sender != chain::Phala::message_origin() {
                        error!("Invalid SystemEvent origin: {:?}", sender);
                        continue;
                    }
                    event_handler.feed(block_number, &event, storage)?;
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
        Ok(())
    }

    fn handle_reward_seed(&mut self, blocknum: chain::BlockNumber, reward_info: &BlockRewardInfo) {
        info!(
            "System::handle_reward_seed({}, {:?}), active={}",
            blocknum, reward_info, self.active
        );

        if !self.active {
            return;
        }

        let x = self.hashed_id ^ reward_info.seed;
        let online_hit = x <= reward_info.online_target;
        let compute_hit = self.comp_elected && x <= reward_info.compute_target;

        // Push queue when necessary
        if online_hit || compute_hit {
            info!(
                "System::handle_reward_seed: x={}, online={}, compute={}, elected={}",
                x, reward_info.online_target, reward_info.compute_target, self.comp_elected,
            );
            self.egress.send(&WorkerReportEvent::Heartbeat {
                machine_id: self.machine_id.clone(),
                block_num: blocknum as u32,
                claim_online: online_hit,
                claim_compute: compute_hit,
            });
        }
    }

    fn handle_new_round(
        &mut self,
        seed: U256,
        worker_snapshot: Option<&super::OnlineWorkerSnapshot>,
    ) -> Result<()> {
        if let Some(worker_snapshot) = worker_snapshot {
            info!("System::handle_new_round: new round");
            self.comp_elected =
                comp_election::elect(seed.low_u64(), &worker_snapshot, &self.machine_id);
        } else {
            info!("System::handle_new_round: no snapshot found; skipping this round");
            self.comp_elected = false;
        }
        Ok(())
    }
}

pub struct EventHandler<'a> {
    pub system: &'a mut System,
    seed: Option<U256>,
    new_round: bool,
    snapshot: Option<super::OnlineWorkerSnapshot>,
}

impl<'a> EventHandler<'a> {
    pub fn feed(
        &mut self,
        block_number: chain::BlockNumber,
        event: &Event,
        storage: &crate::Storage,
    ) -> Result<()> {
        match event {
            Event::WorkerRegistered(_stash, pubkey, _machine_id) => {
                if pubkey == &self.system.id_pubkey {
                    info!("System::handle_event: WorkerRegistered");
                    self.system.active = true;
                }
            }
            Event::WorkerUnregistered(_stash, machine_id) => {
                // Not perfect because we only have machine_id but not pubkey here.
                if &self.system.machine_id == machine_id {
                    info!("System::handle_event: WorkerUnregistered");
                    self.system.active = false;
                }
            }
            Event::RewardSeed(reward_info) => {
                self.seed = Some(reward_info.seed);
                self.system.handle_reward_seed(block_number, &reward_info);
            }
            Event::NewMiningRound(round) => {
                info!("System::handle_event: new mining round ({})", round);
                // Save the snapshot for later use
                self.snapshot = snapshot_online_worker(storage)?;
                self.new_round = true;
            }
        };
        Ok(())
    }
}

fn snapshot_online_worker(trie: &crate::Storage) -> anyhow::Result<Option<OnlineWorkerSnapshot>> {
    // Stats numbers
    let online_workers_key = storage_prefix("Phala", "OnlineWorkers");
    let compute_workers_key = storage_prefix("Phala", "ComputeWorkers");
    let worker_state_key = storage_prefix("Phala", "WorkerState");
    let stake_received_key = storage_prefix("MiningStaking", "StakeReceived");

    fn decode<T: Decode>(mut data: &[u8]) -> Result<T, DecodeError> {
        Decode::decode(&mut data)
    }

    let online_workers: u32 = match trie.get(&online_workers_key) {
        Some(v) => decode(&v).map_err(|_| anyhow::anyhow!("Decode OnlineWorkers failed"))?,
        None => 0,
    };

    let compute_workers: u32 = match trie.get(&compute_workers_key) {
        Some(v) => decode(&v).map_err(|_| anyhow::anyhow!("Decode ComputeWorkers failed"))?,
        None => 0,
    };

    if online_workers == 0 || compute_workers == 0 {
        info!(
            "OnlineWorker or ComputeWorkers is zero ({}, {}). Skipping worker snapshot.",
            online_workers, compute_workers
        );
        return Ok(None);
    }

    info!("- Stats Online Workers: {}", online_workers);
    info!("- Stats Compute Workers: {}", compute_workers);

    // Online workers and stake received
    // TODO.kevin: take attention to the memory usage.
    let worker_data: Vec<(StorageKey, phala_types::WorkerInfo<chain::BlockNumber>)> = trie
        .pairs(&worker_state_key)
        .into_iter()
        .try_fold(Vec::new(), |mut out, value| -> anyhow::Result<_> {
            out.push((
                StorageKey(value.0),
                decode(&value.1).map_err(|_| anyhow::anyhow!("Decode worker data failed"))?,
            ));
            Ok(out)
        })?;

    let stake_received_data: Vec<(StorageKey, chain::Balance)> = trie
        .pairs(&stake_received_key)
        .into_iter()
        .try_fold(Vec::new(), |mut out, value| -> anyhow::Result<_> {
            out.push((
                StorageKey(value.0),
                decode(&value.1)
                    .map_err(|_| anyhow::anyhow!("Decode stake_received_data failed"))?,
            ));
            Ok(out)
        })?;

    let online_worker_data: Vec<_> = worker_data
        .into_iter()
        .filter(|(_k, worker_info)| match worker_info.state {
            WorkerStateEnum::<chain::BlockNumber>::Mining(_)
            | WorkerStateEnum::<chain::BlockNumber>::MiningStopping => true,
            _ => false,
        })
        .collect();

    let stashes: HashSet<&[u8]> = online_worker_data
        .iter()
        .map(|(k, _v)| account_id_from_map_key(&k.0))
        .collect();

    let stake_received_data: Vec<_> = stake_received_data
        .into_iter()
        .filter(|(k, _v)| stashes.contains(account_id_from_map_key(&k.0)))
        .collect();

    debug!("- online_worker_data: vec[{}]", online_worker_data.len());
    debug!("- stake_received_data: vec[{}]", stake_received_data.len());

    // Snapshot fields
    let worker_state_kv = storage_kv_from_data(online_worker_data);
    let stake_received_kv = storage_kv_from_data(stake_received_data);

    Ok(Some(crate::OnlineWorkerSnapshot {
        worker_state_kv,
        stake_received_kv,
        compute_workers,
    }))
}

fn storage_kv_from_data<T>(storage_data: Vec<(StorageKey, T)>) -> Vec<StorageKV<T>>
where
    T: FullCodec + Clone,
{
    storage_data
        .into_iter()
        .map(|(k, v)| crate::StorageKV(k.0, v))
        .collect()
}

fn account_id_from_map_key(key: &[u8]) -> &[u8] {
    // (twox128(module) + twox128(storage) + black2_128_concat(accountid))
    &key[256 / 8..]
}

impl<'a> Drop for EventHandler<'a> {
    fn drop(&mut self) {
        if let (true, Some(seed)) = (self.new_round, self.seed) {
            self.system
                .handle_new_round(seed, self.snapshot.as_ref())
                .expect("System EventHandler::drop() should never fail; qed.");
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
