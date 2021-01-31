use alloc::vec::Vec;
use core::convert::TryFrom;
use codec::{Encode, Decode, FullCodec};

use sp_core::U256;
use sp_runtime::{traits::Hash as HashT, generic::Header};

pub type RawStorageKey = Vec<u8>;
pub type StorageProof = Vec<Vec<u8>>;

#[derive(Debug, Encode, Decode, Clone)]
pub struct StorageKV<T: FullCodec + Clone>(pub RawStorageKey, pub T);

impl<T: FullCodec + Clone> StorageKV<T> {
	pub fn key(&self) -> &RawStorageKey { &self.0 }
	pub fn value(&self) -> &T { &self.1 }
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct OnlineWorkerSnapshot<BlockNumber, Balance>
where
	BlockNumber: FullCodec + Clone,
	Balance: FullCodec + Clone,
{
	pub worker_state_kv: Vec<StorageKV<super::WorkerInfo<BlockNumber>>>,
	pub stake_received_kv: Vec<StorageKV<Balance>>,
	pub online_workers_kv: StorageKV<u32>,
	pub compute_workers_kv: StorageKV<u32>,
    pub proof: StorageProof,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct HeaderToSync<BlockNumber, Hash>
where
	BlockNumber: Copy + Into<U256> + TryFrom<U256> + Clone,
	Hash: HashT
{
    pub header: Header<BlockNumber, Hash>,
    pub justification: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct BlockHeaderWithEvents<BlockNumber, Hash, Balance>
where
	BlockNumber: Copy + Into<U256> + TryFrom<U256> + FullCodec + Clone,
	Hash: HashT,
	Balance: FullCodec + Clone,
{
    pub block_header: Header<BlockNumber, Hash>,
    pub events: Option<Vec<u8>>,
    pub proof: Option<Vec<Vec<u8>>>,
	pub worker_snapshot: Option<OnlineWorkerSnapshot<BlockNumber, Balance>>
}
