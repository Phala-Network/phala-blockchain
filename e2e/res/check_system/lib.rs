#![cfg_attr(not(feature = "std"), no_std)]

use pink_extension as pink;

#[pink::contract(env = PinkEnvironment)]
mod check_system {
    use super::pink;
    use pink::PinkEnvironment;

    #[ink(storage)]
    pub struct CheckSystem {
        value: bool
    }

    impl CheckSystem {
        #[ink(constructor)]
        pub fn default() -> Self {
            let system = pink::system::SystemRef::instance();
            let value = system.get_driver("NotExists".into()).is_none();
            Self {
                value
            }
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }
}
