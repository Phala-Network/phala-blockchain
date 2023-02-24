use super::{in_enclave, load_pink_library};
use once_cell::sync::Lazy;
use pink_capi::{
    helper::InnerType,
    v1::{
        config_t, cross_call_fn_t, ecalls_t, init_t,
        ocall::{executing_dispatch, OCalls},
        output_fn_t, IdentExecute,
    },
};
use std::ffi::c_void;

pub struct Runtime {
    handle: *const c_void,
    ecall: InnerType<cross_call_fn_t>,
}

unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    let filename = "libpink.so.1.0";
    let handle = load_pink_library("1.0");
    if handle.is_null() {
        panic!("Failed to load {filename}");
    }
    let init: init_t = unsafe {
        std::mem::transmute(libc::dlsym(
            handle,
            b"__pink_runtime_init\0".as_ptr() as *const _,
        ))
    };
    let Some(init) = init else {
        panic!("Failed to get initialize entry in {filename}");
    };
    Runtime::from_fn(init, handle, filename)
});

impl Runtime {
    pub fn from_fn(init: InnerType<init_t>, handle: *mut c_void, filename: &str) -> Self {
        let config = config_t {
            is_dylib: if handle.is_null() { 0 } else { 1 },
            enclaved: in_enclave(),
            ocall: Some(ocall),
        };
        let mut ecalls = ecalls_t::default();
        unsafe {
            if init(&config, &mut ecalls) != 0 {
                panic!("Failed to initialize {filename}");
            }
        };
        let Some(ecall) = ecalls.ecall else {
            panic!("Failed to get ecall entry in {filename}");
        };
        Runtime { handle, ecall }
    }

    /// # Safety
    ///
    /// This is only for unit test. The pointer's in the runtime will be invalid if the original
    /// runtime is dropped.
    pub unsafe fn dup(&self) -> Self {
        Runtime {
            handle: std::ptr::null_mut(),
            ecall: self.ecall,
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.handle.is_null() {
            return;
        }
        unsafe {
            libc::dlclose(self.handle as *mut _);
        }
    }
}

impl Runtime {
    pub fn ecall(&self, call_id: u32, data: &[u8]) -> Vec<u8> {
        unsafe extern "C" fn output_fn(ctx: *mut ::core::ffi::c_void, data: *const u8, len: usize) {
            let output = &mut *(ctx as *mut Vec<u8>);
            output.extend_from_slice(std::slice::from_raw_parts(data, len));
        }
        let mut output = Vec::new();
        let ctx = &mut output as *mut _ as *mut c_void;
        unsafe { (self.ecall)(call_id, data.as_ptr(), data.len(), ctx, Some(output_fn)) };
        output
    }
}

environmental::environmental! { current_ocalls: trait OCalls }

pub fn using_ocalls<F, R>(ocalls: &mut impl OCalls, f: F) -> R
where
    F: FnOnce() -> R,
{
    current_ocalls::using(ocalls, f)
}

unsafe extern "C" fn ocall(
    call_id: u32,
    data: *const u8,
    len: usize,
    ctx: *mut ::core::ffi::c_void,
    output_fn: output_fn_t,
) {
    let input = std::slice::from_raw_parts(data, len);
    let output = current_ocalls::with(move |ocalls| {
        executing_dispatch(&mut IdentExecute, ocalls, call_id, input)
    })
    .expect("No OCalls set");
    if let Some(output_fn) = output_fn {
        unsafe { output_fn(ctx, output.as_ptr(), output.len()) };
    }
}
