use im::OrdMap;
use phala_crypto::sr25519::Sr25519SecretKey;
use pink_capi::types::{AccountId, Balance, Hash};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ClusterStorage {
    root: Option<Hash>,
    kv_store: OrdMap<Vec<u8>, (i32, Vec<u8>)>,
}

impl ClusterStorage {
    pub fn root(&self) -> Option<Hash> {
        self.root
    }
    pub fn set_root(&mut self, root: Hash) {
        self.root = Some(root);
    }
    pub fn get(&self, key: &[u8]) -> Option<&(i32, Vec<u8>)> {
        self.kv_store.get(key)
    }
    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>, rc: i32) {
        if rc == 0 {
            return;
        }
        match self.kv_store.get_mut(&key) {
            Some((ref mut old_rc, ref mut old_value)) => {
                *old_rc += rc;
                if *old_rc == 0 {
                    self.kv_store.remove(&key);
                } else {
                    *old_value = value;
                }
            }
            None => {
                self.kv_store.insert(key, (rc, value));
            }
        }
    }
}
