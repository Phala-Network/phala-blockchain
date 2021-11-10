#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;
use pink::PinkEnvironment;

#[pink::contract(env = PinkEnvironment)]
mod proxy {
    use super::*;

    #[ink(storage)]
    pub struct Proxy {}

    impl Proxy {
        #[ink(constructor)]
        pub fn default() -> Self {
            Proxy {}
        }

        #[pink(on_block_end)]
        pub fn on_block_end(&self) {
            pink::push_message(b"foo".to_vec(), b"/bar".to_vec());
        }
    }
}
