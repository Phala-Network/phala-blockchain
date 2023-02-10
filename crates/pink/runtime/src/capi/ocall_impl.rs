use pink_capi::{
    helper::InnerType,
    v1::{cross_call_fn_t, output_fn_t, CrossCall, CrossCallMut, OCall},
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

pub fn set_ocall_fn(ocall: InnerType<cross_call_fn_t>) {
    unsafe {
        OCALL = ocall;
    }
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
