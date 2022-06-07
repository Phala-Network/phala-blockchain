use core::fmt;

use wasmer::{Memory, WasmCell, WasmPtr as BaseWasmPtr};
use wasmer::{FromToNativeWasmType, Item, ValueType};
use wasmer_wasi_types::*;

#[repr(transparent)]
pub struct WasmPtr<T: Copy, Ty = Item>(BaseWasmPtr<T, Ty>);

unsafe impl<T: Copy, Ty> ValueType for WasmPtr<T, Ty> {}
impl<T: Copy, Ty> Copy for WasmPtr<T, Ty> {}

impl<T: Copy, Ty> Clone for WasmPtr<T, Ty> {
    fn clone(&self) -> Self {
        #[allow(clippy::clone_on_copy)]
        Self(self.0.clone())
    }
}

impl<T: Copy, Ty> fmt::Debug for WasmPtr<T, Ty> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T: Copy, Ty> From<i32> for WasmPtr<T, Ty> {
    fn from(offset: i32) -> Self {
        Self::new(offset as _)
    }
}

unsafe impl<T: Copy, Ty> FromToNativeWasmType for WasmPtr<T, Ty> {
    type Native = <BaseWasmPtr<T, Ty> as FromToNativeWasmType>::Native;

    fn to_native(self) -> Self::Native {
        self.0.to_native()
    }
    fn from_native(n: Self::Native) -> Self {
        Self(BaseWasmPtr::from_native(n))
    }
}

impl<T: Copy, Ty> PartialEq for WasmPtr<T, Ty> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Copy, Ty> Eq for WasmPtr<T, Ty> {}

impl<T: Copy, Ty> WasmPtr<T, Ty> {
    #[inline(always)]
    pub fn new(offset: u32) -> Self {
        Self(BaseWasmPtr::new(offset))
    }
}

impl<T: Copy + ValueType> WasmPtr<T, Item> {
    #[inline(always)]
    pub fn deref<'a>(self, memory: &'a Memory) -> Result<WasmCell<'a, T>, __wasi_errno_t> {
        self.0.deref(memory).ok_or(__WASI_EFAULT)
    }
}
