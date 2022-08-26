use anyhow::Result;
use subxt::{metadata::EncodeStaticType, Metadata};

pub fn storage_key(pallet: &str, entry: &str) -> Vec<u8> {
    let address = ::subxt::dynamic::storage_root(pallet, entry);
    ::subxt::storage::utils::storage_address_root_bytes(&address)
}

pub fn paras_heads_key(para_id: u32, metadata: &Metadata) -> Result<Vec<u8>> {
    let id = EncodeStaticType(crate::ParaId(para_id));
    let address = subxt::dynamic::storage("Paras", "Heads", vec![id]);
    Ok(subxt::storage::utils::storage_address_bytes(
        &address, metadata,
    )?)
}
