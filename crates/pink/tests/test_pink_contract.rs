#![allow(clippy::let_unit_value)]

use frame_support::assert_ok;
use hex_literal::hex;
use pink::{
    local_cache,
    types::{Balance, Weight},
    Contract, Storage, TransactionArguments,
};
use sp_runtime::AccountId32;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const ENOUGH: Balance = u128::MAX / 2;

fn tx_args(storage: &mut Storage) -> TransactionArguments {
    TransactionArguments {
        origin: ALICE.clone(),
        now: 1,
        block_number: 1,
        storage,
        transfer: 0,
        gas_limit: Weight::MAX,
        gas_free: true,
        storage_deposit_limit: None,
        callbacks: None,
    }
}

#[test]
fn test_ink_flip() {
    let mut storage = Storage::default();
    storage.deposit(&ALICE, ENOUGH);

    let code_hash = storage
        .upload_code(
            &ALICE,
            include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
            true,
        )
        .unwrap();
    let contract = Contract::new_with_selector(
        code_hash,
        hex!("9bae9d5e"), // init_value
        true,
        vec![],
        tx_args(&mut storage),
    )
    .unwrap()
    .0;

    let result: bool = contract
        .call_with_selector(
            hex!("2f865bd9"), // get
            (),
            false,
            tx_args(&mut storage),
        )
        .0
        .unwrap();

    assert!(result); // Should equal to the init value

    let _: () = contract
        .call_with_selector(
            hex!("633aa551"), // flip
            (),
            false,
            tx_args(&mut storage),
        )
        .0
        .unwrap();

    let result: bool = contract
        .call_with_selector(
            hex!("2f865bd9"), // get
            (),
            false,
            tx_args(&mut storage),
        )
        .0
        .unwrap();

    assert!(!result); // Should be flipped

    let result: (u32, u128) = contract
        .call_with_selector(
            hex!("f7dff04c"), // echo
            (42u32, 24u128),
            false,
            tx_args(&mut storage),
        )
        .0
        .unwrap();
    assert_eq!(result, (42, 24));
}

#[test]
fn test_load_contract_file() {
    assert_ok!(pink::ContractFile::load(include_bytes!(
        "./fixtures/flip/flip.contract"
    )));
}

#[test]
fn test_ink_cross_contract_instanciate() {
    let mut storage = Storage::default();
    storage.deposit(&ALICE, ENOUGH);

    let code_hash = storage
        .upload_code(
            &ALICE,
            include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
            true,
        )
        .unwrap();
    let _flip = Contract::new_with_selector(
        code_hash,
        hex!("9bae9d5e"), // init_value
        true,
        vec![],
        tx_args(&mut storage),
    )
    .unwrap();

    let code_hash = storage
        .upload_code(
            &ALICE,
            include_bytes!("./fixtures/cross/cross.wasm").to_vec(),
            true,
        )
        .unwrap();
    let mut args = tx_args(&mut storage);
    args.transfer = ENOUGH / 100;
    let contract = Contract::new_with_selector(code_hash, hex!("9bae9d5e"), (), vec![], args)
        .unwrap()
        .0;

    let result: bool = contract
        .call_with_selector(
            hex!("c3220014"), // get
            (),
            false,
            tx_args(&mut storage),
        )
        .0
        .unwrap();

    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_mq_egress() {
    let mut storage = Storage::default();
    storage.deposit(&ALICE, ENOUGH);

    let code_hash = storage
        .upload_code(
            &ALICE,
            include_bytes!("./fixtures/mqproxy/mqproxy.wasm").to_vec(),
            true,
        )
        .unwrap();
    let (contract, effects) = Contract::new_with_selector(
        code_hash,
        hex!("ed4b9d1b"), // init_value
        (),
        vec![],
        tx_args(&mut storage),
    )
    .unwrap();

    insta::assert_debug_snapshot!(effects);

    let effects = contract
        .call_with_selector::<()>(
            hex!("6495da7f"), // push_message
            (b"\x42\x42".to_vec(), b"\x24\x24".to_vec()),
            false,
            tx_args(&mut storage),
        )
        .1;
    insta::assert_debug_snapshot!(effects);

    let effects = contract
        .call_with_selector::<()>(
            hex!("d09d68e0"), // push_osp_message
            (b"\x42\x42".to_vec(), b"\x24\x24".to_vec(), Some([0u8; 32])),
            false,
            tx_args(&mut storage),
        )
        .1;
    insta::assert_debug_snapshot!(effects);
}

fn test_with_wasm(wasm: &[u8], constructor: [u8; 4], message: [u8; 4]) {
    let mut storage = Storage::default();
    storage.set_key_seed([1u8; 64]);
    storage.deposit(&ALICE, ENOUGH);
    let code_hash = storage.upload_code(&ALICE, wasm.to_vec(), true).unwrap();

    let (contract, _) =
        Contract::new_with_selector(code_hash, constructor, (), vec![], tx_args(&mut storage))
            .unwrap();

    local_cache::apply_quotas([(contract.address.as_ref(), 1024)]);

    let () = contract
        .call_with_selector(message, (), true, tx_args(&mut storage))
        .0
        .unwrap();
}

#[test]
fn test_signing() {
    test_with_wasm(
        include_bytes!("./fixtures/signing/signing.wasm"),
        hex!("ed4b9d1b"),
        hex!("928b2036"),
    );
}

#[test]
fn test_logging() {
    env_logger::init();
    test_with_wasm(
        include_bytes!("./fixtures/logging.wasm"),
        hex!("ed4b9d1b"),
        hex!("928b2036"),
    );
}

#[test]
fn test_use_cache() {
    test_with_wasm(
        include_bytes!("./fixtures/use_cache/use_cache.wasm"),
        hex!("ed4b9d1b"),
        hex!("928b2036"),
    );
}


#[test]
#[ignore = "for dev"]
fn test_qjs() {
    env_logger::init();

    let mut storage = Storage::default();
    storage.set_key_seed([1u8; 64]);
    storage.deposit(&ALICE, ENOUGH);
    // let checker = include_bytes!("../../../e2e/res/check_system/target/ink/check_system.wasm");
    // let qjs = include_bytes!("../../../e2e/res/qjs.wasm");
    let checker = [];
    let qjs = [];

    let checker_hash = storage.upload_code(&ALICE, checker.to_vec(), true).unwrap();
    let qjs_hash = storage.upload_code(&ALICE, qjs.to_vec(), false).unwrap();

    let (contract, _) = Contract::new_with_selector(
        checker_hash,
        0xed4b9d1b_u32.to_be_bytes(),
        (),
        vec![],
        tx_args(&mut storage),
    )
    .unwrap();

    local_cache::apply_quotas([(contract.address.as_ref(), 1024)]);

    let js = r#"
    (function(){
        console.log("Hello, World!");
        return "Powered by QuickJS in ink!";
    })()
    "#;
    let result: Result<String, String> = contract
        .call_with_selector(
            0xf32e54c5_u32.to_be_bytes(),
            (qjs_hash, js),
            true,
            tx_args(&mut storage),
        )
        .0
        .unwrap();
    println!("evaluate result={result:?}");
}
