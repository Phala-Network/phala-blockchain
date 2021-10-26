#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(env = PinkEnvironment)]
mod proxy {
    extern crate alloc;
    use alloc::vec::Vec;
    use pink_extension::{push_message, push_osp_message, EcdhPublicKey, PinkEnvironment};

    #[ink(storage)]
    pub struct Proxy {}

    impl Proxy {
        #[ink(constructor)]
        pub fn default() -> Self {
            Proxy {}
        }

        #[ink(message)]
        pub fn push_message(&self, message: Vec<u8>, topic: Vec<u8>) {
            push_message(message, topic)
        }

        #[ink(message)]
        pub fn push_osp_message(
            &self,
            message: Vec<u8>,
            topic: Vec<u8>,
            remote_pubkey: Option<EcdhPublicKey>,
        ) {
            push_osp_message(message, topic, remote_pubkey)
        }
    }
}
