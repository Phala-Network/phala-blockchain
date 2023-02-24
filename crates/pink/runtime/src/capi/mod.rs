use pink_capi::v1;
use v1::*;

use phala_sanitized_logger as logger;

pub(crate) use ocall_impl::OCallImpl;

const _: init_t = Some(__pink_runtime_init);

#[no_mangle]
pub unsafe extern "C" fn __pink_runtime_init(
    config: *const config_t,
    ecalls: *mut ecalls_t,
) -> ::core::ffi::c_int {
    let config = unsafe { &*config };
    if config.is_dylib != 0 {
        logger::init(config.enclaved != 0);
    }
    if ecalls.is_null() {
        log::error!("Failed to init runtime: ecalls is null");
        return -1;
    }
    let Some(ocall) = config.ocall else {
        log::error!("Failed to init runtime: ocall is null");
        return -1;
    };
    unsafe {
        ocall_impl::set_ocall_fn(ocall);
        (&mut *ecalls).ecall = Some(ecall);
    }
    0
}

unsafe extern "C" fn ecall(
    call_id: u32,
    data: *const u8,
    len: usize,
    ctx: *mut ::core::ffi::c_void,
    output_fn: output_fn_t,
) {
    let input = unsafe { std::slice::from_raw_parts(data, len) };
    let output = ecall::executing_dispatch(
        &mut ecall_impl::storage(),
        &mut ecall_impl::ECallImpl,
        call_id,
        input,
    );
    if let Some(output_fn) = output_fn {
        unsafe { output_fn(ctx, output.as_ptr(), output.len()) };
    }
}

mod ecall_impl;
mod ocall_impl;
