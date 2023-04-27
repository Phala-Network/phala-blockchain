#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use pink_extension as pink;

#[pink::contract]
mod contract {
    use super::pink;
    #[ink(storage)]
    pub struct Contract {}

    fn start_sidevm() {
        let code_hash = *include_bytes!("./sideprog.wasm.hash");
        pink::start_sidevm(code_hash).expect("Failed to start sidevm");
    }

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            start_sidevm();
            Self {}
        }

        #[ink(message)]
        pub fn version(&self) -> this_crate::VersionTuple {
            this_crate::version_tuple!()
        }

        #[ink(message)]
        pub fn start(&self) {
            start_sidevm();
        }

        #[ink(message)]
        pub fn stop(&self) {
            pink::force_stop_sidevm();
        }

        #[ink(message)]
        pub fn log_test(&self, msg: alloc::string::String) {
            pink::info!("{}", msg);
        }
    }
}
