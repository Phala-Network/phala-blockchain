// #![no_std]
mod patch;

use sgx_types::sgx_status_t;

#[no_mangle]
pub extern "C" fn ecall_init() -> sgx_status_t {
    let _ = std::io::stdout();
    let _ = std::io::stderr();
    let _ = std::io::stdin();
    println!("Hello world with std!");
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("info env logger initialized!");
    sgx_status_t::SGX_ERROR_AE_INVALID_EPIDBLOB
}

#[no_mangle]
pub extern "C" fn ecall_bench_run(index: u32) -> sgx_status_t {
    sgx_status_t::SGX_SUCCESS
}
