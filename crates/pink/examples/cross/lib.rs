#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod cross {
    use flip::FlipRef;
    use hex_literal::hex;

    //--snip--
    #[ink(storage)]
    pub struct MyContract {
        /// The other contract.
        flip: FlipRef,
    }

    impl MyContract {
        /// Instantiate `MyContract with the given
        /// sub-contract codes and some initial value.
        #[ink(constructor)]
        pub fn new() -> Self {
            let hash = hex!("a3f91e98edc8ccfb035946133027dd5a3f8694c70e7a27ffdf8056f7b9cc40ab").into();
            let flip = FlipRef::new(true)
                .endowment(100000)
                .salt_bytes(&[0x00])
                .code_hash(hash)
                .instantiate()
                .expect("failed at instantiating the `OtherContract` contract..");
            Self { flip }
        }

        /// Calls the other contract.
        #[ink(message)]
        pub fn call_other_contract(&self) -> bool {
            self.flip.get()
        }
    }
    //--snip--
}
