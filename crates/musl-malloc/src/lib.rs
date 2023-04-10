use std::alloc::{GlobalAlloc, Layout};
use std::ffi::c_void;

use c::{musl_free, musl_memalign};
mod c {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub struct MuslMalloc;

unsafe impl GlobalAlloc for MuslMalloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        musl_memalign(layout.align(), layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        musl_free(ptr as *mut c_void);
    }
}
