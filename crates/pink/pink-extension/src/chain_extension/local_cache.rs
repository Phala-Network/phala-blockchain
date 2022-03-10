use alloc::borrow::Cow;

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct CacheSetArgs<'a> {
    pub key: Cow<'a, [u8]>,
    pub value: Cow<'a, [u8]>,
}


#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct CacheSetExpireArgs<'a> {
    pub key: Cow<'a, [u8]>,
    /// The expire time from now in seconds.
    pub expire: u64,
}
