mod backend;
pub mod ser;
mod snapshot;
mod transactional;
mod trie;

use parity_scale_codec::Codec;
use parity_scale_codec::Decode;
use rusty_leveldb::Options as LevelDBOptions;
use rusty_leveldb::WriteBatch;
use rusty_leveldb::DB;
use sp_state_machine::TrieBackend;
use sp_trie::MemoryDB;
use std::borrow::BorrowMut;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub use backend::{PhalaTrieBackend, PhalaTrieStorage};
pub use snapshot::TrieSnapshot;
pub use transactional::TransactionalBackend;
pub use trie::TrieEssenceStorage;

/// Storage key.
pub type StorageKey = Vec<u8>;

/// Storage value.
pub type StorageValue = Vec<u8>;

/// In memory array of storage values.
pub type StorageCollection = Vec<(StorageKey, Option<StorageValue>)>;

/// In memory arrays of storage values for multiple child tries.
pub type ChildStorageCollection = Vec<(StorageKey, StorageCollection)>;

pub(crate) const TRIE_MEMORY_MUST_HOLD_LOCK: &str = "memory MUST hold the lock now";
pub(crate) const TRIE_UNDERLYING_DB_MUST_HOLD_LOCK: &str = "disk db MUST hold the lock now";

pub(crate) struct Inner<T>(pub(crate) T, pub(crate) i32);

impl<T> From<Vec<u8>> for Inner<T>
where
    T: for<'a> From<&'a [u8]>,
{
    fn from(v: Vec<u8>) -> Self {
        let (rc, val) = v.split_at(4);
        let value = T::from(val);
        Inner(
            value,
            i32::from_be_bytes(rc.try_into().expect("reference must in Value")),
        )
    }
}

impl<T> Into<Vec<u8>> for Inner<T>
where
    T: AsRef<[u8]>,
{
    fn into(self) -> Vec<u8> {
        let mut underlying = Vec::with_capacity(self.0.as_ref().len() + 4);
        underlying.extend_from_slice(&self.1.to_be_bytes());
        underlying.extend_from_slice(self.0.as_ref());
        underlying
    }
}

const PINK_TRIE_ROOT_STORAGE_KEY: &str = "phaladb_pink_storage_key";
const SYNC_TRIE_ROOT_STORAGE_KEY: &str = "phaladb_sync_storage_key";

// PhalaDB used for keep tract root and other memory
// NOTE: you have to change the root at every checkpoint and save to transaction to
// make the DB work correctly
pub struct PhalaDB<H: hash_db::Hasher> {
    db: Arc<Mutex<DB>>,
    pink_overlay: Arc<Mutex<MemoryDB<H>>>,
    sync_overlay: Arc<Mutex<MemoryDB<H>>>,
    raw_map: BTreeMap<Vec<u8>, Vec<u8>>,
    pub pink_root: Option<H::Out>,
    pub sync_root: Option<H::Out>,
}

impl<H: hash_db::Hasher> PhalaDB<H>
where
    H::Out: Codec + Ord,
{
    pub fn open(path: impl AsRef<Path>) -> Self {
        let options = LevelDBOptions::default();
        let mut db = DB::open(path, options).expect("open underlying leveldb failed");
        let mut proot = db.get(PINK_TRIE_ROOT_STORAGE_KEY.as_bytes());
        let mut sroot = db.get(SYNC_TRIE_ROOT_STORAGE_KEY.as_bytes());
        Self {
            db: Arc::new(Mutex::new(db)),
            pink_overlay: Default::default(),
            sync_overlay: Default::default(),
            raw_map: Default::default(),
            pink_root: if let Some(ref mut v) = proot {
                Some(
                    H::Out::decode(&mut v.as_slice())
                        .expect("merkle root must not be decode failed"),
                )
            } else {
                None
            },
            sync_root: if let Some(ref mut v) = sroot {
                Some(
                    H::Out::decode(&mut v.as_slice())
                        .expect("merkle root must not be decode failed"),
                )
            } else {
                None
            },
        }
    }

    pub fn get_pink_root(&self) -> H::Out {
        self.pink_root
            .expect("pink root should not be None please init it in PRuntime")
    }

    pub fn get_sync_root(&self) -> H::Out {
        self.sync_root
            .expect("sync root should not be None please init it in PRuntime")
    }

    pub fn derive_pink_backend(&self) -> PhalaTrieBackend<H> {
        PhalaTrieBackend(TrieBackend::new(
            PhalaTrieStorage::Essence(TrieEssenceStorage::Disk {
                disk: self.db.clone(),
                memory: self.pink_overlay.clone(),
            }),
            self.get_pink_root(),
        ))
    }

    pub fn derive_pink_snapshot(&self) -> PhalaTrieBackend<H> {
        let mut db = self.db.lock().expect("take snapshot have to hold lock");
        let snapshot = db.get_snapshot();
        PhalaTrieBackend(TrieBackend::new(
            PhalaTrieStorage::Snapshot(TrieSnapshot::Disk {
                db: self.db.clone(),
                snapshot: snapshot.take(),
            }),
            self.get_pink_root(),
        ))
    }

    pub fn derive_sync_backend(&self) -> PhalaTrieBackend<H> {
        PhalaTrieBackend(TrieBackend::new(
            PhalaTrieStorage::Essence(TrieEssenceStorage::Disk {
                disk: self.db.clone(),
                memory: self.sync_overlay.clone(),
            }),
            self.get_sync_root(),
        ))
    }

    pub fn commit_at_checkpoint(&mut self) -> anyhow::Result<()> {
        let mut db = self.db.lock().expect("checkpoint should hold lock on db");
        let mut batch = WriteBatch::new();
        let mut pink = self
            .pink_overlay
            .lock()
            .expect("commit should hold the pink overlay lock");
        let sync = self
            .sync_overlay
            .lock()
            .expect("commit should hold the sync overlay lock");

        // consolidate all changes
        pink.consolidate(sync.to_owned());

        // prepare the overlay changes
        for (key, (value, rc)) in pink.borrow_mut().drain() {
            match DB::get(db.borrow_mut(), key.as_ref()).map(Inner::<sp_trie::DBValue>::from) {
                Some(mut inner) => {
                    inner.1 += rc;
                    batch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
                }
                _ => {
                    let inner = Inner::<sp_trie::DBValue>(value, rc);
                    batch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
                }
            }
        }

        // prepare the raw
        for (key, value) in self.raw_map.iter() {
            batch.put(key, value)
        }

        if let Some(root) = self.pink_root {
            batch.put(PINK_TRIE_ROOT_STORAGE_KEY.as_bytes(), root.as_ref())
        }

        if let Some(ref root) = self.sync_root {
            batch.put(SYNC_TRIE_ROOT_STORAGE_KEY.as_bytes(), root.as_ref())
        }

        if batch.count() > 0 {
            DB::write(db.borrow_mut(), batch, true)?;
        }

        self.raw_map.clear();

        Ok(())
    }

    // force clear temporary
    // NOTE: in phactory have to keep the root available
    pub fn clear_temporary(&mut self) {
        self.pink_overlay = Default::default();
        self.sync_overlay = Default::default();
        self.raw_map = Default::default();
    }

    pub fn get_raw(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut db = self.db.lock().expect("get raw key have to hold the lock");
        DB::get(db.borrow_mut(), key)
    }

    pub fn put_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let _ = self.raw_map.insert(key, value);
    }

    // directly puth dat into without any transaction
    pub fn put_raw_directly(&mut self, key: &[u8], value: &[u8]) {
        let mut db = self.db.lock().expect("put directly have to hold the lock");
        let _ = DB::put(db.borrow_mut(), key, value);
    }
}

pub mod helper {
    use super::*;
    use hash_db::Hasher;
    use parity_scale_codec::Codec;
    use sp_state_machine::TrieBackend;
    use sp_trie::empty_trie_root;
    use sp_trie::LayoutV1;

    pub fn new_in_memory_backend<H: Hasher>() -> PhalaTrieBackend<H>
    where
        H::Out: Codec + Ord,
    {
        PhalaTrieBackend(TrieBackend::new(
            PhalaTrieStorage::Essence(TrieEssenceStorage::Memory(Default::default())),
            empty_trie_root::<LayoutV1<H>>(),
        ))
    }
}
