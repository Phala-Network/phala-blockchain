// Copyright 2017-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Reference-counted memory-based `HashDB` implementation.

use hash_db::{
    AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher as KeyHasher, PlainDB, PlainDBRef, Prefix,
};
pub(crate) use im::ordmap::{Entry, OrdMap as Map};
use std::{borrow::Borrow, cmp::Eq, convert::TryFrom, hash, marker::PhantomData};

use sp_state_machine::{DefaultError, TrieBackendStorage};
use trie_db::DBValue;

pub trait MaybeDebug: std::fmt::Debug {}
impl<T: std::fmt::Debug> MaybeDebug for T {}

pub type GenericMemoryDB<H> = MemoryDB<H, HashKey<H>, trie_db::DBValue>;

impl<H: KeyHasher> TrieBackendStorage<H> for GenericMemoryDB<H>
where
    H::Out: Ord,
{
    fn get(
        &self,
        key: &<H as KeyHasher>::Out,
        prefix: Prefix,
    ) -> Result<Option<DBValue>, DefaultError> {
        Ok(hash_db::HashDB::get(self, key, prefix))
    }
}

/// Reference-counted memory-based `HashDB` implementation.
///
/// Use `new()` to create a new database. Insert items with `insert()`, remove items
/// with `remove()`, check for existence with `contains()` and lookup a hash to derive
/// the data with `get()`. Clear with `clear()` and purge the portions of the data
/// that have no references with `purge()`.
///
pub struct MemoryDB<H, KF, T>
where
    H: KeyHasher,
    KF: KeyFunction<H>,
{
    data: Map<KF::Key, (T, i32)>,
    hashed_null_node: H::Out,
    null_node_data: T,
    _kf: PhantomData<KF>,
}

impl<H, KF, T> Clone for MemoryDB<H, KF, T>
where
    H: KeyHasher,
    KF: KeyFunction<H>,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            hashed_null_node: self.hashed_null_node,
            null_node_data: self.null_node_data.clone(),
            _kf: Default::default(),
        }
    }
}

pub trait KeyFunction<H: KeyHasher> {
    type Key: Send + Sync + Clone + hash::Hash + Eq;

    fn key(hash: &H::Out, prefix: Prefix) -> Self::Key;
}

/// Key function that only uses the hash
pub struct HashKey<H>(PhantomData<H>);

impl<H> Clone for HashKey<H> {
    fn clone(&self) -> Self {
        Self(Default::default())
    }
}

impl<H> core::fmt::Debug for HashKey<H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "HashKey")
    }
}

impl<H: KeyHasher> KeyFunction<H> for HashKey<H> {
    type Key = H::Out;

    fn key(hash: &H::Out, prefix: Prefix) -> H::Out {
        hash_key::<H>(hash, prefix)
    }
}

/// Make database key from hash only.
pub fn hash_key<H: KeyHasher>(key: &H::Out, _prefix: Prefix) -> H::Out {
    *key
}

/// Key function that concatenates prefix and hash.
pub struct PrefixedKey<H>(PhantomData<H>);

impl<H> Clone for PrefixedKey<H> {
    fn clone(&self) -> Self {
        Self(Default::default())
    }
}

impl<H> core::fmt::Debug for PrefixedKey<H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "PrefixedKey")
    }
}

impl<H: KeyHasher> KeyFunction<H> for PrefixedKey<H> {
    type Key = Vec<u8>;

    fn key(hash: &H::Out, prefix: Prefix) -> Vec<u8> {
        prefixed_key::<H>(hash, prefix)
    }
}

/// Derive a database key from hash value of the node (key) and  the node prefix.
pub fn prefixed_key<H: KeyHasher>(key: &H::Out, prefix: Prefix) -> Vec<u8> {
    let mut prefixed_key = Vec::with_capacity(key.as_ref().len() + prefix.0.len() + 1);
    prefixed_key.extend_from_slice(prefix.0);
    if let Some(last) = prefix.1 {
        prefixed_key.push(last);
    }
    prefixed_key.extend_from_slice(key.as_ref());
    prefixed_key
}

impl<H, KF, T> Default for MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: for<'a> From<&'a [u8]> + Clone,
    KF: KeyFunction<H>,
    KF::Key: Ord,
{
    fn default() -> Self {
        Self::from_null_node(&[0u8][..], [0u8][..].into())
    }
}

/// Create a new `MemoryDB` from a given null key/data
impl<H, KF, T> MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: Default + Clone,
    KF: KeyFunction<H>,
    KF::Key: Ord,
{
    /// Remove an element and delete it from storage if reference count reaches zero.
    /// If the value was purged, return the old value.
    pub fn remove_and_purge(&mut self, key: &<H as KeyHasher>::Out, prefix: Prefix) -> Option<T> {
        if key == &self.hashed_null_node {
            return None;
        }
        let key = KF::key(key, prefix);
        match self.data.entry(key) {
            Entry::Occupied(mut entry) => {
                if entry.get().1 == 1 {
                    let (value, _) = entry.remove();
                    Some(value)
                } else {
                    entry.get_mut().1 -= 1;
                    None
                }
            }
            Entry::Vacant(entry) => {
                let value = T::default();
                entry.insert((value, -1));
                None
            }
        }
    }

    // /// Shrinks the capacity of the map as much as possible. It will drop
    // /// down as much as possible while maintaining the internal rules
    // /// and possibly leaving some space in accordance with the resize policy.
    // #[inline]
    // pub fn shrink_to_fit(&mut self) {
    //     self.data.shrink_to_fit();
    // }
}

impl<H, KF, T> MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: for<'a> From<&'a [u8]> + Clone,
    KF: KeyFunction<H>,
    KF::Key: Ord,
{
    /// Create a new `MemoryDB` from a given null key/data
    pub fn from_null_node(null_key: &[u8], null_node_data: T) -> Self {
        MemoryDB {
            data: Map::default(),
            hashed_null_node: H::hash(null_key),
            null_node_data,
            _kf: Default::default(),
        }
    }

    /// Create a new `MemoryDB` from a given inner hash map.
    pub fn from_inner(data: Map<KF::Key, (T, i32)>) -> Self {
        MemoryDB {
            data,
            ..Default::default()
        }
    }

    /// Create a new instance of `Self`.
    pub fn new(data: &[u8]) -> Self {
        Self::from_null_node(data, data.into())
    }

    /// Create a new default instance of `Self` and returns `Self` and the root hash.
    pub fn default_with_root() -> (Self, H::Out) {
        let db = Self::default();
        let root = db.hashed_null_node;

        (db, root)
    }

    /// Clear all data from the database.
    ///
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Return the internal key-value Map, clearing the current state.
    pub fn drain(&mut self) -> Map<KF::Key, (T, i32)> {
        core::mem::take(&mut self.data)
    }

    /// Grab the raw information associated with a key. Returns None if the key
    /// doesn't exist.
    ///
    /// Even when Some is returned, the data is only guaranteed to be useful
    /// when the refs > 0.
    pub fn raw(&self, key: &<H as KeyHasher>::Out, prefix: Prefix) -> Option<(&T, i32)> {
        if key == &self.hashed_null_node {
            return Some((&self.null_node_data, 1));
        }
        self.data
            .get(&KF::key(key, prefix))
            .map(|(value, count)| (value, *count))
    }

    /// Consolidate all the entries of `other` into `self`.
    pub fn consolidate(&mut self, mut other: crate::BackendTransaction<H>)
    where
        KF::Key: From<[u8; 32]>,
        T: From<Vec<u8>>,
    {
        for (key, (value, rc)) in other.drain() {
            let key: [u8; 32] = TryFrom::try_from(&key[key.len() - 32..]).expect("Should never fail");
            let key = key.into();
            match self.data.entry(key) {
                Entry::Occupied(mut entry) => {
                    if entry.get().1 < 0 {
                        entry.get_mut().0 = value.into();
                    }

                    entry.get_mut().1 += rc;
                    if entry.get().1 == 0 {
                        _ = entry.remove();
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert((value.into(), rc));
                }
            }
        }
    }

    /// Get the keys in the database together with number of underlying references.
    pub fn keys(&self) -> Map<KF::Key, i32> {
        self.data
            .iter()
            .filter_map(|(k, v)| {
                if v.1 != 0 {
                    Some((k.clone(), v.1))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl<H, KF, T> MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: for<'a> From<&'a [u8]> + Clone,
    KF: KeyFunction<H>,
    KF::Key: Ord,
    T: serde::Serialize,
{
    pub fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.data.len()))?;
        for (_k, v) in self.data.iter() {
            seq.serialize_element(&v)?;
        }
        seq.end()
    }
}

impl<H, KF, T> PlainDB<H::Out, T> for MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
    KF: Send + Sync + KeyFunction<H>,
    KF::Key: Borrow<[u8]> + for<'a> From<&'a [u8]>,
    KF::Key: Ord,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        match self.data.get(key.as_ref()) {
            Some(&(ref d, rc)) if rc > 0 => Some(d.clone()),
            _ => None,
        }
    }

    fn contains(&self, key: &H::Out) -> bool {
        matches!(self.data.get(key.as_ref()), Some(&(_, x)) if x > 0)
    }

    fn emplace(&mut self, key: H::Out, value: T) {
        match self.data.entry(key.as_ref().into()) {
            Entry::Occupied(mut entry) => {
                let &mut (ref mut old_value, ref mut rc) = entry.get_mut();
                if *rc <= 0 {
                    *old_value = value;
                }
                *rc += 1;
                if *rc == 0 {
                    _ = entry.remove();
                }
            }
            Entry::Vacant(entry) => {
                entry.insert((value, 1));
            }
        }
    }

    fn remove(&mut self, key: &H::Out) {
        match self.data.entry(key.as_ref().into()) {
            Entry::Occupied(mut entry) => {
                let &mut (_, ref mut rc) = entry.get_mut();
                *rc -= 1;
                if *rc == 0 {
                    _ = entry.remove();
                }
            }
            Entry::Vacant(entry) => {
                let value = T::default();
                entry.insert((value, -1));
            }
        }
    }
}

impl<H, KF, T> PlainDBRef<H::Out, T> for MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
    KF: Send + Sync + KeyFunction<H>,
    KF::Key: Borrow<[u8]> + for<'a> From<&'a [u8]>,
    KF::Key: Ord,
{
    fn get(&self, key: &H::Out) -> Option<T> {
        PlainDB::get(self, key)
    }
    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self, key)
    }
}

impl<H, KF, T> HashDB<H, T> for MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
    KF: KeyFunction<H> + Send + Sync,
    KF::Key: Ord,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<T> {
        if key == &self.hashed_null_node {
            return Some(self.null_node_data.clone());
        }

        let key = KF::key(key, prefix);
        match self.data.get(&key) {
            Some(&(ref d, rc)) if rc > 0 => Some(d.clone()),
            _ => None,
        }
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        if key == &self.hashed_null_node {
            return true;
        }

        let key = KF::key(key, prefix);
        matches!(self.data.get(&key), Some(&(_, x)) if x > 0)
    }

    fn emplace(&mut self, key: H::Out, prefix: Prefix, value: T) {
        if value == self.null_node_data {
            return;
        }

        let key = KF::key(&key, prefix);
        match self.data.entry(key) {
            Entry::Occupied(mut entry) => {
                let &mut (ref mut old_value, ref mut rc) = entry.get_mut();
                if *rc <= 0 {
                    *old_value = value;
                }
                *rc += 1;
                if *rc == 0 {
                    _ = entry.remove();
                }
            }
            Entry::Vacant(entry) => {
                entry.insert((value, 1));
            }
        }
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H::Out {
        if T::from(value) == self.null_node_data {
            return self.hashed_null_node;
        }

        let key = H::hash(value);
        HashDB::emplace(self, key, prefix, value.into());
        key
    }

    fn remove(&mut self, key: &H::Out, prefix: Prefix) {
        if key == &self.hashed_null_node {
            return;
        }

        let key = KF::key(key, prefix);
        match self.data.entry(key) {
            Entry::Occupied(mut entry) => {
                let &mut (_, ref mut rc) = entry.get_mut();
                *rc -= 1;
                if *rc == 0 {
                    _ = entry.remove();
                }
            }
            Entry::Vacant(entry) => {
                let value = T::default();
                entry.insert((value, -1));
            }
        }
    }
}

impl<H, KF, T> HashDBRef<H, T> for MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
    KF: KeyFunction<H> + Send + Sync,
    KF::Key: Ord,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<T> {
        HashDB::get(self, key, prefix)
    }
    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        HashDB::contains(self, key, prefix)
    }
}

impl<H, KF, T> AsPlainDB<H::Out, T> for MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
    KF: KeyFunction<H> + Send + Sync,
    KF::Key: Borrow<[u8]> + for<'a> From<&'a [u8]>,
    KF::Key: Ord,
{
    fn as_plain_db(&self) -> &dyn PlainDB<H::Out, T> {
        self
    }
    fn as_plain_db_mut(&mut self) -> &mut dyn PlainDB<H::Out, T> {
        self
    }
}

impl<H, KF, T> AsHashDB<H, T> for MemoryDB<H, KF, T>
where
    H: KeyHasher,
    T: Default + PartialEq<T> + AsRef<[u8]> + for<'a> From<&'a [u8]> + Clone + Send + Sync,
    KF: KeyFunction<H> + Send + Sync,
    KF::Key: Ord,
{
    fn as_hash_db(&self) -> &dyn HashDB<H, T> {
        self
    }
    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<H, T> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{HashDB, HashKey, KeyHasher, MemoryDB};
    use hash_db::EMPTY_PREFIX;
    use keccak_hasher::KeccakHasher;

    #[test]
    fn memorydb_remove_and_purge() {
        let hello_bytes = b"Hello world!";
        let hello_key = KeccakHasher::hash(hello_bytes);

        let mut m = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
        m.remove(&hello_key, EMPTY_PREFIX);
        assert_eq!(m.raw(&hello_key, EMPTY_PREFIX).unwrap().1, -1);
        m.insert(EMPTY_PREFIX, hello_bytes);
        assert_eq!(m.raw(&hello_key, EMPTY_PREFIX), None);

        let mut m = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
        assert!(m.remove_and_purge(&hello_key, EMPTY_PREFIX).is_none());
        assert_eq!(m.raw(&hello_key, EMPTY_PREFIX).unwrap().1, -1);
        m.insert(EMPTY_PREFIX, hello_bytes);
        m.insert(EMPTY_PREFIX, hello_bytes);
        assert_eq!(m.raw(&hello_key, EMPTY_PREFIX).unwrap().1, 1);
        assert_eq!(
            &*m.remove_and_purge(&hello_key, EMPTY_PREFIX).unwrap(),
            hello_bytes
        );
        assert_eq!(m.raw(&hello_key, EMPTY_PREFIX), None);
        assert!(m.remove_and_purge(&hello_key, EMPTY_PREFIX).is_none());
    }

    #[test]
    fn consolidate() {
        let mut main = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
        let mut other = crate::BackendTransaction::<KeccakHasher>::default();
        let remove_key = other.insert(EMPTY_PREFIX, b"doggo");
        main.remove(&remove_key, EMPTY_PREFIX);

        let insert_key = other.insert(EMPTY_PREFIX, b"arf");
        main.emplace(insert_key, EMPTY_PREFIX, "arf".as_bytes().to_vec());

        let negative_remove_key = other.insert(EMPTY_PREFIX, b"negative");
        other.remove(&negative_remove_key, EMPTY_PREFIX); // ref cnt: 0
        other.remove(&negative_remove_key, EMPTY_PREFIX); // ref cnt: -1
        main.remove(&negative_remove_key, EMPTY_PREFIX); // ref cnt: -1

        main.consolidate(other);

        assert_eq!(main.raw(&remove_key, EMPTY_PREFIX), None);
        assert_eq!(
            main.raw(&insert_key, EMPTY_PREFIX).unwrap(),
            (&"arf".as_bytes().to_vec(), 2)
        );
        assert_eq!(
            main.raw(&negative_remove_key, EMPTY_PREFIX).unwrap(),
            (&"negative".as_bytes().to_vec(), -2),
        );
    }

    #[test]
    fn default_works() {
        let mut db = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
        let hashed_null_node = KeccakHasher::hash(&[0u8][..]);
        assert_eq!(db.insert(EMPTY_PREFIX, &[0u8][..]), hashed_null_node);

        let (db2, root) = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default_with_root();
        assert!(db2.contains(&root, EMPTY_PREFIX));
        assert!(db.contains(&root, EMPTY_PREFIX));
    }
}
