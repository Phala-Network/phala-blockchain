use blake2_rfc;

/// Do a Blake2 512-bit hash and place result in `dest`.
pub fn blake2_512_into(data: &[u8], dest: &mut [u8; 64]) {
    dest.copy_from_slice(blake2_rfc::blake2b::blake2b(64, &[], data).as_bytes());
}

/// Do a Blake2 512-bit hash and return result.
pub fn blake2_512(data: &[u8]) -> [u8; 64] {
    let mut r = [0; 64];
    blake2_512_into(data, &mut r);
    r
}

/// Do a Blake2 256-bit hash and place result in `dest`.
pub fn blake2_256_into(data: &[u8], dest: &mut [u8; 32]) {
	dest.copy_from_slice(blake2_rfc::blake2b::blake2b(32, &[], data).as_bytes());
}

/// Do a Blake2 256-bit hash and return result.
pub fn blake2_256(data: &[u8]) -> [u8; 32] {
	let mut r = [0; 32];
	blake2_256_into(data, &mut r);
	r
}
