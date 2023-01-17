use serde::{Deserialize, Serialize};
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

use super::{ChildStorageCollection, StorageCollection};

#[derive(Serialize, Deserialize, TypeInfo, Encode, Decode, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", crate = "serde")]
pub struct StorageChanges {
    pub main_storage_changes: StorageCollection,
    pub child_storage_changes: ChildStorageCollection,
}

#[derive(Serialize, Deserialize, TypeInfo, Encode, Decode, Clone, Debug)]
#[serde(crate = "serde")]
pub struct StorageData {
    pub inner: Vec<(Vec<u8>, Vec<u8>)>,
}
