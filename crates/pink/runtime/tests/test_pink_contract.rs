use assert_matches::assert_matches;
use phala_types::contract::ConvertTo;
use pink_capi::{
    types::{AccountId, Weight},
    v1::ecall::{ECalls, TransactionArguments},
};
use pink_extension::system::System;
use pink_extension_runtime::local_cache;
use sp_core::Pair;
use sp_runtime::AccountId32;

use helpers::{checker_wasm, create_cluster, deploy_checker, TestCluster, ALICE, ENOUGH};
use scale::{Decode, Encode};

use rusty_fork::rusty_fork_test;

use crate::helpers::ink_helpers::Callable;

use ink::{codegen::TraitCallBuilder as _, ToAccountId as _};

mod helpers;

fn tx_args() -> TransactionArguments {
    TransactionArguments {
        origin: ALICE.clone(),
        transfer: 0,
        gas_limit: Weight::MAX,
        gas_free: true,
        storage_deposit_limit: None,
        deposit: 0,
    }
}

fn alice() -> AccountId {
    ALICE.clone()
}

#[test]
fn test_ink_flip() {
    let mut cluster = TestCluster::for_test();

    let code_hash = cluster
        .tx()
        .upload_code(
            alice(),
            include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
            true,
        )
        .unwrap();

    let result = cluster
        .tx()
        .instantiate(code_hash, 0x9bae9d5e, true, vec![], tx_args());

    insta::assert_debug_snapshot!(result);
    insta::assert_debug_snapshot!(cluster.effects.take());
    let contract = result.result.unwrap().account_id;

    let result = cluster.tx().call_typed::<_, bool>(
        &contract,
        0x2f865bd9, // get
        (),
        tx_args(),
    );
    assert_matches!(result, Ok(true));

    let _ = cluster.estimate().call_typed::<_, ()>(
        &contract,
        0x633aa551, // flip
        (),
        tx_args(),
    );
    let result = cluster.tx().call_typed::<_, ()>(
        &contract,
        0x633aa551, // flip
        (),
        tx_args(),
    );
    assert_matches!(result, Ok(()));

    let result = cluster.tx().call_typed::<_, bool>(
        &contract,
        0x2f865bd9, // get
        (),
        tx_args(),
    );
    assert_matches!(result, Ok(false));

    let result = cluster.tx().call_typed::<_, (u32, u128)>(
        &contract,
        0xf7dff04c, // echo
        (42u32, 24u128),
        tx_args(),
    );
    assert_matches!(result, Ok((42, 24)));
}

#[test]
fn test_ink_cross_contract_instantiate() {
    let mut cluster = TestCluster::for_test();

    let _code_hash = cluster
        .tx()
        .upload_code(
            alice(),
            include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
            true,
        )
        .unwrap();

    let code_hash = cluster
        .tx()
        .upload_code(
            alice(),
            include_bytes!("./fixtures/cross/cross.wasm").to_vec(),
            true,
        )
        .unwrap();

    let mut args = tx_args();
    args.transfer = ENOUGH / 100;
    let result = cluster
        .tx()
        .instantiate(code_hash, 0x9bae9d5e, true, vec![], args);
    insta::assert_debug_snapshot!(result);
    insta::assert_debug_snapshot!(cluster.effects.take());
    let contract = result.result.unwrap().account_id;

    let result = cluster.tx().call_typed::<_, bool>(
        &contract,
        0xc3220014, // get
        (),
        tx_args(),
    );
    assert_matches!(result, Ok(true));
}

fn test_with_wasm(wasm: &[u8], constructor: u32, message: u32, query: bool) {
    let mut cluster = TestCluster::for_test();
    let mut cluster = if query { cluster.query() } else { cluster.tx() };

    let code_hash = cluster.upload_code(alice(), wasm.to_vec(), true).unwrap();

    let contract = cluster
        .instantiate_typed(code_hash, constructor, (), vec![], tx_args())
        .expect("Failed to instantiate contract");

    local_cache::apply_quotas([(contract.as_ref(), 1024)]);

    let result = cluster.call_typed::<_, ()>(&contract, message, (), tx_args());
    assert_matches!(result, Ok(()));
}

#[test]
fn test_signing() {
    test_with_wasm(
        include_bytes!("./fixtures/signing/signing.wasm"),
        0xed4b9d1b,
        0x928b2036,
        true,
    );
}

rusty_fork_test! {
    #[test]
    fn test_logging() {
        env_logger::init();
        test_with_wasm(
            include_bytes!("./fixtures/logging.wasm"),
            0xed4b9d1b,
            0x928b2036,
            false,
        );
    }

    #[test]
    fn test_use_cache_in_tx() {
        env_logger::init();
        test_with_wasm(
            include_bytes!("./fixtures/use_cache/use_cache.wasm"),
            0xed4b9d1b,
            0x7ec6db42,
            false,
        );
    }
}

#[test]
fn test_use_cache() {
    test_with_wasm(
        include_bytes!("./fixtures/use_cache/use_cache.wasm"),
        0xed4b9d1b,
        0x928b2036,
        true,
    );
}

#[test]
#[ignore = "for dev"]
fn test_qjs() {
    use scale::{Decode, Encode};

    env_logger::init();

    let mut cluster = TestCluster::for_test();

    let checker = [];
    let qjs = [];
    // let checker = include_bytes!("../../../../e2e/res/check_system/target/ink/check_system.wasm");
    // let qjs = include_bytes!("qjs.wasm");

    #[derive(Debug, Encode, Decode)]
    pub enum Output {
        String(String),
        Bytes(Vec<u8>),
        Undefined,
    }

    let checker_hash = cluster
        .tx()
        .upload_code(alice(), checker.to_vec(), true)
        .unwrap();
    let qjs_hash = cluster
        .tx()
        .upload_code(alice(), qjs.to_vec(), false)
        .unwrap();

    let contract = cluster
        .tx()
        .instantiate_typed(checker_hash, 0xed4b9d1b, (), vec![], tx_args())
        .expect("Failed to instantiate contract");

    local_cache::apply_quotas([(contract.as_ref(), 1024)]);

    let js = r#"
    (function(){
        console.log("Hello, World!");
        // return scriptArgs[1];
        return new Uint8Array([21, 31]);
    })()
    "#;
    let t0 = std::time::Instant::now();
    let args: Vec<String> = vec!["Hello".to_string(), "World".to_string()];
    type LangError = ();
    let result = cluster
        .query()
        .call_typed::<_, Result<Result<Output, String>, LangError>>(
            &contract,
            0xf32e54c5,
            (qjs_hash, js, args),
            tx_args(),
        )
        .expect("Failed to call contract");
    println!("evaluate result={result:?}, dt={:?}", t0.elapsed());
}

#[test]
fn test_bare_runtime() {
    let mut cluster = TestCluster::bare();
    let checker = deploy_checker(&mut cluster, ENOUGH);
    let is_tx = checker
        .call()
        .is_in_transaction()
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(is_tx)
}

#[test]
fn test_get_balance() {
    const TEST_ADDRESS: AccountId32 = AccountId32::new([255u8; 32]);

    let (mut cluster, checker) = create_cluster();

    let balance = 114514;

    cluster.tx().deposit(TEST_ADDRESS.clone(), balance);

    let result = checker
        .call()
        .direct_balance_of(TEST_ADDRESS.convert_to())
        .query(&mut cluster);
    assert!(result.is_err());

    let result = checker
        .call()
        .total_balance_of(TEST_ADDRESS.convert_to())
        .query(&mut cluster);
    assert_eq!(result.unwrap(), balance);
    let result = checker
        .call()
        .total_balance_of(TEST_ADDRESS.convert_to())
        .submit_tx(&mut cluster);
    assert_eq!(result.unwrap(), balance);
}

#[test]
fn test_set_cache_in_tx() {
    let (mut cluster, checker) = create_cluster();
    local_cache::apply_quotas([(checker.to_account_id().as_ref(), 1024)]);

    let ok = checker
        .call()
        .cache_set(b"foo".to_vec(), b"bar".to_vec())
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(ok);
    let value = checker
        .call()
        .cache_get(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(value, Some(b"bar".to_vec()));

    checker
        .call()
        .cache_set_expiration(b"foo".to_vec(), 0)
        .submit_tx(&mut cluster)
        .unwrap();
    let value = checker
        .call()
        .cache_get(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(value, None);

    checker
        .call()
        .cache_set(b"foo".to_vec(), b"baz".to_vec())
        .submit_tx(&mut cluster)
        .unwrap();
    let value = checker
        .call()
        .cache_get(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(value, Some(b"baz".to_vec()));
    checker
        .call()
        .cache_remove(b"foo".to_vec())
        .submit_tx(&mut cluster)
        .unwrap();
    let value = checker
        .call()
        .cache_get(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(value, None);
}

#[test]
fn test_set_cache_in_query() {
    let (mut cluster, checker) = create_cluster();
    local_cache::apply_quotas([(checker.to_account_id().as_ref(), 1024)]);

    let ok = checker
        .call()
        .cache_set(b"foo".to_vec(), b"bar".to_vec())
        .query(&mut cluster)
        .unwrap();
    assert!(ok);
    let value = checker
        .call()
        .cache_get(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(value, Some(b"bar".to_vec()));

    checker
        .call()
        .cache_set_expiration(b"foo".to_vec(), 0)
        .query(&mut cluster)
        .unwrap();
    let value = checker
        .call()
        .cache_get(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(value, None);

    checker
        .call()
        .cache_set(b"foo".to_vec(), b"baz".to_vec())
        .query(&mut cluster)
        .unwrap();
    let value = checker
        .call()
        .cache_get(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(value, Some(b"baz".to_vec()));
    checker
        .call()
        .cache_remove(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    let value = checker
        .call()
        .cache_get(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(value, None);
}

#[test]
fn test_signing2() {
    let (mut cluster, checker) = create_cluster();

    let key = checker
        .call()
        .sr25519_derive_key(b"foo".to_vec())
        .query(&mut cluster)
        .unwrap();
    let key_tx = checker
        .call()
        .sr25519_derive_key(b"foo".to_vec())
        .submit_tx(&mut cluster)
        .unwrap();
    assert_eq!(key, key_tx);

    let pubkey = checker
        .call()
        .sr25519_get_public_key(key.clone())
        .query(&mut cluster)
        .unwrap();
    let pubkey_tx = checker
        .call()
        .sr25519_get_public_key(key.clone())
        .submit_tx(&mut cluster)
        .unwrap();
    assert_eq!(pubkey, pubkey_tx);

    let message = b"hello world";

    let signature = checker
        .call()
        .sr25519_sign(message.to_vec(), key.clone())
        .query(&mut cluster)
        .unwrap();
    let signature_tx = checker
        .call()
        .sr25519_sign(message.to_vec(), key.clone())
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(signature_tx.is_empty());

    let pass = checker
        .call()
        .sr25519_verify(message.to_vec(), pubkey.clone(), signature.clone())
        .query(&mut cluster)
        .unwrap();
    assert!(pass);

    let pass_tx = checker
        .call()
        .sr25519_verify(message.to_vec(), pubkey.clone(), signature.clone())
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(pass_tx);
}

#[test]
fn test_http_req() {
    let (mut cluster, checker) = create_cluster();

    let (status, _) = checker
        .call()
        .http_get("https://example.com/".to_string())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(status, 200);

    let (status, _) = checker
        .call()
        .http_get("https://example.com/".to_string())
        .submit_tx(&mut cluster)
        .unwrap();
    assert_eq!(status, 523);

    let result = checker
        .call()
        .http_get("https://example.com/not_allowed".to_string())
        .query(&mut cluster);
    assert!(result.is_err());

    let (status, _) = checker
        .call()
        .http_get("https://example.com/404".to_string())
        .query(&mut cluster)
        .unwrap();
    assert_eq!(status, 404);
}

#[test]
fn test_batch_http_req() {
    let (mut cluster, checker) = create_cluster();

    let responses = checker
        .call()
        .batch_http_get(vec!["https://example.com/".to_string()], 1)
        .query(&mut cluster)
        .unwrap()
        .unwrap();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].0, 200);

    let result = checker
        .call()
        .batch_http_get(vec!["https://example.com/".to_string()], 1)
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(result.is_err());

    let result = checker
        .call()
        .batch_http_get(vec!["https://example.com/not_allowed".to_string()], 1)
        .submit_tx(&mut cluster)
        .unwrap();
    assert_eq!(result, Err("NotAllowed".to_string()));
}

#[test]
fn test_getrandom() {
    let (mut cluster, checker) = create_cluster();

    let result = checker.call().getrandom(32).query(&mut cluster).unwrap();
    assert_eq!(result.len(), 32);
    assert!(result != vec![0u8; 32]);

    let result = checker
        .call()
        .getrandom(32)
        .submit_tx(&mut cluster)
        .unwrap();
    assert_eq!(result, b"");
}

#[test]
fn test_ecdsa_signing() {
    let (mut cluster, checker) = create_cluster();
    let pair = sp_core::ecdsa::Pair::from_string("//Alice", None).unwrap();
    let secret = pair.seed().to_vec();
    let pubkey = pair.public().0;

    let message = b"hello world";
    let message_hash = sp_core::hashing::keccak_256(message);

    let signature = checker
        .call()
        .ecdsa_sign_prehashed(secret.clone(), message_hash)
        .query(&mut cluster)
        .unwrap();

    let signature_tx = checker
        .call()
        .ecdsa_sign_prehashed(secret.clone(), message_hash)
        .submit_tx(&mut cluster)
        .unwrap();

    assert_eq!(signature, signature_tx);

    let pass = checker
        .call()
        .ecdsa_verify_prehashed(signature, message_hash, pubkey)
        .query(&mut cluster)
        .unwrap();
    assert!(pass);

    let pass_tx = checker
        .call()
        .ecdsa_verify_prehashed(signature, message_hash, pubkey)
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(pass_tx);
}

#[test]
fn test_is_in_transaction() {
    let (mut cluster, checker) = create_cluster();

    let result = checker
        .call()
        .is_in_transaction()
        .query(&mut cluster)
        .unwrap();
    assert!(!result);

    let result = checker
        .call()
        .is_in_transaction()
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(result);
}

#[test]
fn test_upgrade_system() {
    let (mut cluster, _checker) = create_cluster();

    let mut system = cluster.system();
    system
        .call_mut()
        .upgrade_system_contract()
        .submit_tx(&mut cluster)
        .unwrap()
        .unwrap();

    let version = system.call().version().query(&mut cluster).unwrap();
    assert_eq!(version, (1, 0xffff, 0));
}

#[test]
fn test_upgrade_runtime() {
    let (mut cluster, checker) = create_cluster();

    let mut system = cluster.system();
    let result = system
        .call_mut()
        .upgrade_runtime((1, 2))
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(result.is_ok());

    let version = checker
        .call()
        .runtime_version()
        .query(&mut cluster)
        .unwrap();

    assert_eq!(version.0, 1);
}

#[test]
fn test_code_exists() {
    let (mut cluster, checker) = create_cluster();

    let result = checker
        .call()
        .code_exists([0; 32], false)
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(!result);

    let result = checker
        .call()
        .code_exists([0; 32], true)
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(!result);
}

#[test]
fn test_worker_pubkey() {
    let (mut cluster, checker) = create_cluster();

    let result = checker.call().worker_pubkey().query(&mut cluster).unwrap();
    assert_eq!(result, cluster.worker_pubkey);

    let result = checker
        .call()
        .worker_pubkey()
        .submit_tx(&mut cluster)
        .unwrap();
    assert_eq!(result, [0u8; 32]);
}

#[test]
fn test_timestamp() {
    let (mut cluster, checker) = create_cluster();

    let result = checker.call().timestamp().query(&mut cluster).unwrap();
    assert!(result > 0);

    let result = checker.call().timestamp().submit_tx(&mut cluster).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_event_chain_head() {
    let (mut cluster, checker) = create_cluster();

    let result = checker
        .call()
        .event_chain_head()
        .query(&mut cluster)
        .unwrap();
    assert!(result > 0);

    let result = checker
        .call()
        .event_chain_head()
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(result > 0);
}

#[test]
fn test_pink_js_eval() {
    use pink_extension::chain_extension::JsValue;

    let (mut cluster, checker) = create_cluster();

    let result = checker
        .call()
        .pink_eval_js("1 + 2".to_string(), vec![])
        .query(&mut cluster)
        .unwrap();
    assert_eq!(result, JsValue::Exception("Not implemented".to_string()));

    let result = checker
        .call()
        .pink_eval_js("1 + 2".to_string(), vec![])
        .submit_tx(&mut cluster)
        .unwrap();
    assert_eq!(
        result,
        JsValue::Exception("Js evaluation is not supported in transaction".to_string())
    );
}

#[test]
fn can_emit_ink_events() {
    let (mut cluster, _checker) = create_cluster();

    let mut system = cluster.system();
    let res = system
        .call_mut()
        .set_driver("Hello".to_string(), [0; 32].into())
        .submit_tx(&mut cluster)
        .unwrap();
    assert!(res.is_ok());
    assert!(cluster.n_ink_events > 0);
}

rusty_fork_test! {
    #[test]
    fn test_for_errors() {
        env_logger::init();

        let mut cluster = TestCluster::bare();
        let checker = deploy_checker(&mut cluster, ENOUGH);

        // UnknownChainExtensionFunction
        let result = checker.call().call_chain_ext(0xffff).query(&mut cluster);
        let result_tx = checker.call().call_chain_ext(0xffff).submit_tx(&mut cluster);
        assert_eq!(format!("{result:?}"), "Err(Failed to execute call: Module(ModuleError { index: 5, error: [1, 0, 0, 0], message: None }))");
        assert_eq!(format!("{result_tx:?}"), "Err(Failed to execute call: Module(ModuleError { index: 5, error: [1, 0, 0, 0], message: None }))");

        // UnknownChainExtensionId
        let result = checker.call().call_chain_ext(0x010001).query(&mut cluster);
        assert_eq!(format!("{result:?}"), "Err(Failed to execute call: Module(ModuleError { index: 5, error: [0, 0, 0, 0], message: None }))");

        // ContractIoBufferOverflow
        let result = checker
            .call()
            .http_get("https://example.com/large".to_string())
            .query(&mut cluster);
        assert_eq!(format!("{result:?}"), "Err(Failed to execute call: Module(ModuleError { index: 5, error: [2, 0, 0, 0], message: None }))");

        // KeySeedMissing
        let result = checker
            .call()
            .sr25519_derive_key(b"foo".to_vec())
            .query(&mut cluster);
        assert_eq!(format!("{result:?}"), "Err(Failed to execute call: Module(ModuleError { index: 5, error: [3, 0, 0, 0], message: None }))");

        // SystemContractMissing
        let result = checker.call().system_contract_version().query(&mut cluster);
        assert_eq!(format!("{result:?}"), "Err(Failed to execute call: Module(ModuleError { index: 5, error: [5, 0, 0, 0], message: None }))");
    }
}

#[test]
fn test_init_failure() {
    use helpers::ink_helpers::BareDeployWasm;
    use helpers::CheckSystemRef;

    let mut cluster = TestCluster::bare();
    cluster.tx().deposit(ALICE.clone(), ENOUGH);

    let checker_wasm = checker_wasm();

    let result = CheckSystemRef::init_fail()
        .bare_deploy(checker_wasm, &mut cluster)
        .unwrap()
        .result
        .unwrap();
    assert!(result.result.did_revert());

    let result = CheckSystemRef::init_panic()
        .bare_deploy(checker_wasm, &mut cluster)
        .unwrap()
        .result;
    assert_eq!(
        format!("{result:?}"),
        "Err(Module(ModuleError { index: 4, error: [12, 0, 0, 0], message: None }))"
    );
}

#[test]
fn test_cov_log() {
    let (mut cluster, checker) = create_cluster();

    checker
        .call()
        .log("Hello, World!".to_string())
        .query(&mut cluster)
        .unwrap();
}

#[test]
fn test_some_other_ecalls() {
    use ink::ToAccountId as _;

    let (mut cluster, checker) = create_cluster();

    let cluster_id = cluster.query().cluster_id();
    assert_eq!(cluster_id, sp_core::H256::default());

    let git_rev = cluster.query().git_revision();
    assert_eq!(git_rev.len(), 46);

    let code = cluster.query().get_sidevm_code([0u8; 32].into());
    assert!(code.is_none());

    let free_balance = cluster.query().free_balance(ALICE.clone());
    assert!(free_balance > 0);
    let total_balance = cluster.query().total_balance(ALICE.clone());
    assert!(total_balance > 0);

    let code_hash = cluster
        .query()
        .code_hash(checker.to_account_id().convert_to());
    assert!(code_hash.is_some());
}
