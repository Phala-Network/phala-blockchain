use alloc::string::String;
use alloc::vec::Vec;

#[derive(scale::Encode, scale::Decode, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum JsCode {
    Source(String),
    Bytecode(Vec<u8>),
}

#[derive(scale::Encode, scale::Decode, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum JsValue {
    Undefined,
    Null,
    String(String),
    Bytes(Vec<u8>),
    Other(String),
    Exception(String),
}

#[test]
fn coverage() {
    use alloc::string::ToString;

    fn test<T: scale::Encode + scale::Decode + core::fmt::Debug + PartialEq + Eq + Clone>(
        value: T,
    ) {
        let encoded = scale::Encode::encode(&value.clone());
        let decoded = T::decode(&mut &encoded[..]).unwrap();
        assert_eq!(value, decoded);
        assert!(!alloc::format!("{value:?}").is_empty());
    }
    test(JsCode::Source("".to_string()));
    test(JsCode::Bytecode(alloc::vec![]));
    test(JsValue::Undefined);
    test(JsValue::Null);
    test(JsValue::String("".to_string()));
    test(JsValue::Bytes(alloc::vec![]));
    test(JsValue::Other("".to_string()));
    test(JsValue::Exception("".to_string()));
}
