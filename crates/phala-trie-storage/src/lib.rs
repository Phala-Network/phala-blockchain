#[cfg(feature = "serde")]
pub mod ser;

mod fused;
mod kvdb;
mod memdb;

#[cfg(test)]
mod tests;

use fused::DatabaseAdapter;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::iter::FromIterator;

use parity_scale_codec::Codec;
use sp_core::storage::ChildInfo;
use sp_core::Hasher;
use sp_state_machine::{Backend, IterArgs, TrieBackend, TrieBackendBuilder};
use sp_trie::{trie_types::TrieDBMutBuilderV0 as TrieDBMutBuilder, TrieMut};

pub use kvdb::{RocksDB, RocksHashDB};
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
pub type KvdbBackend<H> = TrieBackend<fused::DatabaseAdapter<H>, H>;

pub struct TrieStorage<H: Hasher>(KvdbBackend<H>)
where
    H::Out: Ord;

impl<H: Hasher> Default for TrieStorage<H>
where
    H::Out: Codec + Ord,
{
    fn default() -> Self {
        Self(
            TrieBackendBuilder::new(DatabaseAdapter::default_rocksdb(), Default::default()).build(),
        )
    }
}

impl<H: Hasher> TrieStorage<H>
where
    H::Out: Codec + Ord,
{
    pub fn default_memdb() -> Self {
        Self(TrieBackendBuilder::new(DatabaseAdapter::default_memdb(), Default::default()).build())
    }

    pub fn snapshot(&self) -> Self {
        Self(clone_trie_backend(&self.0))
    }
}

pub fn load_trie_backend<H: Hasher>(
    pairs: impl Iterator<Item = (impl AsRef<[u8]>, impl AsRef<[u8]>)>,
) -> KvdbBackend<H>
where
    H::Out: Codec + Ord,
{
    let mut root = Default::default();
    let mut mdb: MemoryDB<H> = Default::default();
    {
        let mut trie_db = TrieDBMutBuilder::new(&mut mdb, &mut root).build();
        for (key, value) in pairs {
            if trie_db.insert(key.as_ref(), value.as_ref()).is_err() {
                panic!("Insert item into trie DB should not fail");
            }
        }
    }
    TrieBackendBuilder::new(DatabaseAdapter::Rocks(RocksHashDB::load(mdb)), root).build()
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum TrieState<V0, V1> {
    V0(V0),
    V1(V1),
}

#[cfg(feature = "serde")]
pub fn serialize_trie_backend<H: Hasher, S>(
    trie: &KvdbBackend<H>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    H::Out: Codec + Serialize + Ord,
    S: Serializer,
{
    let root = trie.root();
    let db = trie.backend_storage();
    match db {
        DatabaseAdapter::Rocks(db) => (root, db).serialize(serializer),
        DatabaseAdapter::Memory(_) => Err(serde::ser::Error::custom(
            "Cannot serialize memory DB. It's only for unit test only",
        )),
    }
}

#[cfg(feature = "serde")]
pub fn deserialize_trie_backend<'de, H: Hasher, De>(
    deserializer: De,
) -> Result<KvdbBackend<H>, De::Error>
where
    H::Out: Codec + Deserialize<'de> + Ord,
    De: Deserializer<'de>,
{
    let (root, db): (H::Out, RocksHashDB<H>) = Deserialize::deserialize(deserializer)?;
    let db = DatabaseAdapter::Rocks(db);
    let backend = TrieBackendBuilder::new(db, root).build();
    Ok(backend)
}

pub fn clone_trie_backend<H: Hasher>(trie: &KvdbBackend<H>) -> KvdbBackend<H>
where
    H::Out: Codec + Ord,
{
    let root = trie.root();
    let mdb = trie.backend_storage().snapshot();
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
        let mut storage = core::mem::replace(self, Self::default_memdb())
            .0
            .into_storage();
        storage.consolidate_mdb(transaction);
        let _ = core::mem::replace(&mut self.0, TrieBackendBuilder::new(storage, root).build());
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
        let mut iter_args = IterArgs::default();
        iter_args.prefix = Some(prefix.as_ref());

        self.0
            .pairs(iter_args)
            .expect("Should get the pairs iter")
            .map(|pair| {
                let (k, v) = pair.expect("Should get the key and value");
                (k, v)
            })
            .collect()
    }

    pub fn as_trie_backend(&self) -> &KvdbBackend<H> {
        &self.0
    }

    pub fn set_root(&mut self, root: H::Out) {
        let storage = core::mem::replace(self, Self::default_memdb())
            .0
            .into_storage();
        let _ = core::mem::replace(&mut self.0, TrieBackendBuilder::new(storage, root).build());
    }

    pub fn load_proof(&mut self, proof: Vec<Vec<u8>>) {
        use hash_db::HashDB as _;
        let root = *self.root();
        let mut storage = MemoryDB::default();
        for value in proof {
            let hash = storage.insert(hash_db::EMPTY_PREFIX, &value);
            log::debug!("Loaded proof {:?}", hash);
        }
        let storage = DatabaseAdapter::Memory(storage);
        let _ = core::mem::replace(&mut self.0, TrieBackendBuilder::new(storage, root).build());
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
