use crate::std::prelude::v1::*;
use sgx_types::sgx_spid_t;

pub fn decode_spid(raw_hex: &str) -> sgx_spid_t {
    let mut spid = sgx_spid_t::default();
    let raw_hex = raw_hex.trim();

    if raw_hex.len() < 16 * 2 {
        log::warn!("Input spid file len ({}) is incorrect!", raw_hex.len());
        return spid;
    }

    let decoded_vec = hex::decode(raw_hex).expect("Failed to decode SPID hex");
    spid.id.copy_from_slice(&decoded_vec[..16]);
    spid
}
