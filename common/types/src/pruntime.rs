use alloc::vec::Vec;
use codec::{Decode, Encode, FullCodec};
use core::convert::TryFrom;

use sp_core::U256;
use sp_runtime::{generic::Header, traits::Hash as HashT};
use trie_storage::ser::StorageChanges;

pub type RawStorageKey = Vec<u8>;
pub type StorageProof = Vec<Vec<u8>>;

#[derive(Debug, Encode, Decode, Clone)]
pub struct StorageKV<T: FullCodec + Clone>(pub RawStorageKey, pub T);

impl<T: FullCodec + Clone> StorageKV<T> {
    pub fn key(&self) -> &RawStorageKey {
        &self.0
    }
    pub fn value(&self) -> &T {
        &self.1
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct HeaderToSync<BlockNumber, Hash>
where
    BlockNumber: Copy + Into<U256> + TryFrom<U256> + Clone,
    Hash: HashT,
{
    pub header: Header<BlockNumber, Hash>,
    pub justification: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct BlockHeaderWithEvents<BlockNumber, Hash>
where
    BlockNumber: Copy + Into<U256> + TryFrom<U256> + FullCodec + Clone,
    Hash: HashT,
{
    pub block_header: Header<BlockNumber, Hash>,
    pub storage_changes: StorageChanges,
}
