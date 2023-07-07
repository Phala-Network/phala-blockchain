use std::sync::{atomic::AtomicUsize, Arc};

use atomic::Ordering;
use log::info;
use ouroboros::self_referencing;
use redb::{Database, ReadTransaction, ReadableTable, TableDefinition, WriteTransaction};
use serde::{Deserialize, Serialize};

use super::traits::{KvStorage, Transaction};

#[self_referencing]
pub struct OwnedTransaction {
    db: Arc<Database>,
    #[borrows(db)]
    #[covariant]
    tx: ReadTransaction<'this>,
}
const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("pairs");

pub enum Redb {
    Database { db: Arc<Database>, sn: usize },
    Snapshot(Arc<OwnedTransaction>),
}

impl KvStorage for Redb {
    type Transaction<'a> = WriteTransaction<'a>;

    fn new() -> Self
    where
        Self: Sized,
    {
        let (db, sn) = create_db();
        let db = Arc::new(db);
        Redb::Database { db, sn }
    }

    fn snapshot(&self) -> Self
    where
        Self: Sized,
    {
        match self {
            Redb::Database { db, .. } => {
                let tx = OwnedTransactionBuilder {
                    db: db.clone(),
                    tx_builder: |db| db.begin_read().expect("begin_read failed"),
                }
                .build();
                Redb::Snapshot(Arc::new(tx))
            }
            Redb::Snapshot(snap) => Redb::Snapshot(snap.clone()),
        }
    }

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self {
            Redb::Database { db, .. } => {
                let tx = db.begin_read().expect("begin_read failed");
                let table = tx.open_table(TABLE).expect("open_table failed");
                table
                    .get(key)
                    .expect("get failed")
                    .map(|v| v.value().to_vec())
            }
            Redb::Snapshot(snap) => {
                let table = snap
                    .borrow_tx()
                    .open_table(TABLE)
                    .expect("open_table failed");
                table
                    .get(key)
                    .expect("get failed")
                    .map(|v| v.value().to_vec())
            }
        }
    }

    fn transaction<'a>(&'a self) -> Self::Transaction<'a> {
        let Self::Database { db, sn: _ } = self else {
            panic!("transaction() called on snapshot")
        };
        db.begin_write().expect("begin_write failed")
    }

    fn for_each(&self, mut cb: impl FnMut(&[u8], &[u8])) {
        match self {
            Redb::Database { db, .. } => {
                let tx = db.begin_read().expect("begin_read failed");
                let table = tx.open_table(TABLE).expect("open_table failed");
                for result in table.iter().expect("iter over redb failed") {
                    let (k, v) = result.expect("iter over redb failed");
                    cb(k.value(), v.value());
                }
            }
            Redb::Snapshot(snap) => {
                let table = snap
                    .borrow_tx()
                    .open_table(TABLE)
                    .expect("open_table failed");
                for result in table.iter().expect("iter over redb failed") {
                    let (k, v) = result.expect("iter over redb failed");
                    cb(k.value(), v.value());
                }
            }
        }
    }
}

impl Default for Redb {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for Redb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        super::serializing::serialize_as_map(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Redb {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        super::serializing::deserialize_from_map(deserializer)
    }
}

impl Transaction for WriteTransaction<'_> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.open_table(TABLE)
            .expect("open_table failed")
            .get(key)
            .expect("get failed")
            .map(|v| v.value().to_vec())
    }

    fn put(&self, key: &[u8], value: &[u8]) {
        self.open_table(TABLE)
            .expect("open_table failed")
            .insert(key, value)
            .expect("put failed");
    }

    fn delete(&self, key: &[u8]) {
        self.open_table(TABLE)
            .expect("open_table failed")
            .remove(key)
            .expect("delete failed");
    }

    fn commit(self) {
        self.commit().expect("commit failed");
    }
}

fn create_db() -> (Database, usize) {
    let cache_dir = super::cache_dir::get();
    static NEXT_SN: AtomicUsize = AtomicUsize::new(0);
    let sn = NEXT_SN.fetch_add(1, Ordering::SeqCst);
    if sn == 0 {
        if std::path::Path::new(&cache_dir).exists() {
            info!("Removing cache folder: {}", &cache_dir);
            std::fs::remove_dir_all(&cache_dir).expect("Failed to remove cache folder");
        }
        std::fs::create_dir_all(&cache_dir).expect("Failed to create cache folder");
    }
    let path = format!("{cache_dir}/cache-{sn}.redb",);
    let db = Database::builder()
        .set_cache_size(1024 * 1024 * 128)
        .create(&path)
        .expect("Failed to create database");
    (db, sn)
}
