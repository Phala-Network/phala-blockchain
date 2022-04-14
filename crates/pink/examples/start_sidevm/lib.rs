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
            let code = &include_bytes!("./sidevm_httpserver.wasm")[..];
            pink::start_sidevm(code.into(), 100);
            Self {}
        }
        #[ink(message)]
        pub fn test(&self) {
        }
    }
}
