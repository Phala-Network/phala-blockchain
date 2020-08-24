use lib::*;

use ser::{Error, Serialize, SerializeTuple, Serializer};

////////////////////////////////////////////////////////////////////////////////

macro_rules! primitive_impl {
    ($ty:ident, $method:ident $($cast:tt)*) => {
        impl Serialize for $ty {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.$method(*self $($cast)*)
            }
        }
    }
}

primitive_impl!(bool, serialize_bool);
primitive_impl!(isize, serialize_i64 as i64);
primitive_impl!(i8, serialize_i8);
primitive_impl!(i16, serialize_i16);
primitive_impl!(i32, serialize_i32);
primitive_impl!(i64, serialize_i64);
primitive_impl!(usize, serialize_u64 as u64);
primitive_impl!(u8, serialize_u8);
primitive_impl!(u16, serialize_u16);
primitive_impl!(u32, serialize_u32);
primitive_impl!(u64, serialize_u64);
primitive_impl!(f32, serialize_f32);
primitive_impl!(f64, serialize_f64);
primitive_impl!(char, serialize_char);

serde_if_integer128! {
    primitive_impl!(i128, serialize_i128);
    primitive_impl!(u128, serialize_u128);
}

////////////////////////////////////////////////////////////////////////////////

impl Serialize for str {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self)
    }
}

#[cfg(feature = "alloc")]
impl Serialize for String {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self)
    }
}

impl<'a> Serialize for fmt::Arguments<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for Option<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Some(ref value) => serializer.serialize_some(value),
            None => serializer.serialize_none(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T: ?Sized> Serialize for PhantomData<T> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit_struct("PhantomData")
    }
}

////////////////////////////////////////////////////////////////////////////////

// Does not require T: Serialize.
impl<T> Serialize for [T; 0] {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        try!(serializer.serialize_tuple(0)).end()
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T> Serialize for [T; $len]
            where
                T: Serialize,
            {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let mut seq = try!(serializer.serialize_tuple($len));
                    for e in self {
                        try!(seq.serialize_element(e));
                    }
                    seq.end()
                }
            }
        )+
    }
}

array_impls! {
    01 02 03 04 05 06 07 08 09 10
    11 12 13 14 15 16 17 18 19 20
    21 22 23 24 25 26 27 28 29 30
    31 32
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for [T]
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self)
    }
}

#[cfg(feature = "alloc")]
macro_rules! seq_impl {
    ($ty:ident < T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound:ident)* >) => {
        impl<T $(, $typaram)*> Serialize for $ty<T $(, $typaram)*>
        where
            T: Serialize $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound,)*
        {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_seq(self)
            }
        }
    }
}

#[cfg(feature = "alloc")]
seq_impl!(BinaryHeap<T: Ord>);

#[cfg(feature = "alloc")]
seq_impl!(BTreeSet<T: Ord>);

#[cfg(feature = "alloc")]
seq_impl!(LinkedList<T>);

#[cfg(feature = "alloc")]
seq_impl!(Vec<T>);

#[cfg(feature = "alloc")]
seq_impl!(VecDeque<T>);

////////////////////////////////////////////////////////////////////////////////

impl<Idx> Serialize for Range<Idx>
where
    Idx: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use super::SerializeStruct;
        let mut state = try!(serializer.serialize_struct("Range", 2));
        try!(state.serialize_field("start", &self.start));
        try!(state.serialize_field("end", &self.end));
        state.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(range_inclusive)]
impl<Idx> Serialize for RangeInclusive<Idx>
where
    Idx: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use super::SerializeStruct;
        let mut state = try!(serializer.serialize_struct("RangeInclusive", 2));
        try!(state.serialize_field("start", &self.start()));
        try!(state.serialize_field("end", &self.end()));
        state.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(ops_bound, collections_bound))]
impl<T> Serialize for Bound<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Bound::Unbounded => serializer.serialize_unit_variant("Bound", 0, "Unbounded"),
            Bound::Included(ref value) => {
                serializer.serialize_newtype_variant("Bound", 1, "Included", value)
            }
            Bound::Excluded(ref value) => {
                serializer.serialize_newtype_variant("Bound", 2, "Excluded", value)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Serialize for () {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit()
    }
}

#[cfg(feature = "unstable")]
impl Serialize for ! {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        *self
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<$($name),+> Serialize for ($($name,)+)
            where
                $($name: Serialize,)+
            {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let mut tuple = try!(serializer.serialize_tuple($len));
                    $(
                        try!(tuple.serialize_element(&self.$n));
                    )+
                    tuple.end()
                }
            }
        )+
    }
}

tuple_impls! {
    1 => (0 T0)
    2 => (0 T0 1 T1)
    3 => (0 T0 1 T1 2 T2)
    4 => (0 T0 1 T1 2 T2 3 T3)
    5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "alloc")]
macro_rules! map_impl {
    ($ty:ident < K $(: $kbound1:ident $(+ $kbound2:ident)*)*, V $(, $typaram:ident : $bound:ident)* >) => {
        impl<K, V $(, $typaram)*> Serialize for $ty<K, V $(, $typaram)*>
        where
            K: Serialize $(+ $kbound1 $(+ $kbound2)*)*,
            V: Serialize,
            $($typaram: $bound,)*
        {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_map(self)
            }
        }
    }
}

#[cfg(feature = "alloc")]
map_impl!(BTreeMap<K: Ord, V>);

////////////////////////////////////////////////////////////////////////////////

macro_rules! deref_impl {
    (
        $(#[doc = $doc:tt])*
        <$($desc:tt)+
    ) => {
        $(#[doc = $doc])*
        impl <$($desc)+ {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                (**self).serialize(serializer)
            }
        }
    };
}

deref_impl!(<'a, T: ?Sized> Serialize for &'a T where T: Serialize);
deref_impl!(<'a, T: ?Sized> Serialize for &'a mut T where T: Serialize);

#[cfg(feature = "alloc")]
deref_impl!(<T: ?Sized> Serialize for Box<T> where T: Serialize);

#[cfg(all(feature = "rc", feature = "alloc"))]
deref_impl! {
    /// This impl requires the [`"rc"`] Cargo feature of Serde.
    ///
    /// Serializing a data structure containing `Rc` will serialize a copy of
    /// the contents of the `Rc` each time the `Rc` is referenced within the
    /// data structure. Serialization will not attempt to deduplicate these
    /// repeated data.
    ///
    /// [`"rc"`]: https://serde.rs/feature-flags.html#-features-rc
    <T: ?Sized> Serialize for Rc<T> where T: Serialize
}

#[cfg(all(feature = "rc", feature = "alloc"))]
deref_impl! {
    /// This impl requires the [`"rc"`] Cargo feature of Serde.
    ///
    /// Serializing a data structure containing `Arc` will serialize a copy of
    /// the contents of the `Arc` each time the `Arc` is referenced within the
    /// data structure. Serialization will not attempt to deduplicate these
    /// repeated data.
    ///
    /// [`"rc"`]: https://serde.rs/feature-flags.html#-features-rc
    <T: ?Sized> Serialize for Arc<T> where T: Serialize
}

#[cfg(feature = "alloc")]
deref_impl!(<'a, T: ?Sized> Serialize for Cow<'a, T> where T: Serialize + ToOwned);

////////////////////////////////////////////////////////////////////////////////

/// This impl requires the [`"rc"`] Cargo feature of Serde.
///
/// [`"rc"`]: https://serde.rs/feature-flags.html#-features-rc
#[cfg(all(feature = "rc", feature = "alloc"))]
impl<T: ?Sized> Serialize for RcWeak<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.upgrade().serialize(serializer)
    }
}

/// This impl requires the [`"rc"`] Cargo feature of Serde.
///
/// [`"rc"`]: https://serde.rs/feature-flags.html#-features-rc
#[cfg(all(feature = "rc", feature = "alloc"))]
impl<T: ?Sized> Serialize for ArcWeak<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.upgrade().serialize(serializer)
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! nonzero_integers {
    ( $( $T: ident, )+ ) => {
        $(
            #[cfg(num_nonzero)]
            impl Serialize for num::$T {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    self.get().serialize(serializer)
                }
            }
        )+
    }
}

nonzero_integers! {
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroUsize,
}

#[cfg(num_nonzero_signed)]
nonzero_integers! {
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroIsize,
}

// Currently 128-bit integers do not work on Emscripten targets so we need an
// additional `#[cfg]`
serde_if_integer128! {
    nonzero_integers! {
        NonZeroU128,
    }

    #[cfg(num_nonzero_signed)]
    nonzero_integers! {
        NonZeroI128,
    }
}

impl<T> Serialize for Cell<T>
where
    T: Serialize + Copy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.get().serialize(serializer)
    }
}

impl<T> Serialize for RefCell<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.try_borrow() {
            Ok(value) => value.serialize(serializer),
            Err(_) => Err(S::Error::custom("already mutably borrowed")),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T, E> Serialize for Result<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Result::Ok(ref value) => serializer.serialize_newtype_variant("Result", 0, "Ok", value),
            Result::Err(ref value) => {
                serializer.serialize_newtype_variant("Result", 1, "Err", value)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(core_duration)]
impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use super::SerializeStruct;
        let mut state = try!(serializer.serialize_struct("Duration", 2));
        try!(state.serialize_field("secs", &self.as_secs()));
        try!(state.serialize_field("nanos", &self.subsec_nanos()));
        state.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(core_reverse)]
impl<T> Serialize for Reverse<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}
