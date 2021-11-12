use crate::{
    types::{utils::raw_proof, Hash, ParachainApi, RelaychainApi, StorageKey},
    Error,
};
use anyhow::{anyhow, Result};
use codec::Decode;
use codec::Encode;
use phactory_api::blocks::StorageProof;
use phala_node_rpc_ext::MakeInto as _;
use phala_trie_storage::ser::StorageChanges;
use phala_types::messaging::MessageOrigin;
use phaxt::{rpc::ExtraRpcExt as _, subxt};
use serde_json::to_value;
use subxt::Signer;

pub use sp_core::{twox_128, twox_64};

use crate::types::SrSigner;

/// Gets a storage proof for a single storage item
pub async fn read_proof(
    api: &RelaychainApi,
    hash: Option<Hash>,
    storage_key: StorageKey,
) -> Result<StorageProof> {
    api.client
        .rpc()
        .read_proof(vec![storage_key], hash)
        .await
        .map(raw_proof)
        .map_err(Into::into)
}

/// Gets a storage proof for a storage items
pub async fn read_proofs(
    api: &RelaychainApi,
    hash: Option<Hash>,
    storage_keys: Vec<StorageKey>,
) -> Result<StorageProof> {
    api.client
        .rpc()
        .read_proof(storage_keys, hash)
        .await
        .map(raw_proof)
        .map_err(Into::into)
}

// Storage functions

/// Fetch storage changes made by given block.
pub async fn fetch_storage_changes(api: &ParachainApi, hash: &Hash) -> Result<StorageChanges> {
    let response = api
        .client
        .extra_rpc()
        .get_storage_changes(hash, hash)
        .await?;
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
pub async fn fetch_genesis_storage(api: &ParachainApi) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let hash = Some(*api.client.genesis());
    let response = api
        .client
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
        .client
        .rpc()
        .client
        .request("pha_getMqNextSequence", &[to_value(sender_hex)?])
        .await?;
    Ok(seq)
}

pub fn paras_heads_key(para_id: u32) -> StorageKey {
    let id = phaxt::kusama::runtime_types::polkadot_parachain::primitives::Id(para_id);
    let entry = phaxt::kusama::paras::storage::Heads(id);
    phaxt::storage_key(entry)
}

pub fn decode_parachain_heads(head: Vec<u8>) -> Result<Vec<u8>, Error> {
    Decode::decode(&mut head.as_slice()).or(Err(Error::FailedToDecode))
}

/// Updates the nonce from the mempool
pub async fn update_signer_nonce(api: &ParachainApi, signer: &mut SrSigner) -> Result<()> {
    let account_id = signer.account_id().clone();
    let nonce = api
        .storage()
        .system()
        .account(account_id.clone(), None)
        .await?
        .nonce;
    signer.set_nonce(nonce);
    log::info!("Fetch account {} nonce={}", account_id, nonce);
    Ok(())
}
