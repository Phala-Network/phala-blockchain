#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use pink_extension as pink;

#[pink::contract(env = PinkEnvironment)]
mod sidevm_deployer {
    use super::pink;
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use pink::system::{Error, Result};
    use pink::PinkEnvironment;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct SidevmOp {
        /// Owner of the contract
        owner: AccountId,
        /// Contracts that are allowed to deploy sidevm.
        whitelist: Mapping<AccountId, ()>,
    }

    impl SidevmOp {
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::utils::initialize_contract(|me: &mut Self| {
                me.owner = Self::env().caller();
            })
        }
        #[ink(message)]
        pub fn allow(&mut self, contract: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::BadOrigin);
            }
            self.whitelist.insert(contract, &());
            Ok(())
        }
    }

    impl pink::system::SidevmOperation for SidevmOp {
        #[ink(message)]
        fn deploy(&self, code_hash: pink::Hash) -> Result<()> {
            let caller = self.env().caller();
            if !self.whitelist.contains(&caller) {
                return Err(Error::BadOrigin);
            }
            let system = pink::system::SystemRef::instance();
            system.deploy_sidevm_to(caller, code_hash)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;
        use pink_system::System;

        const SYSTEM_ADDR: [u8; 32] = [42u8; 32];
        const SIDEVMOP_ADDR: [u8; 32] = [24u8; 32];

        fn with_callee<T>(callee: [u8; 32], f: impl FnOnce() -> T) -> T {
            let prev = ink_env::test::callee::<PinkEnvironment>();
            ink_env::test::set_callee::<PinkEnvironment>(callee.into());
            let ret = f();
            ink_env::test::set_callee::<PinkEnvironment>(prev);
            ret
        }

        #[ink::test]
        fn should_forbid_non_admin_contract_to_deploy_sidevm() {
            use pink::system::{SidevmOperationRef, SystemRef};

            with_callee(SYSTEM_ADDR, || {
                SystemRef::mock_with(System::default());
            });

            with_callee(SIDEVMOP_ADDR, || {
                let mut sideman = SidevmOp::new();
                sideman
                    .allow([1u8; 32].into())
                    .expect("Failed to allow contract");
                SidevmOperationRef::mock_with(sideman);
            });
            let driver = SidevmOperationRef::instance().expect("Failed to get driver instance");

            let result = driver.deploy(Default::default());
            assert_eq!(result, Err(Error::BadOrigin));
        }

        #[ink::test]
        fn should_forbid_contract_not_in_whitelist() {
            use pink::system::{SidevmOperationRef, SystemRef};
            use pink_extension::system::System as _;
            with_callee(SYSTEM_ADDR, || {
                let mut system = System::default();
                system.grant_admin(SIDEVMOP_ADDR.into()).ok();
                SystemRef::mock_with(system);
            });

            with_callee(SIDEVMOP_ADDR, || {
                SidevmOperationRef::mock_with(SidevmOp::new());
            });
            let driver = SidevmOperationRef::instance().expect("Failed to get driver instance");
            let result = driver.deploy(Default::default());
            assert_eq!(result, Err(Error::BadOrigin));
        }

        #[ink::test]
        fn should_allow_contract_in_whitelist() {
            use pink::system::{SidevmOperationRef, SystemRef};
            use pink_extension::system::System as _;

            with_callee(SYSTEM_ADDR, || {
                let mut system = System::default();
                system.grant_admin(SIDEVMOP_ADDR.into()).ok();
                SystemRef::mock_with(system);
            });

            with_callee(SIDEVMOP_ADDR, || {
                let mut sideman = SidevmOp::new();
                sideman
                    .allow([1u8; 32].into())
                    .expect("Failed to allow contract");
                SidevmOperationRef::mock_with(sideman);
            });
            let driver = SidevmOperationRef::instance().expect("Failed to get driver instance");
            let result = driver.deploy(Default::default());
            assert_eq!(result, Ok(()));
        }
    }
}
