#![cfg_attr(not(feature = "std"), no_std)]

use pink_extension as pink;

#[pink::contract]
mod start_sidevm {
    use super::pink;
    use scale::Encode;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            let hash = *include_bytes!("./sideprog.wasm.hash");
            pink::start_sidevm(hash, true);
            Self {}
        }
        #[pink(on_block_end)]
        pub fn on_block_end(&self) {
            let number = self.env().block_number().encode();
            pink::ext().cache_set(b"block_number", &number).unwrap();
            pink::push_sidevm_message(b"hello".to_vec());
        }
        #[ink(message)]
        pub fn test(&self) {
        }
    }
}
