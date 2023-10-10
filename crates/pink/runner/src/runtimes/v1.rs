use super::{in_enclave, load_pink_library};
use once_cell::sync::Lazy;
use pink_capi::{
    helper::InnerType,
    v1::{
        config_t, cross_call_fn_t, ecalls_t, init_t,
        ocall::{self, OCalls},
        ocalls_t, output_fn_t, IdentExecute,
    },
};
use std::ffi::c_void;

pub struct Runtime {
    handle: *const c_void,
    ecall_fn: InnerType<cross_call_fn_t>,
}

unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

macro_rules! define_runtimes {
    ($(($major: expr, $minor: expr),)*) => {
        pub fn get_runtime(version: (u32, u32)) -> &'static Runtime {
            match version {
                $(
                    ($major, $minor) => {
                        static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::load_by_version(($major, $minor)));
                        &RUNTIME
                    },
                )*
                _ => panic!("Unsupported runtime version: {version:?}"),
            }
        }
        pub fn runtime_versions() -> &'static [(u32, u32)] {
            &[$(($major, $minor)),*]
        }
    };
}

define_runtimes! {
    (1, 0),
    (1, 1),
    (1, 2),
}

impl Default for Runtime {
    fn default() -> Self {
        Self::load_by_version((1, 0))
    }
}

impl Runtime {
    pub fn load_by_version(version: (u32, u32)) -> Self {
        let (major, minor) = version;
        let handle = load_pink_library(version);
        if handle.is_null() {
            panic!("Failed to load pink dylib {major}.{minor}");
        }
        let init: init_t = unsafe {
            std::mem::transmute(libc::dlsym(
                handle,
                b"__pink_runtime_init\0".as_ptr() as *const _,
            ))
        };
        let Some(init) = init else {
            panic!("Failed to get initialize entry in pink dylib {major}.{minor}");
        };
        Runtime::from_fn(init, handle, version)
    }

    pub fn from_fn(init: InnerType<init_t>, handle: *mut c_void, version: (u32, u32)) -> Self {
        let filename = format!("libpink.so.{}.{}", version.0, version.1);
        let is_dylib = !handle.is_null();
        let config = config_t {
            is_dylib: is_dylib as _,
            enclaved: in_enclave(),
            ocalls: ocalls_t {
                ocall: Some(handle_ocall),
                alloc: is_dylib.then_some(ocall_alloc),
                dealloc: is_dylib.then_some(ocall_dealloc),
            },
        };
        let mut ecalls = ecalls_t::default();
        let ret = unsafe { init(&config, &mut ecalls) };
        if ret != 0 {
            panic!("Failed to initialize {filename}, ret={ret}");
        }
        let Some(get_version) = ecalls.get_version else {
            panic!("Failed to get ecall entry in {filename}");
        };
        let (mut major, mut minor) = (0, 0);
        unsafe { get_version(&mut major, &mut minor) };
        if (major, minor) != version {
            panic!("Version mismatch in {filename}, expect {version:?}, got ({major},{minor})");
        }
        let Some(ecall) = ecalls.ecall else {
            panic!("Failed to get ecall entry in {filename}");
        };
        Runtime {
            handle,
            ecall_fn: ecall,
        }
    }

    /// # Safety
    ///
    /// This is for unit test only. The pointer's in the runtime will be invalid if the original
    /// runtime is dropped.
    pub unsafe fn dup(&self) -> Self {
        Runtime {
            handle: std::ptr::null_mut(),
            ecall_fn: self.ecall_fn,
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
    #[tracing::instrument(skip_all)]
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

#[tracing::instrument(name = "ocall", skip_all)]
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

unsafe extern "C" fn ocall_alloc(size: usize, align: usize) -> *mut u8 {
    std::alloc::alloc(std::alloc::Layout::from_size_align(size, align).unwrap())
}

unsafe extern "C" fn ocall_dealloc(ptr: *mut u8, size: usize, align: usize) {
    std::alloc::dealloc(
        ptr,
        std::alloc::Layout::from_size_align(size, align).unwrap(),
    )
}
