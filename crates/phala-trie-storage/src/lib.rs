#[cfg(feature = "serde")]
pub mod ser;
use core::iter::FromIterator;
use parity_scale_codec::Codec;
use pkvdb::LevelDB;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_core::storage::ChildInfo;
use sp_core::Hasher;
use sp_state_machine::{Backend, TrieBackend as TrieStorageBackend};
use sp_std::vec::Vec;
#[cfg(feature = "serde")]
use sp_trie::HashDBT as _;
use sp_trie::{trie_types::TrieDBMutV0 as TrieDBMut, MemoryDB, TrieMut};
use std::path::Path;

/// Storage key.
pub type StorageKey = Vec<u8>;

/// Storage value.
pub type StorageValue = Vec<u8>;

/// In memory array of storage values.
pub type StorageCollection = Vec<(StorageKey, Option<StorageValue>)>;

/// In memory arrays of storage values for multiple child tries.
pub type ChildStorageCollection = Vec<(StorageKey, StorageCollection)>;

/// In memory changes as Transaction also MemoryDB
pub type Transaction<H> = MemoryDB<H>;

pub type MemoryTrieBackend<H> = TrieStorageBackend<MemoryDB<H>, H>;

pub type PkvdbTrieBackend<H> = TrieStorageBackend<LevelDB<H>, H>;

#[cfg(feature = "memorydb")]
pub type TrieBackend<H> = MemoryTrieBackend<H>;

#[cfg(feature = "memorydb")]
pub type TrieStorage<H> = MemoryTrieStorage<H>;

#[cfg(feature = "memorydb")]
pub fn snapshot<H: Hasher>(backend: &TrieBackend<H>) -> TrieBackend<H>
where
    H::Out: Codec,
{
    let root = backend.root();
    let kvs: Vec<_> = backend
        .backend_storage()
        .clone()
        .drain()
        .into_iter()
        .map(|it| it.1)
        .collect();
    let mut mdb = MemoryDB::default();
    for value in kvs {
        for _ in 0..value.1 {
            mdb.insert((&[], None), &value.0);
        }
    }
    MemoryTrieBackend::<H>::new(mdb, *root)
}

#[cfg(not(feature = "memorydb"))]
pub type TrieBackend<H> = PkvdbTrieBackend<H>;

#[cfg(not(feature = "memorydb"))]
pub type TrieStorage<H> = PkvdbTrieStorage<H>;

#[cfg(not(feature = "memorydb"))]
pub fn snapshot<H: Hasher>(backend: &TrieBackend<H>) -> TrieBackend<H>
where
    H::Out: Codec,
{
    let root = backend.root().clone();
    let underlying = backend.backend_storage().clone();
    PkvdbTrieBackend::<H>::new(underlying, root)
}

#[cfg(feature = "serde")]
pub fn serialize_trie_backend<H: Hasher, S>(
    trie: &MemoryTrieBackend<H>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    H::Out: Codec + Serialize,
    S: Serializer,
{
    let root = trie.root();
    let kvs: Vec<_> = trie
        .backend_storage()
        .clone()
        .drain()
        .into_iter()
        .map(|it| it.1)
        .collect();
    (root, kvs).serialize(serializer)
}

#[cfg(feature = "serde")]
pub fn deserialize_trie_backend<'de, H: Hasher, De>(
    deserializer: De,
) -> Result<MemoryTrieBackend<H>, De::Error>
where
    H::Out: Codec + Deserialize<'de>,
    De: Deserializer<'de>,
{
    let (root, kvs): (H::Out, Vec<(Vec<u8>, i32)>) = Deserialize::deserialize(deserializer)?;
    let mut mdb = MemoryDB::default();
    for value in kvs {
        for _ in 0..value.1 {
            mdb.insert((&[], None), &value.0);
        }
    }
    let backend = MemoryTrieBackend::<H>::new(mdb, root);
    Ok(backend)
}

// Transactional Database trait
pub trait TransactionalDB<H: Hasher> {
    /// calculate if the deltas changes the merkle root
    fn calc_root_if_changes<'a>(
        &self,
        delta: &'a StorageCollection,
        child_deltas: &'a ChildStorageCollection,
    ) -> (H::Out, Transaction<H>);

    /// apply changes to underlying storage
    fn apply_changes(&mut self, root: H::Out, transaction: Transaction<H>);

    /// return current merkle root
    fn root(&self) -> &H::Out;

    /// get value from the db over the key
    fn get(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>>;
}

// TrieStorage with pkvdb on gramine protected filesystem
pub struct PkvdbTrieStorage<H: Hasher>(PkvdbTrieBackend<H>);

impl<H: Hasher> PkvdbTrieStorage<H>
where
    H::Out: Codec,
{
    pub fn with_trie_backend(backend: PkvdbTrieBackend<H>) -> Self {
        Self(backend)
    }

    pub fn new(path: impl AsRef<Path>) -> Self {
        Self::with_trie_backend(PkvdbTrieBackend::<H>::new(
            LevelDB::new(path),
            Default::default(),
        ))
    }

    pub fn flush(&self) {
        self.0.backend_storage().flush();
    }
}

impl<H: Hasher> TransactionalDB<H> for PkvdbTrieStorage<H>
where
    H::Out: Codec + Ord,
{
    fn calc_root_if_changes<'a>(
        &self,
        delta: &'a StorageCollection,
        child_deltas: &'a ChildStorageCollection,
    ) -> (H::Out, Transaction<H>) {
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

    fn apply_changes(&mut self, _root: H::Out, transaction: Transaction<H>) {
        self.0.backend_storage().consolidate(transaction)
    }

    fn root(&self) -> &H::Out {
        self.0.root()
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        self.0.storage(key.as_ref()).ok().flatten()
    }
}

// In memory TrieStorage with MemoryDB
pub struct MemoryTrieStorage<H: Hasher>(MemoryTrieBackend<H>);

impl<H: Hasher> MemoryTrieStorage<H>
where
    H::Out: Codec + Ord,
{
    // load trie backend from the pairs
    pub fn load_trie_backend(
        pairs: impl Iterator<Item = (impl AsRef<[u8]>, impl AsRef<[u8]>)>,
    ) -> MemoryTrieBackend<H> {
        let mut root = Default::default();
        let mut mdb = Default::default();
        {
            let mut trie_db = TrieDBMut::new(&mut mdb, &mut root);
            for (key, value) in pairs {
                if trie_db.insert(key.as_ref(), value.as_ref()).is_err() {
                    panic!("Insert item into trie DB should not fail");
                }
            }
        }
        MemoryTrieBackend::<H>::new(mdb, root)
    }

    pub fn clone_trie_backend(trie: &MemoryTrieBackend<H>) -> MemoryTrieBackend<H>
    where
        H::Out: Codec,
    {
        let root = trie.root();
        let kvs: Vec<_> = trie
            .backend_storage()
            .clone()
            .drain()
            .into_iter()
            .map(|it| it.1)
            .collect();
        let mut mdb = MemoryDB::default();
        for value in kvs {
            for _ in 0..value.1 {
                mdb.insert((&[], None), &value.0);
            }
        }
        MemoryTrieBackend::<H>::new(mdb, *root)
    }
}

impl<H: Hasher> TransactionalDB<H> for MemoryTrieStorage<H>
where
    H::Out: Codec + Ord,
{
    fn calc_root_if_changes<'a>(
        &self,
        delta: &'a StorageCollection,
        child_deltas: &'a ChildStorageCollection,
    ) -> (H::Out, Transaction<H>) {
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

    fn apply_changes(&mut self, root: H::Out, transaction: Transaction<H>) {
        let mut storage = core::mem::take(self).0.into_storage();
        storage.consolidate(transaction);
        storage.purge();
        let _ = core::mem::replace(&mut self.0, MemoryTrieBackend::<H>::new(storage, root));
    }

    fn root(&self) -> &H::Out {
        self.0.root()
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        self.0.storage(key.as_ref()).ok().flatten()
    }
}

impl<H: Hasher> MemoryTrieStorage<H>
where
    H::Out: Codec + Ord,
{
    /// Overwrite all data in the trie DB with given key/value pairs.
    pub fn load(&mut self, pairs: impl Iterator<Item = (impl AsRef<[u8]>, impl AsRef<[u8]>)>) {
        let trie = Self::load_trie_backend(pairs);
        let _ = core::mem::replace(&mut self.0, trie);
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

impl<H: Hasher> Default for MemoryTrieStorage<H>
where
    H::Out: Codec,
{
    fn default() -> Self {
        Self(MemoryTrieBackend::<H>::new(
            Default::default(),
            Default::default(),
        ))
    }
}

#[cfg(feature = "serde")]
const _: () = {
    impl<H: Hasher> Serialize for MemoryTrieStorage<H>
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

    impl<'de, H: Hasher> Deserialize<'de> for MemoryTrieStorage<H>
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
