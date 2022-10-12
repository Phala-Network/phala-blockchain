use crate::{IntPtr, IntRet, OcallError, Result, VmMemory};
use log::Level;

const OCALL_N_ARGS: usize = 4;

pub(crate) struct StackedArgs<Args> {
    args: Args,
}

impl StackedArgs<()> {
    pub(crate) const fn empty() -> Self {
        StackedArgs { args: () }
    }
}

impl<A: Nargs> StackedArgs<A> {
    pub(crate) fn load(mut raw: &[IntPtr]) -> Option<Self> {
        Some(check_args_length(StackedArgs {
            args: Nargs::load(&mut raw)?,
        }))
    }

    pub(crate) fn dump(self) -> [IntPtr; OCALL_N_ARGS] {
        let mut ret = [Default::default(); OCALL_N_ARGS];
        let data = check_args_length(self).args.dump();
        ret[..data.len()].copy_from_slice(&data);
        ret
    }
}

impl<A, B> StackedArgs<(A, B)> {
    fn pop(self) -> (A, StackedArgs<B>) {
        let (a, args) = self.args;
        (a, StackedArgs { args })
    }
}

impl<B> StackedArgs<B> {
    fn push<A>(self, arg: A) -> StackedArgs<(A, B)> {
        StackedArgs {
            args: (arg, self.args),
        }
    }
}

pub(crate) trait Nargs {
    const N_ARGS: usize;
    fn load(buf: &mut &[IntPtr]) -> Option<Self>
    where
        Self: Sized;

    // Since #![feature(generic_const_exprs)] is not yet stable, we use OCALL_N_ARGS instead of
    // Self::N_ARGS
    fn dump(self) -> [IntPtr; OCALL_N_ARGS];
}

impl Nargs for () {
    const N_ARGS: usize = 0;
    fn load(_buf: &mut &[IntPtr]) -> Option<Self> {
        Some(())
    }
    fn dump(self) -> [IntPtr; OCALL_N_ARGS] {
        Default::default()
    }
}

impl Nargs for IntPtr {
    const N_ARGS: usize = 1;
    fn load(buf: &mut &[IntPtr]) -> Option<Self> {
        let me = *buf.get(0)?;
        *buf = &buf[1..];
        Some(me)
    }

    fn dump(self) -> [IntPtr; OCALL_N_ARGS] {
        let mut ret = [0; OCALL_N_ARGS];
        ret[0] = self;
        ret
    }
}

impl<A, B> Nargs for (A, B)
where
    A: Nargs,
    B: Nargs,
{
    const N_ARGS: usize = A::N_ARGS + B::N_ARGS;

    fn load(buf: &mut &[IntPtr]) -> Option<Self> {
        let b = B::load(buf)?;
        let a = A::load(buf)?;
        Some((a, b))
    }

    fn dump(self) -> [IntPtr; OCALL_N_ARGS] {
        let (a, b) = self;
        let mut buf = [IntPtr::default(); OCALL_N_ARGS];
        buf[0..B::N_ARGS].copy_from_slice(&b.dump()[0..B::N_ARGS]);
        buf[B::N_ARGS..Self::N_ARGS].copy_from_slice(&a.dump()[..A::N_ARGS]);
        buf
    }
}

// Since the const evaluation of Rust is not powerful enough yet, we use this trick to statically
// check the argument types encode output do not exceed the maximum number of arguments.
pub(crate) trait NotTooManyArgs {
    const TOO_MANY_ARGUMENTS: ();
}
impl<T: Nargs> NotTooManyArgs for T {
    const TOO_MANY_ARGUMENTS: () = [()][(Self::N_ARGS > OCALL_N_ARGS) as usize];
}

pub(crate) fn check_args_length<T: Nargs + NotTooManyArgs>(v: StackedArgs<T>) -> StackedArgs<T> {
    let _ = T::TOO_MANY_ARGUMENTS;
    v
}

pub(crate) trait I32Convertible {
    fn to_i32(&self) -> i32;
    fn from_i32(i: i32) -> Result<Self>
    where
        Self: Sized;
}

pub(crate) trait ArgEncode {
    type Encoded;

    fn encode_arg<A>(self, stack: StackedArgs<A>) -> StackedArgs<(Self::Encoded, A)>;
}

pub(crate) trait ArgDecode<'a> {
    type Encoded;
    fn decode_arg<R>(
        stack: StackedArgs<(Self::Encoded, R)>,
        vm: &'a impl VmMemory,
    ) -> Result<(Self, StackedArgs<R>)>
    where
        Self: Sized;
}

/// Trait for types that can be encoded to a return value of a ocall.
pub trait RetEncode {
    /// Encode the ocall return value into a IntRet
    fn encode_ret(self) -> IntRet;
}

pub(crate) trait RetDecode {
    fn decode_ret(encoded: IntRet) -> Self
    where
        Self: Sized;
}

impl ArgEncode for &[u8] {
    type Encoded = (IntPtr, IntPtr);

    fn encode_arg<A>(self, stack: StackedArgs<A>) -> StackedArgs<(Self::Encoded, A)> {
        let ptr = self.as_ptr() as IntPtr;
        let len = self.len() as IntPtr;
        stack.push((len, ptr))
    }
}

impl<'a> ArgDecode<'a> for &'a [u8] {
    type Encoded = (IntPtr, IntPtr);

    fn decode_arg<A>(
        stack: StackedArgs<(Self::Encoded, A)>,
        vm: &'a impl VmMemory,
    ) -> Result<(Self, StackedArgs<A>)>
    where
        Self: Sized,
    {
        let ((len, ptr), stack) = stack.pop();
        let bytes = vm.slice_from_vm(ptr, len)?;
        Ok((bytes, stack))
    }
}

impl ArgEncode for &str {
    type Encoded = (IntPtr, IntPtr);

    fn encode_arg<A>(self, stack: StackedArgs<A>) -> StackedArgs<(Self::Encoded, A)> {
        let bytes = self.as_bytes();
        bytes.encode_arg(stack)
    }
}

impl<'a> ArgDecode<'a> for &'a str {
    type Encoded = (IntPtr, IntPtr);

    fn decode_arg<A>(
        stack: StackedArgs<(Self::Encoded, A)>,
        vm: &'a impl VmMemory,
    ) -> Result<(Self, StackedArgs<A>)>
    where
        Self: Sized,
    {
        let (bytes, stack): (&[u8], _) = ArgDecode::decode_arg(stack, vm)?;
        Ok((
            core::str::from_utf8(bytes).or(Err(OcallError::InvalidEncoding))?,
            stack,
        ))
    }
}

impl ArgEncode for &mut [u8] {
    type Encoded = (IntPtr, IntPtr);

    fn encode_arg<A>(self, stack: StackedArgs<A>) -> StackedArgs<(Self::Encoded, A)> {
        let ptr = self.as_mut_ptr() as IntPtr;
        let len = self.len() as IntPtr;
        stack.push((len, ptr))
    }
}

impl<'a> ArgDecode<'a> for &'a mut [u8] {
    type Encoded = (IntPtr, IntPtr);

    fn decode_arg<A>(
        stack: StackedArgs<(Self::Encoded, A)>,
        vm: &'a impl VmMemory,
    ) -> Result<(Self, StackedArgs<A>)>
    where
        Self: Sized,
    {
        let ((len, ptr), stack) = stack.pop();
        let bytes = vm.slice_from_vm_mut(ptr, len)?;
        Ok((bytes, stack))
    }
}

impl<B> StackedArgs<B> {
    pub(crate) fn push_arg<Arg: ArgEncode>(self, v: Arg) -> StackedArgs<(Arg::Encoded, B)> {
        v.encode_arg(self)
    }
}

impl<A, B> StackedArgs<(A, B)> {
    pub(crate) fn pop_arg<'a, Arg: ArgDecode<'a, Encoded = A>>(
        self,
        vm: &'a impl VmMemory,
    ) -> Result<(Arg, StackedArgs<B>)> {
        Arg::decode_arg(self, vm)
    }
}

macro_rules! impl_codec_i {
    ($typ: ty) => {
        impl I32Convertible for $typ {
            fn to_i32(&self) -> i32 {
                *self as i32
            }
            fn from_i32(i: i32) -> Result<Self> {
                if i > <$typ>::MAX as i32 || i < (-<$typ>::MAX - 1) as i32 {
                    Err(OcallError::InvalidEncoding)
                } else {
                    Ok(i as Self)
                }
            }
        }
    };
}
impl_codec_i!(i8);
impl_codec_i!(i16);

macro_rules! impl_codec_u {
    ($typ: ty) => {
        impl I32Convertible for $typ {
            fn to_i32(&self) -> i32 {
                *self as i32
            }
            fn from_i32(i: i32) -> Result<Self> {
                if i as u32 > <$typ>::MAX as u32 {
                    Err(OcallError::InvalidEncoding)
                } else {
                    Ok(i as Self)
                }
            }
        }
    };
}
impl_codec_u!(u8);
impl_codec_u!(u16);

macro_rules! impl_codec {
    ($typ: ty) => {
        impl I32Convertible for $typ {
            fn to_i32(&self) -> i32 {
                *self as i32
            }
            fn from_i32(i: i32) -> Result<Self> {
                Ok(i as Self)
            }
        }
    };
}
impl_codec!(i32);
impl_codec!(u32);

macro_rules! impl_codec64 {
    ($typ: ty) => {
        impl ArgEncode for $typ {
            type Encoded = (IntPtr, IntPtr);

            fn encode_arg<R>(self, stack: StackedArgs<R>) -> StackedArgs<(Self::Encoded, R)> {
                let low = (self & 0xffffffff) as IntPtr;
                let high = ((self >> 32) & 0xffffffff) as IntPtr;
                stack.push((low, high))
            }
        }

        impl<'a> ArgDecode<'a> for $typ {
            type Encoded = (IntPtr, IntPtr);

            fn decode_arg<R>(
                stack: StackedArgs<(Self::Encoded, R)>,
                _vm: &'a impl VmMemory,
            ) -> Result<(Self, StackedArgs<R>)>
            where
                Self: Sized,
            {
                let ((low, high), stack) = stack.pop();
                let high = ((high as Self) << 32);
                let v = high & (low as Self);
                Ok((v, stack))
            }
        }
    };
}

impl_codec64!(i64);
impl_codec64!(u64);

impl<I: I32Convertible> ArgEncode for I {
    type Encoded = IntPtr;

    fn encode_arg<R>(self, stack: StackedArgs<R>) -> StackedArgs<(Self::Encoded, R)> {
        stack.push(self.to_i32() as _)
    }
}

impl<'a, I: I32Convertible> ArgDecode<'a> for I {
    type Encoded = IntPtr;

    fn decode_arg<R>(
        stack: StackedArgs<(Self::Encoded, R)>,
        _vm: &'a impl VmMemory,
    ) -> Result<(Self, StackedArgs<R>)>
    where
        Self: Sized,
    {
        let (v, stack) = stack.pop();
        Ok((I::from_i32(v as _)?, stack))
    }
}

impl I32Convertible for bool {
    fn to_i32(&self) -> i32 {
        *self as i32
    }
    fn from_i32(i: i32) -> Result<Self> {
        match i {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(OcallError::InvalidEncoding),
        }
    }
}

impl I32Convertible for OcallError {
    fn to_i32(&self) -> i32 {
        *self as u8 as i32
    }
    fn from_i32(i: i32) -> Result<Self> {
        let code = u8::from_i32(i)?;
        OcallError::try_from(code).or(Err(OcallError::InvalidEncoding))
    }
}

impl I32Convertible for () {
    fn to_i32(&self) -> i32 {
        0
    }
    fn from_i32(i: i32) -> Result<()> {
        if i == 0 {
            Ok(())
        } else {
            Err(OcallError::InvalidEncoding)
        }
    }
}

impl I32Convertible for Level {
    fn to_i32(&self) -> i32 {
        match self {
            Level::Error => 1,
            Level::Warn => 2,
            Level::Info => 3,
            Level::Debug => 4,
            Level::Trace => 5,
        }
    }

    fn from_i32(i: i32) -> Result<Self> {
        match i {
            1 => Ok(Level::Error),
            2 => Ok(Level::Warn),
            3 => Ok(Level::Info),
            4 => Ok(Level::Debug),
            5 => Ok(Level::Trace),
            _ => Err(OcallError::InvalidEncoding),
        }
    }
}

impl<A, B> RetEncode for Result<A, B>
where
    A: I32Convertible,
    B: I32Convertible,
{
    fn encode_ret(self) -> IntRet {
        let (tp, val) = match self {
            Ok(v) => (0, v.to_i32()),
            Err(err) => (1, err.to_i32()),
        };
        ((tp as u32 as i64) << 32) | (val as u32 as i64)
    }
}

impl<A, B> RetDecode for Result<A, B>
where
    A: I32Convertible,
    B: I32Convertible,
{
    fn decode_ret(encoded: IntRet) -> Self {
        let tp = ((encoded >> 32) & 0xffffffff) as i32;
        let val = (encoded & 0xffffffff) as i32;
        if tp == 0 {
            Ok(A::from_i32(val).expect("Invalid ocall return"))
        } else {
            Err(B::from_i32(val).expect("Invalid ocall return"))
        }
    }
}
