use sp_std::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

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
