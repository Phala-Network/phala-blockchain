use std::{
    ffi::c_void,
    fmt::Display,
    marker::PhantomData,
    ptr::null,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use hash_db::Hasher;
use log::info;
use parity_scale_codec::{Decode, Encode};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use sp_state_machine::{DBValue, DefaultError, TrieBackendStorage};

use librocksdb_sys as ffi;
use rocksdb::{
    AsColumnFamilyRef, DBAccess, MultiThreaded, Options, SnapshotWithThreadMode, TransactionDB,
};

use crate::{
    memdb::{HashKey, KeyFunction},
    MemoryDB,
};

pub struct RocksHashDB<H: Hasher> {
    db: Arc<TransactionDB<MultiThreaded>>,
    db_snapshot: *const c_void,
    hashed_null_node: H::Out,
    null_node_data: DBValue,
    sn: usize,
}

fn new_db() -> (TransactionDB<MultiThreaded>, usize) {
    let todo = "Use env or take from arg";
    const CACHE_PATH: &str = "/data/protected_files/caches";
    static NEXT_SN: AtomicUsize = AtomicUsize::new(0);
    let sn = NEXT_SN.fetch_add(1, Ordering::SeqCst);
    if sn == 0 {
        // Remove the entire cache folder if it's the first instance
        if std::path::Path::new(CACHE_PATH).exists() {
            info!("Removing cache folder: {}", CACHE_PATH);
            std::fs::remove_dir_all(CACHE_PATH).expect("Failed to remove cache folder");
        }
        std::fs::create_dir_all(CACHE_PATH).expect("Failed to create cache folder");
    }
    let mut options = Options::default();
    options.set_max_open_files(256);
    options.create_if_missing(true);
    options.set_error_if_exists(true);
    let path = format!("{CACHE_PATH}/cache_{sn}",);
    let db = TransactionDB::open(&options, &Default::default(), &path).expect("Faile to open KVDB");
    (db, sn)
}

impl<H: Hasher> Default for RocksHashDB<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: Hasher> RocksHashDB<H> {
    pub fn new() -> Self {
        let mdb = MemoryDB::<H>::default();
        let (db, sn) = new_db();
        Self {
            db: Arc::new(db),
            db_snapshot: null(),
            hashed_null_node: mdb.hashed_null_node,
            null_node_data: mdb.null_node_data,
            sn,
        }
    }

    pub fn consolidate_mdb(&self, mut other: MemoryDB<H>) {
        self.consolidate(other.drain().into_iter());
    }

    pub fn consolidate(&self, other: impl Iterator<Item = (H::Out, (Vec<u8>, i32))>) {
        if self.is_snapshot() {
            panic!("Consolidate on a snapshot")
        }

        let transaction = self.db.transaction();
        for (key, (value, rc)) in other {
            if rc == 0 {
                continue;
            }

            let pv =
                decode_value(transaction.get(&key)).expect("Failed to get value from transaction");

            let raw_value = match pv {
                None => (value, rc),
                Some((mut d, mut orc)) => {
                    if orc <= 0 {
                        d = value;
                    }

                    orc += rc;

                    if orc == 0 {
                        transaction
                            .delete(&key)
                            .expect("Failed to delete key from transaction");
                        continue;
                    }
                    (d, orc)
                }
            };
            transaction
                .put(&key, raw_value.encode())
                .expect("Failed to put key in transaction");
        }
        transaction.commit().expect("Failed to commit transaction");
    }

    pub fn load(mdb: MemoryDB<H>) -> Self {
        let kvdb = Self::new();
        kvdb.consolidate_mdb(mdb);
        kvdb
    }

    fn rocks_snapshot(&self) -> SnapshotWithThreadMode<'_, Self> {
        SnapshotWithThreadMode::new(self)
    }

    pub fn snapshot(&self) -> Self {
        Self {
            db: self.db.clone(),
            db_snapshot: unsafe { self.db.create_snapshot() as _ },
            hashed_null_node: self.hashed_null_node.clone(),
            null_node_data: self.null_node_data.clone(),
            sn: self.sn,
        }
    }

    fn is_snapshot(&self) -> bool {
        !self.db_snapshot.is_null()
    }

    fn get_r(
        &self,
        key: &H::Out,
    ) -> Result<Option<(sp_state_machine::DBValue, i32)>, DefaultError> {
        let value = if self.is_snapshot() {
            self.rocks_snapshot().get(key)
        } else {
            self.db.get(key)
        };
        decode_value(value)
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
                let me = RocksHashDB::<H>::new();
                let transaction = me.db.transaction();
                while let Some((value, rc)) = seq.next_element::<(Vec<u8>, i32)>()? {
                    let key = H::hash(&value);
                    transaction
                        .put(&key, (value, rc).encode())
                        .expect("Failed to put key in transaction");
                }
                transaction.commit().expect("Failed to commit transaction");
                Ok(me)
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
        for item in self.db.iterator(rocksdb::IteratorMode::Start) {
            let (_, v) = item.expect("Failed to iterate pairs over Database");
            let element: (Vec<u8>, i32) =
                Decode::decode(&mut &v[..]).expect("Failed to decode db value");
            seq.serialize_element(&element)?;
        }
        seq.end()
    }
}

fn decode_value<E: Display>(
    value: Result<Option<Vec<u8>>, E>,
) -> Result<Option<(sp_state_machine::DBValue, i32)>, DefaultError> {
    let value = value.map_err(|err| format!("{}", err))?;
    match value {
        None => return Ok(None),
        Some(value) => {
            let (d, rc): (Vec<u8>, i32) =
                Decode::decode(&mut &value[..]).or(Err("Decode db value failed"))?;
            Ok(Some((d, rc)))
        }
    }
}

unsafe impl<H: Hasher> Send for RocksHashDB<H> {}
unsafe impl<H: Hasher> Sync for RocksHashDB<H> {}

impl<H: Hasher> Drop for RocksHashDB<H> {
    fn drop(&mut self) {
        if self.is_snapshot() {
            unsafe { self.db.release_snapshot(self.db_snapshot as _) }
        }
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

impl<H: Hasher> DBAccess for RocksHashDB<H> {
    unsafe fn create_snapshot(&self) -> *const ffi::rocksdb_snapshot_t {
        if self.is_snapshot() {
            self.db_snapshot as _
        } else {
            self.db.create_snapshot()
        }
    }

    unsafe fn release_snapshot(&self, snapshot: *const ffi::rocksdb_snapshot_t) {
        if !self.is_snapshot() {
            self.db.release_snapshot(snapshot)
        }
    }

    unsafe fn create_iterator(
        &self,
        readopts: &rocksdb::ReadOptions,
    ) -> *mut ffi::rocksdb_iterator_t {
        self.db.create_iterator(readopts)
    }

    unsafe fn create_iterator_cf(
        &self,
        cf_handle: *mut ffi::rocksdb_column_family_handle_t,
        readopts: &rocksdb::ReadOptions,
    ) -> *mut ffi::rocksdb_iterator_t {
        self.db.create_iterator_cf(cf_handle, readopts)
    }

    fn get_opt<K: AsRef<[u8]>>(
        &self,
        key: K,
        readopts: &rocksdb::ReadOptions,
    ) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        self.db.get_opt(key, readopts)
    }

    fn get_cf_opt<K: AsRef<[u8]>>(
        &self,
        cf: &impl AsColumnFamilyRef,
        key: K,
        readopts: &rocksdb::ReadOptions,
    ) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        self.db.get_cf_opt(cf, key, readopts)
    }

    fn get_pinned_opt<K: AsRef<[u8]>>(
        &self,
        key: K,
        readopts: &rocksdb::ReadOptions,
    ) -> Result<Option<rocksdb::DBPinnableSlice>, rocksdb::Error> {
        self.db.get_pinned_opt(key, readopts)
    }

    fn get_pinned_cf_opt<K: AsRef<[u8]>>(
        &self,
        cf: &impl AsColumnFamilyRef,
        key: K,
        readopts: &rocksdb::ReadOptions,
    ) -> Result<Option<rocksdb::DBPinnableSlice>, rocksdb::Error> {
        self.db.get_pinned_cf_opt(cf, key, readopts)
    }

    fn multi_get_opt<K, I>(
        &self,
        keys: I,
        readopts: &rocksdb::ReadOptions,
    ) -> Vec<Result<Option<Vec<u8>>, rocksdb::Error>>
    where
        K: AsRef<[u8]>,
        I: IntoIterator<Item = K>,
    {
        self.db.multi_get_opt(keys, readopts)
    }

    fn multi_get_cf_opt<'b, K, I, W>(
        &self,
        keys_cf: I,
        readopts: &rocksdb::ReadOptions,
    ) -> Vec<Result<Option<Vec<u8>>, rocksdb::Error>>
    where
        K: AsRef<[u8]>,
        I: IntoIterator<Item = (&'b W, K)>,
        W: AsColumnFamilyRef + 'b,
    {
        self.db.multi_get_cf_opt(keys_cf, readopts)
    }
}
