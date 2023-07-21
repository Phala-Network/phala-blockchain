#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pink_extension as pink;

#[pink::contract(env = PinkEnvironment)]
mod tokenomic {
    use super::pink;
    use pink::system::{ContractDeposit, DriverError as Error, SystemRef};
    use pink::PinkEnvironment;

    type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct WeightChanged {
        #[ink(topic)]
        contract_id: AccountId,
        weight: u32,
    }

    #[ink(storage)]
    pub struct PhatTokenomic {}

    impl PhatTokenomic {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        fn ensure_system(&self) -> Result<AccountId> {
            let caller = self.env().caller();
            if caller == pink::ext().system_contract_id() {
                return Ok(caller);
            }
            Err(Error::BadOrigin)
        }

        #[ink(message)]
        pub fn version(&self) -> this_crate::VersionTuple {
            this_crate::version_tuple!()
        }
    }

    impl ContractDeposit for PhatTokenomic {
        #[ink(message)]
        fn change_deposit(&mut self, contract_id: AccountId, deposit: Balance) -> Result<()> {
            self.ensure_system()?;
            const CENTS: Balance = 10_000_000_000;
            let system = SystemRef::instance();
            let weight = deposit / CENTS;
            let weight = weight.try_into().unwrap_or(u32::MAX);
            system.set_contract_weight(contract_id.clone(), weight)?;
            self.env().emit_event(WeightChanged {
                contract_id,
                weight
            });
            Ok(())
        }
    }
}
