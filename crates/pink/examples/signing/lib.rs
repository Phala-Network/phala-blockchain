#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;


#[pink::contract(env=PinkEnvironment)]
mod signing {
    use super::pink;
    use pink::PinkEnvironment;
    use pink::chain_extension::signing as sig;

    #[ink(storage)]
    pub struct Signing {}

    impl Signing {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn test(&self) {
            use sig::SigType;

            let privkey = sig::derive_sr25519_key(b"a spoon of salt");
            let pubkey = sig::get_public_key(&privkey, SigType::Sr25519);
            let message = b"hello world";
            let signature = sig::sign(message, &privkey, SigType::Sr25519);
            let pass = sig::verify(message, &pubkey, &signature, SigType::Sr25519);
            assert!(pass);
            let pass = sig::verify(b"Fake", &pubkey, &signature, SigType::Sr25519);
            assert!(!pass);
        }
    }
}
