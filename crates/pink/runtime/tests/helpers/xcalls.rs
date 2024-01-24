use core::ffi::c_void;
use pink_capi::{
    helper::InnerType,
    v1::{
        config_t, cross_call_fn_t, ecalls_t, init_t,
        ocall::{self, OCalls},
        ocalls_t, output_fn_t, IdentExecute,
    },
};

pub struct Runtime {
    ecall_fn: InnerType<cross_call_fn_t>,
}

impl Runtime {
    pub fn dup(&self) -> Self {
        Self {
            ecall_fn: self.ecall_fn,
        }
    }
    pub fn from_fn(init: InnerType<init_t>) -> Self {
        let config = config_t {
            is_dylib: 0,
            enclaved: 0,
            ocalls: ocalls_t {
                ocall: Some(handle_ocall),
                alloc: None,
                dealloc: None,
            },
        };
        let mut ecalls = ecalls_t::default();
        unsafe { init(&config, &mut ecalls) };
        let ecall = ecalls
            .ecall
            .expect("Failed to get ecall entry in {filename}");
        Runtime { ecall_fn: ecall }
    }

    pub fn ecall(&self, call_id: u32, data: &[u8]) -> Vec<u8> {
        unsafe extern "C" fn output_fn(ctx: *mut ::core::ffi::c_void, data: *const u8, len: usize) {
            let output = &mut *(ctx as *mut Vec<u8>);
            output.extend_from_slice(std::slice::from_raw_parts(data, len));
        }
        let mut output = Vec::new();
        let ctx = &mut output as *mut _ as *mut c_void;
        unsafe { (self.ecall_fn)(call_id, data.as_ptr(), data.len(), ctx, Some(output_fn)) };
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

unsafe extern "C" fn handle_ocall(
    call_id: u32,
    data: *const u8,
    len: usize,
    ctx: *mut ::core::ffi::c_void,
    output_fn: output_fn_t,
) {
    let input = std::slice::from_raw_parts(data, len);
    let output = current_ocalls::with(move |ocalls| {
        ocall::dispatch(&mut IdentExecute, ocalls, call_id, input)
    })
    .expect("No OCalls set");
    if let Some(output_fn) = output_fn {
        unsafe { output_fn(ctx, output.as_ptr(), output.len()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pink_runtime::capi::__pink_runtime_init;
    use rusty_fork::rusty_fork_test;

    #[test]
    fn init_should_fail_without_ocall() {
        let config = config_t {
            is_dylib: 0,
            enclaved: 0,
            ocalls: ocalls_t {
                ocall: None,
                alloc: None,
                dealloc: None,
            },
        };
        let mut ecalls = ecalls_t::default();
        let ret = unsafe { __pink_runtime_init(&config, &mut ecalls) };
        assert!(ret != 0);
        assert!(ecalls.ecall.is_none());
    }

    #[test]
    fn init_should_fail_with_null_ecalls() {
        let config = config_t {
            is_dylib: 0,
            enclaved: 0,
            ocalls: ocalls_t {
                ocall: Some(handle_ocall),
                alloc: None,
                dealloc: None,
            },
        };
        let ret = unsafe { __pink_runtime_init(&config, std::ptr::null_mut()) };
        assert!(ret != 0);
    }

    rusty_fork_test! {
        #[test]
        fn init_with_dylib_enabled() {
            let config = config_t {
                is_dylib: 1,
                enclaved: 0,
                ocalls: ocalls_t {
                    ocall: Some(handle_ocall),
                    alloc: None,
                    dealloc: None,
                },
            };
            let mut ecalls = ecalls_t::default();
            let ret = unsafe { __pink_runtime_init(&config, &mut ecalls) };
            assert!(ret == 0);
            assert!(ecalls.ecall.is_some());
            let (mut major, mut minor) = (0u32, 0u32);
            unsafe { ecalls.get_version.unwrap()(&mut major, &mut minor) };
            assert!((major, minor) == pink_runtime::version())
        }

        #[test]
        fn init_with_enclave_enabled() {
            let config = config_t {
                is_dylib: 1,
                enclaved: 1,
                ocalls: ocalls_t {
                    ocall: Some(handle_ocall),
                    alloc: None,
                    dealloc: None,
                },
            };
            let mut ecalls = ecalls_t::default();
            let ret = unsafe { __pink_runtime_init(&config, &mut ecalls) };
            assert!(ret == 0);
            assert!(ecalls.ecall.is_some());
        }
    }
}
