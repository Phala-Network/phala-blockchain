use rocksdb::{
    AsColumnFamilyRef, DBAccess, DBIteratorWithThreadMode, IteratorMode, SnapshotWithThreadMode,
};

use super::{ffi, DBError, Database};

pub struct Snapshot {
    db: Database,
    snapshot_ptr: *const ffi::rocksdb_snapshot_t,
}

unsafe impl Send for Snapshot {}
unsafe impl Sync for Snapshot {}

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe { self.db.release_snapshot(self.snapshot_ptr) }
    }
}

impl Snapshot {
    pub fn new(db: Database) -> Self {
        Self {
            snapshot_ptr: unsafe { db.create_snapshot() },
            db,
        }
    }

    pub fn iterator(&self, mode: IteratorMode) -> DBIteratorWithThreadMode<Self> {
        self.rocks_snapshot().iterator(mode)
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DBError> {
        self.rocks_snapshot().get(key)
    }

    fn rocks_snapshot(&self) -> SnapshotWithThreadMode<Self> {
        SnapshotWithThreadMode::new(self)
    }
}

impl DBAccess for Snapshot {
    unsafe fn create_snapshot(&self) -> *const ffi::rocksdb_snapshot_t {
        self.snapshot_ptr
    }

    unsafe fn release_snapshot(&self, _snapshot: *const ffi::rocksdb_snapshot_t) {}

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
