#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;

pub use indeterministic_functions::*;

#[ink::contract]
mod indeterministic_functions {
    use alloc::string::String;
    use scale::{Decode, Encode};
    use serde::{Deserialize, Serialize};

    #[derive(Decode, Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Usd {
        pub usd: u128,
    }

    #[derive(Serialize, Deserialize)]
    struct SerdeUsd {
        usd: f64,
    }

    #[ink(storage)]
    pub struct IndeterministicFunctions {}

    impl IndeterministicFunctions {
        #[ink(constructor)]
        #[allow(clippy::should_implement_trait)]
        pub fn default() -> Self {
            IndeterministicFunctions {}
        }

        #[ink(message)]
        pub fn parse_usd(&self, json: String) -> Option<Usd> {
            let decoded: SerdeUsd = serde_json::from_str(&json).ok()?;
            Some(Usd {
                usd: (decoded.usd * 100.0) as u128,
            })
        }
    }
}
