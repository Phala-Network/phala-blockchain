#![feature(panic_unwind)]
#![feature(c_variadic)]

use core::sync::atomic::{AtomicU32, Ordering};

use log::{error, warn, info};

use sgx_types::sgx_status_t;
use sgx_tstd::sync::SgxMutex;

mod pal_sgx;
mod libc_hacks;

use pal_sgx::SgxPlatform;
use phactory::{benchmark, Phactory};


lazy_static::lazy_static! {
    static ref APPLICATION: SgxMutex<Phactory<SgxPlatform>> = SgxMutex::new(Phactory::new(SgxPlatform));
}


#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ecall_handle(
    action: u8,
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len_ptr: *mut usize,
    output_buf_len: usize,
) -> sgx_status_t {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };

    let mut factory = APPLICATION.lock().unwrap();
    let output = factory.handle_scale_api(action, input_slice);

    let output_len = output.len();
    if output_len <= output_buf_len {
        unsafe {
            core::ptr::copy_nonoverlapping(output.as_ptr(), output_ptr, output_len);
            *output_len_ptr = output_len;
            sgx_status_t::SGX_SUCCESS
        }
    } else {
        warn!("Too much output. Buffer overflow.");
        sgx_status_t::SGX_ERROR_FAAS_BUFFER_TOO_SHORT
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]

pub extern "C" fn ecall_init(args: *const u8, args_len: usize) -> sgx_status_t {
    static INITIALIZED: AtomicU32 = AtomicU32::new(0);
    if INITIALIZED.fetch_add(1, Ordering::SeqCst) != 0 {
        panic!("Enclave already initialized.");
    }

    use parity_scale_codec::Decode;
    let mut args_buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let args = match phactory_api::ecall_args::InitArgs::decode(&mut args_buf) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Decode args failed: {:?}", err);
            return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
        }
    };

    APPLICATION.lock().unwrap().set_sealing_path(args.sealing_path);

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&args.log_filter)).init();

    benchmark::reset_iteration_counter();

    info!("Enclave init OK");
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_bench_run(index: u32) -> sgx_status_t {
    if !benchmark::puasing() {
        info!("[{}] Benchmark thread started", index);
        benchmark::run();
    }
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ecall_prpc_request(
    path: *const u8,
    path_len: usize,
    data: *const u8,
    data_len: usize,
    status_code: *mut u16,
    output_ptr: *mut u8,
    output_buf_len: usize,
    output_len_ptr: *mut usize,
) -> sgx_status_t {
    let mut factory = APPLICATION.lock().unwrap();
    let (code, data) = factory.dispatch_prpc_request(path, path_len, data, data_len, output_buf_len);
    let (code, data) = if data.len() > output_buf_len {
        error!("ecall_prpc_request: output buffer too short");
        (500, vec![])
    } else {
        (code, data)
    };
    info!("pRPC status code: {}, data len: {}", code, data.len());
    unsafe {
        *status_code = code;
        let len = data.len().min(output_buf_len);
        core::ptr::copy_nonoverlapping(data.as_ptr(), output_ptr, len);
        *output_len_ptr = len;
    }
    sgx_status_t::SGX_SUCCESS
}
