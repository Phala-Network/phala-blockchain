use crate::std::prelude::v1::*;
use anyhow::Result;
use core::fmt;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

use parity_scale_codec::{Decode, Encode, Error as DecodeError, FullCodec};
use phala_types::{
    pruntime::StorageKV, BlockRewardInfo, SignedWorkerMessage, WorkerMessagePayload,
    WorkerStateEnum,
};
use sp_core::{ecdsa, hashing::blake2_256, storage::StorageKey, U256};

use crate::contracts::AccountIdWrapper;
use crate::light_validation::utils::storage_prefix;
use crate::msg_channel::MsgChannel;
use crate::OnlineWorkerSnapshot;

mod comp_election;

pub type CommandIndex = u64;
type PhalaEvent = phala::RawEvent<sp_runtime::AccountId32, u128>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionStatus {
    Ok,
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
    pub command: String,
    pub status: TransactionStatus,
}

#[derive(Default)]
pub struct System {
    // Keys and identity
    pub id_key: Option<ecdsa::Pair>,
    pub id_pubkey: Vec<u8>,
    pub hashed_id: U256,
    pub machine_id: Vec<u8>,
    // Computation task electino
    pub comp_elected: bool,
    // Transaction
    pub receipts: BTreeMap<CommandIndex, TransactionReceipt>,
    // Messageing
    pub egress: MsgChannel,
}

impl System {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_id(&mut self, pair: &ecdsa::Pair) {
        let pubkey = ecdsa::Public::from(pair.clone());
        let raw_pubkey: &[u8] = pubkey.as_ref();
        let pkh = blake2_256(raw_pubkey);
        self.id_key = Some(pair.clone());
        self.id_pubkey = raw_pubkey.to_vec();
        self.hashed_id = pkh.into();
        info!("System::set_id: hashed identity key: {:?}", self.hashed_id);
    }

    pub fn set_machine_id(&mut self, machine_id: Vec<u8>) {
        self.machine_id = machine_id;
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
                Request::GetWorkerEgress { start_sequence } => {
                    let pending_msgs: Vec<SignedWorkerMessage> = self
                        .egress
                        .queue
                        .iter()
                        .filter(|msg| msg.data.sequence >= start_sequence)
                        .cloned()
                        .collect();
                    Ok(Response::GetWorkerEgress {
                        length: pending_msgs.len(),
                        encoded_egress_b64: base64::encode(&pending_msgs.encode()),
                    })
                } // If we add more unhandled queries:
                  //   _ => Err(Error::Other("Unknown command".to_string()))
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp,
        }
    }

    pub fn feed_event(&mut self) -> EventHandler {
        EventHandler {
            system: self,
            seed: None,
            snapshot: None,
            new_round: false,
        }
    }

    fn handle_reward_seed(
        &mut self,
        blocknum: chain::BlockNumber,
        reward_info: &BlockRewardInfo,
    ) -> Result<()> {
        info!(
            "System::handle_reward_seed({}, {:?})",
            blocknum, reward_info
        );
        let x = self.hashed_id ^ reward_info.seed;
        let online_hit = x <= reward_info.online_target;
        let compute_hit = self.comp_elected && x <= reward_info.compute_target;

        // Push queue when necessary
        if online_hit || compute_hit {
            info!(
                "System::handle_reward_seed: x={}, online={}, compute={}, elected={}",
                x, reward_info.online_target, reward_info.compute_target, self.comp_elected,
            );
            self.egress.push(
                WorkerMessagePayload::Heartbeat {
                    block_num: blocknum as u32,
                    claim_online: online_hit,
                    claim_compute: compute_hit,
                },
                self.id_key
                    .as_ref()
                    .expect("Id key not set in System contract"),
            );
        }
        Ok(())
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
        event: &PhalaEvent,
        storage: &crate::Storage,
    ) -> Result<()> {
        match event {
            // Reset the egress queue once we detected myself is re-registered
            phala::RawEvent::WorkerRegistered(_stash, pubkey, _machine_id) => {
                if pubkey == &self.system.id_pubkey {
                    info!("System::handle_event: Reset MsgChannel due to WorkerRegistered");
                    self.system.egress = Default::default();
                }
            }
            phala::RawEvent::WorkerReset(_stash, machine_id) => {
                // Not perfect because we only have machine_id but not pubkey here.
                if machine_id == &self.system.machine_id {
                    info!("System::handle_event: Reset MsgChannel due to WorkerReset");
                    self.system.egress = Default::default();
                }
            }
            // Handle other events
            phala::RawEvent::WorkerMessageReceived(_stash, pubkey, seq) => {
                // Advance the egress queue messages
                if pubkey == &self.system.id_pubkey {
                    info!("System::handle_event: Message confirmed (seq={})", seq);
                    self.system.egress.received(*seq);
                }
            }
            phala::RawEvent::RewardSeed(reward_info) => {
                self.seed = Some(reward_info.seed);
                self.system.handle_reward_seed(block_number, &reward_info)?;
            }
            phala::RawEvent::NewMiningRound(round) => {
                info!("System::handle_event: new mining round ({})", round);
                // Save the snapshot for later use
                self.snapshot = snapshot_online_worker(storage)?;
                self.new_round = true;
            }
            _ => (),
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
    let online_workers_kv = StorageKV::<u32>(online_workers_key, online_workers);
    let compute_workers_kv = StorageKV::<u32>(compute_workers_key, compute_workers);
    let worker_state_kv = storage_kv_from_data(online_worker_data);
    let stake_received_kv = storage_kv_from_data(stake_received_data);

    Ok(Some(crate::OnlineWorkerSnapshot {
        worker_state_kv,
        stake_received_kv,
        online_workers_kv,
        compute_workers_kv,
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
    GetWorkerEgress { start_sequence: u64 },
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
