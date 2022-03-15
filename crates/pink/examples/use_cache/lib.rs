#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;

#[pink::contract(env=PinkEnvironment)]
mod use_cache {
    use super::pink;
    use pink::chain_extension::pink_extension_instance as ext;
    use pink::PinkEnvironment;

    #[ink(storage)]
    pub struct UseCache {}

    impl UseCache {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn test(&self) {
            assert!(ext().cache_set(b"key", b"value").is_ok());
            assert_eq!(ext().cache_get(b"key"), Some(b"value".to_vec()));
            assert_eq!(ext().cache_remove(b"key"), Some(b"value".to_vec()));
            assert_eq!(ext().cache_get(b"key"), None);
        }
    }
}
