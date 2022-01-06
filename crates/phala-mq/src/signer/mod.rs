
use crate::SignedMessage;

pub trait MessageVerifier {
    fn verify(&self, message: &SignedMessage) -> bool;
}

#[cfg(feature = "queue")]
pub mod signers {
    use alloc::vec::Vec;
    use sp_core::{crypto::Pair as PairTrait, sr25519};
    use serde::{Serialize, Deserialize};
    use phala_serde_more as more;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Sr25519Signer {
        #[serde(with = "more::key_bytes")]
        key: sr25519::Pair,
    }

    impl Sr25519Signer {
        fn sign(&self, data: &[u8]) -> Vec<u8> {
            self.key.sign(data).0.to_vec()
        }
    }

    impl From<sr25519::Pair> for Sr25519Signer {
        fn from(key: sr25519::Pair) -> Self {
            Self { key }
        }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub enum MessageSigner {
        Sr25519(Sr25519Signer),
        Test(Vec<u8>),
    }

    impl MessageSigner {
        pub fn sign(&self, data: &[u8]) -> Vec<u8> {
            match self {
                Self::Sr25519(signer) => signer.sign(data),
                Self::Test(k) => k.to_vec(),
            }
        }
    }

    impl From<sr25519::Pair> for MessageSigner {
        fn from(key: sr25519::Pair) -> Self {
            Self::Sr25519(Sr25519Signer::from(key))
        }
    }
}
