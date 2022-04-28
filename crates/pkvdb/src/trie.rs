// define the trie storage used in state_machine and for memorydb ?

use std::path::Path;

use crate::leveldb::Kvdb;
use crate::snapshot::SnapshotDB;
use hash_db::HashDB;
use hash_db::Hasher;
use hash_db::Prefix;
use parity_scale_codec::Codec;
use sp_state_machine::Backend;
use sp_state_machine::InMemoryBackend;
use sp_state_machine::TrieBackend;
use sp_state_machine::TrieBackendStorage;
pub use sp_trie::DBValue;
use sp_trie::MemoryDB;

// for batch commit txs
pub type MemoryTrieBackend<H> = InMemoryBackend<H>;

// reexport the state_machine Result type used for impl the TrieBackendStorage
type Result<T> = sp_std::result::Result<T, sp_state_machine::DefaultError>;

// just a wrapper for Kvdb and then the Kvdb could be used
// as the TrieBackend(inner)
impl<H: Hasher> TrieBackendStorage<H> for Kvdb<H, DBValue> {
    type Overlay = MemoryDB<H>;

    fn get(&self, key: &H::Out, prefix: Prefix) -> Result<Option<DBValue>> {
        Ok(HashDB::get(self, key, prefix))
    }
}

// Pink will take a snapshot from the LevelDB and then create a new TrieBackend
impl<H: Hasher> TrieBackendStorage<H> for SnapshotDB<H, DBValue> {
    type Overlay = MemoryDB<H>;

    fn get(&self, key: &H::Out, prefix: Prefix) -> Result<Option<DBValue>> {
        Ok(HashDB::get(self, key, prefix))
    }
}

// Pink to commit transaction
pub trait CommitTransaction<H: Hasher>: Backend<H> {
    fn commit_transaction(&mut self, root: H::Out, transaction: Self::Transaction);
}


impl<H: Hasher> CommitTransaction<H> for MemoryTrieBackend<H>
where
    H::Out: Codec + Ord,
{
    fn commit_transaction(&mut self, root: H::Out, transaction: MemoryDB<H>) {
        self.apply_transaction(root, transaction)
    }
}

// use for pink to take snapshot
pub trait TakeSnapshot {
    type Snapshot;
    fn snapshot(&self) -> Self::Snapshot;
}

impl<H: Hasher> TakeSnapshot for MemoryTrieBackend<H>
where
    H::Out: Codec + Ord,
{
    type Snapshot = Self;
    fn snapshot(&self) -> Self::Snapshot {
        let root = self.root();
        let kvs: Vec<_> = self
            .backend_storage()
            .clone()
            .drain()
            .into_iter()
            .map(|it| it.1)
            .collect();
        let mut db = MemoryDB::default();
        for value in kvs {
            for _ in 0..value.1 {
                db.insert((&[], None), &value.0);
            }
        }
        InMemoryBackend::new(db, *root)
    }
}

// helper type for outer usage

pub type PhalaTrieStorage<H> = TrieBackend<PhalaStorage<H>, H>;

pub enum PhalaStorage<H: Hasher> {
    LevelDB(Kvdb<H, DBValue>),
    CombinedLevelDB(Kvdb<H, DBValue>),
    SnapshotDB(SnapshotDB<H, DBValue>),
}

impl<H: Hasher> PhalaStorage<H> 
where 
    H::Out: Codec + Ord
{

   pub fn open(path: impl AsRef<Path>, with_checkpoint: bool) -> PhalaStorage<H> {
       let db = Kvdb::new(path);
       if with_checkpoint {
           let db = db.as_checkpoint_db();
           PhalaStorage::CombinedLevelDB(db)
       } else {
           PhalaStorage::LevelDB(db)
       }
   }
 
}

impl<H: Hasher> PhalaStorage<H> {
    fn as_dyn_trie_storage(&self) -> &dyn TrieBackendStorage<H, Overlay = MemoryDB<H>> {
        match self {
            Self::LevelDB(l) => l,
            Self::SnapshotDB(s) => s,
            Self::CombinedLevelDB(c) => c,
        }
    }
}

impl<H: Hasher> TrieBackendStorage<H> for PhalaStorage<H> {
    type Overlay = MemoryDB<H>;

    fn get(&self, key: &H::Out, prefix: Prefix) -> Result<Option<DBValue>> {
        let reference = self.as_dyn_trie_storage();
        TrieBackendStorage::get(reference, key, prefix)
    }
}

impl<H: Hasher> CommitTransaction<H> for TrieBackend<PhalaStorage<H>, H>
where
    H::Out: Codec + Ord,
{
    fn commit_transaction(&mut self, root: H::Out, transaction: Self::Transaction) {
        let storage = self.backend_storage();
        match storage {
            PhalaStorage::LevelDB(l) => {
                let storage = l.clone();
                storage.consolidate(transaction);
                *self = TrieBackend::new(PhalaStorage::LevelDB(storage), root);
            }
            PhalaStorage::SnapshotDB(_) => {
                unimplemented!("snapshot can not commit a transaction just for reading");
            },
            PhalaStorage::CombinedLevelDB(c) => {
                c.submit(transaction)
            }
        }
    }
}

impl<H: Hasher> TakeSnapshot for TrieBackend<PhalaStorage<H>, H>
where
    H::Out: Codec + Ord,
{
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        let storage = self.backend_storage();
        let root = self.root();
        match storage {
            PhalaStorage::LevelDB(l) => {
                let storage = l.take_sanpshot();
                TrieBackend::new(PhalaStorage::SnapshotDB(storage), *root)
            }
            PhalaStorage::SnapshotDB(_) => {
                unimplemented!("snapshot can not generate snapshot again")
            },
            PhalaStorage::CombinedLevelDB(c) => {
                let storage = c.take_sanpshot();
                TrieBackend::new(PhalaStorage::SnapshotDB(storage), *root)
            }
        }
    }
}
