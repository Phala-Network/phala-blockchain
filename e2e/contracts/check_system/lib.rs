#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use pink_extension as pink;

#[pink::contract(env = PinkEnvironment)]
mod check_system {
    use super::pink;
    use alloc::vec::Vec;
    use pink::system::{ContractDeposit, DriverError, Result, SystemRef};
    use pink::PinkEnvironment;

    use crate::js;
    use alloc::string::String;
    use indeterministic_functions::Usd;

    #[ink(storage)]
    pub struct CheckSystem {
        on_block_end_called: bool,
    }

    impl CheckSystem {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                on_block_end_called: false,
            }
        }

        #[ink(message)]
        pub fn on_block_end_called(&self) -> bool {
            self.on_block_end_called
        }

        #[ink(message)]
        pub fn set_hook(&mut self, gas_limit: u64) {
            let mut system = pink::system::SystemRef::instance();
            _ = system.set_hook(
                pink::HookPoint::OnBlockEnd,
                self.env().account_id(),
                0x01,
                gas_limit,
            );
        }

        #[ink(message, selector = 0x01)]
        pub fn on_block_end(&mut self) {
            if self.env().caller() != self.env().account_id() {
                return;
            }
            self.on_block_end_called = true
        }

        #[ink(message)]
        pub fn start_sidevm(&self) -> bool {
            let hash = *include_bytes!("./sideprog.wasm.hash");
            let system = pink::system::SystemRef::instance();
            system
                .deploy_sidevm_to(self.env().account_id(), hash)
                .expect("Failed to deploy sidevm");
            true
        }

        #[ink(message)]
        pub fn cache_set(&self, key: Vec<u8>, value: Vec<u8>) -> bool {
            pink::ext().cache_set(&key, &value).is_ok()
        }

        #[ink(message)]
        pub fn cache_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
            pink::ext().cache_get(&key)
        }

        #[ink(message)]
        pub fn parse_usd(&self, delegate: Hash, json: String) -> Option<Usd> {
            // The ink sdk currently does not generate typed API for delegate calls. So we have to
            // use this low level approach to call `IndeterministicFunctions::parse_usd()`.
            use ink::env::call;
            let result = call::build_call::<PinkEnvironment>()
                .call_type(call::DelegateCall::new(delegate))
                .exec_input(
                    call::ExecutionInput::new(call::Selector::new(0xafead99e_u32.to_be_bytes()))
                        .push_arg(json),
                )
                .returns::<ink::MessageResult<Option<Usd>>>()
                .invoke();
            pink::info!("parse_usd result: {result:?}");
            result.unwrap()
        }

        #[ink(message)]
        pub fn eval_js(
            &self,
            delegate: Hash,
            script: String,
            args: Vec<String>,
        ) -> Result<js::Output, String> {
            js::eval(delegate, &script, args)
        }

        #[ink(message)]
        pub fn eval_js_bytecode(
            &self,
            delegate: Hash,
            script: Vec<u8>,
            args: Vec<String>,
        ) -> Result<js::Output, String> {
            js::eval_bytecode(delegate, script, args)
        }
    }

    impl ContractDeposit for CheckSystem {
        #[ink(message)]
        fn change_deposit(
            &mut self,
            contract_id: AccountId,
            deposit: Balance,
        ) -> Result<(), DriverError> {
            const CENTS: Balance = 10_000_000_000;
            let system = SystemRef::instance();
            let weight = deposit / CENTS;
            system.set_contract_weight(contract_id, weight as u32)?;
            Ok(())
        }
    }
}

// Haven't much thought on the API shape so far. So it isn't turned into a crate yet.
mod js {
    use super::*;
    use alloc::string::String;
    use alloc::vec::Vec;
    use ink::primitives::Hash;
    use scale::{Decode, Encode};

    #[derive(Debug, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Output {
        String(String),
        Bytes(Vec<u8>),
        Undefined,
    }

    pub fn eval(delegate: Hash, script: &str, args: Vec<String>) -> Result<Output, String> {
        use ink::env::call;

        let result = call::build_call::<pink::PinkEnvironment>()
            .call_type(call::DelegateCall::new(delegate))
            .exec_input(
                call::ExecutionInput::new(call::Selector::new(0x49bfcd24_u32.to_be_bytes()))
                    .push_arg(script)
                    .push_arg(args),
            )
            .returns::<ink::MessageResult<Result<Output, String>>>()
            .invoke();
        pink::info!("eval result: {result:?}");
        result.unwrap()
    }

    pub fn eval_bytecode(
        delegate: Hash,
        script: alloc::vec::Vec<u8>,
        args: Vec<String>,
    ) -> Result<Output, String> {
        use ink::env::call;

        let result = call::build_call::<pink::PinkEnvironment>()
            .call_type(call::DelegateCall::new(delegate))
            .exec_input(
                call::ExecutionInput::new(call::Selector::new(0xbf0ec203_u32.to_be_bytes()))
                    .push_arg(script)
                    .push_arg(args),
            )
            .returns::<ink::MessageResult<Result<Output, String>>>()
            .invoke();
        pink::info!("eval result: {result:?}");
        result.unwrap()
    }
}
