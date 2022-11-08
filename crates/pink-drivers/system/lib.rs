#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use pink_extension as pink;

pub use system::System;

#[pink::contract(env = PinkEnvironment)]
mod system {
    use super::pink;
    use alloc::string::String;
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use pink::system::{ContractDeposit, ContractDepositRef, Error, Result};
    use pink::{HookPoint, PinkEnvironment};

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
        #[ink(constructor, selector = 0xed4b9d1b)]
        pub fn default() -> Self {
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
            if self.administrators.contains(&caller) {
                return Ok(caller);
            }
            Err(Error::BadOrigin)
        }

        fn ensure_owner_or_admin(&self) -> Result<AccountId> {
            self.ensure_owner().or_else(|_| self.ensure_admin())
        }

        fn ensure_self(&self) -> Result<AccountId> {
            let caller = self.env().caller();
            if caller == self.env().account_id() {
                Ok(caller)
            } else {
                Err(Error::BadOrigin)
            }
        }
    }

    impl pink::system::System for System {
        #[ink(message)]
        fn version(&self) -> (u16, u16) {
            (0, 1)
        }

        #[ink(message)]
        fn grant_admin(&mut self, contract_id: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.administrators.insert(contract_id, &());
            Ok(())
        }

        #[ink(message)]
        fn set_driver(&mut self, name: String, contract_id: AccountId) -> Result<()> {
            self.ensure_owner_or_admin()?;
            match name.as_str() {
                "PinkLogger" => {
                    pink::set_log_handler(contract_id);
                }
                _ => {}
            }
            self.drivers.insert(name, &contract_id);
            Ok(())
        }

        #[ink(message)]
        fn get_driver(&self, name: String) -> Option<AccountId> {
            self.drivers.get(&name)
        }

        #[ink(message)]
        fn deploy_sidevm_to(&self, contract_id: AccountId, code_hash: pink::Hash) -> Result<()> {
            self.ensure_admin()?;
            pink::deploy_sidevm_to(contract_id, code_hash);
            Ok(())
        }

        #[ink(message)]
        fn stop_sidevm_at(&self, contract_id: AccountId) -> Result<()> {
            self.ensure_admin()?;
            pink::stop_sidevm_at(contract_id);
            Ok(())
        }

        #[ink(message)]
        fn set_hook(
            &mut self,
            hook: HookPoint,
            contract: AccountId,
            selector: u32,
            gas_limit: u64,
        ) -> Result<()> {
            self.ensure_admin()?;
            pink::set_hook(hook, contract, selector, gas_limit);
            Ok(())
        }

        #[ink(message)]
        fn set_contract_weight(&self, contract_id: AccountId, weight: u32) -> Result<()> {
            self.ensure_admin()?;
            pink::set_contract_weight(contract_id, weight);
            Ok(())
        }

        #[ink(message)]
        fn total_balance_of(&self, account: AccountId) -> Balance {
            pink::ext().balance_of(account).0
        }

        #[ink(message)]
        fn free_balance_of(&self, account: AccountId) -> Balance {
            pink::ext().balance_of(account).1
        }
    }

    impl ContractDeposit for System {
        #[ink(message)]
        fn change_deposit(&mut self, contract_id: AccountId, deposit: Balance) -> Result<()> {
            self.ensure_self()?;
            let flags = ink_env::CallFlags::default().set_allow_reentry(true);
            match ContractDepositRef::instance_with_call_flags(flags) {
                Some(mut driver) => driver.change_deposit(contract_id, deposit),
                None => Ok(()),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;
        use pink::system::SystemRef;

        const OWNER: [u8; 32] = [2u8; 32];

        fn test_system() -> SystemRef {
            ink_env::test::set_caller::<PinkEnvironment>(OWNER.into());
            SystemRef::mock_with(System::default());
            SystemRef::instance()
        }

        #[ink::test]
        fn grant_admin_permissions() {
            let mut system = test_system();
            // The generated SystemRef would set current callee as caller before forwarding the call
            ink_env::test::set_callee::<PinkEnvironment>(OWNER.into());
            assert_eq!(system.grant_admin([42u8; 32].into()), Ok(()));

            ink_env::test::set_callee::<PinkEnvironment>([42u8; 32].into());
            assert_eq!(system.grant_admin([43u8; 32].into()), Err(Error::BadOrigin));
            assert_eq!(system.set_driver("Test".into(), Default::default()), Ok(()));

            ink_env::test::set_callee::<PinkEnvironment>([43u8; 32].into());
            assert_eq!(
                system.set_driver("Test".into(), Default::default()),
                Err(Error::BadOrigin)
            );
        }

        #[ink::test]
        fn set_driver_permissions() {
            let driver_name = "Hello";
            let mut system = test_system();
            ink_env::test::set_callee::<PinkEnvironment>(OWNER.into());

            let driver = system.get_driver(driver_name.into());
            assert_eq!(driver, None);

            // The owner can set driver
            let driver_id = [1u8; 32].into();
            assert_eq!(system.set_driver(driver_name.into(), driver_id), Ok(()));
            let driver = system.get_driver(driver_name.into());
            assert_eq!(driver, Some(driver_id));

            // The others can not set driver
            ink_env::test::set_callee::<PinkEnvironment>([42u8; 32].into());
            assert_eq!(
                system.set_driver(driver_name.into(), [2u8; 32].into()),
                Err(Error::BadOrigin)
            );
            assert_eq!(driver, Some(driver_id));

            // The others can set driver after granted admin
            ink_env::test::set_callee::<PinkEnvironment>(OWNER.into());
            assert_eq!(system.grant_admin([42u8; 32].into()), Ok(()));
            ink_env::test::set_callee::<PinkEnvironment>([42u8; 32].into());
            assert_eq!(
                system.set_driver(driver_name.into(), [2u8; 32].into()),
                Ok(())
            );
        }

        #[ink::test]
        fn deploy_sidevm_permissions() {
            let mut system = test_system();
            ink_env::test::set_callee::<PinkEnvironment>(OWNER.into());

            // The owner can not deploy a sidevm
            assert_eq!(
                system.deploy_sidevm_to(Default::default(), Default::default()),
                Err(Error::BadOrigin)
            );

            assert_eq!(system.grant_admin(OWNER.into()), Ok(()));

            // The owner can deploy after grant the permision to itself
            assert_eq!(
                system.deploy_sidevm_to(Default::default(), Default::default()),
                Ok(())
            );
        }
    }
}
