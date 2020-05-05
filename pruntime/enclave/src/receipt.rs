use crate::std::prelude::v1::*;
use crate::std::vec::Vec;
use serde::{de, Serialize, Deserialize, Serializer, Deserializer};
use std::collections::{BTreeMap};
use crate::contracts::{AccountIdWrapper};

pub type TransactionHash = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionStatus {
	Ok,
	InsufficientBalance,
	NoBalance,
	UnknownError,
	BadContractId,
	SymbolExist,
	AssetIdNotFound,
	NotAssetOwner,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionReceipt {
	pub account: AccountIdWrapper,
	pub block_num: chain::BlockNumber,
	pub tx_hash: TransactionHash,
	pub contract_id: u32,
	pub command: String,
	pub status: TransactionStatus,
}

pub struct ReceiptStore {
	pub receipts: BTreeMap<TransactionHash, TransactionReceipt>,
}

impl ReceiptStore {
	pub fn new() -> Self {
		ReceiptStore { receipts: BTreeMap::<TransactionHash, TransactionReceipt>::new() }
	}

	pub fn add_receipt(&mut self, tx_hash: TransactionHash, tr: TransactionReceipt) {
		self.receipts.insert(tx_hash, tr);
	}

	pub fn get_receipt(&self, tx_hash: TransactionHash) -> Option<&TransactionReceipt> {
		self.receipts.get(&tx_hash)
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
	NotAuthorized,
	Other(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
	QueryReceipt {
		tx_hash: String,
	},
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
	QueryReceipt {
		receipt: TransactionReceipt
	},
	Error(Error),
}
