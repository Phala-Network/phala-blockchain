use crate::{
    types::{utils::raw_proof, Hash, ParachainApi, RelaychainApi, StorageKey},
    Error,
};
use anyhow::{Context, Result};
use codec::Decode;
use codec::Encode;
use phactory_api::blocks::StorageProof;
use phala_node_rpc_ext::MakeInto as _;
use phala_trie_storage::ser::StorageChanges;
use phala_types::messaging::MessageOrigin;
use phaxt::{rpc::ExtraRpcExt as _, subxt, BlockNumber, RpcClient};
use serde_json::to_value;
use subxt::rpc::rpc_params;

pub use sp_core::{twox_128, twox_64};

use crate::types::SrSigner;

/// Gets a storage proof for a single storage item
pub async fn read_proof(
    api: &RelaychainApi,
    hash: Option<Hash>,
    storage_key: &[u8],
) -> Result<StorageProof> {
    api.rpc()
        .read_proof(vec![storage_key], hash)
        .await
        .map(raw_proof)
        .map_err(Into::into)
}

/// Gets a storage proof for a storage items
pub async fn read_proofs(
    api: &RelaychainApi,
    hash: Option<Hash>,
    storage_keys: impl IntoIterator<Item = &[u8]>,
) -> Result<StorageProof> {
    let mut keys = vec![];
    // Retrieve the actual storage keys in case they are prefixed
    for prefix in storage_keys {
        let full_keys = api.storage_keys(prefix, hash).await?;
        keys.extend(full_keys);
    }
    api.rpc()
        .read_proof(keys.iter().map(|k| &k[..]), hash)
        .await
        .map(raw_proof)
        .map_err(Into::into)
}

// Storage functions

/// Fetch storage changes made by given block.
pub async fn fetch_storage_changes(
    client: &RpcClient,
    from: &Hash,
    to: &Hash,
) -> Result<Vec<StorageChanges>> {
    let response = client
        .extra_rpc()
        .get_storage_changes(from, to)
        .await?
        .into_iter()
        .map(|changes| StorageChanges {
            // TODO.kevin: get rid of this convert
            main_storage_changes: changes.main_storage_changes.into_(),
            child_storage_changes: changes.child_storage_changes.into_(),
        })
        .collect();
    Ok(response)
}

/// Fetch the genesis storage.
pub async fn fetch_genesis_storage(api: &ParachainApi) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let hash = Some(api.genesis_hash());
    fetch_genesis_storage_at(api, hash).await
}

async fn fetch_genesis_storage_at(
    api: &ParachainApi,
    hash: Option<sp_core::H256>,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let response = api
        .extra_rpc()
        .storage_pairs(StorageKey(vec![]), hash)
        .await?;
    let storage = response.into_iter().map(|(k, v)| (k.0, v.0)).collect();
    Ok(storage)
}

/// Fetch best next sequence for given sender considering the txpool
pub async fn mq_next_sequence(
    api: &ParachainApi,
    sender: &MessageOrigin,
) -> Result<u64, subxt::Error> {
    let sender_scl = sender.encode();
    let sender_hex = hex::encode(sender_scl);
    let seq: u64 = api
        .rpc()
        .request("pha_getMqNextSequence", rpc_params![to_value(sender_hex)?])
        .await?;
    Ok(seq)
}

pub fn decode_parachain_heads(head: Vec<u8>) -> Result<Vec<u8>, Error> {
    Decode::decode(&mut head.as_slice()).or(Err(Error::FailedToDecode))
}

/// Updates the nonce from the mempool
pub async fn update_signer_nonce(api: &ParachainApi, signer: &mut SrSigner) -> Result<()> {
    let account_id = signer.account_id().clone();
    let nonce = api.extra_rpc().account_nonce(&account_id).await?;
    signer.set_nonce(nonce);
    log::info!("Fetch account {} nonce={}", account_id, nonce);
    Ok(())
}

pub async fn search_suitable_genesis_for_worker(
    api: &ParachainApi,
    pubkey: &[u8],
    prefer: Option<BlockNumber>,
) -> Result<(BlockNumber, Vec<(Vec<u8>, Vec<u8>)>)> {
    let ceil = match prefer {
        Some(ceil) => ceil,
        None => api.latest_finalized_block_number().await?,
    };
    let block = get_worker_unregistered_block(api, pubkey, ceil)
        .await
        .context("Failed to search state for worker")?;
    let block_hash = api
        .rpc()
        .block_hash(Some(block.into()))
        .await
        .context("Failed to resolve block number")?
        .ok_or_else(|| anyhow::anyhow!("Block number {block} not found"))?;
    let genesis = fetch_genesis_storage_at(api, Some(block_hash))
        .await
        .context("Failed to fetch genesis storage")?;
    Ok((block, genesis))
}

async fn get_worker_unregistered_block(
    api: &ParachainApi,
    worker: &[u8],
    latest_block: u32,
) -> Result<u32> {
    let added_at = api.worker_added_at(worker).await?;
    log::info!("Worker added at={added_at:?}");
    let block = added_at.unwrap_or(latest_block + 1).saturating_sub(1);
    log::info!("Choosing genesis state at {block} ");
    Ok(block)
}
