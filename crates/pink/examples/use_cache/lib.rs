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

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;
        #[ink::test]
        fn it_works() {
            use pink_extension::chain_extension::mock;
            use std::{cell::RefCell, collections::HashMap, rc::Rc};

            let storage: Rc<RefCell<HashMap<Vec<u8>, Vec<u8>>>> = Default::default();

            {
                let storage = storage.clone();
                mock::mock_cache_set(move |k, v| {
                    storage.borrow_mut().insert(k.to_vec(), v.to_vec());
                    Ok(())
                });
            }
            {
                let storage = storage.clone();
                mock::mock_cache_get(move |k| storage.borrow().get(k).map(|v| v.to_vec()));
            }
            {
                let storage = storage.clone();
                mock::mock_cache_remove(move |k| {
                    storage.borrow_mut().remove(k).map(|v| v.to_vec())
                });
            }

            let contract = UseCache::default();
            contract.test();
        }
    }
}
