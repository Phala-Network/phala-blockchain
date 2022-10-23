#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use pink_extension as pink;

#[pink::contract]
mod contract {
    use super::pink;
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            let code_hash = *include_bytes!("./sideprog.wasm.hash");
            pink::start_sidevm(code_hash).expect("Failed to start sidevm");
            Self {}
        }

        #[ink(message)]
        pub fn log_test(&self, msg: alloc::string::String) {
            pink::info!("{}", msg);
        }
    }
}
