#![cfg_attr(not(feature = "std"), no_std)]

use pink_extension as pink;

#[pink::contract(env = PinkEnvironment)]
mod tokenomic {
    use super::pink;
    use pink::system::{ContractDeposit, Error, Result, SystemRef};
    use pink::PinkEnvironment;

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
    }

    impl ContractDeposit for PhatTokenomic {
        #[ink(message)]
        fn change_deposit(&mut self, contract_id: AccountId, deposit: Balance) -> Result<()> {
            self.ensure_system()?;
            const CENTS: Balance = 10_000_000_000;
            let system = SystemRef::instance();
            let weight = deposit / CENTS;
            system.set_contract_weight(contract_id, weight.try_into().unwrap_or(u32::MAX))
        }
    }
}
