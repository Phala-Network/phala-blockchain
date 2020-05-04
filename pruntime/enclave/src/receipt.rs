use crate::std::prelude::v1::*;
use crate::std::vec::Vec;
use serde::{de, Serialize, Deserialize, Serializer, Deserializer};
use std::collections::{BTreeMap};
use crate::contracts::{AccountIdWrapper};
use crate::std::time::SystemTime;
use crate::std::untrusted::time::SystemTimeEx;

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
	pub block_num: chain::BlockNumber,
	pub tx_hash: String,
	pub contract_id: u32,
	pub command: String,
	pub status: TransactionStatus,
	pub timestamp: u64,
}

impl Default for TransactionReceipt {
	fn default() -> Self {
		TransactionReceipt {
			block_num: 0,
			tx_hash: String::from(""),
			contract_id: 0,
			command: String::from(""),
			status: TransactionStatus::Ok,
			timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64,
		}
	}
}

pub struct Receipt {
	pub tx_receipts: BTreeMap<AccountIdWrapper, Vec<TransactionReceipt>>,
}

impl Receipt {
	pub fn new() -> Self {
		let tx_receipts = BTreeMap::<AccountIdWrapper, Vec<TransactionReceipt>>::new();
		Receipt { tx_receipts }
	}

	pub fn add_receipt(&mut self, acc: AccountIdWrapper, tr: TransactionReceipt) {
		match self.tx_receipts.clone().get_mut(&acc) {
			Some(receipts) => {
				receipts.push(tr);
				self.tx_receipts.insert(acc, receipts.to_vec());
				//println!("receipt len:{:}", receipts.len());
			},
			None => {
				let mut r = Vec::new();
				r.push(tr);
				self.tx_receipts.insert(acc, r.clone());
				//println!("receipt len:{:}", r.len());
			},
		}
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
		account: AccountIdWrapper,
		tx_hash: String,
	},
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
	QueryReceipt {
		receipts: Vec<TransactionReceipt>
	},
	Error(Error),
}
