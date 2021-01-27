use alloc::vec::Vec;
use codec::{Encode, Decode, FullCodec};

pub type RawStorageKey = Vec<u8>;
pub type StorageProof = Vec<Vec<u8>>;

#[derive(Debug, Encode, Decode)]
pub struct StorageKV<T: FullCodec>(pub RawStorageKey, pub T);

#[derive(Debug, Encode, Decode)]
pub struct OnlineWorkerSnapshot<BlockNumber, Balance>
where
	BlockNumber: FullCodec,
	Balance: FullCodec,
{
	pub worker_state_kv: Vec<StorageKV<super::WorkerInfo<BlockNumber>>>,
	pub stake_received_kv: Vec<StorageKV<Balance>>,
	pub online_workers_kv: StorageKV<u32>,
	pub compute_workers_kv: StorageKV<u32>,
    pub proof: StorageProof,
}
