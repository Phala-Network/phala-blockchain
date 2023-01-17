use alloc::vec::Vec;
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

pub struct SerAsSeq<K, V>(pub im::HashMap<K, V>);

impl<K, V> Serialize for SerAsSeq<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for (_k, v) in self.0.iter() {
            seq.serialize_element(&v)?;
        }
        seq.end()
    }
}
