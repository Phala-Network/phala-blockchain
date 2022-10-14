extern crate alloc;

#[cfg(feature = "serde")]
pub mod ser;

mod memdb;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use core::iter::FromIterator;

use alloc::vec::Vec;

use parity_scale_codec::Codec;
use sp_core::storage::ChildInfo;
use sp_core::Hasher;
use sp_state_machine::{Backend, TrieBackend, TrieBackendBuilder};
use sp_trie::{
    TrieMut,
    trie_types::{
        TrieDBMutBuilderV0 as TrieDBMutBuilder,
    },
};

pub use memdb::GenericMemoryDB as MemoryDB;

/// Storage key.
pub type StorageKey = Vec<u8>;

/// Storage value.
pub type StorageValue = Vec<u8>;

/// In memory array of storage values.
pub type StorageCollection = Vec<(StorageKey, Option<StorageValue>)>;

/// In memory arrays of storage values for multiple child tries.
pub type ChildStorageCollection = Vec<(StorageKey, StorageCollection)>;

pub type InMemoryBackend<H> = TrieBackend<MemoryDB<H>, H>;
pub struct TrieStorage<H: Hasher>(InMemoryBackend<H>);

impl<H: Hasher> Default for TrieStorage<H>
where
    H::Out: Codec,
{
    fn default() -> Self {
        Self(TrieBackendBuilder::new(Default::default(), Default::default()).build())
    }
}

pub fn load_trie_backend<H: Hasher>(
    pairs: impl Iterator<Item = (impl AsRef<[u8]>, impl AsRef<[u8]>)>,
) -> TrieBackend<MemoryDB<H>, H>
where
    H::Out: Codec,
{
    let mut root = Default::default();
    let mut mdb = Default::default();
    {
        let mut trie_db = TrieDBMutBuilder::new(&mut mdb, &mut root).build();
        for (key, value) in pairs {
            if trie_db.insert(key.as_ref(), value.as_ref()).is_err() {
                panic!("Insert item into trie DB should not fail");
            }
        }
    }
    TrieBackendBuilder::new(mdb, root).build()
}

#[cfg(feature = "serde")]
pub fn serialize_trie_backend<H: Hasher, S>(
    trie: &TrieBackend<MemoryDB<H>, H>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    H::Out: Codec + Serialize,
    S: Serializer,
{
    let root = trie.root();
    let kvs: im::HashMap<_, _> = trie.backend_storage().clone().drain();
    (root, kvs).serialize(serializer)
}

#[cfg(feature = "serde")]
pub fn deserialize_trie_backend<'de, H: Hasher, De>(
    deserializer: De,
) -> Result<TrieBackend<MemoryDB<H>, H>, De::Error>
where
    H::Out: Codec + Deserialize<'de>,
    De: Deserializer<'de>,
{
    let (root, kvs): (H::Out, im::HashMap<_, (Vec<u8>, i32)>) =
        Deserialize::deserialize(deserializer)?;
    let mdb = MemoryDB::from_inner(kvs);
    let backend = TrieBackendBuilder::new(mdb, root).build();
    Ok(backend)
}

pub fn clone_trie_backend<H: Hasher>(
    trie: &TrieBackend<MemoryDB<H>, H>,
) -> TrieBackend<MemoryDB<H>, H>
where
    H::Out: Codec,
{
    let root = trie.root();
    let mdb = trie
        .backend_storage()
        .clone();
    TrieBackendBuilder::new(mdb, *root).build()
}

impl<H: Hasher> TrieStorage<H>
where
    H::Out: Codec + Ord,
{
    /// Overwrite all data in the trie DB with given key/value pairs.
    pub fn load(&mut self, pairs: impl Iterator<Item = (impl AsRef<[u8]>, impl AsRef<[u8]>)>) {
        let trie = load_trie_backend(pairs);
        let _ = core::mem::replace(&mut self.0, trie);
    }

    /// Calculate the new state root given storage changes. Returns the new root and a transaction to apply.
    #[allow(clippy::ptr_arg)]
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

    /// Apply storage changes calculated from `calc_root_if_changes`.
    pub fn apply_changes(&mut self, root: H::Out, transaction: MemoryDB<H>) {
        let mut storage = core::mem::take(self).0.into_storage();
        storage.consolidate(transaction);
        let _ = core::mem::replace(
            &mut self.0,
            TrieBackendBuilder::new(storage, root).build()
        );
    }

    pub fn purge(&mut self) {}

    /// Return the state root hash
    pub fn root(&self) -> &H::Out {
        self.0.root()
    }

    /// Given storage key return storage value
    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        self.0.storage(key.as_ref()).ok().flatten()
    }

    /// Return storage pairs which start with given storage key prefix
    pub fn pairs(&self, prefix: impl AsRef<[u8]>) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.pairs_into(prefix)
    }

    fn pairs_into<R: FromIterator<(Vec<u8>, Vec<u8>)>>(&self, prefix: impl AsRef<[u8]>) -> R {
        self.0
            .keys(prefix.as_ref())
            .into_iter()
            .map(|key| {
                let value = self.get(&key).expect("Reflected key should exists");
                (key, value)
            })
            .collect()
    }
}

#[cfg(feature = "serde")]
const _: () = {
    impl<H: Hasher> Serialize for TrieStorage<H>
    where
        H::Out: Codec + Serialize + Ord,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serialize_trie_backend(&self.0, serializer)
        }
    }

    impl<'de, H: Hasher> Deserialize<'de> for TrieStorage<H>
    where
        H::Out: Codec + Deserialize<'de> + Ord,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(Self(deserialize_trie_backend(deserializer)?))
        }
    }
};
