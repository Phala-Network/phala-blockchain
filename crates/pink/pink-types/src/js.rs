use alloc::string::String;
use alloc::vec::Vec;

#[derive(scale::Encode, scale::Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum JsCode {
    Source(String),
    Bytecode(Vec<u8>),
}

#[derive(scale::Encode, scale::Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum JsValue {
    Undefined,
    Null,
    String(String),
    Bytes(Vec<u8>),
    Other(String),
    Exception(String),
}
