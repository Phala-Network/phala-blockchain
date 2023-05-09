use phala_trie_storage::RocksDB;
use pink_capi::{types::Hash, v1::ocall::StorageChanges};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct ClusterStorage {
    root: Option<Hash>,
    kv_store: RocksDB,
}

impl Clone for ClusterStorage {
    fn clone(&self) -> Self {
        Self {
            root: self.root,
            kv_store: self.kv_store.snapshot(),
        }
    }
}

impl ClusterStorage {
    pub fn root(&self) -> Option<Hash> {
        self.root
    }

    pub fn set_root(&mut self, root: Hash) {
        self.root = Some(root);
    }

    pub fn get(&self, key: &[u8]) -> Option<(i32, Vec<u8>)> {
        let (value, rc) = self
            .kv_store
            .get_r(key)
            .expect("Failed to get key from RocksDB")?;
        Some((rc, value))
    }

    pub fn commit(&mut self, root: Hash, changes: StorageChanges) {
        self.kv_store.consolidate(changes.into_iter());
        self.set_root(root);
    }
}
