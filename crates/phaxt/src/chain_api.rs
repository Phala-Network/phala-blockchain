use anyhow::anyhow;
use subxt::ext::scale_value::At;
use subxt::rpc::NumberOrHex;

use anyhow::Result;

use crate::{BlockNumber, ChainApi, Hash};

impl ChainApi {
    pub async fn relay_parent_number(&self) -> Result<BlockNumber> {
        let hash = self
            .rpc()
            .block_hash(Some(subxt::rpc::BlockNumber::from(NumberOrHex::Number(1))))
            .await?;
        let addr = subxt::dynamic::storage_root("ParachainSystem", "ValidationData");
        let validation_data = self
            .storage()
            .fetch(&addr, hash)
            .await?
            .ok_or(anyhow!("ValidationData not found"))?;
        let number = validation_data
            .at("relay_parent_number")
            .ok_or(anyhow!("No relay_parent_number"))?
            .as_u128()
            .ok_or(anyhow!("No relay_parent_number"))?;
        Ok(number as _)
    }

    pub fn paras_heads_key(&self, para_id: u32) -> Result<Vec<u8>> {
        crate::dynamic::paras_heads_key(para_id, &self.metadata())
    }

    pub async fn current_set_id(&self, block_hash: Option<Hash>) -> Result<u64> {
        let address = subxt::dynamic::storage_root("Grandpa", "CurrentSetId");
        let set_id = self
            .storage()
            .fetch(&address, block_hash)
            .await?
            .ok_or(anyhow!("No set id"))?;
        Ok(set_id.as_u128().ok_or(anyhow!("Invalid set id"))? as _)
    }
}
