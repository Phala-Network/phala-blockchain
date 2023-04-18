pub mod tx;

pub fn storage_key(pallet: &str, entry: &str) -> Vec<u8> {
    ::subxt::dynamic::storage_root(pallet, entry).to_root_bytes()
}
