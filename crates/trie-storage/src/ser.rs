use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use super::{ChildStorageCollection, StorageCollection};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", crate = "serde")]
pub struct StorageChanges {
    pub main_storage_changes: StorageCollection,
    pub child_storage_changes: ChildStorageCollection,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "serde")]
pub struct StorageData {
    pub inner: Vec<(Vec<u8>, Vec<u8>)>,
}
