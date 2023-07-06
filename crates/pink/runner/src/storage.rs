use im::OrdMap;
use phala_trie_storage::{default_db_type, DBType, RocksDB};
use pink_capi::{types::Hash, v1::ocall::StorageChanges};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

enum StorageAdapter {
    RocksDB(RocksDB),
    Memory(OrdMap<Vec<u8>, (i32, Vec<u8>)>),
}

impl Default for StorageAdapter {
    fn default() -> Self {
        match default_db_type() {
            DBType::Memory => StorageAdapter::Memory(Default::default()),
            DBType::RocksDB => StorageAdapter::RocksDB(Default::default()),
        }
    }
}

impl Serialize for StorageAdapter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            StorageAdapter::RocksDB(db) => db.serialize(serializer),
            StorageAdapter::Memory(mdb) => mdb.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for StorageAdapter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match default_db_type() {
            DBType::Memory => Deserialize::deserialize(deserializer).map(StorageAdapter::Memory),
            DBType::RocksDB => Deserialize::deserialize(deserializer).map(StorageAdapter::RocksDB),
        }
    }
}

impl Clone for StorageAdapter {
    fn clone(&self) -> Self {
        match self {
            StorageAdapter::RocksDB(kvdb) => StorageAdapter::RocksDB(kvdb.snapshot()),
            StorageAdapter::Memory(map) => StorageAdapter::Memory(map.clone()),
        }
    }
}

impl StorageAdapter {
    fn get(&self, key: &[u8]) -> Option<(Vec<u8>, i32)> {
        match self {
            StorageAdapter::RocksDB(kvdb) => {
                kvdb.get_r(key).expect("Failed to get key from RocksDB")
            }
            StorageAdapter::Memory(mdb) => {
                let (rc, v) = mdb.get(key).cloned()?;
                Some((v, rc))
            }
        }
    }

    fn consolidate<K: AsRef<[u8]>>(&mut self, other: impl Iterator<Item = (K, (Vec<u8>, i32))>) {
        match self {
            StorageAdapter::RocksDB(kvdb) => kvdb.consolidate(other),
            StorageAdapter::Memory(mdb) => {
                for (key, (value, rc)) in other {
                    if rc == 0 {
                        continue;
                    }

                    let key = key.as_ref();

                    let pv = mdb.get(key).cloned();

                    let raw_value = match pv {
                        None => (rc, value),
                        Some((mut orc, mut d)) => {
                            if orc <= 0 {
                                d = value;
                            }

                            orc += rc;

                            if orc == 0 {
                                mdb.remove(key);
                                continue;
                            }
                            (orc, d)
                        }
                    };
                    mdb.insert(key.to_vec(), raw_value);
                }
            }
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ClusterStorage {
    root: Option<Hash>,
    kv_store: StorageAdapter,
}

impl ClusterStorage {
    pub fn default_memdb() -> Self {
        Self {
            root: None,
            kv_store: StorageAdapter::Memory(Default::default()),
        }
    }

    pub fn root(&self) -> Option<Hash> {
        self.root
    }

    pub fn set_root(&mut self, root: Hash) {
        self.root = Some(root);
    }

    pub fn get(&self, key: &[u8]) -> Option<(i32, Vec<u8>)> {
        let (value, rc) = self.kv_store.get(key)?;
        Some((rc, value))
    }

    pub fn commit(&mut self, root: Hash, changes: StorageChanges) {
        self.kv_store.consolidate(changes.into_iter());
        self.set_root(root);
    }
}
