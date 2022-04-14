use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher, PlainDB, PlainDBRef, Prefix};
use rusty_leveldb::WriteBatch;
use rusty_leveldb::DB;
use std::cell::RefCell;
use std::primitives::i32;

// TODO: George for more effective performance maybe we could use the single snapshot for queries
pub struct Kvdb<H, T> {
    leveldb: RefCell<DB>,
}

// inner kv represention
struct Inner<T>(T, i32);

impl<T> From<Vec<u8>> for Inner<T>
where
    T: for<'a> From<&'a [u8]>,
{
    fn from(v: Vec<u8>) -> Self {
        let rc = i32::from_be_bytes(&v[0..4]);
        let value = T::from(&v[4..]);
        Inner(value, rc)
    }
}

impl<T> Into<Vec<u8>> for Inner<T>
where
    T: AsRef<[u8]>,
{
    fn into(self) -> Vec<u8> {
        let mut underlying = Vec::with_capcity(T.as_ref().len() + 4);
        underlying.extend_from_slice(self.1.to_be_bytes());
        underlying.extend_from_slice(self.0.as_ref());
        underlying
    }
}

impl<H, T> PlainDB<H::Out, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        let db_mut = self.leveldb.borrow_mut();
        match DB::get(db_mut.deref_mut(), key.as_ref()) {
            Ok(v) => v.map(Inner::<T>::from),
            _ => None,
        }
    }

    fn contains(&self, key: &H::Out) -> bool {
        let db_mut = self.leveldb.borrow_mut();
        match DB::get(db_mut.deref_mut(), &snapshot, k.as_ref()) {
            Ok(opt) => opt.is_some(),
            Err(_) => false,
        }
    }

    fn emplace(&mut self, key: H::Out, value: T) {
        let db_mut = self.leveldb.borrow_mut();
        match DB::get(db_mut.deref_mut(), key.as_ref()) {
            Ok(opt) => {
                let mut writebatch = WriteBatch::new();
                if let Some(mut inner) = opt.map(Inner::<T>::from) {
                    inner.1 += 1;
                    writebatch.put(key.as_ref(), inner.into::<Vec<u8>>().as_ref());
                    let _ = DB::write(db_mut.deref_mut(), writebatch, true);
                } else {
                    let inner = Inner::<T>(value, 1);
                    writebatch.put(key.as_ref(), inner.into::<Vec<u8>>().as_ref());
                    let _ = DB::write(db_mut.deref_mut(), writebatch, true);
                }
            }
            _ => {
                // FIXME: how to resolve the IO error over the leveldb directly panic ?
            }
        }
    }

    fn remove(&mut self, key: H::Out) {
        let db_mut = self.leveldb.borrow_mut();
        match DB::get(db_mut.deref_mut(), key.as_ref()) {
            Ok(opt) => {
                if let Some(mut inner) = opt.map(Inner::<T>::from){
                    if inner.1 > 1 {
                        inner.1 -= 1;
                        let writebatch = WriteBatch::new();
                        writebatch.put(key.as_ref(), inner.into::<Vec<u8>>().as_ref());
                        let _ = DB::write(db_mut.deref_mut(), writebatch, true);
                    } else {
                        let _ = DB::delte(db_mut.deref_mut(), key.as_ref());
                    }
                }

            },
            _ => {
                // FIXME: panic too ?
            }
        }
    }
}

impl<H, T> PlainDBRef<H::Out, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        PlainDB::get(self, key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self, key)
    }
}

impl<H, T> HashDB<H, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send,
{
    fn get(&self, key: &H::Out, _prefix: Prefix) -> Option<T> {
        unimplemented!()
    }

    fn contains(&self, key: &H::Out, _prefix: Prefix) -> bool {
        unimplemented!()
    }

    fn emplace(&mut self, key: &H::Out, _prefix: Prefix, value: T) {
        unimplemented!()
    }

    fn insert(&mut self, key: &H::Out, _prefix: Prefix, value: &[u8]) {
        unimplemented!()
    }

    fn remove(&mut self, key: &H::Out, _prefix: Prefix) {
        unimplemented!()
    }
}

impl<H, T> HashDBRef<H, T> for Kvdb<H, T>
where
    H: Hasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send,
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
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + send,
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
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send,
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
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
