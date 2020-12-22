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


pub type CommandIndex = u64;
type PhalaEvent = phala::RawEvent<sp_runtime::AccountId32, u128, u128>;

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
    pub id_key: Option<ecdsa::Pair>,
    pub id_pubkey: Vec<u8>,
    pub hashed_id: U256,
    pub machine_id: Vec<u8>,
    pub receipts: BTreeMap<CommandIndex, TransactionReceipt>,
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

    pub fn handle_event(&mut self, blocknum: chain::BlockNumber, event: &PhalaEvent) -> Result<(), Error> {
        match event {
            phala::RawEvent::WorkerRegistered(_stash, pubkey, _machine_id) => {
                // Reset the egress queue once we detected myself is re-registered
                if pubkey == &self.id_pubkey {
                    println!("System::handle_event: Reset MsgChannel");
                    self.egress = Default::default();
                }
            },
            phala::RawEvent::WorkerMessageReceived(_stash, pubkey, seq) => {
                println!("System::handle_event: Message confirmed (seq={})", seq);
                // Advance the egress queue messages
                if pubkey == &self.id_pubkey {
                    self.egress.received(*seq);
                }
            },
            phala::RawEvent::RewardSeed(reward_info) => {
                self.handle_reward_seed(blocknum, &reward_info)?;
            },
            _ => ()
        };
        Ok(())
    }

    fn handle_reward_seed(&mut self, blocknum: chain::BlockNumber, reward_info: &BlockRewardInfo)
    -> Result<(), Error> {
        println!("System::handle_reward_seed({}, {:?})", blocknum, reward_info);
        let x = self.hashed_id ^ reward_info.seed;
        let online_hit = x <= reward_info.online_target;
        // TODO: consider the compute_target only if we are chosen:
        // let _compute_hit = x <= self.compute_target;

        // Push queue when necessary
        if online_hit {
            println!("System::handle_reward_seed: online hit ({} < {})!", x, reward_info.online_target);
            self.egress.push(WorkerMessagePayload::Heartbeat {
                block_num: blocknum as u32,
                claim_online: true,
                claim_compute: false,
            },
            self.id_key.as_ref().expect("Id key not set in System contract"));
        }
        Ok(())
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
