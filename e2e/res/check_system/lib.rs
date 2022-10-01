#![cfg_attr(not(feature = "std"), no_std)]

use pink_extension as pink;

#[pink::contract(env = PinkEnvironment)]
mod check_system {
    use super::pink;
    use pink::system::{ContractDeposit, Result, SystemRef};
    use pink::PinkEnvironment;

    #[ink(storage)]
    pub struct CheckSystem {
        on_block_end_called: bool,
    }

    impl CheckSystem {
        #[ink(constructor)]
        pub fn default() -> Self {
            let system = pink::system::SystemRef::instance();
            _ = system.get_driver("NotExists".into());
            Self {
                on_block_end_called: false,
            }
        }

        #[ink(message)]
        pub fn on_block_end_called(&self) -> bool {
            self.on_block_end_called
        }

        #[ink(message)]
        pub fn set_hook(&self) {
            let mut system = pink::system::SystemRef::instance();
            _ = system.set_hook(pink::HookPoint::OnBlockEnd, self.env().account_id(), 0x01);
        }

        #[ink(message, selector = 0x01)]
        pub fn on_block_end(&mut self) {
            if !pink::predefined_accounts::is_runtime(&self.env().caller()) {
                return;
            }
            self.on_block_end_called = true
        }
    }

    impl ContractDeposit for CheckSystem {
        #[ink(message)]
        fn change_deposit(&mut self, contract_id: AccountId, deposit: Balance) -> Result<()> {
            const CENTS: Balance = 10_000_000_000;
            let system = SystemRef::instance();
            let weight = deposit / CENTS;
            system.set_contract_weight(contract_id, weight as u32)
        }
    }
}
