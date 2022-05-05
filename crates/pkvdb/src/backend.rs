use crate::snapshot::TrieSnapshot as PhalaTrieStorageSnapshot;
use crate::trie::TrieEssenceStorage as PhalaTrieStorageEssence;
use crate::{ChildStorageCollection, StorageCollection};
use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher, PlainDB, PlainDBRef, Prefix};
use parity_scale_codec::Codec;
use sp_state_machine::Backend;
use sp_state_machine::TrieBackend;
use sp_state_machine::TrieBackendStorage;
use sp_std::borrow::Borrow;
use sp_std::convert::AsRef;
use sp_storage::ChildInfo;
use sp_trie::empty_trie_root;
use sp_trie::DBValue;
use sp_trie::LayoutV1;
use sp_trie::MemoryDB;
use std::sync::Arc;

type Result<T> = sp_std::result::Result<T, sp_state_machine::DefaultError>;

// underlying trie storage abstraction used to combined with the sp_state_machine::TrieBackend
pub enum PhalaTrieStorage<H: Hasher> {
    Essence(PhalaTrieStorageEssence<H>),
    Snapshot(PhalaTrieStorageSnapshot<H>),
    Empty,
}

static NULL_NODE_DATA: DBValue = DBValue::new();
#[inline]
fn get_null_hashed_node<H: Hasher>() -> H::Out {
    H::hash(&NULL_NODE_DATA)
}

impl<H: Hasher> Default for PhalaTrieStorage<H> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<H: Hasher> PhalaTrieStorage<H> {
    fn as_dyn_hashdb(&self) -> &dyn HashDB<H, DBValue> {
        match self {
            Self::Essence(db) => db.as_hash_db(),
            Self::Snapshot(snapshot) => snapshot.as_hash_db(),
            _ => unimplemented!(),
        }
    }

    fn as_dyn_hashdb_mut(&mut self) -> &mut dyn HashDB<H, DBValue> {
        match self {
            Self::Essence(db) => db.as_hash_db_mut(),
            Self::Snapshot(snapshot) => snapshot.as_hash_db_mut(),
            _ => unimplemented!(),
        }
    }
}

impl<H: Hasher> PhalaTrieStorage<H>
where
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
    H::Out: AsRef<[u8]>,
{
    fn as_dyn_plaindb(&self) -> &dyn PlainDB<H::Out, DBValue> {
        match self {
            Self::Essence(db) => db.as_plain_db(),
            Self::Snapshot(snapshot) => snapshot.as_plain_db(),
            _ => unimplemented!(),
        }
    }

    fn as_dyn_plaindb_mut(&mut self) -> &mut dyn PlainDB<H::Out, DBValue> {
        match self {
            Self::Essence(db) => db.as_plain_db_mut(),
            Self::Snapshot(snapshot) => snapshot.as_plain_db_mut(),
            _ => unimplemented!(),
        }
    }
}

impl<H: Hasher> PlainDB<H::Out, DBValue> for PhalaTrieStorage<H>
where
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        PlainDB::get(self.as_dyn_plaindb(), key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self.as_dyn_plaindb(), key)
    }

    fn emplace(&mut self, key: H::Out, value: DBValue) {
        PlainDB::emplace(self.as_dyn_plaindb_mut(), key, value)
    }

    fn remove(&mut self, key: &H::Out) {
        PlainDB::remove(self.as_dyn_plaindb_mut(), key)
    }
}

impl<H> PlainDBRef<H::Out, DBValue> for PhalaTrieStorage<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        PlainDB::get(self.as_dyn_plaindb(), key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self.as_dyn_plaindb(), key)
    }
}

impl<H> AsPlainDB<H::Out, DBValue> for PhalaTrieStorage<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn as_plain_db(&self) -> &dyn PlainDB<H::Out, DBValue> {
        self
    }

    fn as_plain_db_mut(&mut self) -> &mut dyn PlainDB<H::Out, DBValue> {
        self
    }
}

impl<H: Hasher> HashDB<H, DBValue> for PhalaTrieStorage<H> {
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        if *key == get_null_hashed_node::<H>() {
            return Some(NULL_NODE_DATA.clone());
        }
        HashDB::get(self.as_dyn_hashdb(), key, prefix)
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        if *key == get_null_hashed_node::<H>() {
            return true;
        }
        HashDB::contains(self.as_dyn_hashdb(), key, prefix)
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H::Out {
        HashDB::insert(self.as_dyn_hashdb_mut(), prefix, value)
    }

    fn emplace(&mut self, key: H::Out, prefix: Prefix, value: DBValue) {
        HashDB::emplace(self.as_dyn_hashdb_mut(), key, prefix, value)
    }

    fn remove(&mut self, _key: &H::Out, _prefix: Prefix) {
        unimplemented!("HashDB remove is not supported on Snapshot")
    }
}

impl<H: Hasher> HashDBRef<H, DBValue> for PhalaTrieStorage<H> {
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        HashDB::get(self.as_hash_db(), key, prefix)
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        HashDB::contains(self.as_hash_db(), key, prefix)
    }
}

impl<H: Hasher> AsHashDB<H, DBValue> for PhalaTrieStorage<H> {
    fn as_hash_db(&self) -> &dyn HashDB<H, DBValue> {
        self
    }

    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<H, DBValue> {
        self
    }
}

// impl the trait for being used in TrieBackend
impl<H: Hasher> TrieBackendStorage<H> for PhalaTrieStorage<H> {
    type Overlay = MemoryDB<H>;

    fn get(&self, key: &H::Out, prefix: Prefix) -> Result<Option<DBValue>> {
        Ok(HashDB::get(self.as_dyn_hashdb(), key, prefix))
    }
}

// Phala defined TrieBackend used in anywhere need TrieBackend abstraction
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

impl<H: Hasher> PhalaTrieBackend<H>
where
    H::Out: Codec + Ord,
{
    // calc if current changes make the root change?
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
}
