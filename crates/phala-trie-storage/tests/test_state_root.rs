use phala_trie_storage::*;
use serde::{Deserialize, Serialize};
use sp_core::Hasher;
use sp_runtime::{traits::Hash, StateVersion};
use sp_trie::LayoutV0 as Layout;
use sp_trie::TrieConfiguration as _;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct NativeBlakeTwo256;

impl Hasher for NativeBlakeTwo256 {
    type Out = sp_core::H256;
    type StdHasher = hash256_std_hasher::Hash256StdHasher;
    const LENGTH: usize = 32;

    fn hash(s: &[u8]) -> Self::Out {
        sp_core::hashing::blake2_256(s).into()
    }
}

impl Hash for NativeBlakeTwo256 {
    type Output = sp_core::H256;

    fn trie_root(input: Vec<(Vec<u8>, Vec<u8>)>, _: StateVersion) -> Self::Output {
        Layout::<Self>::trie_root(input)
    }

    fn ordered_trie_root(input: Vec<Vec<u8>>, _: StateVersion) -> Self::Output {
        Layout::<Self>::ordered_trie_root(input)
    }
}

/// Storage key.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "serde")]
pub struct TestStorageKey(#[serde(with = "impl_serde::serialize")] Vec<u8>);

/// Storage value.
type TestStorageValue = TestStorageKey;

/// In memory array of storage values.
type TestStorageCollection = Vec<(TestStorageKey, Option<TestStorageValue>)>;

/// In memory arrays of storage values for multiple child tries.
type TestChildStorageCollection = Vec<(TestStorageKey, TestStorageCollection)>;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "serde")]
struct RpcResponse {
    result: Vec<Changes>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", crate = "serde")]
struct Changes {
    main_storage_changes: TestStorageCollection,
    child_storage_changes: TestChildStorageCollection,
}

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}

fn load_roots() -> Vec<String> {
    let json_str = std::fs::read_to_string(data_dir().join("state_roots.txt")).unwrap();
    json_str.split_whitespace().map(|v| v.into()).collect()
}

fn load_changes() -> Vec<Changes> {
    let json_str = std::fs::read_to_string(data_dir().join("changes.json")).unwrap();
    let response: RpcResponse = serde_json::from_str(json_str.as_str()).unwrap();
    let changes = response.result;
    changes
}

fn load_genesis_trie() -> TrieStorage<NativeBlakeTwo256> {
    let mut trie: TrieStorage<NativeBlakeTwo256> = Default::default();

    let json_str = std::fs::read_to_string(data_dir().join("db-0.json")).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let kvs: HashMap<String, String> =
        serde_json::from_value(json_value["genesis"]["raw"]["top"].clone()).unwrap();
    let decoded = kvs
        .iter()
        .map(|(k, v)| (hex::decode(&k[2..]).unwrap(), hex::decode(&v[2..]).unwrap()));
    trie.load(decoded);
    trie
}

fn map_storage_collection(collection: TestStorageCollection) -> StorageCollection {
    collection
        .into_iter()
        .map(|(k, v)| (k.0, v.map(|v| v.0)))
        .collect()
}

#[test]
fn test_genesis_root() {
    let trie = load_genesis_trie();
    let roots = load_roots();
    assert_eq!(format!("{:?}", trie.root()), roots[0]);
}

#[test]
fn test_apply_main_changes() {
    let mut trie = load_genesis_trie();
    let changes = load_changes();
    let roots = load_roots();

    for (number, change) in changes.into_iter().skip(1).take(30).enumerate() {
        let main_storage_changes = map_storage_collection(change.main_storage_changes);
        let child_storage_changes: Vec<_> = change
            .child_storage_changes
            .into_iter()
            .map(|(k, v)| (k.0, map_storage_collection(v)))
            .collect();

        let (root, trans) =
            trie.calc_root_if_changes(&main_storage_changes, &child_storage_changes);
        trie.apply_changes(root, trans);
        assert_eq!(format!("{:?}", trie.root()), roots[number + 1]);
    }
}
