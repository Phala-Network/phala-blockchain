use std::collections::HashSet;
use codec::FullCodec;
use sp_core::{storage::StorageKey, twox_128};
use phala_types::pruntime::{
    StorageKV, RawStorageKey, StorageProof,
    OnlineWorkerSnapshot,
};
use subxt::{EventsDecoder, Store, RawEventWrapper as Raw};

use super::XtClient;
use crate::{
    runtimes,
    error::Error,
    types::{
        Runtime, Hash, BlockNumber, AccountId, RawEvents, Balance,
        utils::raw_proof,
    }
};

/// Gets a single storage item
pub  async fn get_storage(
    client: &XtClient, hash: Option<Hash>, storage_key: StorageKey
) -> Result<Option<Vec<u8>>, Error>
{
    let storage = client.rpc.storage(&storage_key, hash).await?;
    Ok(storage.map(|data| (&data.0[..]).to_vec()))
}

/// Gets a storage proof for a single storage item
pub async fn read_proof(client: &XtClient, hash: Option<Hash>, storage_key: StorageKey)
-> Result<StorageProof, Error>
{
    client.read_proof(vec![storage_key], hash).await
        .map(raw_proof)
        .map_err(Into::into)
}

/// Fetches the raw event at a certain block
pub async fn fetch_events(client: &XtClient, hash: &Hash)
-> Result<Option<(RawEvents, StorageProof, RawStorageKey)>, Error> {
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

/// Constructs a EventsDecoder for Phala Network
pub fn get_event_decoder(xt: &XtClient) -> EventsDecoder::<Runtime> {
    use runtimes::phala::PhalaModuleEventsDecoder;
    use subxt::balances::BalancesEventsDecoder;
    use subxt::staking::StakingEventsDecoder;
    use subxt::session::SessionEventsDecoder;

    let metadata = xt.metadata().clone();
    let mut decoder = EventsDecoder::<Runtime>::new(metadata);
    decoder.with_phala_module();
    decoder.with_balances();
    decoder.with_staking();
    decoder.with_session();
    decoder
}

/// Takes a snapshot of the necessary information for calculating compute works at a certain block
pub async fn snapshot_online_worker_at(xt: &XtClient, hash: Option<Hash>)
-> Result<OnlineWorkerSnapshot<BlockNumber, Balance>, Error> {
    use runtimes::{phala::*, mining_staking::*};
    use phala_types::*;
    // Stats numbers
    let online_workers_store = <OnlineWorkers<_> as Default>::default();
    let compute_workers_store = <ComputeWorkers<_> as Default>::default();
    let online_workers_key = online_workers_store.key(xt.metadata())?;
    let compute_workers_key = compute_workers_store.key(xt.metadata())?;
    let online_workers: u32 = xt.fetch_or_default(&online_workers_store, hash).await?;
    let compute_workers: u32 = xt.fetch_or_default(&compute_workers_store, hash).await?;
    if compute_workers == 0 {
        println!("ComputeWorkers is zero. Skipping worker snapshot.");
        return Err(Error::ComputeWorkerNotEnabled);
    }
    println!("- Stats Online Workers: {}", online_workers);
    println!("- Stats Compute Workers: {}", compute_workers);
    // Online workers and stake received
    let worker_data =
        fetch_map::<WorkerStateStore<_>>(xt, hash).await?;
    let staked_data =
        fetch_map::<StakeReceivedStore<_>>(xt, hash).await?;
    let online_worker_data: Vec<_> = worker_data
        .into_iter()
        .filter(|(_k, worker_info)|
            match worker_info.state {
                WorkerStateEnum::<BlockNumber>::Mining(_)
                | WorkerStateEnum::<BlockNumber>::MiningStopping => true,
                _ => false,
            }
        ).collect();
    let stashes: HashSet<AccountId> = online_worker_data
        .iter()
        .map(|(k, _v)| account_id_from_map_key(&k.0))
        .collect();
    let online_staked_data: Vec<_> = staked_data
        .into_iter()
        .filter(|(k, _v)| stashes.contains(&account_id_from_map_key(&k.0)))
        .collect();
    println!("- online_worker_data: vec[{}]", online_worker_data.len());
    println!("- online_staked_data: vec[{}]", online_staked_data.len());

    // Proof of all the storage keys
    let mut all_keys: Vec<StorageKey> = online_worker_data
        .iter().map(|(k, _)| k)
        .chain(online_staked_data.iter().map(|(k, _)| k))
        .cloned()
        .collect();
    all_keys.push(online_workers_key.clone());
    all_keys.push(compute_workers_key.clone());
    println!("- All Storage Keys: vec[{}]", all_keys.len());
    let read_proof = xt.read_proof(all_keys, hash).await.map_err(Into::<Error>::into)?;
    let proof = raw_proof(read_proof);

    // Snapshot fields
    let online_workers_kv = StorageKV::<u32>(online_workers_key.0, online_workers);
    let compute_workers_kv = StorageKV::<u32>(compute_workers_key.0, compute_workers);
    let worker_state_kv = storage_kv_from_data(online_worker_data);
    let stake_received_kv = storage_kv_from_data(online_staked_data);
    Ok(OnlineWorkerSnapshot {
        worker_state_kv,
        stake_received_kv,
        online_workers_kv,
        compute_workers_kv,
        proof,
    })
}

/// Check if the given raw event (in `Vec<u8>`) contains `PhalaModule.NewMiningRound`
pub fn check_round_end_event(
    decoder: &EventsDecoder::<Runtime>, value: &Vec<u8>
) -> Result<bool, Error> {
    let raw_events = decoder.decode_events(&mut value.as_slice())?;
    for (_phase, raw) in &raw_events {
        if let Raw::Event(event) = raw {
            if event.module == "PhalaModule" && event.variant == "NewMiningRound" {
                return Ok(true)
            }
        }
    }
    Ok(false)
}

pub async fn test_round_end_event(xt: &XtClient, decoder: &EventsDecoder::<Runtime>, hash: &Hash)
-> Result<bool, Error> {
    let (value, _proof, _key) = fetch_events(xt, hash).await?.unwrap();
    check_round_end_event(decoder, &value)
}

// Storage functions

/// Fetches all the StorageMap entries from Substrate
async fn fetch_map<F>(
    xt: &XtClient, hash: Option<Hash>
) -> Result<Vec<(StorageKey, <F as Store<Runtime>>::Returns)>, Error>
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

/// Converts the raw data `Vec<(StorageKey, T)>` to `Vec<StorageKV<T>>`
fn storage_kv_from_data<T>(storage_data: Vec<(StorageKey, T)>) -> Vec<StorageKV<T>>
where
    T: FullCodec + Clone
{
    storage_data
        .into_iter()
        .map(|(k, v)| StorageKV(k.0, v))
        .collect()
}

// Utility functions

/// Calculates the Substrate storage key prefix
fn storage_value_key_vec(module: &str, storage_key_name: &str) -> Vec<u8> {
    let mut key = twox_128(module.as_bytes()).to_vec();
    key.extend(&twox_128(storage_key_name.as_bytes()));
    key
}

/// Extract the last 256 bits as the AccountId (unsafe)
fn account_id_from_map_key(key: &[u8]) -> AccountId {
    // TODO: decode the key regularly
    // (twox128(module) + twox128(storage) + black2_128_concat(accountid))
    let mut raw_key: [u8; 32] = Default::default();
    raw_key.copy_from_slice(&key[(key.len() - 32)..]);
    AccountId::from(raw_key)
}
