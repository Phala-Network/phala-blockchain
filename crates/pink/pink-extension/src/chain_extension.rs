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
    pub reason_phrase: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Extensions for the ink runtime defined by fat contract.
#[ink::chain_extension]
pub trait PinkExt {
    type ErrorCode = ErrorCode;

    // func_id refer to https://github.com/patractlabs/PIPs/blob/main/PIPs/pip-100.md
    #[ink(extension = 0xff000001, handle_status = false, returns_result = false)]
    fn http_request(request: HttpRequest) -> HttpResponse;
}

/// Make a simple HTTP GET request
///
/// # Arguments
/// url: The URL to GET
/// headers: The headers to send with the request
///
/// # Example
/// ```ignore
/// use pink_extension::http_get;
/// let response = http_get!("https://example.com/");
/// assert_eq!(response.status_code, 200);
/// ```
#[macro_export]
macro_rules! http_get {
    ($url: expr, $headers: expr) => {{
        use pink_extension::chain_extension::{HttpRequest, HttpResponse};
        let headers = $headers;
        let request = HttpRequest {
            url: $url.into(),
            method: "GET".into(),
            headers,
            body: Default::default(),
        };
        Self::env().extension().http_request(request)
    }};
    ($url: expr) => {{
        $crate::http_get!($url, Default::default())
    }};
}


/// Make a simple HTTP POST request
///
/// # Arguments
/// url: The URL to POST
/// data: The payload to POST
/// headers: The headers to send with the request
///
/// # Example
/// ```ignore
/// use pink_extension::http_get;
/// let response = http_post!("https://example.com/", b"Hello, world!");
/// assert_eq!(response.status_code, 200);
/// ```
#[macro_export]
macro_rules! http_post {
    ($url: expr, $data: expr, $headers: expr) => {{
        use pink_extension::chain_extension::{HttpRequest, HttpResponse};
        let headers = $headers;
        let body = $data.into();
        let request = HttpRequest {
            url: $url.into(),
            method: "POST".into(),
            headers,
            body,
        };
        Self::env().extension().http_request(request)
    }};
    ($url: expr, $data: expr) => {{
        $crate::http_post!($url, $data, Default::default())
    }};
}
