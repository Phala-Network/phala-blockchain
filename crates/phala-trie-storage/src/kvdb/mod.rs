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
