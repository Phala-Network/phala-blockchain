extern crate alloc;

#[cfg(feature = "serde")]
pub mod ser;

mod kvdb;
mod memdb;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use core::iter::FromIterator;

use alloc::vec::Vec;

use parity_scale_codec::Codec;
use sp_core::storage::ChildInfo;
use sp_core::Hasher;
use sp_state_machine::{Backend, TrieBackend};
use sp_trie::{trie_types::TrieDBMutV0 as TrieDBMut, TrieMut};

pub use kvdb::{KeyValueDB, global_init};
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
pub type KeyValueDBBackend<H> = TrieBackend<KeyValueDB<H>, H>;
pub struct TrieStorage<H: Hasher>(KeyValueDBBackend<H>);

impl<H: Hasher> Default for TrieStorage<H>
where
    H::Out: Codec,
{
    fn default() -> Self {
        Self(TrieBackend::new(KeyValueDB::new(), Default::default()))
    }
}

pub fn load_trie_backend<H: Hasher>(
    pairs: impl Iterator<Item = (impl AsRef<[u8]>, impl AsRef<[u8]>)>,
) -> TrieBackend<KeyValueDB<H>, H>
where
    H::Out: Codec,
{
    let mut root = Default::default();
    let mut mdb: MemoryDB<H> = Default::default();
    {
        let mut trie_db = TrieDBMut::new(&mut mdb, &mut root);
        for (key, value) in pairs {
            if trie_db.insert(key.as_ref(), value.as_ref()).is_err() {
                panic!("Insert item into trie DB should not fail");
            }
        }
    }
    TrieBackend::new(KeyValueDB::load(mdb), root)
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum TrieState<V0, V1> {
    V0(V0),
    V1(V1),
}

#[cfg(feature = "serde")]
pub fn serialize_trie_backend<H: Hasher, S>(
    trie: &TrieBackend<KeyValueDB<H>, H>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    H::Out: Codec + Serialize,
    S: Serializer,
{
    let root = trie.root();
    let backup_ver: u32 = 0;
    TrieState::<(), _>::V1((root, backup_ver)).serialize(serializer)
}

#[cfg(feature = "serde")]
pub fn deserialize_trie_backend<'de, H: Hasher, De>(
    deserializer: De,
) -> Result<TrieBackend<KeyValueDB<H>, H>, De::Error>
where
    H::Out: Codec + Deserialize<'de>,
    De: Deserializer<'de>,
{
    let (root, db) = match Deserialize::deserialize(deserializer)? {
        TrieState::V0((root, kvs)) => {
            let mdb = MemoryDB::<H>::from_inner(kvs);
            (root, KeyValueDB::load(mdb))
        }
        TrieState::V1((root, ver)) => {
            let ver: u32 = ver;
            let db = todo!();
            (root, db)
        }
    };
    let backend = TrieBackend::new(db, root);
    Ok(backend)
}

pub fn clone_trie_backend<H: Hasher>(
    trie: &TrieBackend<KeyValueDB<H>, H>,
) -> TrieBackend<KeyValueDB<H>, H>
where
    H::Out: Codec,
{
    let root = trie.root();
    let mdb = trie.backend_storage().snapshot();
    TrieBackend::new(mdb, *root)
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
        let storage = core::mem::take(self).0.into_storage();
        storage.consolidate(transaction);
        let _ = core::mem::replace(&mut self.0, TrieBackend::new(storage, root));
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
