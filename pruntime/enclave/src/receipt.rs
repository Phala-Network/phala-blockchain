use crate::std::prelude::v1::*;
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap};
use crate::contracts::AccountIdWrapper;

pub type CommandIndex = u64;

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

pub struct ReceiptStore {
	pub receipts: BTreeMap<CommandIndex, TransactionReceipt>,
}

impl ReceiptStore {
	pub fn new() -> Self {
		ReceiptStore { receipts: BTreeMap::<CommandIndex, TransactionReceipt>::new() }
	}

	pub fn add_receipt(&mut self, command_index: CommandIndex, tr: TransactionReceipt) {
		self.receipts.insert(command_index, tr);
	}

	pub fn get_receipt(&self, command_index: CommandIndex) -> Option<&TransactionReceipt> {
		self.receipts.get(&command_index)
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
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
	QueryReceipt {
		receipt: TransactionReceipt
	},
	Error(Error),
}
