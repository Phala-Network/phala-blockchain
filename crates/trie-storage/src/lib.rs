#![no_std]

extern crate alloc;

#[cfg(feature = "sgx")]
pub extern crate serde_sgx as serde;

#[cfg(not(feature = "sgx"))]
pub extern crate serde_std as serde;

pub mod ser;

use alloc::vec::Vec;

use parity_scale_codec::Codec;
use sp_core::storage::ChildInfo;
use sp_core::Hasher;
use sp_state_machine::{Backend, TrieBackend};
use sp_trie::{trie_types::TrieDBMut, MemoryDB, TrieMut};

/// Storage key.
pub type StorageKey = Vec<u8>;

/// Storage value.
pub type StorageValue = Vec<u8>;

/// In memory array of storage values.
pub type StorageCollection = Vec<(StorageKey, Option<StorageValue>)>;

/// In memory arrays of storage values for multiple child tries.
pub type ChildStorageCollection = Vec<(StorageKey, StorageCollection)>;

pub struct TrieStorage<H: Hasher>(TrieBackend<MemoryDB<H>, H>);

impl<H: Hasher> Default for TrieStorage<H>
where
    H::Out: Codec,
{
    fn default() -> Self {
        Self(TrieBackend::new(Default::default(), Default::default()))
    }
}

impl<H: Hasher> TrieStorage<H>
where
    H::Out: Codec + Ord,
{
    /// Overwrite all data in the trie DB with given key/value pairs.
    pub fn load(&mut self, pairs: impl Iterator<Item = (impl AsRef<[u8]>, impl AsRef<[u8]>)>) {
        let trie_be = core::mem::replace(self, Default::default()).0;
        let mut root = *trie_be.root();
        let mut mdb = trie_be.into_storage();
        {
            let mut trie_db = TrieDBMut::new(&mut mdb, &mut root);
            for (key, value) in pairs {
                match trie_db.insert(key.as_ref(), value.as_ref()) {
                    Err(_) => panic!("Insert item into trie DB should not fail"),
                    _ => (),
                }
            }
        }
        let _ = core::mem::replace(&mut self.0, TrieBackend::new(mdb, root));
    }

    /// Apply storage changes grabbed from chain node to the trie DB.
    pub fn apply_changes(
        &mut self,
        delta: StorageCollection,
        child_deltas: ChildStorageCollection,
    ) {
        let child_deltas: Vec<(ChildInfo, StorageCollection)> = child_deltas
            .into_iter()
            .map(|(k, v)| {
                let chinfo = ChildInfo::new_default(k.as_ref());
                (chinfo, v)
            })
            .collect();
        let (root, transaction) = self.0.full_storage_root(
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
        );
        // self.0.apply_transaction(root, transaction);  // TrieBackend has this api for std
        {
            // workaround for no_std
            self.0.backend_storage_mut().consolidate(transaction);
            let trie_be = core::mem::replace(self, Default::default()).0;
            let _ = core::mem::replace(&mut self.0, TrieBackend::new(trie_be.into_storage(), root));
        }
    }

    /// Return the state root hash
    pub fn root(&self) -> &H::Out {
        self.0.root()
    }
}
