use crate::std::slice;
use sp_core::hashing::blake2_256;
use sp_core::H256;
use parity_scale_codec::Encode;

pub fn header_hash<H: Encode>(header: &H) -> H256 {
	let data = header.encode();
	blake2_256(&data.as_slice()).into()
}
