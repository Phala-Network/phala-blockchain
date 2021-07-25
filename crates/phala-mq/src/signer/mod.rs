use alloc::vec::Vec;

use crate::SignedMessage;

pub trait MessageSigner {
    fn sign(&self, data: &[u8]) -> Vec<u8>;
}
pub trait MessageVerifier {
    fn verify(&self, message: &SignedMessage) -> bool;
}

#[cfg(feature = "signers")]
mod signers {
    use super::MessageSigner;
    use alloc::vec::Vec;
    use sp_core::{crypto::Pair as PairTrait, sr25519};

    impl MessageSigner for sr25519::Pair {
        fn sign(&self, data: &[u8]) -> Vec<u8> {
            PairTrait::sign(self, data).0.to_vec()
        }
    }
}
