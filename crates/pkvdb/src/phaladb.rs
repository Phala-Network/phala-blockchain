//! phala kvdb layer
//!
//! To implement an adaptor for underlying kv system

// phactory
// firstly feed block transaction into the database
// secondly call every transaction over the database

// phactory use the instance for
// keep track the system and runtime state
// derive snapshot for Pink read and write
// belive that every call into pink will generate a new StateMachine

use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher, PlainDB, PlainDBRef, Prefix};
use rusty_leveldb::OuterSnapshot;
use rusty_leveldb::Snapshot;
use rusty_leveldb::{Options as LevelDBOptions, WriteBatch, DB};
use sp_database::Change;
use sp_state_machine::TrieBackendStorage;
use sp_std::borrow::Borrow;
use sp_std::borrow::BorrowMut;
use sp_std::convert::AsRef;
use sp_std::sync::Arc;
use sp_std::sync::Mutex;
use sp_trie::DBValue;
use sp_trie::MemoryDB;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;

const MUST_HOLD_LOCK: &str = "operate on underlying kv must hold the lock qed";
const MUST_NOT_EMPTY: &str = "underlying kv must not empty";

// directly use the sp_database transaction type to commit the system state
pub type Transaction<H> = sp_database::Transaction<H>;

type Result<T> = sp_std::result::Result<T, sp_state_machine::DefaultError>;

struct Inner<T>(pub(crate) T, pub(crate) i32);

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

// PhalaDB
#[derive(Default)]
pub struct PhalaDB<H: Hasher> {
    pub(crate) disk: Option<Arc<Mutex<DB>>>,
    // TODO:george replace the mutex though now we just write thread-safe
    pub(crate) memory: Arc<Mutex<MemoryDB<H>>>,
    pub(crate) only_memory: bool, // only use in memory make replay well
}

//
impl<H: Hasher> PhalaDB<H> {
    // create a phaladb in memory
    pub fn new_in_memory() -> Self {
        Self {
            disk: None,
            memory: Arc::new(Mutex::new(Default::default())),
            only_memory: true,
        }
    }

    // open a phaladb at a path
    pub fn new_with_disk(path: impl AsRef<Path>) -> Self {
        let options = LevelDBOptions::default();
        // TODO:george should open the gramine env when the gramine is stable
        let leveldb = DB::open(path, options).expect("open underlying leveldb failed");
        Self {
            disk: Some(Arc::new(Mutex::new(leveldb))),
            memory: Default::default(),
            only_memory: false,
        }
    }

    // take a snapshot
    // TODO: george implement the snapshot over leveldb to supply a specificed view
    // currently only take a snapshot over the current disk view
    pub fn take_snapshot(&self, _block_id: u64) -> PhalaSnapshot<H> {
        if self.only_memory {
            let memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
            let memory = memory_mut.borrow().deref().clone();
            PhalaSnapshot::Memory(memory)
        } else {
            let mut db_mut = self
                .disk
                .as_ref()
                .expect(MUST_NOT_EMPTY)
                .lock()
                .expect(MUST_HOLD_LOCK);
            let snapshot = DB::get_snapshot(db_mut.borrow_mut());
            let db = self.disk.as_ref().expect(MUST_NOT_EMPTY).clone();
            PhalaSnapshot::Disk {
                db,
                snapshot: snapshot.take(),
            }
        }
    }

    // finalized the memory into the disk storage
    // NOTE: outer TrieBackend root is not change same as the value before finalized
    pub fn finalized(&self, phactory_transaction: Transaction<H>) {
        if self.only_memory {
            panic!("Snapshot only supports read only")
        }
        let mut writebatch = WriteBatch::new();
        let mut db_mut = self
            .disk
            .as_ref()
            .expect(MUST_NOT_EMPTY)
            .lock()
            .expect(MUST_HOLD_LOCK);
        let mut memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        for (key, (value, rc)) in memory_mut.borrow_mut().drain() {
            match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<sp_trie::DBValue>::from) {
                Some(mut inner) => {
                    inner.1 += rc;
                    writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
                }
                None => {
                    let inner = Inner::<sp_trie::DBValue>(value, rc);
                    writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
                }
            }
        }

        let outer = phactory_transaction
            .0
            .into_iter()
            .map(|change| match change {
                Change::Set(col, key, value) => (col as u8, key, Some(value)),
                Change::Remove(col, key) => (col as u8, key, None),
                _ => unimplemented!(),
            });
        for item in outer {
            match item {
                (_, key, Some(value)) => {
                    writebatch.put(&key, &value);
                }
                (_, key, None) => {
                    writebatch.delete(&key);
                }
            }
        }
        if writebatch.count() > 0 {
            let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
        }
    }

    pub fn commit_in_memory_transaction(&self, transaction: MemoryDB<H>) {
        let mut memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);

        memory_mut.borrow_mut().consolidate(transaction);
    }

    // used to get he raw key
    pub fn get_raw(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut db_mut = self
            .disk
            .as_ref()
            .expect(MUST_NOT_EMPTY)
            .lock()
            .expect(MUST_HOLD_LOCK);
        DB::get(db_mut.borrow_mut(), key)
    }
}

impl<H: Hasher> PlainDB<H::Out, DBValue> for PhalaDB<H>
where
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        let mut db_mut = self
            .disk
            .as_ref()
            .expect(MUST_NOT_EMPTY)
            .lock()
            .expect(MUST_HOLD_LOCK);
        let memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        PlainDB::get(memory_mut.borrow().deref(), key).or_else(|| {
            if self.only_memory {
                return None;
            }
            match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<DBValue>::from) {
                Some(Inner(value, rc)) if rc > 0 => Some(value),
                _ => None,
            }
        })
    }

    fn contains(&self, key: &H::Out) -> bool {
        let mut db_mut = self
            .disk
            .as_ref()
            .expect(MUST_NOT_EMPTY)
            .lock()
            .expect(MUST_HOLD_LOCK);
        let memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        if PlainDB::contains(memory_mut.borrow().deref(), key) {
            true
        } else {
            if self.only_memory {
                return false;
            }
            match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<DBValue>::from) {
                Some(Inner(_, rc)) if rc > 0 => true,
                _ => false,
            }
        }
    }

    fn emplace(&mut self, key: H::Out, value: DBValue) {
        let mut memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        PlainDB::emplace(memory_mut.borrow_mut().deref_mut(), key, value)
    }

    fn remove(&mut self, key: &H::Out) {
        let _ = self
            .disk
            .as_ref()
            .expect(MUST_NOT_EMPTY)
            .lock()
            .expect(MUST_HOLD_LOCK);
        let mut memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        PlainDB::remove(memory_mut.borrow_mut().deref_mut(), key)
    }
}

impl<H> PlainDBRef<H::Out, DBValue> for PhalaDB<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        PlainDB::get(self, key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self, key)
    }
}

impl<H> AsPlainDB<H::Out, DBValue> for PhalaDB<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn as_plain_db(&self) -> &dyn PlainDB<H::Out, DBValue> {
        self
    }

    fn as_plain_db_mut(&mut self) -> &mut dyn PlainDB<H::Out, DBValue> {
        self
    }
}

impl<H: Hasher> HashDB<H, DBValue> for PhalaDB<H>
where
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        let mut db_mut = self
            .disk
            .as_ref()
            .expect(MUST_NOT_EMPTY)
            .lock()
            .expect(MUST_HOLD_LOCK);
        let memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        HashDB::get(memory_mut.borrow().deref(), key, prefix).or_else(|| {
            if self.only_memory {
                return None;
            }
            match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<DBValue>::from) {
                Some(Inner(value, rc)) if rc > 0 => Some(value),
                _ => None,
            }
        })
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        let mut db_mut = self
            .disk
            .as_ref()
            .expect(MUST_NOT_EMPTY)
            .lock()
            .expect(MUST_HOLD_LOCK);
        let memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        if HashDB::contains(memory_mut.borrow().deref(), key, prefix) {
            true
        } else {
            if self.only_memory {
                return false;
            }
            match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<DBValue>::from) {
                Some(Inner(_, rc)) if rc > 0 => true,
                _ => false,
            }
        }
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H::Out {
        let mut memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        HashDB::insert(memory_mut.borrow_mut().deref_mut(), prefix, value)
    }

    fn emplace(&mut self, key: H::Out, prefix: Prefix, value: DBValue) {
        let mut memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        HashDB::emplace(memory_mut.borrow_mut().deref_mut(), key, prefix, value)
    }

    fn remove(&mut self, key: &H::Out, prefix: Prefix) {
        let mut memory_mut = self.memory.lock().expect(MUST_HOLD_LOCK);
        HashDB::remove(memory_mut.borrow_mut().deref_mut(), key, prefix)
    }
}

impl<H> HashDBRef<H, DBValue> for PhalaDB<H>
where
    H: Hasher,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        HashDB::get(self, key, prefix)
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        HashDB::contains(self, key, prefix)
    }
}

impl<H> AsHashDB<H, DBValue> for PhalaDB<H>
where
    H: Hasher,
{
    fn as_hash_db(&self) -> &dyn HashDB<H, DBValue> {
        self
    }

    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<H, DBValue> {
        self
    }
}

// SnapshotDB
// TODO: george reduce the data race over the underlying DB in multi query condition this should be a bottlenack
pub enum PhalaSnapshot<H: Hasher> {
    Memory(MemoryDB<H>),
    Disk {
        db: Arc<Mutex<DB>>,
        snapshot: OuterSnapshot,
    },
}

impl<H: Hasher> PlainDB<H::Out, DBValue> for PhalaSnapshot<H>
where
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        match self {
            Self::Memory(db) => PlainDB::get(db, key),
            Self::Disk { db, snapshot } => {
                let mut db_mut = db.lock().expect(MUST_HOLD_LOCK);
                if let Ok(item) = DB::get_at(
                    db_mut.borrow_mut(),
                    &Snapshot::from(snapshot.clone()),
                    key.as_ref(),
                ) {
                    match item.map(Inner::<DBValue>::from) {
                        Some(Inner(value, rc)) if rc > 0 => Some(value),
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    fn contains(&self, key: &H::Out) -> bool {
        match self {
            Self::Memory(db) => PlainDB::contains(db, key),
            Self::Disk { db, snapshot } => {
                let mut db_mut = db.lock().expect(MUST_HOLD_LOCK);
                if let Ok(item) = DB::get_at(
                    db_mut.borrow_mut(),
                    &Snapshot::from(snapshot.clone()),
                    key.as_ref(),
                ) {
                    match item.map(Inner::<DBValue>::from) {
                        Some(Inner(_, rc)) if rc > 0 => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
    }

    fn emplace(&mut self, key: H::Out, value: DBValue) {
        match self {
            Self::Memory(db) => PlainDB::emplace(db, key, value),
            Self::Disk { .. } => unimplemented!("PlainDB emplace is not supported on SnapshotDB"),
        }
    }

    fn remove(&mut self, key: &H::Out) {
        match self {
            Self::Memory(db) => PlainDB::remove(db, key),
            Self::Disk { .. } => unimplemented!("PlainDB remove is not supported on SnapshotDB"),
        }
    }
}

impl<H> PlainDBRef<H::Out, DBValue> for PhalaSnapshot<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        PlainDB::get(self, key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self, key)
    }
}

impl<H> AsPlainDB<H::Out, DBValue> for PhalaSnapshot<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn as_plain_db(&self) -> &dyn PlainDB<H::Out, DBValue> {
        self
    }

    fn as_plain_db_mut(&mut self) -> &mut dyn PlainDB<H::Out, DBValue> {
        self
    }
}

static NULL_NODE_DATA: DBValue = DBValue::new();
#[inline]
fn get_null_hashed_node<H: Hasher>() -> H::Out {
    H::hash(&NULL_NODE_DATA)
}

impl<H: Hasher> HashDB<H, DBValue> for PhalaSnapshot<H>
where
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        if *key == get_null_hashed_node::<H>() {
            return Some(NULL_NODE_DATA.clone());
        }
        match self {
            Self::Memory(db) => HashDB::get(db, key, prefix),
            Self::Disk { db, snapshot } => {
                let mut db_mut = db.lock().expect(MUST_HOLD_LOCK);
                if let Ok(item) = DB::get_at(
                    db_mut.borrow_mut(),
                    &Snapshot::from(snapshot.clone()),
                    key.as_ref(),
                ) {
                    match item.map(Inner::<DBValue>::from) {
                        Some(Inner(value, rc)) if rc > 0 => Some(value),
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        if *key == get_null_hashed_node::<H>() {
            return true;
        }
        match self {
            Self::Memory(db) => HashDB::contains(db, key, prefix),
            Self::Disk { db, snapshot } => {
                let mut db_mut = db.lock().expect(MUST_HOLD_LOCK);
                if let Ok(item) = DB::get_at(
                    db_mut.borrow_mut(),
                    &Snapshot::from(snapshot.clone()),
                    key.as_ref(),
                ) {
                    match item.map(Inner::<DBValue>::from) {
                        Some(Inner(_, rc)) if rc > 0 => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
    }

    fn insert(&mut self, _prefix: Prefix, _value: &[u8]) -> H::Out {
        unimplemented!("HasDB insert is not supported on Snapshot")
    }

    /// Like `insert()`, except you provide the key and the data is all moved.
    fn emplace(&mut self, _key: H::Out, _prefix: Prefix, _value: DBValue) {
        unimplemented!("HashDB emplace is not supported on Snapshot")
    }

    fn remove(&mut self, _key: &H::Out, _prefix: Prefix) {
        unimplemented!("HashDB remove is not supported on Snapshot")
    }
}

impl<H> HashDBRef<H, DBValue> for PhalaSnapshot<H>
where
    H: Hasher,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        HashDB::get(self, key, prefix)
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        HashDB::contains(self, key, prefix)
    }
}

impl<H> AsHashDB<H, DBValue> for PhalaSnapshot<H>
where
    H: Hasher,
{
    fn as_hash_db(&self) -> &dyn HashDB<H, DBValue> {
        self
    }

    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<H, DBValue> {
        self
    }
}

pub enum PhalaTrieStorage<H: Hasher> {
    Mutual(PhalaDB<H>),
    Shared(PhalaSnapshot<H>),
    Empty,
}

impl<H: Hasher> Default for PhalaTrieStorage<H> {
    fn default() -> Self {
        PhalaTrieStorage::Empty
    }
}

impl<H: Hasher> PhalaTrieStorage<H> {
    fn as_dyn_hashdb(&self) -> &dyn HashDB<H, DBValue> {
        match self {
            Self::Mutual(db) => db.as_hash_db(),
            Self::Shared(snapshot) => snapshot.as_hash_db(),
            _ => unimplemented!(),
        }
    }

    fn as_dyn_hashdb_mut(&mut self) -> &mut dyn HashDB<H, DBValue> {
        match self {
            Self::Mutual(db) => db.as_hash_db_mut(),
            Self::Shared(snapshot) => snapshot.as_hash_db_mut(),
            _ => unimplemented!(),
        }
    }
}

impl<H: Hasher> PhalaTrieStorage<H>
where
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
    H::Out: AsRef<[u8]>,
{
    fn as_dyn_plaindb(&self) -> &dyn PlainDB<H::Out, DBValue> {
        match self {
            Self::Mutual(db) => db.as_plain_db(),
            Self::Shared(snapshot) => snapshot.as_plain_db(),
            _ => unimplemented!(),
        }
    }

    fn as_dyn_plaindb_mut(&mut self) -> &mut dyn PlainDB<H::Out, DBValue> {
        match self {
            Self::Mutual(db) => db.as_plain_db_mut(),
            Self::Shared(snapshot) => snapshot.as_plain_db_mut(),
            _ => unimplemented!(),
        }
    }
}

impl<H: Hasher> TrieBackendStorage<H> for PhalaTrieStorage<H> {
    type Overlay = MemoryDB<H>;

    fn get(&self, key: &H::Out, prefix: Prefix) -> Result<Option<DBValue>> {
        Ok(HashDB::get(self.as_dyn_hashdb(), key, prefix))
    }
}

impl<H: Hasher> PlainDB<H::Out, DBValue> for PhalaTrieStorage<H>
where
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        PlainDB::get(self.as_dyn_plaindb(), key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self.as_dyn_plaindb(), key)
    }

    fn emplace(&mut self, key: H::Out, value: DBValue) {
        PlainDB::emplace(self.as_dyn_plaindb_mut(), key, value)
    }

    fn remove(&mut self, key: &H::Out) {
        PlainDB::remove(self.as_dyn_plaindb_mut(), key)
    }
}

impl<H> PlainDBRef<H::Out, DBValue> for PhalaTrieStorage<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        PlainDB::get(self.as_dyn_plaindb(), key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self.as_dyn_plaindb(), key)
    }
}

impl<H> AsPlainDB<H::Out, DBValue> for PhalaTrieStorage<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn as_plain_db(&self) -> &dyn PlainDB<H::Out, DBValue> {
        self
    }

    fn as_plain_db_mut(&mut self) -> &mut dyn PlainDB<H::Out, DBValue> {
        self
    }
}

impl<H: Hasher> HashDB<H, DBValue> for PhalaTrieStorage<H> {
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        if *key == get_null_hashed_node::<H>() {
            return Some(NULL_NODE_DATA.clone());
        }
        HashDB::get(self.as_dyn_hashdb(), key, prefix)
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        if *key == get_null_hashed_node::<H>() {
            return true;
        }
        HashDB::contains(self.as_dyn_hashdb(), key, prefix)
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H::Out {
        HashDB::insert(self.as_dyn_hashdb_mut(), prefix, value)
    }

    fn emplace(&mut self, key: H::Out, prefix: Prefix, value: DBValue) {
        HashDB::emplace(self.as_dyn_hashdb_mut(), key, prefix, value)
    }

    fn remove(&mut self, _key: &H::Out, _prefix: Prefix) {
        unimplemented!("HashDB remove is not supported on Snapshot")
    }
}

impl<H: Hasher> HashDBRef<H, DBValue> for PhalaTrieStorage<H> {
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        HashDB::get(self.as_hash_db(), key, prefix)
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        HashDB::contains(self.as_hash_db(), key, prefix)
    }
}

impl<H: Hasher> AsHashDB<H, DBValue> for PhalaTrieStorage<H> {
    fn as_hash_db(&self) -> &dyn HashDB<H, DBValue> {
        self
    }

    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<H, DBValue> {
        self
    }
}
