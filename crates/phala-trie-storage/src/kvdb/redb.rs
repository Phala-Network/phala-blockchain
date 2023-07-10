use std::sync::{atomic::AtomicUsize, Arc};

use atomic::Ordering;
use log::info;
use ouroboros::self_referencing;
use redb::{
    Database, ReadOnlyTable, ReadTransaction, ReadableTable, TableDefinition, WriteTransaction,
};
use serde::{Deserialize, Serialize};

use super::traits::{KvStorage, Transaction};

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("pairs");

#[self_referencing]
pub struct OwnedTransaction {
    db: Arc<Database>,
    #[borrows(db)]
    #[covariant]
    tx: ReadTransaction<'this>,
}

#[self_referencing]
pub struct OwnedTable {
    tx: OwnedTransaction,
    #[borrows(tx)]
    #[covariant]
    table: ReadOnlyTable<'this, &'static [u8], &'static [u8]>,
}

impl OwnedTransaction {
    fn into_table(self) -> OwnedTable {
        OwnedTableBuilder {
            tx: self,
            table_builder: |tx| {
                tx.borrow_tx()
                    .open_table(TABLE)
                    .expect("Failed to open table in redb")
            },
        }
        .build()
    }
}

pub enum Redb {
    Database { db: Arc<Database>, sn: usize },
    Snapshot(Arc<OwnedTable>),
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
                    tx_builder: |db| {
                        db.begin_read()
                            .expect("Failed to create read transaction for snapshot")
                    },
                }
                .build();
                Redb::Snapshot(Arc::new(tx.into_table()))
            }
            Redb::Snapshot(snap) => Redb::Snapshot(snap.clone()),
        }
    }

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self {
            Redb::Database { db, .. } => {
                let tx = db
                    .begin_read()
                    .expect("Failed to create read transaction for get");
                let table = tx.open_table(TABLE).expect("Failed to open table in redb");
                table
                    .get(key)
                    .expect("Failed to get value from database, the database may be corrupted")
                    .map(|v| v.value().to_vec())
            }
            Redb::Snapshot(snap) => {
                let table = snap.borrow_table();
                table
                    .get(key)
                    .expect("Failed to get value from snapshot, the database may be corrupted")
                    .map(|v| v.value().to_vec())
            }
        }
    }

    fn transaction(&self) -> Self::Transaction<'_> {
        let Self::Database { db, sn: _ } = self else {
            panic!("transaction() called on snapshot")
        };
        db.begin_write()
            .expect("Failed to create write transaction")
    }

    fn for_each(&self, mut cb: impl FnMut(&[u8], &[u8])) {
        match self {
            Redb::Database { db, .. } => {
                let tx = db
                    .begin_read()
                    .expect("Failed to create read transaction for iteration");
                let table = tx
                    .open_table(TABLE)
                    .expect("Failed to open table for iteration");
                for result in table.iter().expect("Failed to iterate over redb") {
                    let (k, v) = result.expect("Failed to iter over redb");
                    cb(k.value(), v.value());
                }
            }
            Redb::Snapshot(snap) => {
                let table = snap.borrow_table();
                for result in table.iter().expect("Failed to iterate over redb snapshot") {
                    let (k, v) = result.expect("Failed to iter over redb snapshot");
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
            .expect("Failed to open table in redb")
            .get(key)
            .expect("Failed to get value from database, the database may be corrupted")
            .map(|v| v.value().to_vec())
    }

    fn put(&self, key: &[u8], value: &[u8]) {
        self.open_table(TABLE)
            .expect("Failed to open table in redb")
            .insert(key, value)
            .expect("Failed to put value into database, the database may be corrupted");
    }

    fn delete(&self, key: &[u8]) {
        self.open_table(TABLE)
            .expect("Failed to open table in redb")
            .remove(key)
            .expect("Failed to delete value from database, the database may be corrupted");
    }

    fn commit(self) {
        self.commit().expect("Failed to commit transaction");
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
        .create(path)
        .expect("Failed to create database");
    (db, sn)
}
