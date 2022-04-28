// define the rawdb common behavior for storage the phactory and system config

use crate::leveldb::Kvdb;
use hash_db::Hasher;
use rusty_leveldb::WriteBatch;
use rusty_leveldb::DB;
use sp_std::collections::btree_map::BTreeMap;
use std::borrow::BorrowMut;

#[derive(Default, Clone)]
pub struct Transction {
    pub(crate) puts: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) deletes: Vec<Vec<u8>>,
}

impl Transction {
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.puts.insert(key, value);
    }

    pub fn delete(&mut self, key: Vec<u8>) {
        self.deletes.push(key);
    }
}

pub trait RawDB {
    // put kv into underlying database witout hashing
    fn put(&mut self, key: &[u8], value: &[u8]);

    // get value from underlying database use newest snapshot
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

    fn begin(&self) -> Transction;

    fn commit(&mut self, transaction: Transction);
}

impl<H, T> RawDB for Kvdb<H, T>
where
    H: Hasher,
{
    fn put(&mut self, key: &[u8], value: &[u8]) {
        let mut db_mut = self.leveldb.lock().expect("put have to hold the lock");
        let _ = DB::put(db_mut.borrow_mut(), key, value);
    }

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut db_mut = self.leveldb.lock().expect("get have to hold the lock");
        DB::get(db_mut.borrow_mut(), key)
    }

    fn begin(&self) -> Transction {
        Default::default()
    }

    fn commit(&mut self, transaction: Transction) {
        let mut writebatch = WriteBatch::new();
        for (key, value) in &transaction.puts {
            writebatch.put(key, value);
        }
        for key in &transaction.deletes {
            writebatch.delete(key);
        }
        let mut db_mut = self.leveldb.lock().expect("commit have to hold the lock");
        let _ = DB::write(db_mut.borrow_mut(), writebatch, true);
    }
}
