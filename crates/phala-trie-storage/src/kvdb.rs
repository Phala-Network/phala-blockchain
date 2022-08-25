use std::{
    ffi::c_void,
    fmt::Display,
    ptr::null,
    sync::{Arc, Mutex},
};

use hash_db::Hasher;
use parity_scale_codec::{Decode, Encode};
use sp_state_machine::{DBValue, DefaultError, TrieBackendStorage};

use librocksdb_sys as ffi;
use rocksdb::{
    AsColumnFamilyRef, DBAccess, MultiThreaded, Options, SnapshotWithThreadMode, TransactionDB,
};

use crate::{
    memdb::{HashKey, KeyFunction},
    MemoryDB,
};

static GLOBAL_KVDB_INSTANCE: Mutex<Option<Arc<TransactionDB<MultiThreaded>>>> = Mutex::new(None);

pub struct KeyValueDB<H: Hasher> {
    db: Arc<TransactionDB<MultiThreaded>>,
    db_snapshot: *const c_void,
    hashed_null_node: H::Out,
    null_node_data: DBValue,
}

pub fn global_init(from_scratch: bool) {
    let mut guard = GLOBAL_KVDB_INSTANCE.lock().unwrap();
    if guard.is_some() {
        panic!("Global KVDB has already been initialized")
    }
    let mut options = Options::default();
    options.set_max_open_files(1000);
    if from_scratch {
        options.create_if_missing(true);
        options.set_error_if_exists(true);
    } else {
        options.create_if_missing(false);
        options.set_error_if_exists(false);
    };
    let db = TransactionDB::open(&options, &Default::default(), "./state_db")
        .expect("Faile to open KVDB");
    *guard = Some(Arc::new(db));
}

impl<H: Hasher> KeyValueDB<H> {
    pub fn new() -> Self {
        let mdb = MemoryDB::<H>::default();
        Self {
            db: GLOBAL_KVDB_INSTANCE
                .lock()
                .unwrap()
                .as_ref()
                .expect("Global KVDB not initialized")
                .clone(),
            db_snapshot: null(),
            hashed_null_node: mdb.hashed_null_node,
            null_node_data: mdb.null_node_data,
        }
    }

    pub fn consolidate(&self, mut other: MemoryDB<H>) {
        if self.is_snapshot() {
            panic!("Consolidate on a snapshot")
        }

        let transaction = self.db.transaction();
        for (key, (value, rc)) in other.drain() {
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
        // let db = TransactionDB::open_default("./state").expect("Open KVDB failed");
        let kvdb = Self::new();
        kvdb.consolidate(mdb);
        kvdb
    }

    pub fn rocks_snapshot(&self) -> SnapshotWithThreadMode<'_, Self> {
        SnapshotWithThreadMode::new(self)
    }

    pub fn snapshot(&self) -> Self {
        Self {
            db: self.db.clone(),
            db_snapshot: unsafe { self.db.create_snapshot() as _ },
            hashed_null_node: self.hashed_null_node.clone(),
            null_node_data: self.null_node_data.clone(),
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
            SnapshotWithThreadMode::<Self>::new(self).get(key)
        } else {
            self.db.get(key)
        };
        decode_value(value)
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

unsafe impl<H: Hasher> Send for KeyValueDB<H> {}
unsafe impl<H: Hasher> Sync for KeyValueDB<H> {}

impl<H: Hasher> Drop for KeyValueDB<H> {
    fn drop(&mut self) {
        if self.is_snapshot() {
            unsafe { self.db.release_snapshot(self.db_snapshot as _) }
        }
    }
}

impl<H: Hasher> TrieBackendStorage<H> for KeyValueDB<H> {
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

impl<H: Hasher> DBAccess for KeyValueDB<H> {
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
