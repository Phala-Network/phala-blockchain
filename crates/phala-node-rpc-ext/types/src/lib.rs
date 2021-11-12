use scale::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Serialize, Deserialize};

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

