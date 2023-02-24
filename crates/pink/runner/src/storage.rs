use im::OrdMap;
use pink_capi::types::Hash;
use serde::{Deserialize, Serialize};

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

    fn update(&mut self, key: Vec<u8>, value: Vec<u8>, rc: i32) {
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

    pub fn commit(&mut self, root: Hash, changes: Vec<(Vec<u8>, (Vec<u8>, i32))>) {
        for (key, (value, rc)) in changes {
            self.update(key, value, rc);
        }
        self.set_root(root);
    }
}
