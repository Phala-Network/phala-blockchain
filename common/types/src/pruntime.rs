use alloc::vec::Vec;
use codec::{Encode, Decode};

pub type RawStorageKey = Vec<u8>;
pub type StorageProof = Vec<Vec<u8>>;

#[derive(Debug, Encode)]
pub struct StorageKV(pub RawStorageKey, pub Vec<u8>);

#[derive(Debug, Encode)]
pub struct OnlineWorkerSnapshot {
    pub worker_state_kv: Vec<StorageKV>,
    pub online_workers_kv: StorageKV,
    pub proof: StorageProof,
}
