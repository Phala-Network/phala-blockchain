use serde::{Serialize, Deserialize};
use alloc::vec::Vec;

use super::{StorageCollection, ChildStorageCollection};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StorageChanges {
    pub main_storage_changes: StorageCollection,
    pub child_storage_changes: ChildStorageCollection,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageData {
    pub inner: Vec<(Vec<u8>, Vec<u8>)>,
}
