use crate::std::prelude::v1::*;
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

use phala_types::{BlockRewardInfo, SignedWorkerMessage, WorkerMessagePayload};
use sp_core::U256;
use sp_core::ecdsa;
use sp_core::hashing::blake2_256;
use parity_scale_codec::Encode;

use crate::contracts::AccountIdWrapper;
use crate::msg_channel::MsgChannel;

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
    BadAccountData,
    BadAccountInfo,
    BadLedgerInfo,
    BadTrustedStateData,
    BadTrustedState,
    InvalidAccount,
    BadTransactionWithProof,
    FailedToVerify,
    FailedToGetTransaction,
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
        println!("System::set_id: hashed identity key: {:?}", self.hashed_id);
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

    pub fn handle_query(&mut self, accid_origin: Option<&chain::AccountId>, req: Request)
    -> Response {
        let inner = || -> Result<Response, Error> {
            match req {
                Request::QueryReceipt { command_index } => {
                    match self.get_receipt(command_index) {
                        Some(receipt) => {
                            let origin = accid_origin.ok_or(Error::NotAuthorized)?;
                            if receipt.account == AccountIdWrapper(origin.clone()) {
                                Ok(Response::QueryReceipt { receipt: receipt.clone() })
                            } else {
                                Err(Error::NotAuthorized)
                            }
                        },
                        None => Err(Error::Other(String::from("Transaction hash not found"))),
                    }
                },
                Request::GetWorkerEgress { start_sequence } => {
                    let pending_msgs: Vec<SignedWorkerMessage> = self.egress.queue
                        .iter()
                        .filter(|msg| msg.data.sequence >= start_sequence)
                        .cloned()
                        .collect();
                    Ok(Response::GetWorkerEgress {
                        length: pending_msgs.len(),
                        encoded_egreee_b64: base64::encode(&pending_msgs.encode())
                    })
                },
                // If we add more unhandled queries:
                //   _ => Err(Error::Other("Unknown command".to_string()))
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp
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

    fn handle_reward_seed(&mut self, blocknum: chain::BlockNumber, reward_info: &BlockRewardInfo)
    -> Result<(), Error> {
        println!("System::handle_reward_seed({}, {:?})", blocknum, reward_info);
        let x = self.hashed_id ^ reward_info.seed;
        let online_hit = x <= reward_info.online_target;
        let compute_hit = self.comp_elected && x <= reward_info.compute_target;

        // Push queue when necessary
        if online_hit || compute_hit {
            println!(
                "System::handle_reward_seed: x={}, online={}, compute={}, elected={}",
                x,
                reward_info.online_target,
                reward_info.compute_target,
                self.comp_elected,
            );
            self.egress.push(
                WorkerMessagePayload::Heartbeat {
                    block_num: blocknum as u32,
                    claim_online: online_hit,
                    claim_compute: compute_hit,
                },
                self.id_key.as_ref().expect("Id key not set in System contract"));
        }
        Ok(())
    }

    fn handle_new_round(
        &mut self, seed: U256, worker_snapshot: Option<&super::OnlineWorkerSnapshot>
    ) -> Result<(), Error> {
        if let Some(worker_snapshot) = worker_snapshot {
            println!("System::handle_new_round: new round");
            self.comp_elected = comp_election::elect(
                seed.low_u64(), &worker_snapshot, &self.machine_id);
        } else {
            println!("System::handle_new_round: no snapshot found; skipping this round");
            self.comp_elected = false;
        }
        Ok(())
    }
}

pub struct EventHandler<'a> {
    pub system: &'a mut System,
    seed: Option<U256>,
    new_round: bool,
    snapshot: Option<&'a super::OnlineWorkerSnapshot>,
}

impl<'a> EventHandler<'a> {
    pub fn feed(
        &mut self, block_context: &'a super::BlockHeaderWithEvents, event: &PhalaEvent
    ) -> Result<(), Error>
    {
        match event {
            // Reset the egress queue once we detected myself is re-registered
            phala::RawEvent::WorkerRegistered(_stash, pubkey, _machine_id) => {
                if pubkey == &self.system.id_pubkey {
                    println!("System::handle_event: Reset MsgChannel due to WorkerRegistered");
                    self.system.egress = Default::default();
                }
            },
            phala::RawEvent::WorkerRenewed(_stash, machine_id) => {
                // Not perfect because we only have machine_id but not pubkey here.
                if machine_id == &self.system.machine_id {
                    println!("System::handle_event: Reset MsgChannel due to WorkerRenewed");
                    self.system.egress = Default::default();
                }
            },
            // Handle other events
            phala::RawEvent::WorkerMessageReceived(_stash, pubkey, seq) => {
                println!("System::handle_event: Message confirmed (seq={})", seq);
                // Advance the egress queue messages
                if pubkey == &self.system.id_pubkey {
                    self.system.egress.received(*seq);
                }
            },
            phala::RawEvent::RewardSeed(reward_info) => {
                let blocknum = block_context.block_header.number;
                self.seed = Some(reward_info.seed);
                self.system.handle_reward_seed(blocknum, &reward_info)?;
            },
            phala::RawEvent::NewMiningRound(round) => {
                println!("System::handle_event: new mining round ({})", round);
                // Save the snapshot for later use
                self.snapshot = block_context.worker_snapshot.as_ref();
                self.new_round = true;
            },
            _ => ()
        };
        Ok(())
    }
}

impl<'a> Drop for EventHandler<'a> {
    fn drop(&mut self) {
        if let (true, Some(seed)) = (self.new_round, self.seed) {
            self.system.handle_new_round(seed, self.snapshot)
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
	QueryReceipt {
		command_index: CommandIndex,
    },
    GetWorkerEgress {
        start_sequence: u64,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
	QueryReceipt {
		receipt: TransactionReceipt
    },
    GetWorkerEgress {
        length: usize,
        encoded_egreee_b64: String,
    },
	Error(Error),
}
