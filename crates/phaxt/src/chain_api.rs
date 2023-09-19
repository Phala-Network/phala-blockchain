use anyhow::{anyhow, Context, Result};
use subxt::dynamic::Value;
use subxt::ext::scale_encode::EncodeAsType;
use subxt::ext::scale_value::At;
use subxt::rpc::types::{BlockNumber as SubxtBlockNumber, NumberOrHex};
use subxt::storage::Storage;

use parity_scale_codec::{Decode, Encode};
use phala_types::{VersionedWorkerEndpoints, WorkerPublicKey};

use crate::{BlockNumber, ChainApi, Config, Hash, RpcClient};

impl ChainApi {
    async fn storage_at(&self, hash: Option<Hash>) -> Result<Storage<Config, RpcClient>> {
        let snap = match hash {
            Some(hash) => self.storage().at(hash),
            None => self.storage().at_latest().await?,
        };
        Ok(snap)
    }

    pub fn storage_key(
        &self,
        pallet_name: &str,
        entry_name: &str,
        key: &impl EncodeAsType,
    ) -> Result<Vec<u8>> {
        let address = subxt::dynamic::storage(pallet_name, entry_name, vec![key]);
        Ok(self.0.storage().address_bytes(&address)?)
    }

    pub fn paras_heads_key(&self, para_id: u32) -> Result<Vec<u8>> {
        let id = crate::ParaId(para_id);
        self.storage_key("Paras", "Heads", &id)
    }

    pub async fn relay_parent_number(&self) -> Result<BlockNumber> {
        let hash = self
            .rpc()
            .block_hash(Some(SubxtBlockNumber::from(NumberOrHex::Number(1))))
            .await
            .context("Failed get the HASH of block 1")?;
        let addr = subxt::dynamic::storage_root("ParachainSystem", "ValidationData");
        let validation_data = self
            .storage()
            .at(hash.expect("hash must not None"))
            .fetch(&addr)
            .await
            .context("Failed to fetch validation data")?
            .ok_or_else(|| anyhow!("ValidationData not found"))?
            .to_value()?;
        let number = validation_data
            .at("relay_parent_number")
            .ok_or_else(|| anyhow!("No relay_parent_number"))?
            .as_u128()
            .ok_or_else(|| anyhow!("No relay_parent_number"))?;
        Ok(number as _)
    }

    pub async fn current_set_id(&self, block_hash: Option<Hash>) -> Result<u64> {
        let address = subxt::dynamic::storage_root("Grandpa", "CurrentSetId");
        let set_id = self
            .storage_at(block_hash)
            .await?
            .fetch(&address)
            .await
            .context("Failed to get current set_id")?
            .ok_or_else(|| anyhow!("No set id"))?;
        Ok(set_id
            .to_value()?
            .as_u128()
            .ok_or_else(|| anyhow!("Invalid set id"))? as _)
    }

    pub async fn get_paraid(&self, hash: Option<Hash>) -> Result<u32> {
        let address = subxt::dynamic::storage_root("ParachainInfo", "ParachainId");
        let id = self
            .storage_at(hash)
            .await?
            .fetch(&address)
            .await
            .context("Failed to get current set_id")?
            .ok_or_else(|| anyhow!("No paraid found"))?
            .to_value()?;
        let id = id
            .at(0)
            .ok_or_else(|| anyhow!("Invalid paraid"))?
            .as_u128()
            .ok_or_else(|| anyhow!("Invalid paraid"))?;
        Ok(id as _)
    }

    pub async fn worker_registered_at(
        &self,
        block_number: BlockNumber,
        worker: &[u8],
    ) -> Result<bool> {
        let hash = self
            .rpc()
            .block_hash(Some(block_number.into()))
            .await?
            .ok_or_else(|| anyhow!("Block number not found"))?;
        let worker = Value::from_bytes(worker);
        let address = subxt::dynamic::storage("PhalaRegistry", "Workers", vec![worker]);
        let registered = self
            .storage()
            .at(hash)
            .fetch(&address)
            .await
            .context("Failed to get worker info")?
            .is_some();
        Ok(registered)
    }

    pub async fn worker_added_at(&self, worker: &[u8]) -> Result<Option<BlockNumber>> {
        let worker = Value::from_bytes(worker);
        let address = subxt::dynamic::storage("PhalaRegistry", "WorkerAddedAt", vec![worker]);
        let Some(block) = self
            .storage()
            .at_latest()
            .await?
            .fetch(&address)
            .await
            .context("Failed to get worker info")?
        else {
            return Ok(None);
        };
        let block_number = block
            .to_value()?
            .as_u128()
            .ok_or_else(|| anyhow!("Invalid block number in WorkerAddedAt"))?;
        Ok(Some(block_number as _))
    }

    async fn fetch<K: Encode, V: Decode>(
        &self,
        pallet: &str,
        name: &str,
        key: Option<K>,
    ) -> Result<Option<V>> {
        let mut args = vec![];
        if let Some(key) = key {
            let key = Value::from_bytes(key.encode());
            args.push(key);
        }
        let address = subxt::dynamic::storage(pallet, name, args);
        let Some(data) = self
            .storage()
            .at_latest()
            .await?
            .fetch(&address)
            .await
            .context("Failed to get worker endpoints")?
        else {
            return Ok(None);
        };
        Ok(Some(Decode::decode(&mut &data.encoded()[..])?))
    }

    pub async fn get_workers(&self, cluster_id: Hash) -> Result<Vec<WorkerPublicKey>> {
        let result = self
            .fetch("PhalaPhatContracts", "ClusterWorkers", Some(&cluster_id))
            .await?;
        Ok(result.unwrap_or_default())
    }

    pub async fn get_endpoints(&self, worker: &WorkerPublicKey) -> Result<Vec<String>> {
        let result = self
            .fetch("PhalaRegistry", "Endpoints", Some(worker))
            .await?;
        let Some(VersionedWorkerEndpoints::V1(endpoints)) = result else {
            return Ok(vec![]);
        };
        Ok(endpoints)
    }

    pub async fn storage_keys(&self, prefix: &[u8], hash: Option<Hash>) -> Result<Vec<Vec<u8>>> {
        let page = 100;
        let mut keys: Vec<Vec<u8>> = vec![];

        loop {
            let start_key = keys.last().map(|k| &k[..]);
            let result = self
                .rpc()
                .storage_keys_paged(prefix, page, start_key, hash)
                .await?;
            if result.is_empty() {
                break;
            }
            result.into_iter().for_each(|key| keys.push(key.0));
        }
        Ok(keys)
    }

    pub async fn latest_finalized_block_number(&self) -> Result<BlockNumber> {
        let latest_block_hash = self
            .rpc()
            .finalized_head()
            .await
            .context("Failed to get finalized head")?;
        let block = self
            .rpc()
            .header(Some(latest_block_hash))
            .await
            .context("Failed to get block number")?
            .ok_or(anyhow::anyhow!("Block number not found"))?
            .number;
        Ok(block)
    }
}
