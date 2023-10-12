#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use pink_extension as pink;

#[pink::contract(env = PinkEnvironment)]
mod check_system {
    use super::pink;
    use alloc::vec::Vec;
    use pink::system::{ContractDeposit, DriverError, Result, SystemRef};
    use pink::PinkEnvironment;

    use phat_js as js;
    use alloc::string::String;
    use indeterministic_functions::Usd;

    #[ink(storage)]
    pub struct CheckSystem {
        on_block_end_called: bool,
    }

    impl CheckSystem {
        #[ink(constructor)]
        #[allow(clippy::should_implement_trait)]
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
                .returns::<Option<Usd>>()
                .invoke();
            pink::info!("parse_usd result: {result:?}");
            result
        }

        #[ink(message)]
        pub fn eval_js(
            &self,
            delegate: Hash,
            script: String,
            args: Vec<String>,
        ) -> Result<js::Output, String> {
            js::eval_with(delegate, &script, &args)
        }

        #[ink(message)]
        pub fn eval_js_bytecode(
            &self,
            delegate: Hash,
            script: Vec<u8>,
            args: Vec<String>,
        ) -> Result<js::Output, String> {
            js::eval_bytecode_with(delegate, &script, &args)
        }

        #[ink(message)]
        pub fn runtime_version(&self) -> (u32, u32) {
            pink::ext().runtime_version()
        }

        #[ink(message)]
        pub fn batch_http_get(&self, urls: Vec<String>, timeout_ms: u64) -> Vec<(u16, String)> {
            pink::ext()
                .batch_http_request(
                    urls.into_iter()
                        .map(|url| pink::chain_extension::HttpRequest {
                            url,
                            method: "GET".into(),
                            headers: Default::default(),
                            body: Default::default(),
                        })
                        .collect(),
                    timeout_ms,
                )
                .unwrap()
                .into_iter()
                .map(|result| match result {
                    Ok(response) => (
                        response.status_code,
                        String::from_utf8(response.body).unwrap_or_default(),
                    ),
                    Err(err) => (524, alloc::format!("Error: {err:?}")),
                })
                .collect()
        }
        #[ink(message)]
        pub fn http_get(&self, url: String) -> (u16, String) {
            let response = pink::ext().http_request(pink::chain_extension::HttpRequest {
                url,
                method: "GET".into(),
                headers: Default::default(),
                body: Default::default(),
            });
            (
                response.status_code,
                String::from_utf8(response.body).unwrap_or_default(),
            )
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
