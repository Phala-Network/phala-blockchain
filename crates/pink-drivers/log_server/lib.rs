#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use pink_extension as pink;

#[pink::contract]
mod contract {
    use super::pink;

    use pink::system::DriverError as Error;

    type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct Contract {
        owner: AccountId,
    }

    fn start_sidevm() {
        let code_hash = *include_bytes!("./sideprog.wasm.hash");
        pink::start_sidevm(code_hash).expect("Failed to start sidevm");
    }

    impl Contract {
        #[ink(constructor)]
        #[allow(clippy::should_implement_trait)]
        pub fn default() -> Self {
            start_sidevm();
            Self {
                owner: Self::env().caller(),
            }
        }

        #[ink(message)]
        pub fn version(&self) -> this_crate::VersionTuple {
            this_crate::version_tuple!()
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        pub fn start(&self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::BadOrigin);
            }
            start_sidevm();
            Ok(())
        }

        #[ink(message)]
        pub fn stop(&self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::BadOrigin);
            }
            pink::force_stop_sidevm();
            Ok(())
        }

        #[ink(message)]
        pub fn log_test(&self, msg: alloc::string::String) {
            pink::info!("{}", msg);
        }
    }
}
