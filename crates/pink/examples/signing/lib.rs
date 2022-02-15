#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;


#[pink::contract(env=PinkEnvironment)]
mod signing {
    use super::pink;
    use pink::{PinkEnvironment, sign, verify, derive_sr25519_pair, public_key_for, chain_extension::SigType};

    #[ink(storage)]
    pub struct Signing {}

    impl Signing {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn test(&self) {
            let (privkey, pubkey) = derive_sr25519_pair!(b"a spoon of salt");
            assert_eq!(pubkey, public_key_for!(&privkey, SigType::Sr25519));
            let message = b"hello world".as_ref();
            let signature = sign!(message, &privkey, SigType::Sr25519);
            let pass = verify!(message, &pubkey, &signature, SigType::Sr25519);
            assert!(pass);
            let pass = verify!(b"Fake".as_ref(), &pubkey, &signature, SigType::Sr25519);
            assert!(!pass);
        }
    }
}
