use sp_state_machine::{DefaultError, TrieBackendStorage};

use super::*;

pub enum DatabaseAdapter<H: Hasher> {
    Rocks(RocksHashDB<H>),
    /// The memory is only for unittest
    Memory(MemoryDB<H>),
}

impl<H: Hasher> DatabaseAdapter<H> {
    pub fn default_rocksdb() -> Self {
        Self::Rocks(RocksHashDB::new())
    }

    pub fn default_memdb() -> Self {
        Self::Memory(MemoryDB::default())
    }

    pub fn snapshot(&self) -> Self {
        match self {
            DatabaseAdapter::Rocks(kvdb) => DatabaseAdapter::Rocks(kvdb.snapshot()),
            DatabaseAdapter::Memory(mdb) => DatabaseAdapter::Memory(mdb.clone()),
        }
    }

    pub fn consolidate_mdb(&mut self, other: MemoryDB<H>) {
        match self {
            DatabaseAdapter::Rocks(kvdb) => kvdb.consolidate_mdb(other),
            DatabaseAdapter::Memory(mdb) => mdb.consolidate(other),
        }
    }
}

impl<H: Hasher> TrieBackendStorage<H> for DatabaseAdapter<H> {
    type Overlay = MemoryDB<H>;

    fn get(
        &self,
        key: &H::Out,
        prefix: hash_db::Prefix,
    ) -> Result<Option<sp_state_machine::DBValue>, DefaultError> {
        match self {
            DatabaseAdapter::Rocks(kvdb) => kvdb.get(key, prefix),
            DatabaseAdapter::Memory(mdb) => mdb.get(key, prefix),
        }
    }
}
