use anyhow::{anyhow, Context, Result};
use parity_scale_codec::Encode;
use subxt::dynamic::Value;
use subxt::ext::frame_metadata;
use subxt::rpc::types::{BlockNumber as SubxtBlockNumber, NumberOrHex};
use subxt::ext::scale_value::At;

use crate::{BlockNumber, ChainApi, Hash};

impl ChainApi {
    pub fn storage_key(
        &self,
        pallet_name: &str,
        entry_name: &str,
        key: &impl Encode,
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

    pub async fn current_set_id(&self, block_hash: Hash) -> Result<u64> {
        let address = subxt::dynamic::storage_root("Grandpa", "CurrentSetId");
        let set_id = self
            .storage()
            .at(block_hash)
            .fetch(&address)
            .await
            .context("Failed to get current set_id")?
            .ok_or_else(|| anyhow!("No set id"))?;
        Ok(set_id
            .to_value()?
            .as_u128()
            .ok_or_else(|| anyhow!("Invalid set id"))? as _)
    }

    pub async fn get_paraid(&self, hash: Hash) -> Result<u32> {
        let address = subxt::dynamic::storage_root("ParachainInfo", "ParachainId");
        let id = self
            .storage()
            .at(hash)
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
            .fetch(&address)
            .await
            .context("Failed to get worker info")? else {
                return Ok(None);
            };
        let block_number = block
            .to_value()?
            .as_u128()
            .ok_or_else(|| anyhow!("Invalid block number in WorkerAddedAt"))?;
        Ok(Some(block_number as _))
    }
}
