use serde::{Deserialize, Serialize};
use sp_runtime::traits::BlakeTwo256;
use std::collections::HashMap;
use std::path::PathBuf;
use trie_storage::*;

/// Storage key.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestStorageKey(#[serde(with = "impl_serde::serialize")] Vec<u8>);

/// Storage value.
type TestStorageValue = TestStorageKey;

/// In memory array of storage values.
type TestStorageCollection = Vec<(TestStorageKey, Option<TestStorageValue>)>;

/// In memory arrays of storage values for multiple child tries.
type TestChildStorageCollection = Vec<(TestStorageKey, TestStorageCollection)>;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RpcResponse {
    result: Vec<Changes>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
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
    let mut changes = response.result;
    changes.reverse();
    changes
}

fn load_genesis_trie() -> TrieStorage<BlakeTwo256> {
    let mut trie: TrieStorage<BlakeTwo256> = Default::default();

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

    for (number, change) in changes.into_iter().take(30).enumerate() {
        let main_storage_changes = map_storage_collection(change.main_storage_changes);
        let child_storage_changes: Vec<_> = change
            .child_storage_changes
            .into_iter()
            .map(|(k, v)| (k.0, map_storage_collection(v)))
            .collect();

        trie.apply_changes(main_storage_changes, child_storage_changes);
        assert_eq!(format!("{:?}", trie.root()), roots[number + 1]);
    }
}
