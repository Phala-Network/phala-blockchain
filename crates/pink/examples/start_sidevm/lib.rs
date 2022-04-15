#![cfg_attr(not(feature = "std"), no_std)]

use pink_extension as pink;

#[pink::contract]
mod start_sidevm {
    use super::pink;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            let code = &include_bytes!("./sidevm_recv_messages.wasm")[..];
            pink::start_sidevm(code.into(), 100);
            Self {}
        }
        #[pink(on_block_end)]
        pub fn on_block_end(&self) {
            pink::push_sidevm_message(b"hello".to_vec());
        }
        #[ink(message)]
        pub fn test(&self) {
        }
    }
}
