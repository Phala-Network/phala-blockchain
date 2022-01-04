#![feature(panic_unwind)]
#![feature(c_variadic)]

use core::sync::atomic::{AtomicU32, Ordering};
use parity_scale_codec::Decode;

use log::{error, info, warn};

use sgx_tstd::sync::SgxMutex;
use sgx_types::sgx_status_t;

mod libc_hacks;
mod pal_sgx;

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

    libc_hacks::init();

    let mut args_buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let args = match phactory_api::ecall_args::InitArgs::decode(&mut args_buf) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Decode args failed: {:?}", err);
            return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
        }
    };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&args.log_filter))
        .init();

    if args.enable_checkpoint {
        match Phactory::restore_from_checkpoint(&SgxPlatform, &args.sealing_path) {
            Ok(Some(mut factory)) => {
                info!("Loaded checkpoint");
                factory.set_args(args.clone());
                *APPLICATION.lock().unwrap() = factory;
                return sgx_status_t::SGX_SUCCESS;
            }
            Err(err) => {
                error!("Failed to load checkpoint: {:?}", err);
                if !args.skip_corrupted_checkpoint {
                    return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
                }
                info!("Skipped corrupted checkpoint");
            }
            Ok(None) => {
                info!("No checkpoint found");
            },
        }
    } else {
        info!("No checkpoint file specified.");
    }

    APPLICATION.lock().unwrap().init(args.clone());

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
pub extern "C" fn ecall_async_reactor_run() {
    SgxPlatform::async_reactor_run();
}

#[no_mangle]
pub extern "C" fn ecall_async_executor_run() {
    SgxPlatform::async_executor_run();
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
    let (code, data) =
        unsafe { factory.dispatch_prpc_request(path, path_len, data, data_len, output_buf_len) };
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
