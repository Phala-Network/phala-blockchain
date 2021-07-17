use anyhow::{anyhow, Result};
use codec::Encode;
use sp_core::{storage::StorageKey, twox_128, twox_64};
use phala_types::{messaging::MessageOrigin};
use enclave_api::blocks::{StorageProof, ParaId};
use super::runtimes;

use super::XtClient;
use crate::{Error, types::{Hash, utils::raw_proof}};
use trie_storage::ser::StorageChanges;
use rpc_ext::MakeInto as _;
use codec::Decode;

/// Gets a single storage item
pub  async fn get_storage(
    client: &XtClient, hash: Option<Hash>, storage_key: StorageKey
) -> Result<Option<Vec<u8>>>
{
    let storage = client.rpc.storage(&storage_key, hash).await?;
    Ok(storage.map(|data| (&data.0[..]).to_vec()))
}

/// Gets a storage proof for a single storage item
pub async fn read_proof(client: &XtClient, hash: Option<Hash>, storage_key: StorageKey)
-> Result<StorageProof>
{
    client.read_proof(vec![storage_key], hash).await
        .map(raw_proof)
        .map_err(Into::into)
}

// Storage functions

/// Fetch storage changes made by given block.
pub async fn fetch_storage_changes(client: &XtClient, hash: &Hash) -> Result<StorageChanges> {
    let response = client.rpc.get_storage_changes(hash, hash).await?;
    let first = response
        .into_iter()
        .next()
        .ok_or(anyhow!(crate::error::Error::BlockNotFound))?;
    Ok(StorageChanges {
        main_storage_changes: first.main_storage_changes.into_(),
        child_storage_changes: first.child_storage_changes.into_(),
    })
}

/// Fetch the genesis storage.
pub async fn fetch_genesis_storage(client: &XtClient) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let hash = Some(*client.genesis());
    let response = client.rpc.storage_pairs(StorageKey(vec![]), hash).await?;
    let storage = response.into_iter().map(|(k, v)| (k.0, v.0)).collect();
    Ok(storage)
}

/// Fetch latest sequences for given sender
pub async fn fetch_mq_ingress_seq(client: &XtClient, sender: MessageOrigin) -> Result<u64> {
    client
        .fetch_or_default(&runtimes::phala_mq::OffchainIngressStore::new(sender), None)
        .await
        .or(Ok(0))
}

pub fn get_para_head_key(para_id: &ParaId) -> StorageKey {
    StorageKey(storage_map_key_vec("Paras", "Heads", &para_id.encode()))
}

pub fn get_parachain_heads(
    head: Vec<u8>,
) -> Result<Vec<u8>, Error> {
    Decode::decode(&mut head.as_slice()).or(Err(Error::FailedToDecode))
}

// Utility functions

/// Calculates the Substrate storage key prefix
pub fn storage_value_key_vec(module: &str, storage_key_name: &str) -> Vec<u8> {
    let mut key = twox_128(module.as_bytes()).to_vec();
    key.extend(&twox_128(storage_key_name.as_bytes()));
    key
}

/// Calculates the Substrate storage key prefix for a StorageMap
fn storage_map_key_vec(module: &str, storage_item: &str, item_key: &[u8]) -> Vec<u8> {
    let mut key = storage_value_key_vec(module, storage_item);
    let hash = twox_64(&item_key);
    key.extend(&hash);
    key.extend(item_key);
    key
}
