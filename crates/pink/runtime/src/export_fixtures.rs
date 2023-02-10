pub fn load_test_wasm(name: &str) -> Vec<u8> {
    match name {
        "hooks_test" => include_bytes!("../tests/fixtures/hooks_test/hooks_test.wasm").to_vec(),
        _ => panic!("{name} not found"),
    }
}
