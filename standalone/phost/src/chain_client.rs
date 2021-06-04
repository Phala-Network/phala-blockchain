use std::collections::HashSet;
use anyhow::{anyhow, Result};
use codec::FullCodec;
use log::{debug, error, info};
use sp_core::{storage::StorageKey, twox_128};
use phala_types::pruntime::{
    StorageKV, RawStorageKey, StorageProof,
    OnlineWorkerSnapshot,
};
use subxt::{EventsDecoder, Store, RawEventWrapper as Raw};

use super::XtClient;
use crate::{
    runtimes,
    types::{
        Runtime, Hash, BlockNumber, AccountId, RawEvents,
        utils::raw_proof,
    }
};
use trie_storage::ser::StorageChanges;
use rpc_ext::MakeInto as _;

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

/// Fetches the raw event at a certain block
pub async fn fetch_events(client: &XtClient, hash: &Hash)
-> Result<Option<(RawEvents, StorageProof, RawStorageKey)>> {
    let key = storage_value_key_vec("System", "Events");
    let storage_key = StorageKey(key.clone());
    let result = match get_storage(&client, Some(hash.clone()), storage_key.clone()).await? {
        Some(value) => {
            let proof = read_proof(&client, Some(hash.clone()), storage_key).await?;
            Some((value, proof, key))
        },
        None => None,
    };
    Ok(result)
}

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

pub async fn fetch_genesis_storage(client: &XtClient) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let hash = Some(*client.genesis());
    let response = client.rpc.storage_pairs(StorageKey(vec![]), hash).await?;
    let storage = response.into_iter().map(|(k, v)| (k.0, v.0)).collect();
    Ok(storage)
}

// Storage functions

/// Fetches all the StorageMap entries from Substrate
async fn fetch_map<F>(
    xt: &XtClient, hash: Option<Hash>
) -> Result<Vec<(StorageKey, <F as Store<Runtime>>::Returns)>>
where
    F: Store<Runtime>,
    <F as Store<Runtime>>::Returns: FullCodec + Clone
{
    let mut data = Vec::<(StorageKey, <F as Store<Runtime>>::Returns)>::new();
    let mut iter = xt.iter::<F>(hash).await
        .expect("failed to iterate");
    while let Some((k, v)) = iter.next().await.expect("kv iteration failed") {
        data.push((k.clone(), v.clone()));
    }
    Ok(data)
}

// Utility functions

/// Calculates the Substrate storage key prefix
pub fn storage_value_key_vec(module: &str, storage_key_name: &str) -> Vec<u8> {
    let mut key = twox_128(module.as_bytes()).to_vec();
    key.extend(&twox_128(storage_key_name.as_bytes()));
    key
}
