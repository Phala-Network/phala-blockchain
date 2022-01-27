use alloc::string::String;
use alloc::vec::Vec;

use ink_lang as ink;

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ErrorCode {}

impl ink_env::chain_extension::FromStatusCode for ErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[ink::chain_extension]
pub trait PinkExt {
    type ErrorCode = ErrorCode;

    // func_id refer to https://github.com/patractlabs/PIPs/blob/main/PIPs/pip-100.md
    #[ink(extension = 0xff000001, handle_status = false, returns_result = false)]
    fn http_request(request: HttpRequest) -> HttpResponse;
}

#[macro_export]
macro_rules! http_get {
    ($url: expr) => {{
        use pink_extension::chain_extension::{HttpRequest, HttpResponse};
        let request = HttpRequest {
            url: $url.into(),
            method: "GET".into(),
            headers: Default::default(),
            body: Default::default(),
        };
        Self::env().extension().http_request(request)
    }};
}
