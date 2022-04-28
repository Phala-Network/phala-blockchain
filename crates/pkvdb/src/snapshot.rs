// specificed type SnapshotStorage
use crate::leveldb::Inner;
use crate::leveldb::Kvdb;
use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher, PlainDB, PlainDBRef, Prefix};
use rusty_leveldb::OuterSnapshot;
use rusty_leveldb::Snapshot;
use rusty_leveldb::DB;
use std::borrow::BorrowMut;
use std::sync::Arc;

// TODO: SnapshotDB is only used in contract query in pink but if there need some write operations ?

pub struct SnapshotDB<H, T>
where
    H: Hasher,
{
    pub(crate) inner: Kvdb<H, T>,
    pub(crate) snapshot: Arc<OuterSnapshot>,
    pub(crate) null_node_data: T,
    pub(crate) hashed_null_node: H::Out,
}

impl<H, T> PlainDB<H::Out, T> for SnapshotDB<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        let mut db_mut = self.inner.leveldb.lock().expect("database lock must hold");
        let snapshot = self.snapshot.as_ref().clone();
        if let Ok(item) = DB::get_at(db_mut.borrow_mut(), &Snapshot::from(snapshot), key.as_ref()) {
            match item.map(Inner::<T>::from) {
                Some(Inner(value, rc)) if rc > 0 => Some(value),
                _ => None,
            }
        } else {
            None
        }
    }

    fn contains(&self, key: &H::Out) -> bool {
        let mut db_mut = self.inner.leveldb.lock().expect("database lock must hold");
        let snapshot = self.snapshot.as_ref().clone();
        if let Ok(item) = DB::get_at(db_mut.borrow_mut(), &Snapshot::from(snapshot), key.as_ref()) {
            match item.map(Inner::<T>::from) {
                Some(Inner(_, rc)) if rc > 0 => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn emplace(&mut self, key: H::Out, value: T) {
        self.inner.as_plain_db_mut().emplace(key, value);
    }

    fn remove(&mut self, key: &H::Out) {
        self.inner.as_plain_db_mut().remove(key)
    }
}

impl<H, T> HashDB<H, T> for SnapshotDB<H, T>
where
    H: Hasher,
    T: Default + PartialEq + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out, _prefix: Prefix) -> Option<T> {
        if key == &self.hashed_null_node {
            return Some(self.null_node_data.clone());
        }

        let mut db_mut = self.inner.leveldb.lock().expect("database lock must hold");
        let snapshot = self.snapshot.as_ref().clone();
        if let Ok(item) = DB::get_at(db_mut.borrow_mut(), &Snapshot::from(snapshot), key.as_ref()) {
            match item.map(Inner::<T>::from) {
                Some(Inner(value, rc)) if rc > 0 => Some(value),
                _ => None,
            }
        } else {
            None
        }
    }

    fn contains(&self, key: &H::Out, _prefix: Prefix) -> bool {
        if key == &self.hashed_null_node {
            return true;
        }

        let mut db_mut = self.inner.leveldb.lock().expect("database lock must hold");
        let snapshot = self.snapshot.as_ref().clone();
        if let Ok(item) = DB::get_at(db_mut.borrow_mut(), &Snapshot::from(snapshot), key.as_ref()) {
            match item.map(Inner::<T>::from) {
                Some(Inner(_, rc)) if rc > 0 => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn emplace(&mut self, key: H::Out, prefix: Prefix, value: T) {
        self.inner.as_hash_db_mut().emplace(key, prefix, value);
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H::Out {
        self.inner.as_hash_db_mut().insert(prefix, value)
    }

    fn remove(&mut self, key: &H::Out, prefix: Prefix) {
        self.inner.as_hash_db_mut().remove(key, prefix);
    }
}

impl<H, T> PlainDBRef<H::Out, T> for SnapshotDB<H, T>
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

impl<H, T> HashDBRef<H, T> for SnapshotDB<H, T>
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

impl<H, T> AsPlainDB<H::Out, T> for SnapshotDB<H, T>
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

impl<H, T> AsHashDB<H, T> for SnapshotDB<H, T>
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
