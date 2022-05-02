mod phaladb;
pub mod ser;

use parity_scale_codec::Codec;
pub use phaladb::PhalaDB;
pub use phaladb::Transaction;

use hash_db::Hasher;
use phaladb::PhalaTrieStorage;
use sp_state_machine::Backend;
use sp_state_machine::MemoryDB;
use sp_state_machine::TrieBackend;
use sp_storage::ChildInfo;
use sp_trie::empty_trie_root;
use sp_trie::LayoutV1;
use std::path::Path;

pub struct PhalaTrieBackend<H: Hasher>(pub TrieBackend<PhalaTrieStorage<H>, H>);

impl<H: Hasher> Default for PhalaTrieBackend<H>
where
    H::Out: Codec + Ord,
{
    fn default() -> Self {
        Self(TrieBackend::new(
            PhalaTrieStorage::Empty,
            empty_trie_root::<LayoutV1<H>>(),
        ))
    }
}

pub trait TransactionalBackend<H: Hasher> {
    // NOTE: is not thread-safe so keep this method run with lock
    fn commit_transaction(&mut self, root: H::Out, transaction: MemoryDB<H>);
}

impl<H: Hasher> TransactionalBackend<H> for PhalaTrieBackend<H>
where
    H::Out: Codec + Ord,
{
    fn commit_transaction(&mut self, root: H::Out, transaction: MemoryDB<H>) {
        let storage = std::mem::take(self).0.into_storage();
        match &storage {
            PhalaTrieStorage::Mutual(db) => {
                db.commit_in_memory_transaction(transaction);
            }
            _ => unimplemented!("snapshot database is not supported commit transaction from pink"),
        }
        let backend = TrieBackend::new(storage, root);
        let _ = std::mem::replace(&mut self.0, backend);
    }
}

pub fn new_memory_backend<H: Hasher>() -> PhalaTrieBackend<H>
where
    H::Out: Codec + Ord,
{
    let storage = PhalaTrieStorage::Mutual(PhalaDB::<H>::new_in_memory());
    PhalaTrieBackend(TrieBackend::new(storage, empty_trie_root::<LayoutV1<H>>()))
}

pub fn new_disk_backend<H: Hasher, F: Fn(&PhalaDB<H>) -> H::Out>
(
    path: impl AsRef<Path>,
    retrieve_root: F,
) -> PhalaTrieBackend<H> 
where
    H::Out: Codec + Ord,
{
    let db = PhalaDB::<H>::new_with_disk(path);
    let root = retrieve_root(&db);
    PhalaTrieBackend(TrieBackend::new(PhalaTrieStorage::Mutual(db), root))
}

// used in phactory

/// Storage key.
pub type StorageKey = Vec<u8>;

/// Storage value.
pub type StorageValue = Vec<u8>;

/// In memory array of storage values.
pub type StorageCollection = Vec<(StorageKey, Option<StorageValue>)>;

/// In memory arrays of storage values for multiple child tries.
pub type ChildStorageCollection = Vec<(StorageKey, StorageCollection)>;

impl<H: Hasher> PhalaTrieBackend<H>
where
    H::Out: Codec + Ord,
{
    pub fn calc_root_if_changes<'a>(
        &self,
        delta: &'a StorageCollection,
        child_deltas: &'a ChildStorageCollection,
    ) -> (H::Out, MemoryDB<H>) {
        let child_deltas: Vec<(ChildInfo, &StorageCollection)> = child_deltas
            .iter()
            .map(|(k, v)| {
                let chinfo = ChildInfo::new_default(k);
                (chinfo, v)
            })
            .collect();
        self.0.full_storage_root(
            delta
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref().map(|v| v.as_ref()))),
            child_deltas.iter().map(|(k, v)| {
                (
                    k,
                    v.iter()
                        .map(|(k, v)| (k.as_ref(), v.as_ref().map(|v| v.as_ref()))),
                )
            }),
            sp_core::storage::StateVersion::V0,
        )
    }

    // TODO:george should use more specificed methods to access the system state
    pub fn get_raw(&self, key: &[u8]) -> Option<Vec<u8>> {
        let storage = self.0.backend_storage();
        match storage {
            PhalaTrieStorage::Mutual(db) => db.get_raw(key),
            _ => None,
        }
    }

    pub fn begin_transaction(&self) -> Transaction<H::Out> {
        Default::default()
    }

    pub fn finalized_block_with_outer_transaction(&self, transaction: Transaction<H>) {
        let storage = self.0.backend_storage();
        match storage {
            PhalaTrieStorage::Mutual(db) => db.finalized(transaction),
            _ => unimplemented!("only the main storage could finalized after dispatching block")
        }
    }

}
