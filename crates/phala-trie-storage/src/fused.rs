use sp_state_machine::{DefaultError, TrieBackendStorage};

use super::*;

pub enum DatabaseAdapter<H: Hasher> {
    RocksDB(RocksHashDB<H>),
    Memory(MemoryDB<H>),
}

impl<H: Hasher> DatabaseAdapter<H> {
    pub fn default_rocksdb() -> Self {
        Self::RocksDB(RocksHashDB::new())
    }

    pub fn default_memdb() -> Self {
        Self::Memory(MemoryDB::default())
    }

    pub fn load(mdb: MemoryDB<H>, db_type: crate::DBType) -> Self {
        match db_type {
            DBType::Memory => Self::Memory(mdb),
            DBType::RocksDB => Self::RocksDB(RocksHashDB::load(mdb)),
        }
    }

    pub fn new(typ: crate::DBType) -> Self {
        match typ {
            DBType::Memory => Self::Memory(Default::default()),
            DBType::RocksDB => Self::RocksDB(Default::default()),
        }
    }

    pub fn snapshot(&self) -> Self {
        match self {
            DatabaseAdapter::RocksDB(kvdb) => DatabaseAdapter::RocksDB(kvdb.snapshot()),
            DatabaseAdapter::Memory(mdb) => DatabaseAdapter::Memory(mdb.clone()),
        }
    }

    pub fn consolidate_mdb(&mut self, other: MemoryDB<H>) {
        match self {
            DatabaseAdapter::RocksDB(kvdb) => kvdb.consolidate_mdb(other),
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
            DatabaseAdapter::RocksDB(kvdb) => kvdb.get(key, prefix),
            DatabaseAdapter::Memory(mdb) => mdb.get(key, prefix),
        }
    }
}
