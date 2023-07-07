use sp_state_machine::{DefaultError, TrieBackendStorage};

use super::*;

pub enum DatabaseAdapter<H: Hasher> {
    Memory(MemoryDB<H>),
    RocksDB(HashRocksDB<H>),
    Redb(HashRedb<H>),
}

impl<H: Hasher> DatabaseAdapter<H> {
    pub fn default_memdb() -> Self {
        Self::Memory(MemoryDB::default())
    }

    pub fn load(mdb: MemoryDB<H>, db_type: crate::DBType) -> Self {
        match db_type {
            DBType::Memory => Self::Memory(mdb),
            DBType::RocksDB => Self::RocksDB(HashRocksDB::load(mdb)),
            DBType::Redb => Self::Redb(HashRedb::load(mdb)),
        }
    }

    pub fn new(typ: crate::DBType) -> Self {
        match typ {
            DBType::Memory => Self::Memory(Default::default()),
            DBType::RocksDB => Self::RocksDB(Default::default()),
            DBType::Redb => Self::Redb(Default::default()),
        }
    }

    pub fn snapshot(&self) -> Self {
        match self {
            DatabaseAdapter::Memory(mdb) => DatabaseAdapter::Memory(mdb.clone()),
            DatabaseAdapter::RocksDB(kvdb) => DatabaseAdapter::RocksDB(kvdb.snapshot()),
            DatabaseAdapter::Redb(db) => DatabaseAdapter::Redb(db.snapshot()),
        }
    }

    pub fn consolidate_mdb(&mut self, other: MemoryDB<H>) {
        match self {
            DatabaseAdapter::Memory(mdb) => mdb.consolidate(other),
            DatabaseAdapter::RocksDB(kvdb) => kvdb.consolidate(other),
            DatabaseAdapter::Redb(db) => db.consolidate(other),
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
            DatabaseAdapter::Memory(mdb) => mdb.get(key, prefix),
            DatabaseAdapter::RocksDB(kvdb) => kvdb.get(key, prefix),
            DatabaseAdapter::Redb(db) => db.get(key, prefix),
        }
    }
}
