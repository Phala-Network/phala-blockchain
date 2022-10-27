use sgx_api_lite as sgx;

#[test]
#[ignore = "only works under sgx"]
fn it_works() {
    // In Enclave A
    let my_target_info = sgx::target_info().unwrap();
    let target_info_bytes = sgx::encode(&my_target_info);

    // In Enclave B
    let its_target_info = unsafe { sgx::decode(target_info_bytes).unwrap() };
    let report = sgx::report(its_target_info, &[0; 64]).unwrap();
    let report_bytes = sgx::encode(&report);

    // In Enclave A
    let recv_report = unsafe { sgx::decode(report_bytes).unwrap() };
    let rv = sgx::verify(recv_report);
    assert!(rv.is_ok());
    println!("It works!");
}
