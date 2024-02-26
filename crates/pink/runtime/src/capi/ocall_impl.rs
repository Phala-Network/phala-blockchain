use pink_capi::{
    helper::InnerType,
    v1::{cross_call_fn_t, ocalls_t, output_fn_t, CrossCall, CrossCallMut, OCall},
};

static mut OCALL: InnerType<cross_call_fn_t> = _default_ocall;

unsafe extern "C" fn _default_ocall(
    _call_id: u32,
    _data: *const u8,
    _len: usize,
    _ctx: *mut ::core::ffi::c_void,
    _output: output_fn_t,
) {
    panic!("No ocall function provided");
}

pub(super) fn set_ocall_fn(ocalls: ocalls_t) -> Result<(), &'static str> {
    let Some(ocall) = ocalls.ocall else {
        return Err("No ocall function provided");
    };
    unsafe {
        OCALL = ocall;
        #[cfg(feature = "allocator")]
        if let Some(alloc) = ocalls.alloc {
            allocator::ALLOC_FUNC = alloc;
        }
        #[cfg(feature = "allocator")]
        if let Some(dealloc) = ocalls.dealloc {
            allocator::DEALLOC_FUNC = dealloc;
        }
    }
    Ok(())
}

pub(crate) struct OCallImpl;
impl CrossCallMut for OCallImpl {
    fn cross_call_mut(&mut self, call_id: u32, data: &[u8]) -> Vec<u8> {
        self.cross_call(call_id, data)
    }
}
impl CrossCall for OCallImpl {
    fn cross_call(&self, id: u32, data: &[u8]) -> Vec<u8> {
        unsafe extern "C" fn output_fn(ctx: *mut ::core::ffi::c_void, data: *const u8, len: usize) {
            let output = &mut *(ctx as *mut Vec<u8>);
            output.extend_from_slice(std::slice::from_raw_parts(data, len));
        }
        unsafe {
            let mut output = Vec::new();
            let ctx = &mut output as *mut _ as *mut ::core::ffi::c_void;
            OCALL(id, data.as_ptr(), data.len(), ctx, Some(output_fn));
            output
        }
    }
}
impl OCall for OCallImpl {}

#[cfg(feature = "allocator")]
mod allocator {
    //! # Rust memory allocator in dynamic runtimes
    //!
    //! By default, Rust std comes with a default allocator which uses the alloc
    //! APIs from libc. As a result, every dynamic library written in Rust will
    //! have its own allocator. This is not what we want because we have a common
    //! allocator in the main executable which have some metrices and statistics.
    //! If we use the default allocator, the statistics will be not accurate.
    //! So we make this allocator in the runtime and delegate the calls to the
    //! allocator in the main executable.

    use pink_capi::{helper::InnerType, v1};
    use std::alloc::{GlobalAlloc, Layout, System};

    pub(super) static mut ALLOC_FUNC: InnerType<v1::alloc_fn_t> = system_alloc;
    pub(super) static mut DEALLOC_FUNC: InnerType<v1::dealloc_fn_t> = system_dealloc;
    unsafe extern "C" fn system_alloc(size: usize, align: usize) -> *mut u8 {
        // Safety: The layout is valid because it is always passed from the PinkAllocator below.
        System.alloc(Layout::from_size_align(size, align).unwrap_unchecked())
    }
    unsafe extern "C" fn system_dealloc(ptr: *mut u8, size: usize, align: usize) {
        // Safety: The layout is valid because it is always passed from the PinkAllocator below.
        System.dealloc(ptr, Layout::from_size_align(size, align).unwrap_unchecked())
    }

    struct PinkAllocator;

    #[global_allocator]
    static ALLOCATOR: PinkAllocator = PinkAllocator;

    unsafe impl GlobalAlloc for PinkAllocator {
        unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
            ALLOC_FUNC(layout.size(), layout.align())
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
            DEALLOC_FUNC(ptr, layout.size(), layout.align())
        }
    }

    #[test]
    fn can_set_ocalls() {
        let ocalls = v1::ocalls_t {
            ocall: Some(super::_default_ocall),
            alloc: Some(system_alloc),
            dealloc: Some(system_dealloc),
        };
        assert!(super::set_ocall_fn(ocalls).is_ok());
        // can allocate
        let _ = vec![1, 2, 3];
    }
}

#[test]
#[should_panic]
fn default_ocall_should_panic() {
    let ocall = OCallImpl;
    ocall.cross_call(0, &[]);
}
