use alloc::vec::Vec;

use crate::Result;

pub type Key = Vec<u8>;
pub type Value = Vec<u8>;
pub type QueueIndex = u32;

pub struct KvTransaction {
    pub accessed_keys: Vec<Key>,
    pub version_updates: Vec<Key>,
    pub value_updates: Vec<(Key, Option<Value>)>,
    pub queue_head: Option<QueueIndex>,
    pub queue_lock: Option<Key>,
}

pub trait BumpVersion {
    fn bump_version(&self, version: Option<Value>) -> Result<Value>;
}

pub trait KvSnapshot {
    /// Should be the block hash for a blockchain backend
    fn snapshot_id(&self) -> Result<Value>;

    /// Get a storage value from the snapshot
    fn get(&self, key: &[u8]) -> Result<Option<Value>>;

    /// Batch get storage values from the snapshot
    #[allow(clippy::type_complexity)]
    fn batch_get(&self, keys: &[Key]) -> Result<Vec<(Key, Option<Value>)>> {
        keys.iter()
            .map(|key| {
                let key = key.clone();
                Ok((key.clone(), self.get(&key)?))
            })
            .collect()
    }
}

pub trait KvSession {
    fn get(&mut self, key: &[u8]) -> Result<Option<Value>>;
    fn put(&mut self, key: &[u8], value: Value);
    fn delete(&mut self, key: &[u8]);
}

pub trait QueueSession {
    fn pop(&mut self) -> Result<Option<Value>>;
}

pub trait QueueIndexCodec {
    fn encode(number: QueueIndex) -> Vec<u8>;
    fn decode(raw: impl AsRef<[u8]>) -> Result<QueueIndex>;
}

pub trait AccessTracking {
    fn read(&mut self, key: &[u8]);
    fn write(&mut self, key: &[u8]);
    /// Returns (access list, version updates)
    fn collect_into(self) -> (Vec<Key>, Vec<Key>);
}

// pub trait Concat {
//     fn concat(self, other: &Self) -> Self;
// }

// impl Concat for String {
//     fn concat(self, other: &Self) -> Self {
//         self + other
//     }
// }

// impl Concat for Vec<u8> {
//     fn concat(mut self, other: &Self) -> Self {
//         self.extend_from_slice(&other);
//         self
//     }
// }
