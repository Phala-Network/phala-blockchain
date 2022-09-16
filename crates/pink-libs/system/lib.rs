#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use pink_extension as pink;

pub use system::System;

#[pink::contract(env = PinkEnvironment)]
mod system {
    use super::pink;
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use pink::system::{Error, Result};
    use pink::PinkEnvironment;
    use alloc::string::String;

    /// Pink's system contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct System {
        /// The owner of the contract(the cluster).
        owner: AccountId,
        /// The administrators
        administrators: Mapping<AccountId, ()>,
        /// The drivers
        drivers: Mapping<String, AccountId>,
    }

    impl System {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|me: &mut Self| me.owner = Self::env().caller())
        }

        fn ensure_owner(&self) -> Result<AccountId> {
            let caller = self.env().caller();
            if caller == self.owner {
                Ok(caller)
            } else {
                Err(Error::BadOrigin)
            }
        }

        fn ensure_admin(&self) -> Result<AccountId> {
            let caller = self.env().caller();
            if caller == self.owner {
                return Ok(caller);
            }
            if self.administrators.contains(&caller) {
                return Ok(caller);
            }
            Err(Error::BadOrigin)
        }
    }

    impl pink::system::System for System {
        #[ink(message)]
        fn grant_admin(&mut self, contract_id: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.administrators.insert(contract_id, &());
            Ok(())
        }

        #[ink(message)]
        fn set_driver(&mut self, name: String, contract_id: AccountId) -> Result<()> {
            self.ensure_admin()?;
            self.drivers.insert(name, &contract_id);
            Ok(())
        }

        #[ink(message)]
        fn get_driver(&self, name: String) -> Option<AccountId> {
            self.drivers.get(&name)
        }

        #[ink(message)]
        fn deploy_sidevm_to(
            &self,
            contract_id: AccountId,
            code_hash: pink::Hash,
        ) -> Result<()> {
            self.ensure_admin()?;
            pink::deploy_sidevm_to(contract_id, code_hash);
            Ok(())
        }

        #[ink(message)]
        fn stop_sidevm_at(
            &self,
            contract_id: AccountId,
        ) -> Result<()> {
            self.ensure_admin()?;
            pink::stop_sidevm_at(contract_id);
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn it_works() {
            use pink::system::SystemRef;

            SystemRef::mock_with(System::new());

            let driver_name = "Hello";
            let mut system = SystemRef::instance();

            let driver = system.get_driver(driver_name.into());
            assert_eq!(driver, None);

            let driver_id = [1u8; 32].into();
            system
                .set_driver(driver_name.into(), driver_id)
                .expect("Set driver failed");
            let driver = system.get_driver(driver_name.into());
            assert_eq!(driver, Some(driver_id));
        }
    }
}
