use frame_support::assert_ok;
use pink::Contract;
use sp_runtime::AccountId32;
use hex_literal::hex;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

#[test]
fn test_ink_flip() {
    let mut contract = Contract::new_with_selector(
        ALICE.clone(),
        include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
        hex!("9bae9d5e"), // init_value
        true,
        vec![],
    )
    .unwrap();

    let result: bool = contract
        .call_with_selector(
            ALICE.clone(),
            hex!("2f865bd9"), // get
            (),
            false,
        )
        .unwrap();

    assert_eq!(result, true); // Should equal to the init value

    let _: () = contract
        .call_with_selector(
            ALICE.clone(),
            hex!("633aa551"), // flip
            (),
            false,
        )
        .unwrap();

    let result: bool = contract
        .call_with_selector(
            ALICE.clone(),
            hex!("2f865bd9"), // get
            (),
            false,
        )
        .unwrap();

    assert_eq!(result, false); // Should be flipped

    let result: (u32, u128) = contract
        .call_with_selector(
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
    assert_ok!(pink::ContractFile::load(include_bytes!("./fixtures/flip/flip.contract")));
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
