#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use pink_extension as pink;

pub use system::System;

#[pink::contract(env = PinkEnvironment)]
mod system {
    use super::pink;
    use alloc::string::String;
    use alloc::vec::Vec;
    use ink::{codegen::Env, storage::Mapping};
    use pink::system::{CodeType, ContractDeposit, ContractDepositRef, DriverError, Error, Result};
    use pink::{HookPoint, PinkEnvironment};

    use this_crate::{version_tuple, VersionTuple};

    /// A new driver is set.
    #[ink(event)]
    pub struct DriverChanged {
        #[ink(topic)]
        name: String,
        previous: Option<AccountId>,
        current: AccountId,
    }

    /// A new administrator is added.
    #[ink(event)]
    pub struct AdministratorAdded {
        user: AccountId,
    }

    /// Pink's system contract.
    #[ink(storage)]
    pub struct System {
        /// The owner of the contract(the cluster).
        owner: AccountId,
        /// The administrators
        administrators: Mapping<AccountId, ()>,
        /// The drivers (deprecated)
        drivers: Mapping<String, AccountId>,
        /// The drivers
        drivers2: Mapping<String, (BlockNumber, AccountId)>,
        /// The history of drivers
        #[allow(clippy::type_complexity)]
        drivers_history: Mapping<String, Vec<(BlockNumber, AccountId)>>,
    }

    impl System {
        #[ink(constructor, selector = 0xed4b9d1b)]
        #[allow(clippy::should_implement_trait)]
        pub fn default() -> Self {
            Self {
                owner: Self::env().caller(),
                administrators: Default::default(),
                drivers: Default::default(),
                drivers2: Default::default(),
                drivers_history: Default::default(),
            }
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }
    }

    impl System {
        fn ensure_owner(&self) -> Result<AccountId> {
            let caller = self.env().caller();
            if caller == self.owner {
                Ok(caller)
            } else {
                Err(Error::PermisionDenied)
            }
        }

        fn ensure_admin(&self) -> Result<AccountId> {
            let caller = self.env().caller();
            if self.administrators.contains(caller) {
                return Ok(caller);
            }
            Err(Error::PermisionDenied)
        }

        fn ensure_owner_or_admin(&self) -> Result<AccountId> {
            self.ensure_owner().or_else(|_| self.ensure_admin())
        }

        fn ensure_self(&self) -> Result<AccountId> {
            let caller = self.env().caller();
            if caller == self.env().account_id() {
                Ok(caller)
            } else {
                Err(Error::PermisionDenied)
            }
        }

        fn ensure_min_runtime_version(&self, version: (u32, u32)) -> Result<()> {
            if pink::ext().runtime_version() >= version {
                Ok(())
            } else {
                Err(Error::ConditionNotMet)
            }
        }

        fn version(&self) -> VersionTuple {
            version_tuple!()
        }
    }

    impl pink::system::System for System {
        #[ink(message)]
        fn version(&self) -> VersionTuple {
            self.version()
        }

        #[ink(message)]
        fn grant_admin(&mut self, contract_id: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.administrators.insert(contract_id, &());
            self.env()
                .emit_event(AdministratorAdded { user: contract_id });
            Ok(())
        }

        #[ink(message)]
        fn set_driver(&mut self, name: String, contract_id: AccountId) -> Result<()> {
            self.ensure_owner_or_admin()?;
            #[allow(clippy::single_match)]
            match name.as_str() {
                "PinkLogger" => {
                    pink::set_log_handler(contract_id);
                }
                _ => {}
            }

            let previous = self.get_driver2(name.clone());
            if let Some((block, previous)) = previous {
                if previous == contract_id {
                    return Ok(());
                }
                let mut history = self.drivers_history.get(&name).unwrap_or_default();
                history.push((block, previous));
                self.drivers_history.insert(&name, &history);
            }
            self.drivers2
                .insert(&name, &(self.env().block_number(), contract_id));
            self.env().emit_event(DriverChanged {
                name,
                previous: previous.map(|(_, id)| id),
                current: contract_id,
            });
            Ok(())
        }

        #[ink(message)]
        fn get_driver(&self, name: String) -> Option<AccountId> {
            self.get_driver2(name).map(|(_, id)| id)
        }

        #[ink(message)]
        fn get_driver2(&self, name: String) -> Option<(BlockNumber, AccountId)> {
            self.drivers2
                .get(&name)
                .or_else(|| self.drivers.get(&name).map(|id| (0, id)))
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

        #[ink(message)]
        fn is_admin(&self, contract_id: AccountId) -> bool {
            self.administrators.contains(contract_id)
        }

        #[ink(message)]
        fn upgrade_system_contract(&mut self) -> Result<()> {
            let caller = self.ensure_owner()?;
            pink::info!("Upgrading system contract...");
            let Some(code_hash) = pink::ext().import_latest_system_code(caller) else {
                pink::error!("No new version of system contract found.");
                return Err(Error::CodeNotFound);
            };
            let my_code_hash = self
                .env()
                .code_hash(&self.env().account_id())
                .expect("Code hash should exists here.");
            if my_code_hash == code_hash.into() {
                pink::info!("No new version of system contract found.");
                return Ok(());
            }
            // Call the `do_upgrade` from the new version of system contract.
            ink::env::set_code_hash(&code_hash).expect("System code should exists here");
            let flags = ink::env::CallFlags::default().set_allow_reentry(true);
            pink::system::SystemRef::instance_with_call_flags(flags)
                .do_upgrade(self.version())
                // panic here to revert the state change.
                .expect("Failed to call do_upgrade on the new system code");
            pink::info!("System contract upgraded successfully.");
            Ok(())
        }

        #[ink(message)]
        fn do_upgrade(&self, from_version: VersionTuple) -> Result<()> {
            self.ensure_self()?;
            self.ensure_min_runtime_version((1, 0))?;
            if from_version >= self.version() {
                pink::error!("The system contract is already upgraded.");
                return Err(Error::ConditionNotMet);
            }
            Ok(())
        }

        /// Upgrade the contract runtime
        ///
        /// Be careful when using this function, it would panic the worker if the
        /// runtime version is not supported.
        #[ink(message)]
        fn upgrade_runtime(&mut self, version: (u32, u32)) -> Result<()> {
            let _ = self.ensure_owner()?;
            pink::info!("Upgrading pink contract runtime...");
            pink::upgrade_runtime(version);
            Ok(())
        }

        /// Check if the code is already uploaded to the cluster with given code hash.
        #[ink(message)]
        fn code_exists(&self, code_hash: [u8; 32], code_type: CodeType) -> bool {
            pink::ext().code_exists(code_hash, code_type.is_sidevm())
        }

        #[ink(message)]
        fn code_hash(&self, account: AccountId) -> Option<ink::primitives::Hash> {
            self.env().code_hash(&account).ok()
        }

        #[ink(message)]
        fn driver_history(&self, name: String) -> Option<Vec<(BlockNumber, AccountId)>> {
            self.drivers_history.get(name)
        }

        #[ink(message)]
        fn current_event_chain_head(&self) -> (u64, pink::Hash) {
            pink::ext().current_event_chain_head()
        }
    }

    impl ContractDeposit for System {
        #[ink(message)]
        fn change_deposit(
            &mut self,
            contract_id: AccountId,
            deposit: Balance,
        ) -> Result<(), DriverError> {
            self.ensure_self()?;
            let flags = ink::env::CallFlags::default().set_allow_reentry(true);
            match ContractDepositRef::instance_with_call_flags(flags) {
                Some(mut driver) => driver.change_deposit(contract_id, deposit),
                None => Ok(()),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use pink::system::SystemRef;

        const OWNER: [u8; 32] = [2u8; 32];

        fn test_system() -> SystemRef {
            ink::env::test::set_caller::<PinkEnvironment>(OWNER.into());
            SystemRef::mock_with(System::default());
            SystemRef::instance()
        }

        #[ink::test]
        fn grant_admin_permissions() {
            let mut system = test_system();
            // The generated SystemRef would set current callee as caller before forwarding the call
            ink::env::test::set_callee::<PinkEnvironment>(OWNER.into());
            assert_eq!(system.grant_admin([42u8; 32].into()), Ok(()));

            ink::env::test::set_callee::<PinkEnvironment>([42u8; 32].into());
            assert_eq!(system.grant_admin([43u8; 32].into()), Err(Error::BadOrigin));
            assert_eq!(system.set_driver("Test".into(), Default::default()), Ok(()));

            ink::env::test::set_callee::<PinkEnvironment>([43u8; 32].into());
            assert_eq!(
                system.set_driver("Test".into(), Default::default()),
                Err(Error::BadOrigin)
            );
        }

        #[ink::test]
        fn set_driver_permissions() {
            let driver_name = "Hello";
            let mut system = test_system();
            ink::env::test::set_callee::<PinkEnvironment>(OWNER.into());

            let driver = system.get_driver(driver_name.into());
            assert_eq!(driver, None);

            // The owner can set driver
            let driver_id = [1u8; 32].into();
            assert_eq!(system.set_driver(driver_name.into(), driver_id), Ok(()));
            let driver = system.get_driver(driver_name.into());
            assert_eq!(driver, Some(driver_id));

            // The others can not set driver
            ink::env::test::set_callee::<PinkEnvironment>([42u8; 32].into());
            assert_eq!(
                system.set_driver(driver_name.into(), [2u8; 32].into()),
                Err(Error::BadOrigin)
            );
            assert_eq!(driver, Some(driver_id));

            // The others can set driver after granted admin
            ink::env::test::set_callee::<PinkEnvironment>(OWNER.into());
            assert_eq!(system.grant_admin([42u8; 32].into()), Ok(()));
            ink::env::test::set_callee::<PinkEnvironment>([42u8; 32].into());
            assert_eq!(
                system.set_driver(driver_name.into(), [2u8; 32].into()),
                Ok(())
            );
        }

        #[ink::test]
        fn deploy_sidevm_permissions() {
            let mut system = test_system();
            ink::env::test::set_callee::<PinkEnvironment>(OWNER.into());

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
