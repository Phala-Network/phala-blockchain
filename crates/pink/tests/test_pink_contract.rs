use pink::Contract;
use sp_runtime::AccountId32;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

#[test]
fn test_ink_flip() {
    let mut contract = Contract::new_with_selector(
        ALICE.clone(),
        include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
        [0x9b, 0xae, 0x9d, 0x5e], // init_value
        true,
        vec![],
    )
    .unwrap();

    let result: bool = contract
        .call_with_selector(
            ALICE.clone(),
            [0x2f, 0x86, 0x5b, 0xd9], // get
            (),
        )
        .unwrap();

    assert_eq!(result, true); // Should equal to the init value

    let _: () = contract
        .call_with_selector(
            ALICE.clone(),
            [0x63, 0x3a, 0xa5, 0x51], // flip
            (),
        )
        .unwrap();

    let result: bool = contract
        .call_with_selector(
            ALICE.clone(),
            [0x2f, 0x86, 0x5b, 0xd9], // get
            (),
        )
        .unwrap();

    assert_eq!(result, false); // Should be flipped

    let result: (u32, u128) = contract
        .call_with_selector(
            ALICE.clone(),
            [0xf7, 0xdf, 0xf0, 0x4c], // echo
            (42u32, 24u128),
        )
        .unwrap();
    assert_eq!(result, (42, 24));
}
