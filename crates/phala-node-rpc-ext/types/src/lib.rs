use scale::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// Storage key.
#[derive(Serialize, Deserialize, Clone, Debug, Encode, Decode, TypeInfo)]
pub struct StorageKey(#[serde(with = "impl_serde::serialize")] pub Vec<u8>);

/// Storage value.
pub type StorageValue = StorageKey;

/// In memory array of storage values.
pub type StorageCollection<K, V> = Vec<(K, Option<V>)>;

/// In memory arrays of storage values for multiple child tries.
pub type ChildStorageCollection<K, V> = Vec<(K, StorageCollection<K, V>)>;

#[derive(Serialize, Deserialize, Clone, Debug, Encode, Decode, TypeInfo)]
#[serde(rename_all = "camelCase")]
pub struct StorageChanges {
    /// A value of `None` means that it was deleted.
    pub main_storage_changes: StorageCollection<StorageKey, StorageValue>,
    /// All changes to the child storages.
    pub child_storage_changes: ChildStorageCollection<StorageKey, StorageValue>,
}

/// Response for the `pha_getStorageChanges` RPC.
pub type GetStorageChangesResponse = Vec<StorageChanges>;

#[derive(Serialize, Deserialize, Clone, Debug, Encode, Decode, TypeInfo)]
#[serde(rename_all = "camelCase")]
pub struct StorageChangesWithRoot {
    pub changes: StorageChanges,
    pub state_root: Vec<u8>,
}

/// Response for the `pha_getStorageChangesWithRoot` RPC.
pub type GetStorageChangesResponseWithRoot = Vec<StorageChangesWithRoot>;

// Stuffs to convert ChildStorageCollection and StorageCollection types,
// in order to dump the keys values into hex strings instead of list of dec numbers.
pub trait MakeInto<T>: Sized {
    fn into_(self) -> T;
}

impl MakeInto<StorageKey> for Vec<u8> {
    fn into_(self) -> StorageKey {
        StorageKey(self)
    }
}

impl MakeInto<Vec<u8>> for StorageKey {
    fn into_(self) -> Vec<u8> {
        self.0
    }
}

impl<F: MakeInto<T>, T> MakeInto<Option<T>> for Option<F> {
    fn into_(self) -> Option<T> {
        self.map(|v| v.into_())
    }
}

impl<T1, T2, F1, F2> MakeInto<(T1, T2)> for (F1, F2)
where
    F1: MakeInto<T1>,
    F2: MakeInto<T2>,
{
    fn into_(self) -> (T1, T2) {
        (self.0.into_(), self.1.into_())
    }
}

impl<F: MakeInto<T>, T> MakeInto<Vec<T>> for Vec<F> {
    fn into_(self) -> Vec<T> {
        self.into_iter().map(|v| v.into_()).collect()
    }
}
