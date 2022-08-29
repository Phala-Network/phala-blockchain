use anyhow::Result;
use subxt::{metadata::EncodeStaticType, Metadata};

pub mod tx;

pub fn storage_key(pallet: &str, entry: &str) -> Vec<u8> {
    let address = ::subxt::dynamic::storage_root(pallet, entry);
    ::subxt::storage::utils::storage_address_root_bytes(&address)
}
