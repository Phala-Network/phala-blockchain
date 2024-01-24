//! Utilities to create and verify off-chain attestation

use alloc::vec::Vec;
use core::fmt;
use pink::chain_extension::{signing, SigType};
use scale::{Decode, Encode};

/// A signed payload produced by a [`Generator`], and can be validated by [`Verifier`].
#[derive(Clone, Encode, Decode, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Attestation {
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
    // TODO: add metadata
}

/// An attestation verifier
#[derive(Debug, Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Verifier {
    pub pubkey: Vec<u8>,
}

impl Verifier {
    /// Verifies an attestation
    pub fn verify(&self, attestation: &Attestation) -> bool {
        signing::verify(
            &attestation.data,
            &self.pubkey,
            &attestation.signature,
            SigType::Sr25519,
        )
    }

    /// Verifies an attestation and decodes the inner data
    pub fn verify_as<T: Decode>(&self, attestation: &Attestation) -> Option<T> {
        if !self.verify(attestation) {
            return None;
        }
        Decode::decode(&mut &attestation.data[..]).ok()
    }
}

/// An attestation generator
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Generator {
    pub privkey: Vec<u8>,
}

impl Generator {
    /// Produces a signed attestation with the given `data`
    pub fn sign<T: Clone + Encode + Decode>(&self, data: T) -> Attestation {
        let encoded = Encode::encode(&data);
        let signature = signing::sign(&encoded, &self.privkey, SigType::Sr25519);
        Attestation {
            data: encoded,
            signature,
        }
    }
}

impl fmt::Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We don't want to leak the privkey to anyone
        write!(f, "Generator")
    }
}

/// Creates a pair of attestation utility to do off-chain attestation
pub fn create(salt: &[u8]) -> (Generator, Verifier) {
    let privkey = signing::derive_sr25519_key(salt);
    let pubkey = signing::get_public_key(&privkey, SigType::Sr25519);
    (Generator { privkey }, Verifier { pubkey })
}

#[cfg(test)]
mod test {
    use super::*;

    #[ink::test]
    fn it_works() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        // Generate an attestation and verify it later
        #[derive(Clone, Encode, Decode, scale_info::TypeInfo)]
        struct SomeData {
            x: u32,
        }

        let (generator, verifier) = create(b"salt");
        let attestation = generator.sign(SomeData { x: 123 });
        assert!(verifier.verify(&attestation));
    }
}
