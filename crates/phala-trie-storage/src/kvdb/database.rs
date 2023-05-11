use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use log::info;
use parity_scale_codec::{Decode, Encode};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};
use sp_state_machine::DefaultError;

use rocksdb::{Error as DBError, IteratorMode, MultiThreaded, Options, TransactionDB};

use super::{Database, Snapshot};
pub enum RocksDB {
    Database { db: Database, sn: usize },
    Snapshot(Arc<Snapshot>),
}

impl RocksDB {
    pub fn new() -> Self {
        let (db, sn) = create_db();
        Self::Database {
            db: Arc::new(db),
            sn,
        }
    }

    pub fn snapshot(&self) -> Self {
        match self {
            Self::Database { db, .. } => Self::Snapshot(Arc::new(Snapshot::new(db.clone()))),
            Self::Snapshot(snap) => Self::Snapshot(snap.clone()),
        }
    }

    pub fn consolidate<K: AsRef<[u8]>>(&self, other: impl Iterator<Item = (K, (Vec<u8>, i32))>) {
        let RocksDB::Database { db, .. } = self else {
            panic!("Consolidate on a snapshot")
        };

        let transaction = db.transaction();
        for (key, (value, rc)) in other {
            if rc == 0 {
                continue;
            }

            let key = key.as_ref();

            let pv =
                decode_value(transaction.get(key)).expect("Failed to get value from transaction");

            let raw_value = match pv {
                None => (value, rc),
                Some((mut d, mut orc)) => {
                    if orc <= 0 {
                        d = value;
                    }

                    orc += rc;

                    if orc == 0 {
                        transaction
                            .delete(key)
                            .expect("Failed to delete key from transaction");
                        continue;
                    }
                    (d, orc)
                }
            };
            transaction
                .put(key, raw_value.encode())
                .expect("Failed to put key in transaction");
        }
        transaction.commit().expect("Failed to commit transaction");
    }

    #[cfg(test)]
    pub(crate) fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DBError> {
        let Self::Database { db, .. } = self else {
            panic!("Put on a snapshot")
        };
        db.put(key, value)
    }

    #[cfg(test)]
    pub(crate) fn delete(&self, key: &[u8]) -> Result<(), DBError> {
        let Self::Database { db, .. } = self else {
            panic!("Delete on a snapshot")
        };
        db.delete(key)
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DBError> {
        match self {
            RocksDB::Database { db, .. } => db.get(key),
            RocksDB::Snapshot(snap) => snap.get(key),
        }
    }

    pub fn get_r(
        &self,
        key: &[u8],
    ) -> Result<Option<(sp_state_machine::DBValue, i32)>, DefaultError> {
        decode_value(self.get(key))
    }
}

impl Default for RocksDB {
    fn default() -> Self {
        Self::new()
    }
}

// The global cache dir for unit tests.
environmental::environmental!(test_cached_path: String);

#[cfg(test)]
pub(crate) fn with_cache_dir<T>(cache_dir: &str, f: impl FnOnce() -> T) -> T {
    let mut cache_dir = cache_dir.to_string();
    test_cached_path::using(&mut cache_dir, f)
}

pub(crate) fn create_db() -> (TransactionDB<MultiThreaded>, usize) {
    let test_path = test_cached_path::with(|path| path.clone());
    let cache_path = &test_path
        .or_else(|| std::env::var("PHALA_TRIE_CACHE_PATH").ok())
        .unwrap_or_else(|| "data/protected_files/caches".to_string());
    static NEXT_SN: AtomicUsize = AtomicUsize::new(0);
    let sn = NEXT_SN.fetch_add(1, Ordering::SeqCst);
    if sn == 0 && std::path::Path::new(cache_path).exists() {
        info!("Removing cache folder: {}", cache_path);
        std::fs::remove_dir_all(cache_path).expect("Failed to remove cache folder");
    }
    let mut options = Options::default();
    options.set_max_open_files(256);
    options.create_if_missing(true);
    options.set_error_if_exists(true);
    let path = format!("{cache_path}/cache_{sn}",);
    let db = TransactionDB::open(&options, &Default::default(), path).expect("Faile to open KVDB");
    (db, sn)
}

impl Serialize for RocksDB {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut ser = serializer.serialize_map(None)?;
        /// To deduplicate the two match arms
        macro_rules! ser_iter {
            ($iter: expr) => {
                for item in $iter {
                    let (key, v) = item.expect("Failed to iterate pairs over Database");
                    let (value, rc): (Vec<u8>, i32) =
                        Decode::decode(&mut &v[..]).expect("Failed to decode db value");
                    ser.serialize_entry(&key, &(rc, value))?;
                }
            };
        }
        match self {
            RocksDB::Database { db, .. } => {
                ser_iter!(db.iterator(IteratorMode::Start))
            }
            RocksDB::Snapshot(snap) => {
                ser_iter!(snap.iterator(IteratorMode::Start))
            }
        }
        ser.end()
    }
}

impl<'de> Deserialize<'de> for RocksDB {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MapVisitor;
        impl<'de> Visitor<'de> for MapVisitor {
            type Value = RocksDB;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (db, sn) = create_db();
                let transaction = db.transaction();
                while let Some((key, (rc, value))) = seq.next_entry::<Vec<u8>, (i32, Vec<u8>)>()? {
                    transaction
                        .put(&key, (value, rc).encode())
                        .expect("Failed to put key in transaction");
                }
                transaction.commit().expect("Failed to commit transaction");
                Ok(RocksDB::Database {
                    db: Arc::new(db),
                    sn,
                })
            }
        }
        deserializer.deserialize_map(MapVisitor)
    }
}

fn decode_value<E: Display>(
    value: Result<Option<Vec<u8>>, E>,
) -> Result<Option<(sp_state_machine::DBValue, i32)>, DefaultError> {
    let value = value.map_err(|err| err.to_string())?;
    match value {
        None => Ok(None),
        Some(value) => {
            let (d, rc): (Vec<u8>, i32) =
                Decode::decode(&mut &value[..]).or(Err("Decode db value failed"))?;
            Ok(Some((d, rc)))
        }
    }
}

#[test]
fn serde_works() {
    let cache_dir = tempfile::tempdir().unwrap();
    with_cache_dir(cache_dir.path().to_str().unwrap(), || {
        let db = RocksDB::new();
        db.put(b"foo", &(vec![42u8], 1i32).encode()).unwrap();
        let ser = serde_cbor::to_vec(&db).unwrap();
        let de: RocksDB = serde_cbor::from_slice(&ser).unwrap();
        assert_eq!(de.get_r(b"foo").unwrap(), Some((vec![42], 1)));
    });
}

#[test]
fn snapshot_works() {
    let cache_dir = tempfile::tempdir().unwrap();
    with_cache_dir(cache_dir.path().to_str().unwrap(), || {
        let db = RocksDB::new();
        db.put(b"foo", b"bar").unwrap();
        assert_eq!(db.get(b"foo").unwrap().unwrap(), b"bar");
        let snapshot = db.snapshot();
        assert_eq!(snapshot.get(b"foo").unwrap().unwrap(), b"bar");
        db.delete(b"foo").unwrap();
        assert_eq!(db.get(b"foo").unwrap(), None);
        assert_eq!(snapshot.get(b"foo").unwrap().unwrap(), b"bar");
    });
}
