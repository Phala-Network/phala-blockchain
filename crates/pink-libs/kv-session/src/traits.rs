use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::Result;

pub struct KvTransaction<K, V> {
    pub accessed_keys: Vec<K>,
    pub version_updates: Vec<K>,
    pub value_updates: Vec<(K, Option<V>)>,
}

pub trait BumpVersion<V> {
    fn bump_version(&self, version: Option<V>) -> Result<V>;
}

pub trait KvSnapshot {
    type Key;
    type Value;

    /// Should be the block hash for a blockchain backend
    fn snapshot_id(&self) -> Result<Self::Value>;

    /// Get a storage value from the snapshot
    fn get(&self, key: &impl ToOwned<Owned = Self::Key>) -> Result<Option<Self::Value>>;

    /// Batch get storage values from the snapshot
    #[allow(clippy::type_complexity)]
    fn batch_get(&self, keys: &[Self::Key]) -> Result<Vec<(Self::Key, Option<Self::Value>)>>
    where
        Self::Key: Clone,
    {
        keys.iter()
            .map(|key| {
                let key = key.clone();
                Ok((key.clone(), self.get(&key)?))
            })
            .collect()
    }
}

pub trait KvSnapshotExt: KvSnapshot {
    fn prefixed(self, prefix: Self::Key) -> PrefixedKvSnapshot<Self::Key, Self>
    where
        Self: Sized;
}

impl<K, V, T> KvSnapshotExt for T
where
    K: Concat + Clone,
    T: KvSnapshot<Key = K, Value = V>,
{
    fn prefixed(self, prefix: Self::Key) -> PrefixedKvSnapshot<K, Self> {
        PrefixedKvSnapshot {
            inner: self,
            prefix,
        }
    }
}

#[derive(Clone)]
pub struct PrefixedKvSnapshot<K, DB> {
    inner: DB,
    prefix: K,
}

impl<K, V, DB> KvSnapshot for PrefixedKvSnapshot<K, DB>
where
    K: Concat + Clone,
    DB: KvSnapshot<Key = K, Value = V>,
{
    type Key = K;

    type Value = V;

    fn get(&self, key: &impl ToOwned<Owned = Self::Key>) -> Result<Option<Self::Value>> {
        let key = self.prefix.clone().concat(key.to_owned());
        self.inner.get(&key)
    }

    fn snapshot_id(&self) -> Result<Self::Value> {
        self.inner.snapshot_id()
    }
}

pub trait KvSession {
    type Key;
    type Value;
    fn get(
        &mut self,
        key: &(impl ToOwned<Owned = Self::Key> + ?Sized),
    ) -> Result<Option<Self::Value>>;
    fn put(&mut self, key: &(impl ToOwned<Owned = Self::Key> + ?Sized), value: Self::Value);
    fn delete(&mut self, key: &(impl ToOwned<Owned = Self::Key> + ?Sized));
}

pub trait AccessTracking {
    type Key;
    fn read(&mut self, key: &Self::Key);
    fn write(&mut self, key: &Self::Key);
    /// Returns (access list, version updates)
    fn collect_into(self) -> (Vec<Self::Key>, Vec<Self::Key>);
}

pub trait Concat {
    fn concat(self, other: Self) -> Self;
}

impl Concat for String {
    fn concat(self, other: Self) -> Self {
        self + &other
    }
}

impl Concat for Vec<u8> {
    fn concat(mut self, other: Self) -> Self {
        self.extend_from_slice(&other);
        self
    }
}
