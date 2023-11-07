#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use pink_extension as pink;

#[pink::contract(env = PinkEnvironment)]
mod check_system {
    use super::pink;
    use alloc::vec::Vec;
    use pink::system::{ContractDeposit, DriverError, Result, SystemRef};
    use pink::PinkEnvironment;

    use alloc::string::String;
    use indeterministic_functions::Usd;
    use phat_js as js;

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

        #[ink(message)]
        pub fn system_contract_version(&self) -> (u16, u16, u16) {
            pink::system::SystemRef::instance().version()
        }

        #[ink(message)]
        pub fn eval_javascript(
            &self,
            script: String,
            args: Vec<String>,
        ) -> Result<js::Output, String> {
            js::eval(&script, &args)
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

    #[cfg(test)]
    mod tests {
        use drink::session::Session;
        use drink_pink_runtime::{ExecMode, PinkRuntime};
        use ink::codegen::TraitCallBuilder;

        use super::test_helper::{call, deploy_bundle};
        use super::CheckSystemRef;

        #[drink::contract_bundle_provider]
        enum BundleProvider {}

        #[drink::test]
        fn it_works() -> Result<(), Box<dyn std::error::Error>> {
            let mut session = Session::<PinkRuntime>::new()?;
            session.execute_with(|| {
                PinkRuntime::setup_cluster().expect("Failed to setup cluster");
            });

            let checker_bundle = BundleProvider::local()?;
            let checker = deploy_bundle(
                &mut session,
                checker_bundle,
                CheckSystemRef::default().salt_bytes(vec![]),
            )?;
            let ver = call(
                &mut session,
                checker.call().system_contract_version(),
                false,
            )?;
            assert_eq!(ver, (1, 0, 0));

            let eval_result = call(
                &mut session,
                checker.call().eval_javascript("'Hello'".into(), vec![]),
                false,
            )?;
            assert_eq!(eval_result, Ok(phat_js::Output::String("Hello".into())));

            let (status, _body) = call(
                &mut session,
                checker.call().http_get("https://httpbin.org/get".into()),
                false,
            )?;
            assert_eq!(status, 200);

            PinkRuntime::execute_in_mode(ExecMode::Transaction, move || {
                let (status, _body) = call(
                    &mut session,
                    checker.call().http_get("https://httpbin.org/get".into()),
                    false,
                )?;
                assert_eq!(status, 523);
                Ok(())
            })
        }
    }

    #[cfg(test)]
    mod test_helper {
        use ::ink::{
            env::{
                call::{
                    utils::{ReturnType, Set, Unset},
                    Call, CallBuilder, CreateBuilder, ExecutionInput, FromAccountId,
                },
                Environment,
            },
            primitives::Hash,
        };
        use drink::{errors::MessageResult, runtime::Runtime, session::Session, ContractBundle};
        use drink_pink_runtime::PinkRuntime;
        use pink_extension::{Balance, ConvertTo};
        use scale::{Decode, Encode};

        const DEFAULT_GAS_LIMIT: u64 = 1_000_000_000_000_000;

        pub fn deploy_bundle<Env, Contract, Args>(
            session: &mut Session<PinkRuntime>,
            bundle: ContractBundle,
            constructor: CreateBuilder<
                Env,
                Contract,
                Unset<Hash>,
                Unset<u64>,
                Unset<Balance>,
                Set<ExecutionInput<Args>>,
                Set<Vec<u8>>,
                Set<ReturnType<Contract>>,
            >,
        ) -> Result<Contract, String>
        where
            Env: Environment<Hash = Hash, Balance = Balance>,
            Contract: FromAccountId<Env>,
            Args: Encode,
            Env::AccountId: From<[u8; 32]>,
        {
            session.execute_with(move || {
                let caller = PinkRuntime::default_actor();
                let code_hash = PinkRuntime::upload_code(caller.clone(), bundle.wasm, true)?;
                let constructor = constructor
                    .code_hash(code_hash.0.into())
                    .endowment(0)
                    .gas_limit(DEFAULT_GAS_LIMIT);
                let params = constructor.params();
                let input_data = params.exec_input().encode();
                let account_id = PinkRuntime::instantiate(
                    caller,
                    0,
                    params.gas_limit(),
                    None,
                    code_hash,
                    input_data,
                    params.salt_bytes().clone(),
                )?;
                Ok(Contract::from_account_id(account_id.convert_to()))
            })
        }

        pub fn call<Env, Args, Ret>(
            session: &mut Session<PinkRuntime>,
            call_builder: CallBuilder<
                Env,
                Set<Call<Env>>,
                Set<ExecutionInput<Args>>,
                Set<ReturnType<Ret>>,
            >,
            deterministic: bool,
        ) -> Result<Ret, String>
        where
            Env: Environment<Hash = Hash, Balance = Balance>,
            Args: Encode,
            Ret: Decode,
        {
            session.execute_with(move || {
                let origin = PinkRuntime::default_actor();
                let params = call_builder.params();
                let data = params.exec_input().encode();
                let callee = params.callee();
                let address: [u8; 32] = callee.as_ref().try_into().or(Err("Invalid callee"))?;
                let result = PinkRuntime::call(
                    origin,
                    address.into(),
                    0,
                    DEFAULT_GAS_LIMIT,
                    None,
                    data,
                    deterministic,
                )?;
                let ret = MessageResult::<Ret>::decode(&mut &result[..])
                    .map_err(|e| format!("Failed to decode result: {}", e))?
                    .map_err(|e| format!("Failed to execute call: {}", e))?;
                Ok(ret)
            })
        }
    }
}
