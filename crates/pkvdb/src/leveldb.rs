use crate::snapshot;
use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher, PlainDB, PlainDBRef, Prefix};
use rusty_leveldb::gramine_env::GramineEnv;
use rusty_leveldb::{LdbIterator, Options as LevelDBOptions, WriteBatch, DB};
pub use sp_trie::MemoryDB as Transaction;
use sp_trie::MemoryDB;
use std::borrow::BorrowMut;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

pub type LevelDB<H> = Kvdb<H, sp_trie::DBValue>;

pub struct Kvdb<H, T>
where
    H: Hasher,
{
    pub(crate) leveldb: Arc<Mutex<DB>>,
    pub(crate) null_node_data: T,
    pub(crate) hashed_null_node: H::Out,
    pub(crate) pending_transactions: Arc<Option<Mutex<MemoryDB<H>>>>,
}

impl<H, T> Clone for Kvdb<H, T>
where
    H: Hasher,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            leveldb: self.leveldb.clone(),
            null_node_data: self.null_node_data.clone(),
            hashed_null_node: self.hashed_null_node,
            pending_transactions: self.pending_transactions.clone(),
        }
    }
}

impl<H, T> Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    pub fn with_null_node<P: AsRef<Path>>(path: P, null_key: &[u8], null_node_data: T) -> Self {
        let mut options = LevelDBOptions::default();
        options.env = Rc::new(Box::new(GramineEnv::new()));
        let db = DB::open(path, options).expect("underlying database must open");
        Kvdb {
            leveldb: Arc::new(Mutex::new(db)),
            hashed_null_node: H::hash(null_key),
            null_node_data,
            pending_transactions: Arc::new(None),
        }
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self::with_null_node(path, &[0u8][..], [0u8][..].into())
    }

    pub fn as_checkpoint_db(mut self) -> Self {
        self.pending_transactions = Arc::new(Some(Mutex::new(Default::default())));
        self
    }

    pub fn purge(&self) {
        // TODO:george this approach is not sound for performance
        if let Ok(mut db_mut) = self.leveldb.lock() {
            let mut writebatch = WriteBatch::new();
            if let Ok(mut iter) = DB::new_iter(db_mut.borrow_mut()) {
                while let Some((key, val)) = iter.next() {
                    if i32::from_be_bytes(
                        val[0..4].try_into().expect("reference must in the Value"),
                    ) == 0
                    {
                        writebatch.delete(&key);
                    }
                }
                let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
            }
        }
    }

    // only for test
    pub fn clear(&self) {
        if let Ok(mut db_mut) = self.leveldb.lock() {
            let mut writebatch = WriteBatch::new();
            if let Ok(mut iter) = DB::new_iter(db_mut.borrow_mut()) {
                while let Some((key, _)) = iter.next() {
                    writebatch.delete(&key);
                }
                let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
            }
        }
    }

    pub fn raw(&self, key: &H::Out, _prefix: Prefix) -> Option<(T, i32)> {
        if key == &self.hashed_null_node {
            return Some((self.null_node_data.clone(), 1));
        }
        let mut db_mut = self.leveldb.lock().expect("database lock must hold");
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(Inner(value, rc)) => Some((value, rc)),
            _ => None,
        }
    }

    pub fn flush(&self) {
        let mut db = self
            .leveldb
            .lock()
            .expect("should always get the leveldb reference");
        let _ = DB::flush(db.borrow_mut());
    }

    // take_read_snapshot any write operation will panic
    pub fn take_sanpshot(&self) -> snapshot::SnapshotDB<H, T> {
        let mut db_mut = self
            .leveldb
            .lock()
            .expect("take snapshot have to hold the lock");
        let snapshot = DB::get_snapshot(db_mut.borrow_mut());
        let outer = snapshot.take();
        snapshot::SnapshotDB {
            inner: self.clone(),
            snapshot: Arc::new(outer),
            null_node_data: self.null_node_data.clone(),
            hashed_null_node: self.hashed_null_node.clone(),
        }
    }
}

impl<H> Kvdb<H, sp_trie::DBValue>
where
    H: Hasher,
{
    pub fn consolidate(&self, mut transaction: Transaction<H>) {
        if let Ok(mut db_mut) = self.leveldb.lock() {
            let mut writebatch = WriteBatch::new();

            for (key, (value, rc)) in transaction.drain() {
                match DB::get(db_mut.borrow_mut(), key.as_ref())
                    .map(Inner::<sp_trie::DBValue>::from)
                {
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
            if writebatch.count() > 0 {
                let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
            }
        }
    }

    pub fn submit(&self, transaction: Transaction<H>) {
        if let Some(guard) = self.pending_transactions.deref() {
            let mut pending = guard
                .lock()
                .expect("submit transaction have to hold the lock");
            pending.borrow_mut().consolidate(transaction);
        } else {
            self.consolidate(transaction);
        }
    }

    pub fn commit_with_raw(&self, raw: Option<crate::raw::Transction>) {
        let mut db_mut = self
            .leveldb
            .lock()
            .expect("commit all transaction should hold the lock");
        let mut writebatch = WriteBatch::new();
        if let Some(transaction) = raw {
            for (key, value) in &transaction.puts {
                writebatch.put(key, value);
            }
            for key in &transaction.deletes {
                writebatch.delete(key);
            }
        }

        let mut pending_mut = self
            .pending_transactions
            .as_ref()
            .as_ref()
            .expect("pending must exists")
            .lock()
            .expect("take pending transactions should hold the lock");
            
        for (key, (value, rc)) in pending_mut.borrow_mut().drain() {
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
        if writebatch.count() > 0 {
            let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
        }
    }
}

// inner kv represention
// first 4 bytes is the reference count and left data vector is the data self
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

impl<H, T> PlainDB<H::Out, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        //FIXME:george the leveldb should panic over the lock faild
        if let Ok(mut db_mut) = self.leveldb.lock() {
            match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
                Some(Inner(value, rc)) if rc > 0 => Some(value),
                _ => None,
            }
        } else {
            None
        }
    }

    fn contains(&self, key: &H::Out) -> bool {
        let mut db_mut = self.leveldb.lock().expect("database lock must hold");
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(Inner(_, rc)) if rc > 0 => true,
            _ => false,
        }
    }

    fn emplace(&mut self, key: H::Out, value: T) {
        let mut db_mut = self.leveldb.lock().expect("database lock must hold");
        let mut writebatch = WriteBatch::new();
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(mut inner) => {
                inner.1 += 1;
                writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
            }
            _ => {
                let inner = Inner::<T>(value, 1);
                writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
            }
        }
        let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
    }

    fn remove(&mut self, key: &H::Out) {
        let mut db_mut = self.leveldb.lock().expect("database lock must hold");
        let mut writebatch = WriteBatch::new();
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(mut inner) => {
                inner.1 -= 1;
                writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
            }
            _ => {
                let inner = Inner::<T>(T::default(), -1);
                writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
            }
        }
        let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
    }
}

// for phala we don't need the key function as generic
// and we just use the simple deref key function in memory db
impl<H, T> HashDB<H, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out, _prefix: Prefix) -> Option<T> {
        if key == &self.hashed_null_node {
            return Some(self.null_node_data.clone());
        }

        let mut db_mut = self.leveldb.lock().expect("database lock must hold");
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(Inner(value, rc)) if rc > 0 => Some(value),
            _ => None,
        }
    }

    fn contains(&self, key: &H::Out, _prefix: Prefix) -> bool {
        if key == &self.hashed_null_node {
            return true;
        }

        let mut db_mut = self.leveldb.lock().expect("database lock must hold");
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(Inner(_, rc)) if rc > 0 => true,
            _ => false,
        }
    }

    fn emplace(&mut self, key: H::Out, _prefix: Prefix, value: T) {
        if value == self.null_node_data {
            return;
        }
        let mut db_mut = self.leveldb.lock().expect("database lock must hold");
        let mut writebatch = WriteBatch::new();
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(mut inner) => {
                inner.1 += 1;
                writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
            }
            _ => {
                let inner = Inner::<T>(value, 1);
                writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
            }
        }
        let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H::Out {
        if T::from(value) == self.null_node_data {
            return self.hashed_null_node;
        }
        let key = H::hash(value);
        HashDB::emplace(self, key, prefix, value.into());
        key
    }

    fn remove(&mut self, key: &H::Out, _prefix: Prefix) {
        if key == &self.hashed_null_node {
            return;
        }

        let mut db_mut = self.leveldb.lock().expect("database lock must hold");
        let mut writebatch = WriteBatch::new();
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(mut inner) => {
                inner.1 -= 1;
                writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
            }
            _ => {
                let inner = Inner::<T>(T::default(), -1);
                writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
            }
        }
        let _ = DB::write(db_mut.deref_mut(), writebatch, true);
    }
}

impl<H, T> PlainDBRef<H::Out, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        PlainDB::get(self, key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self, key)
    }
}

impl<H, T> HashDBRef<H, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<T> {
        HashDB::get(self, key, prefix)
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        HashDB::contains(self, key, prefix)
    }
}

impl<H, T> AsPlainDB<H::Out, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn as_plain_db(&self) -> &dyn PlainDB<H::Out, T> {
        self
    }

    fn as_plain_db_mut(&mut self) -> &mut dyn PlainDB<H::Out, T> {
        self
    }
}

impl<H, T> AsHashDB<H, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn as_hash_db(&self) -> &dyn HashDB<H, T> {
        self
    }

    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<H, T> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::KeccakHasher;

    static TMP_DB_PATH: &str = "/tmp/test_kv";

    #[test]
    fn purge() {
        let value_bytes = b"purge_value";
        let key = KeccakHasher::hash(value_bytes);
        let mut kv = LevelDB::<KeccakHasher>::new(TMP_DB_PATH);

        let return_key = kv.insert(hash_db::EMPTY_PREFIX, value_bytes);
        assert_eq!(key, return_key);
        let raw = kv.raw(&key, hash_db::EMPTY_PREFIX);
        assert!(raw.is_some());
        assert_eq!(raw.unwrap().1, 1);
        let _ = kv.insert(hash_db::EMPTY_PREFIX, value_bytes);
        let raw = kv.raw(&key, hash_db::EMPTY_PREFIX);
        assert_eq!(raw.unwrap().1, 2);
        kv.as_hash_db_mut().remove(&key, hash_db::EMPTY_PREFIX);
        let raw = kv.raw(&key, hash_db::EMPTY_PREFIX);
        assert_eq!(raw.unwrap().1, 1);
        kv.as_hash_db_mut().remove(&key, hash_db::EMPTY_PREFIX);
        let raw = kv.raw(&key, hash_db::EMPTY_PREFIX);
        assert_eq!(raw.unwrap().1, 0);
        kv.purge();
        let raw = kv.raw(&key, hash_db::EMPTY_PREFIX);
        assert!(raw.is_none());

        let _ = kv.insert(hash_db::EMPTY_PREFIX, value_bytes);
        let raw = kv.raw(&key, hash_db::EMPTY_PREFIX);
        assert_eq!(raw.unwrap().1, 1);
        kv.as_hash_db_mut().remove(&key, hash_db::EMPTY_PREFIX);
        let raw = kv.raw(&key, hash_db::EMPTY_PREFIX);
        assert_eq!(raw.unwrap().1, 0);
    }

    #[test]
    fn consolidate() {
        let origin_bytes = b"before_consolidate";
        let delta_bytes = b"after_consolidate";
        let delta_remove_bytes = b"remove_with_consolidate";
        let origin_key = KeccakHasher::hash(origin_bytes);
        let delta_key = KeccakHasher::hash(delta_bytes);
        let delta_remove_key = KeccakHasher::hash(delta_remove_bytes);

        let mut transaction = Transaction::<KeccakHasher>::default();
        let return_key = transaction
            .as_hash_db_mut()
            .insert(hash_db::EMPTY_PREFIX, delta_bytes);
        let _ = transaction
            .as_hash_db_mut()
            .remove(&delta_remove_key, hash_db::EMPTY_PREFIX);
        assert_eq!(return_key, delta_key);

        let mut kv = LevelDB::<KeccakHasher>::new(TMP_DB_PATH);
        kv.clear(); // make sure cleaning env

        let return_key = kv
            .as_hash_db_mut()
            .insert(hash_db::EMPTY_PREFIX, origin_bytes);
        assert_eq!(return_key, origin_key);
        let _ = kv
            .as_hash_db_mut()
            .insert(hash_db::EMPTY_PREFIX, delta_remove_bytes);
        kv.consolidate(transaction);

        let raw = kv.raw(&origin_key, hash_db::EMPTY_PREFIX);
        assert_eq!(raw.unwrap().1, 1);
        let raw = kv.raw(&delta_key, hash_db::EMPTY_PREFIX);
        assert_eq!(raw.unwrap().1, 1);
        let raw = kv.raw(&delta_remove_key, hash_db::EMPTY_PREFIX);
        assert_eq!(raw.unwrap().1, 0);
    }

    #[test]
    fn it_works() {
        let value = b"works_default_value";
        let mut kv = LevelDB::<KeccakHasher>::new(TMP_DB_PATH);
        let hash = kv.as_hash_db_mut();
        let key = hash.insert(hash_db::EMPTY_PREFIX, value);
        let result = hash.get(&key, hash_db::EMPTY_PREFIX);
        assert!(result.is_some());
        assert_eq!(&result.unwrap(), value);
    }
}
