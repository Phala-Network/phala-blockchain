use blake2_rfc;

/// Do a Blake2 128-bit hash and place result in `dest`.
pub fn blake2_128_into(data: &[u8], dest: &mut [u8; 16]) {
	dest.copy_from_slice(blake2_rfc::blake2b::blake2b(16, &[], data).as_bytes());
}

/// Do a Blake2 128-bit hash and return result.
pub fn blake2_128(data: &[u8]) -> [u8; 16] {
	let mut r = [0; 16];
	blake2_128_into(data, &mut r);
	r
}
