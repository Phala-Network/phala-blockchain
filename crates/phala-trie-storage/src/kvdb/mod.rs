use std::sync::Arc;

use librocksdb_sys as ffi;
use rocksdb::{Error as DBError, MultiThreaded, TransactionDB};

pub use database::RocksDB;
pub use hashdb::RocksHashDB;
pub use snapshot::Snapshot;

type Database = Arc<TransactionDB<MultiThreaded>>;

mod database;
mod hashdb;
mod snapshot;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_works() {
        let db = RocksDB::new();
        db.put(b"foo", b"bar").unwrap();
        assert_eq!(db.get(b"foo").unwrap().unwrap(), b"bar");
        let snapshot = db.snapshot();
        assert_eq!(snapshot.get(b"foo").unwrap().unwrap(), b"bar");
        db.delete(b"foo").unwrap();
        assert_eq!(db.get(b"foo").unwrap(), None);
        assert_eq!(snapshot.get(b"foo").unwrap().unwrap(), b"bar");
    }
}
