use crate::hasher::{Hasher, Twox128};
use alloc::vec::Vec;

/// Returns the prefix of an storage item
pub fn storage_prefix(pallet_name: &str, storage_name: &str) -> [u8; 32] {
    // Copied from Substrate function: `storage_prefix()`
    let pallet_hash = Twox128::hash(pallet_name.as_bytes());
    let storage_hash = Twox128::hash(storage_name.as_bytes());

    let mut final_key = [0u8; 32];
    final_key[..16].copy_from_slice(&pallet_hash);
    final_key[16..].copy_from_slice(&storage_hash);
    final_key
}

/// Returns the storage key of a storage map entry
pub fn storage_map_prefix<H: Hasher>(prefix: &[u8], key1: &[u8]) -> Vec<u8> {
    let key1_hashed = H::hash(key1);

    let mut final_key = Vec::with_capacity(prefix.len() + key1_hashed.as_ref().len() + key1.len());
    final_key.extend_from_slice(prefix);
    final_key.extend_from_slice(key1_hashed.as_ref());
    final_key
}

/// Returns the storage key of a storage double map entry
pub fn storage_double_map_prefix<H1: Hasher, H2: Hasher>(
    prefix: &[u8],
    key1: &[u8],
    key2: &[u8],
) -> Vec<u8> {
    let key1_hashed = H1::hash(key1);
    let key2_hashed = H2::hash(key2);

    let mut final_key = Vec::with_capacity(
        prefix.len()
            + key1_hashed.as_ref().len()
            + key1.len()
            + key2_hashed.as_ref().len()
            + key2.len(),
    );
    final_key.extend_from_slice(prefix);
    final_key.extend_from_slice(key1_hashed.as_ref());
    final_key.extend_from_slice(key2_hashed.as_ref());
    final_key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hasher::Blake2_128Concat;
    #[test]
    fn storage_key_is_correct() {
        use scale::Encode;
        let map_key =
            hex_literal::hex!("0202020202020202020202020202020202020202020202020202020202020202")
                .to_vec();
        let key = storage_double_map_prefix::<Blake2_128Concat, Blake2_128Concat>(
            &storage_prefix("PhatRollupAnchor", "States")[..],
            &hex_literal::hex!("0101010101010101010101010101010101010101010101010101010101010101"),
            &map_key.encode(),
        );
        assert_eq!(
            key,
            hex_literal::hex!("6e5134eca327aece93f5faddaec7c0d751f254b22584f9f893c604003c293742c035f853fcd0f0589e30c9e2dc1a0f57010101010101010101010101010101010101010101010101010101010101010135e8cfc0722c6a15a223941231244028800202020202020202020202020202020202020202020202020202020202020202")
        );
    }
}
