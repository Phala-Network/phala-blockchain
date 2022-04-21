use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher, PlainDB, PlainDBRef, Prefix};
use rusty_leveldb::LdbIterator;
use rusty_leveldb::WriteBatch;
use rusty_leveldb::DB;
use sp_state_machine::backend::Consolidate;
use std::borrow::BorrowMut;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Mutex;

pub struct Kvdb<H, T>
where
    H: Hasher,
{
    leveldb: Mutex<DB>,
    null_node_data: T,
    hashed_null_node: H::Out,
}

// this is just an empty implementation for the TrieBackendEssence bound
impl<H, T> Consolidate for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send,
{
    fn consolidate(&mut self, _other: Self) {
        unimplemented!()
    }
}

impl<H, T> Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
{
    // FIXME: should add a path to leveldb
    pub fn with_null_node(_null_key: &[u8], _null_node_data: T) -> Self {
        unimplemented!()
    }

    pub fn new(_path: impl AsRef<Path>) -> Self {
        // use the zero vector as the default null key and data ?
        unimplemented!()
    }

    pub fn purge(&mut self) {
        // TODO:george this approach is not sound for performance 
        if let Ok(mut db_mut) = self.leveldb.lock() {
            let mut writebatch = WriteBatch::new();
            if let Ok(mut iter) = DB::new_iter(db_mut.borrow_mut()) {
                while let Some((key, val)) = iter.next() {
                    if i32::from_be_bytes(val[0..4].try_into().unwrap()) <= 0 {
                        writebatch.delete(&key);
                    }
                }
                let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
            }
        }
    }

    // TODO: if we really need the consolidate function same as the MemoryDB ?
    // in substrate state-machine the data is from the block backend and we could belive the
    // contract is stored in block as externics stable but we just support the same semantics for
    // phala runtime
}

// inner kv represention
// first 4 bytes is the reference count and left data vector is the data self
struct Inner<T>(T, i32);

impl<T> From<Vec<u8>> for Inner<T>
where
    T: for<'a> From<&'a [u8]>,
{
    fn from(v: Vec<u8>) -> Self {
        let (rc, val) = v.split_at(4);
        let value = T::from(val);
        Inner(value, i32::from_be_bytes(rc.try_into().unwrap()))
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
        let mut db_mut = self.leveldb.lock().unwrap();
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(Inner(_, rc)) if rc > 0 => true,
            _ => false,
        }
    }

    fn emplace(&mut self, key: H::Out, value: T) {
        let mut db_mut = self.leveldb.lock().unwrap();
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
        let mut db_mut = self.leveldb.lock().unwrap();
        let mut writebatch = WriteBatch::new();
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(mut inner) => {
                if inner.1 > 1 {
                    inner.1 -= 1;
                    writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
                    let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
                } else {
                    let _ = DB::delete(db_mut.borrow_mut(), key.as_ref());
                }
            }
            _ => {}
        }
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

        let mut db_mut = self.leveldb.lock().unwrap();
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(Inner(value, rc)) if rc > 0 => Some(value),
            _ => None,
        }
    }

    fn contains(&self, key: &H::Out, _prefix: Prefix) -> bool {
        if key == &self.hashed_null_node {
            return true;
        }

        let mut db_mut = self.leveldb.lock().unwrap();
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(Inner(_, rc)) if rc > 0 => true,
            _ => false,
        }
    }

    fn emplace(&mut self, key: H::Out, _prefix: Prefix, value: T) {
        if value == self.null_node_data {
            return;
        }
        let mut db_mut = self.leveldb.lock().unwrap();
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

        let mut db_mut = self.leveldb.lock().unwrap();
        match DB::get(db_mut.borrow_mut(), key.as_ref()).map(Inner::<T>::from) {
            Some(mut inner) => {
                if inner.1 > 1 {
                    inner.1 -= 1;
                    let mut writebatch = WriteBatch::new();
                    writebatch.put(key.as_ref(), Into::<Vec<u8>>::into(inner).as_ref());
                    let _ = DB::write(db_mut.deref_mut(), writebatch, true);
                } else {
                    let _ = DB::delete(db_mut.deref_mut(), key.as_ref());
                }
            }
            _ => {}
        }
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
    use hash_db::Hasher;
    use sp_core::KeccakHasher; 

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
