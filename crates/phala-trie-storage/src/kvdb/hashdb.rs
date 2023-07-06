use hash_db::Hasher;

use parity_scale_codec::Decode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_state_machine::{DefaultError, TrieBackendStorage};

use crate::{
    memdb::{HashKey, KeyFunction},
    MemoryDB,
};

use super::{traits::KvStorage, DecodedDBValue};

pub struct HashDB<H: Hasher, DB> {
    inner: DB,
    hashed_null_node: H::Out,
    null_node_data: Vec<u8>,
}

impl<H: Hasher, DB: KvStorage> Default for HashDB<H, DB> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: Hasher, DB: KvStorage> HashDB<H, DB> {
    pub fn from_inner(inner: DB) -> Self {
        let mdb = MemoryDB::<H>::default();
        Self {
            inner,
            hashed_null_node: mdb.hashed_null_node,
            null_node_data: mdb.null_node_data,
        }
    }

    pub fn new() -> Self {
        Self::from_inner(DB::new())
    }

    pub fn consolidate(&self, mut other: MemoryDB<H>) {
        self.inner.consolidate(other.drain().into_iter());
    }

    pub fn load(mdb: MemoryDB<H>) -> Self {
        let kvdb = Self::new();
        kvdb.consolidate(mdb);
        kvdb
    }

    pub fn snapshot(&self) -> Self {
        Self {
            inner: self.inner.snapshot(),
            hashed_null_node: self.hashed_null_node,
            null_node_data: self.null_node_data.clone(),
        }
    }

    fn get_decoded(&self, key: &H::Out) -> Option<DecodedDBValue> {
        let raw = self.inner.get(key.as_ref())?;
        Some(DecodedDBValue::decode(&mut &raw[..]).expect("Failed to decode DB value"))
    }
}

impl<H: Hasher, DB: KvStorage + Send + Sync> TrieBackendStorage<H> for HashDB<H, DB> {
    type Overlay = MemoryDB<H>;

    fn get(&self, key: &H::Out, prefix: hash_db::Prefix) -> Result<Option<Vec<u8>>, DefaultError> {
        if key == &self.hashed_null_node {
            return Ok(Some(self.null_node_data.clone()));
        }
        let key = HashKey::<H>::key(key, prefix);
        match self.get_decoded(&key) {
            None => Ok(None),
            Some((d, rc)) => {
                if rc > 0 {
                    Ok(Some(d))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl<H: Hasher, DB: KvStorage> Serialize for HashDB<H, DB> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serializing::serialize_as_seq(&self.inner, serializer)
    }
}
impl<'de, H: Hasher, DB: KvStorage> Deserialize<'de> for HashDB<H, DB> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(HashDB::from_inner(super::serializing::deserialize_from_seq::<
            _,
            DB,
            H,
        >(deserializer)?))
    }
}

#[test]
fn serde_hashdb_works() {
    use hash_db::HashDB;
    use sp_core::Blake2Hasher;
    use parity_scale_codec::Encode;
    use crate::HashRocksDB;

    let cache_dir = tempfile::tempdir().unwrap();
    super::with_cache_dir(cache_dir.path().to_str().unwrap(), || {
        let mut mdb = MemoryDB::default();
        mdb.insert((&[], None), &(b"foo".to_vec(), 2).encode());
        let db = HashRocksDB::<Blake2Hasher>::new();
        db.consolidate(mdb);
        let cobr = serde_cbor::to_vec(&db).unwrap();
        let _: HashRocksDB<Blake2Hasher> = serde_cbor::from_slice(&cobr).unwrap();
    });
}
