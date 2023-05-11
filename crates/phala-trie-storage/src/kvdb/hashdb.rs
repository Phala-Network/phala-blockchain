use std::{marker::PhantomData, sync::Arc};

use hash_db::Hasher;

use parity_scale_codec::{Decode, Encode};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use sp_state_machine::{DBValue, DefaultError, TrieBackendStorage};

use rocksdb::IteratorMode;

use crate::{
    kvdb::database::create_db,
    memdb::{HashKey, KeyFunction},
    MemoryDB,
};

use super::RocksDB;
pub struct RocksHashDB<H: Hasher> {
    inner: RocksDB,
    hashed_null_node: H::Out,
    null_node_data: DBValue,
}

impl<H: Hasher> Default for RocksHashDB<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: Hasher> RocksHashDB<H> {
    pub fn from_inner(inner: RocksDB) -> Self {
        let mdb = MemoryDB::<H>::default();
        Self {
            inner,
            hashed_null_node: mdb.hashed_null_node,
            null_node_data: mdb.null_node_data,
        }
    }

    pub fn new() -> Self {
        Self::from_inner(RocksDB::new())
    }

    pub fn consolidate_mdb(&self, mut other: MemoryDB<H>) {
        self.inner.consolidate(other.drain().into_iter());
    }

    pub fn load(mdb: MemoryDB<H>) -> Self {
        let kvdb = Self::new();
        kvdb.consolidate_mdb(mdb);
        kvdb
    }

    pub fn snapshot(&self) -> Self {
        Self {
            inner: self.inner.snapshot(),
            hashed_null_node: self.hashed_null_node,
            null_node_data: self.null_node_data.clone(),
        }
    }

    fn get_r(
        &self,
        key: &H::Out,
    ) -> Result<Option<(sp_state_machine::DBValue, i32)>, DefaultError> {
        self.inner.get_r(key.as_ref())
    }
}

impl<'de, H: Hasher> Deserialize<'de> for RocksHashDB<H> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VecVisitor<H> {
            marker: PhantomData<H>,
        }
        impl<'de, H: Hasher> Visitor<'de> for VecVisitor<H> {
            type Value = RocksHashDB<H>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let (db, sn) = create_db();
                let transaction = db.transaction();
                while let Some((value, rc)) = seq.next_element::<(Vec<u8>, i32)>()? {
                    let key = H::hash(&value);
                    transaction
                        .put(key, (value, rc).encode())
                        .expect("Failed to put key in transaction");
                }
                transaction.commit().expect("Failed to commit transaction");
                let db = RocksHashDB::from_inner(RocksDB::Database {
                    db: Arc::new(db),
                    sn,
                });
                Ok(db)
            }
        }
        deserializer.deserialize_seq(VecVisitor {
            marker: PhantomData,
        })
    }
}

impl<H: Hasher> Serialize for RocksHashDB<H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        /// To deduplicate the two match arms
        macro_rules! ser_iter {
            ($iter: expr) => {
                for item in $iter {
                    let (_, v) = item.expect("Failed to iterate pairs over Database");
                    let element: (Vec<u8>, i32) =
                        Decode::decode(&mut &v[..]).expect("Failed to decode db value");
                    seq.serialize_element(&element)?;
                }
            };
        }
        match &self.inner {
            RocksDB::Database { db, .. } => {
                ser_iter!(db.iterator(IteratorMode::Start));
            }
            RocksDB::Snapshot(snap) => {
                ser_iter!(snap.iterator(IteratorMode::Start));
            }
        }
        seq.end()
    }
}

impl<H: Hasher> TrieBackendStorage<H> for RocksHashDB<H> {
    type Overlay = MemoryDB<H>;

    fn get(
        &self,
        key: &H::Out,
        prefix: hash_db::Prefix,
    ) -> Result<Option<sp_state_machine::DBValue>, DefaultError> {
        if key == &self.hashed_null_node {
            return Ok(Some(self.null_node_data.clone()));
        }
        let key = HashKey::<H>::key(key, prefix);
        match self.get_r(&key)? {
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

#[test]
fn serde_hashdb_works() {
    use hash_db::HashDB;
    use sp_core::Blake2Hasher;

    let cache_dir = tempfile::tempdir().unwrap();
    crate::kvdb::database::with_cache_dir(cache_dir.path().to_str().unwrap(), || {
        let mut mdb = MemoryDB::default();
        mdb.insert((&[], None), &(b"foo".to_vec(), 2).encode());
        let db = RocksHashDB::<Blake2Hasher>::new();
        db.consolidate_mdb(mdb);
        let cobr = serde_cbor::to_vec(&db).unwrap();
        let _: RocksHashDB<Blake2Hasher> = serde_cbor::from_slice(&cobr).unwrap();
    });
}
