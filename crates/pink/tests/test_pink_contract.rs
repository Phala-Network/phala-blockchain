use frame_support::assert_ok;
use hex_literal::hex;
use pink::Contract;
use pink_extension::PinkEvent;
use sp_runtime::AccountId32;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

#[test]
fn test_ink_flip() {
    let mut storage = Contract::new_storage();
    let code_hash = storage
        .upload_code(
            ALICE.clone(),
            include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
        )
        .unwrap();
    let mut contract = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        code_hash,
        hex!("9bae9d5e"), // init_value
        true,
        vec![],
        vec![],
        0,
        0,
    )
    .unwrap()
    .0;

    let result: bool = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("2f865bd9"), // get
            (),
            false,
            0,
            0,
        )
        .unwrap()
        .0;

    assert_eq!(result, true); // Should equal to the init value

    let _: () = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("633aa551"), // flip
            (),
            false,
            0,
            0,
        )
        .unwrap()
        .0;

    let result: bool = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("2f865bd9"), // get
            (),
            false,
            0,
            0,
        )
        .unwrap()
        .0;

    assert_eq!(result, false); // Should be flipped

    let result: (u32, u128) = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("f7dff04c"), // echo
            (42u32, 24u128),
            false,
            0,
            0,
        )
        .unwrap()
        .0;
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
    let mut storage = Contract::new_storage();
    let code_hash = storage
        .upload_code(
            ALICE.clone(),
            include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
        )
        .unwrap();
    let _flip = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        code_hash,
        hex!("9bae9d5e"), // init_value
        true,
        vec![],
        vec![],
        0,
        0,
    )
    .unwrap();

    let code_hash = storage
        .upload_code(
            ALICE.clone(),
            include_bytes!("./fixtures/cross/cross.wasm").to_vec(),
        )
        .unwrap();
    let mut contract = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        code_hash,
        hex!("9bae9d5e"),
        (),
        vec![],
        vec![],
        0,
        0,
    )
    .unwrap()
    .0;

    let result: bool = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("c3220014"), // get
            (),
            false,
            0,
            0,
        )
        .unwrap()
        .0;

    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_mq_egress() {
    let mut storage = Contract::new_storage();
    let code_hash = storage
        .upload_code(
            ALICE.clone(),
            include_bytes!("./fixtures/mqproxy/mqproxy.wasm").to_vec(),
        )
        .unwrap();
    let (mut contract, effects) = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        code_hash,
        hex!("ed4b9d1b"), // init_value
        (),
        vec![],
        vec![],
        1,
        0,
    )
    .unwrap();

    insta::assert_debug_snapshot!(effects);

    let (_, effects): ((), _) = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("6495da7f"), // push_message
            (b"\x42\x42".to_vec(), b"\x24\x24".to_vec()),
            false,
            1,
            0,
        )
        .unwrap();
    insta::assert_debug_snapshot!(effects);

    let (_, effects): ((), _) = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("d09d68e0"), // push_osp_message
            (b"\x42\x42".to_vec(), b"\x24\x24".to_vec(), Some([0u8; 32])),
            false,
            1,
            0,
        )
        .unwrap();
    insta::assert_debug_snapshot!(effects);
}

#[test]
fn test_on_block_end() {
    let mut storage = Contract::new_storage();
    let code_hash = storage
        .upload_code(
            ALICE.clone(),
            include_bytes!("./fixtures/hooks_test/hooks_test.wasm").to_vec(),
        )
        .unwrap();
    let (mut contract, effects) = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        code_hash,
        hex!("ed4b9d1b"), // init_value
        (),
        vec![],
        vec![],
        1,
        0,
    )
    .unwrap();

    insta::assert_debug_snapshot!(contract);

    insta::assert_debug_snapshot!(effects);

    for (account, event) in effects.pink_events {
        if let PinkEvent::OnBlockEndSelector(selector) = event {
            if account == contract.address {
                contract.set_on_block_end_selector(selector);
            }
        }
    }

    let effects = contract.on_block_end(&mut storage, 1, 1).unwrap();

    insta::assert_debug_snapshot!(effects);
}

#[test]
fn test_signing() {
    let mut storage = Contract::new_storage();
    storage.set_key_seed([1u8; 64]);
    let code_hash = storage
        .upload_code(
            ALICE.clone(),
            include_bytes!("./fixtures/signing/signing.wasm").to_vec(),
        )
        .unwrap();

    let (mut contract, _) = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        code_hash,
        hex!("ed4b9d1b"),
        (),
        vec![],
        vec![],
        1,
        0,
    )
    .unwrap();

    let _: ((), _) = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("928b2036"),
            (),
            false,
            1,
            0,
        )
        .unwrap();
}
