use sp_core_hashing::*;

/// Hasher to use to hash keys to insert to storage.
pub trait Hasher: 'static {
    type Output: AsRef<[u8]>;
    fn hash(x: &[u8]) -> Self::Output;
}

pub struct Twox64Concat;
impl Hasher for Twox64Concat {
    type Output = Vec<u8>;
    fn hash(x: &[u8]) -> Vec<u8> {
        twox_64(x)
            .iter()
            .chain(x.iter())
            .cloned()
            .collect::<Vec<_>>()
    }
}

pub struct Blake2_128Concat;
impl Hasher for Blake2_128Concat {
    type Output = Vec<u8>;
    fn hash(x: &[u8]) -> Vec<u8> {
        blake2_128(x)
            .iter()
            .chain(x.iter())
            .cloned()
            .collect::<Vec<_>>()
    }
}

pub struct Blake2_128;
impl Hasher for Blake2_128 {
    type Output = [u8; 16];
    fn hash(x: &[u8]) -> [u8; 16] {
        blake2_128(x)
    }
}

pub struct Blake2_256;
impl Hasher for Blake2_256 {
    type Output = [u8; 32];
    fn hash(x: &[u8]) -> [u8; 32] {
        blake2_256(x)
    }
}

pub struct Twox128;
impl Hasher for Twox128 {
    type Output = [u8; 16];
    fn hash(x: &[u8]) -> [u8; 16] {
        twox_128(x)
    }
}

pub struct Twox256;
impl Hasher for Twox256 {
    type Output = [u8; 32];
    fn hash(x: &[u8]) -> [u8; 32] {
        twox_256(x)
    }
}
