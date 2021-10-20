use frame_support::assert_ok;
use hex_literal::hex;
use pink::Contract;
use sp_runtime::AccountId32;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

#[test]
fn test_ink_flip() {
    let mut storage = Contract::new_storage();
    let mut contract = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
        hex!("9bae9d5e"), // init_value
        true,
        vec![],
    )
    .unwrap();

    let result: bool = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("2f865bd9"), // get
            (),
            false,
        )
        .unwrap();

    assert_eq!(result, true); // Should equal to the init value

    let _: () = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("633aa551"), // flip
            (),
            false,
        )
        .unwrap();

    let result: bool = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("2f865bd9"), // get
            (),
            false,
        )
        .unwrap();

    assert_eq!(result, false); // Should be flipped

    let result: (u32, u128) = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("f7dff04c"), // echo
            (42u32, 24u128),
            false,
        )
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
    let mut storage = Contract::new_storage();
    let _flip = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
        hex!("9bae9d5e"), // init_value
        true,
        vec![],
    )
    .unwrap();

    let mut contract = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        include_bytes!("./fixtures/cross/cross.wasm").to_vec(),
        hex!("9bae9d5e"),
        (),
        vec![],
    )
    .unwrap();

    let result: bool = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("c3220014"), // get
            (),
            false,
        )
        .unwrap();

    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_mq_egress() {
    let mut storage = Contract::new_storage();
    let mut contract = Contract::new_with_selector(
        &mut storage,
        ALICE.clone(),
        include_bytes!("./fixtures/mqproxy/mqproxy.wasm").to_vec(),
        hex!("ed4b9d1b"), // init_value
        (),
        vec![],
    )
    .unwrap();

    let _: () = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("6495da7f"), // push_message
            (b"\x42\x42".to_vec(), b"\x24\x24".to_vec()),
            false,
        )
        .unwrap();

    let _: () = contract
        .call_with_selector(
            &mut storage,
            ALICE.clone(),
            hex!("d09d68e0"), // push_osp_message
            (
                b"\x42\x42".to_vec(),
                b"\x24\x24".to_vec(),
                Some([0u8; 32]),
            ),
            false,
        )
        .unwrap();

    let messages = pink::runtime::take_mq_egress();
    insta::assert_debug_snapshot!(messages);
}
