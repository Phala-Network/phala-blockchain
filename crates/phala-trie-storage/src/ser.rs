use hash_db::Hasher;
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

pub struct SerAsSeq<'a, H: Hasher>(pub &'a crate::MemoryDB<H>)
where
    H::Out: Ord;

impl<H: Hasher> Serialize for SerAsSeq<'_, H>
where
    H::Out: Ord,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
