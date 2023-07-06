pub use snapshot::Snapshot;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use log::info;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_state_machine::DefaultError;

use librocksdb_sys as ffi;
use rocksdb::{Error as DBError, IteratorMode, MultiThreaded, Options, Transaction, TransactionDB};

mod snapshot;

type Database = Arc<TransactionDB<MultiThreaded>>;

use super::{decode_value, traits::KvStorage};
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
        KvStorage::consolidate(self, other)
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
        let value = self.get(key).map_err(|err| err.to_string())?;
        decode_value(value).or(Err("Decode db value failed".into()))
    }
}

impl Default for RocksDB {
    fn default() -> Self {
        Self::new()
    }
}

impl KvStorage for RocksDB {
    type Transaction<'a> = Transaction<'a, TransactionDB<MultiThreaded>>;

    fn transaction<'a>(&'a self) -> Self::Transaction<'a> {
        let RocksDB::Database { db, .. } = self else {
            panic!("Consolidate on a snapshot")
        };
        db.transaction()
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        Self::new()
    }

    fn for_each(&self, mut cb: impl FnMut(&[u8], &[u8])) {
        /// To deduplicate the two match arms
        macro_rules! db_iter {
            ($iter: expr) => {
                for item in $iter {
                    let (k, v) = item.expect("Failed to iterate pairs over Database");
                    cb(&k, &v);
                }
            };
        }
        match &self {
            RocksDB::Database { db, .. } => {
                db_iter!(db.iterator(IteratorMode::Start));
            }
            RocksDB::Snapshot(snap) => {
                db_iter!(snap.iterator(IteratorMode::Start));
            }
        }
    }

    fn snapshot(&self) -> Self
    where
        Self: Sized,
    {
        self.snapshot()
    }

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get(key).expect("Failed to get key")
    }
}

impl<'a> super::traits::Transaction for Transaction<'a, TransactionDB<MultiThreaded>> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get(key).expect("Failed to get key")
    }

    fn put(&self, key: &[u8], value: &[u8]) {
        self.put(key, value).expect("Failed to put key");
    }

    fn delete(&self, key: &[u8]) {
        self.delete(key).expect("Failed to delete key");
    }

    fn commit(self) {
        self.commit().expect("Failed to commit transaction");
    }
}

fn create_db() -> (TransactionDB<MultiThreaded>, usize) {
    let cache_dir = super::cache_dir::get();
    static NEXT_SN: AtomicUsize = AtomicUsize::new(0);
    let sn = NEXT_SN.fetch_add(1, Ordering::SeqCst);
    if sn == 0 && std::path::Path::new(&cache_dir).exists() {
        info!("Removing cache folder: {}", &cache_dir);
        std::fs::remove_dir_all(&cache_dir).expect("Failed to remove cache folder");
    }
    let mut options = Options::default();
    options.set_max_open_files(256);
    options.create_if_missing(true);
    options.set_error_if_exists(true);
    let path = format!("{cache_dir}/cache-{sn}.rocksdb",);
    let db = TransactionDB::open(&options, &Default::default(), path).expect("Faile to open KVDB");
    (db, sn)
}

impl Serialize for RocksDB {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        super::serializing::serialize_as_map(self, serializer)
    }
}

impl<'de> Deserialize<'de> for RocksDB {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        super::serializing::deserialize_from_map(deserializer)
    }
}

#[test]
fn serde_works() {
    use parity_scale_codec::Encode;

    let tmp_dir = tempfile::tempdir().unwrap();
    super::with_cache_dir(tmp_dir.path().to_str().unwrap(), || {
        let db = RocksDB::new();
        db.put(b"foo", &(vec![42u8], 1i32).encode()).unwrap();
        let ser = serde_cbor::to_vec(&db).unwrap();
        let de: RocksDB = serde_cbor::from_slice(&ser).unwrap();
        assert_eq!(de.get_r(b"foo").unwrap(), Some((vec![42], 1)));
    });
}

#[test]
fn snapshot_works() {
    let tmp_dir = tempfile::tempdir().unwrap();
    super::with_cache_dir(tmp_dir.path().to_str().unwrap(), || {
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
