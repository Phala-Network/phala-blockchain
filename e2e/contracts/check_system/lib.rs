#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub use check_system::*;

#[ink::contract(env = PinkEnvironment)]
mod check_system {
    use alloc::vec::Vec;
    use pink::chain_extension::{signing as sig, JsCode, JsValue};
    use pink::system::{ContractDeposit, DriverError, Result, SystemRef};
    use pink::{PinkEnvironment, WorkerId};

    use alloc::string::{String, ToString};
    use indeterministic_functions::Usd;
    use phat_js as js;

    #[ink(storage)]
    pub struct CheckSystem {
        on_block_end_called: bool,
        flag: String,
    }

    impl CheckSystem {
        #[ink(constructor)]
        #[allow(clippy::should_implement_trait)]
        pub fn default() -> Self {
            Self {
                on_block_end_called: false,
                flag: String::new(),
            }
        }

        #[ink(constructor)]
        pub fn init_fail() -> Result<Self, ()> {
            pink::info!("init_fail");
            ink::env::debug_println!("init_fail");
            Err(())
        }

        #[ink(constructor)]
        pub fn init_panic() -> Self {
            pink::info!("panic in constructor");
            panic!("panic in constructor")
        }

        #[ink(message)]
        pub fn on_block_end_called(&self) -> bool {
            self.on_block_end_called
        }

        #[ink(message)]
        pub fn set_flag(&mut self, flag: String) {
            self.flag = flag;
        }

        #[ink(message)]
        pub fn flag(&self) -> String {
            self.flag.clone()
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
        pub fn cache_set_expiration(&self, key: Vec<u8>, value: u64) {
            pink::ext().cache_set_expiration(&key, value)
        }

        #[ink(message)]
        pub fn cache_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
            pink::ext().cache_get(&key)
        }

        #[ink(message)]
        pub fn cache_remove(&self, key: Vec<u8>) -> Option<Vec<u8>> {
            pink::ext().cache_remove(&key)
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
        pub fn batch_http_get(
            &self,
            urls: Vec<String>,
            timeout_ms: u64,
        ) -> Result<Vec<(u16, String)>, String> {
            let responses = pink::ext()
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
                .map_err(|err| alloc::format!("{err:?}"))?
                .into_iter()
                .map(|result| match result {
                    Ok(response) => (
                        response.status_code,
                        String::from_utf8(response.body).unwrap_or_default(),
                    ),
                    Err(err) => (524, alloc::format!("Error: {err:?}")),
                })
                .collect();
            Ok(responses)
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
        pub fn stop_sidevm(&mut self) {
            pink::force_stop_sidevm()
        }

        #[ink(message)]
        pub fn deploy_paid_sidevm(
            &mut self,
            wokers: Vec<WorkerId>,
            ttl: u32,
            mem_pages: u32,
            pay: Balance,
        ) -> Result<(), pink::system::DriverError> {
            use pink::system::SidevmOperationRef;
            use pink::ResultExt;

            const HASH: [u8; 32] = *include_bytes!("./sideprog.wasm.hash");
            const CODE_LEN: usize = include_bytes!("./sideprog.wasm").len();
            let driver = SidevmOperationRef::instance()
                .ok_or(pink::system::Error::DriverNotFound)
                .log_err("SidevmDeployer not found")?;
            driver
                .set_value_transferred(pay)
                .deploy_to_workers(HASH, CODE_LEN as _, wokers, mem_pages, ttl)
                .log_err("Failed to deploy sidevm")
        }

        #[ink(message)]
        pub fn calc_paid_sidevm_price(
            &mut self,
            n_wokers: u32,
            mem_pages: u32,
        ) -> Result<Balance, pink::system::DriverError> {
            use pink::system::SidevmOperationRef;

            const CODE_LEN: usize = include_bytes!("./sideprog.wasm").len();
            let driver =
                SidevmOperationRef::instance().ok_or(pink::system::Error::DriverNotFound)?;
            driver.calc_price(CODE_LEN as _, mem_pages, n_wokers)
        }

        #[ink(message)]
        pub fn set_sidevm_deadline(
            &mut self,
            deadline: BlockNumber,
            pay: Balance,
        ) -> Result<(), pink::system::DriverError> {
            use pink::system::SidevmOperationRef;
            use pink::ResultExt;
            let driver =
                SidevmOperationRef::instance().ok_or(pink::system::Error::DriverNotFound)?;
            driver
                .set_value_transferred(pay)
                .update_deadline(deadline)
                .log_err("Failed to update deadline")
        }

        #[ink(message)]
        pub fn query_sidevm(&self, action: String) -> Result<Vec<u8>, String> {
            use sideabi::Request;

            let request = match action.as_str() {
                "ping" => Request::Ping,
                "callback" => Request::Callback {
                    call_data: ink::selector_bytes!("sidevm_callbak").to_vec(),
                },
                _ => return Err("Invalid action".into()),
            };
            let request = pink_json::to_vec(&request).map_err(|err| err.to_string())?;
            pink::query_local_sidevm(self.env().account_id(), request)
        }

        #[ink(message)]
        pub fn sidevm_callbak(&self) -> u8 {
            42
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

        #[ink(message)]
        pub fn pink_eval_js(&self, script: String, args: Vec<String>) -> JsValue {
            pink::ext().js_eval(alloc::vec![JsCode::Source(script)], args)
        }

        #[ink(message)]
        pub fn direct_balance_of(&self, address: AccountId) -> (Balance, Balance) {
            pink::ext().balance_of(address)
        }

        #[ink(message)]
        pub fn free_balance_of(&self, address: AccountId) -> Balance {
            pink::system::SystemRef::instance().free_balance_of(address)
        }

        #[ink(message)]
        pub fn total_balance_of(&self, address: AccountId) -> Balance {
            pink::system::SystemRef::instance().total_balance_of(address)
        }

        #[ink(message)]
        pub fn sr25519_derive_key(&self, salt: Vec<u8>) -> Vec<u8> {
            sig::derive_sr25519_key(&salt)
        }

        #[ink(message)]
        pub fn sr25519_get_public_key(&self, key: Vec<u8>) -> Vec<u8> {
            sig::get_public_key(&key, sig::SigType::Sr25519)
        }

        #[ink(message)]
        pub fn sr25519_sign(&self, message: Vec<u8>, key: Vec<u8>) -> Vec<u8> {
            sig::sign(&message, &key, sig::SigType::Sr25519)
        }

        #[ink(message)]
        pub fn sr25519_verify(
            &self,
            message: Vec<u8>,
            pubkey: Vec<u8>,
            signature: Vec<u8>,
        ) -> bool {
            sig::verify(&message, &pubkey, &signature, sig::SigType::Sr25519)
        }

        #[ink(message)]
        pub fn getrandom(&self, len: u8) -> Vec<u8> {
            pink::ext().getrandom(len)
        }

        #[ink(message)]
        pub fn is_in_transaction(&self) -> bool {
            pink::ext().is_in_transaction()
        }

        #[ink(message)]
        pub fn ecdsa_sign_prehashed(&self, key: Vec<u8>, message_hash: [u8; 32]) -> [u8; 65] {
            sig::ecdsa_sign_prehashed(&key, message_hash)
        }

        #[ink(message)]
        pub fn ecdsa_verify_prehashed(
            &self,
            signature: [u8; 65],
            message_hash: [u8; 32],
            pubkey: [u8; 33],
        ) -> bool {
            sig::ecdsa_verify_prehashed(signature, message_hash, pubkey)
        }

        #[ink(message)]
        pub fn timestamp(&self) -> u64 {
            pink::ext().untrusted_millis_since_unix_epoch()
        }

        #[ink(message)]
        pub fn worker_pubkey(&self) -> [u8; 32] {
            pink::ext().worker_pubkey()
        }

        #[ink(message)]
        pub fn code_exists(&self, code_hash: [u8; 32], is_sidevm: bool) -> bool {
            pink::ext().code_exists(code_hash, is_sidevm)
        }

        #[ink(message)]
        pub fn event_chain_head(&self) -> u64 {
            pink::ext().current_event_chain_head().0
        }

        #[ink(message)]
        pub fn call_chain_ext(&self, ext_id: u32) {
            ::ink::env::chain_extension::ChainExtensionMethod::build(ext_id)
                .input::<()>()
                .output::<(), false>()
                .ignore_error_code()
                .call(&());
        }

        #[ink(message)]
        pub fn log(&self, msg: String) {
            pink::info!("{msg}");
            ink::env::debug_println!("{msg}");
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
        use pink_drink::{Callable, DeployBundle, PinkRuntime};
        use ink::codegen::TraitCallBuilder;
        use pink::chain_extension::JsValue;

        use super::CheckSystemRef;

        #[drink::contract_bundle_provider]
        enum BundleProvider {}

        #[test]
        fn it_works() -> Result<(), Box<dyn std::error::Error>> {
            tracing_subscriber::fmt::init();
            let mut session = Session::<PinkRuntime>::new()?;
            let mut checker = CheckSystemRef::default()
                .deploy_bundle(&BundleProvider::local()?, &mut session)
                .expect("Failed to deploy checker contract");
            checker
                .call_mut()
                .set_flag("42".into())
                .submit_tx(&mut session)?;
            let flag = checker.call().flag().query(&mut session)?;
            assert_eq!(flag, "42");

            // Can eval js via pink extension
            let js_code = r#"
                async function main() {
                    const response = await fetch("https://httpbin.org/get");
                    const json = await response.json();
                    Sidevm.inspect(json);
                    return json.url;
                }
                main()
                    .then((v) => scriptOutput = v)
                    .catch(console.error)
                    .finally(() => process.exit(0))
            "#;
            let url = checker
                .call()
                .pink_eval_js(js_code.into(), vec![])
                .query(&mut session)?;
            assert_eq!(url, JsValue::String("https://httpbin.org/get".into()));
            Ok(())
        }
    }
}
