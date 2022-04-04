use crate::{IntPtr, OcallEnv, Result};

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
}

impl Nargs for () {
    const N_ARGS: usize = 0;
    fn load(buf: &mut &[IntPtr]) -> Option<Self> {
        Some(())
    }
}

impl Nargs for IntPtr {
    const N_ARGS: usize = 1;
    fn load(buf: &mut &[IntPtr]) -> Option<Self> {
        let me = *buf.get(0)?;
        *buf = &buf[1..];
        Some(me)
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
}

pub(crate) trait NotTooManyArgs {
    const TOO_MANY_ARGUMENTS: ();
}
impl<T: Nargs> NotTooManyArgs for T {
    const TOO_MANY_ARGUMENTS: () = [()][(Self::N_ARGS > 4) as usize];
}

pub(crate) fn check_args_length<T: Nargs + NotTooManyArgs>(v: StackedArgs<T>) -> StackedArgs<T> {
    let _ = T::TOO_MANY_ARGUMENTS;
    v
}

pub(crate) trait ArgEncode<A> {
    type Encoded;

    fn encode(self, stack: StackedArgs<A>) -> StackedArgs<(Self::Encoded, A)>;
}

pub(crate) trait ArgDecode<'a, A> {
    type Encoded;
    fn decode(
        stack: StackedArgs<(Self::Encoded, A)>,
        env: &'a impl OcallEnv,
    ) -> Result<(Self, StackedArgs<A>)>
    where
        Self: Sized;
}

impl<A> ArgEncode<A> for &[u8] {
    type Encoded = (IntPtr, IntPtr);

    fn encode(self, stack: StackedArgs<A>) -> StackedArgs<(Self::Encoded, A)> {
        let ptr = self.as_ptr() as IntPtr;
        let len = self.len() as IntPtr;
        stack.push((ptr, len))
    }
}

impl<'a, A> ArgDecode<'a, A> for &'a [u8] {
    type Encoded = (IntPtr, IntPtr);

    fn decode(
        stack: StackedArgs<(Self::Encoded, A)>,
        env: &'a impl OcallEnv,
    ) -> Result<(Self, StackedArgs<A>)>
    where
        Self: Sized,
    {
        let ((ptr, len), stack) = stack.pop();
        Ok((env.slice_from_vm(ptr, len)?, stack))
    }
}

impl<A> ArgEncode<A> for &mut [u8] {
    type Encoded = (IntPtr, IntPtr);

    fn encode(self, stack: StackedArgs<A>) -> StackedArgs<(Self::Encoded, A)> {
        let ptr = self.as_mut_ptr() as IntPtr;
        let len = self.len() as IntPtr;
        stack.push((ptr, len))
    }
}

impl<'a, A> ArgDecode<'a, A> for &'a mut [u8] {
    type Encoded = (IntPtr, IntPtr);

    fn decode(
        stack: StackedArgs<(Self::Encoded, A)>,
        env: &'a impl OcallEnv,
    ) -> Result<(Self, StackedArgs<A>)>
    where
        Self: Sized,
    {
        let ((ptr, len), stack) = stack.pop();
        Ok((env.slice_from_vm_mut(ptr, len)?, stack))
    }
}

impl<B> StackedArgs<B> {
    pub(crate) fn encode<Arg: ArgEncode<B>>(self, v: Arg) -> StackedArgs<(Arg::Encoded, B)> {
        v.encode(self)
    }
}

impl<A, B> StackedArgs<(A, B)> {
    pub(crate) fn decode<'a, Arg: ArgDecode<'a, B, Encoded = A>>(
        self,
        env: &'a impl OcallEnv,
    ) -> Result<(Arg, StackedArgs<B>)> {
        Arg::decode(self, env)
    }
}

macro_rules! impl_codec {
    ($typ: ty) => {
        impl<R> ArgEncode<R> for $typ {
            type Encoded = IntPtr;

            fn encode(self, stack: StackedArgs<R>) -> StackedArgs<(Self::Encoded, R)> {
                stack.push(self as _)
            }
        }

        impl<'a, R> ArgDecode<'a, R> for $typ {
            type Encoded = IntPtr;

            fn decode(stack: StackedArgs<(Self::Encoded, R)>, _env: &'a impl OcallEnv) -> Result<(Self, StackedArgs<R>)>
            where
                Self: Sized,
            {
                let (v, stack) = stack.pop();
                Ok((v as _, stack))
            }
        }
    };
    ($typ: ty, $($other: ty),*) => {
        impl_codec!($typ);
        impl_codec!($($other),*);
    }
}

impl_codec!(i8, u8, i16, u16, i32, u32);

macro_rules! impl_codec64 {
    ($typ: ty) => {
        impl<R> ArgEncode<R> for $typ {
            type Encoded = (IntPtr, IntPtr);

            fn encode(self, stack: StackedArgs<R>) -> StackedArgs<(Self::Encoded, R)> {
                let low = (self & 0xffffffff) as IntPtr;
                let high = ((self >> 32) & 0xffffffff) as IntPtr;
                stack.push((low, high))
            }
        }

        impl<'a, R> ArgDecode<'a, R> for $typ {
            type Encoded = (IntPtr, IntPtr);

            fn decode(
                stack: StackedArgs<(Self::Encoded, R)>,
                _env: &'a impl OcallEnv,
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
