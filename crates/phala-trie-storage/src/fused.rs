use sp_state_machine::{DefaultError, TrieBackendStorage};

use super::*;

pub enum RocksOrMemoryDB<H: Hasher> {
    Rocks(RocksHashDB<H>),
    /// The memory is only for unittest
    Memory(MemoryDB<H>),
}

impl<H: Hasher> RocksOrMemoryDB<H> {
    pub fn default_rocksdb() -> Self {
        Self::Rocks(RocksHashDB::new())
    }

    pub fn default_memdb() -> Self {
        Self::Memory(MemoryDB::default())
    }

    pub fn snapshot(&self) -> Self {
        match self {
            RocksOrMemoryDB::Rocks(kvdb) => RocksOrMemoryDB::Rocks(kvdb.snapshot()),
            RocksOrMemoryDB::Memory(mdb) => RocksOrMemoryDB::Memory(mdb.clone()),
        }
    }

    pub fn consolidate_mdb(&mut self, other: MemoryDB<H>) {
        match self {
            RocksOrMemoryDB::Rocks(kvdb) => kvdb.consolidate_mdb(other),
            RocksOrMemoryDB::Memory(mdb) => mdb.consolidate(other),
        }
    }
}

impl<H: Hasher> TrieBackendStorage<H> for RocksOrMemoryDB<H> {
    type Overlay = MemoryDB<H>;

    fn get(
        &self,
        key: &H::Out,
        prefix: hash_db::Prefix,
    ) -> Result<Option<sp_state_machine::DBValue>, DefaultError> {
        match self {
            RocksOrMemoryDB::Rocks(kvdb) => kvdb.get(key, prefix),
            RocksOrMemoryDB::Memory(mdb) => mdb.get(key, prefix),
        }
    }
}
