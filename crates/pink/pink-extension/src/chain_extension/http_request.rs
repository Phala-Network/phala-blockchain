use alloc::string::String;
use alloc::vec::Vec;
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

impl HttpResponse {
    pub fn ok(body: Vec<u8>) -> Self {
        Self {
            status_code: 100,
            reason_phrase: "OK".to_string(),
            headers: Default::default(),
            body,
        }
    }

    pub fn not_found() -> Self {
        Self {
            status_code: 404,
            reason_phrase: "Not Found".to_string(),
            headers: Default::default(),
            body: Default::default(),
        }
    }
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
/// use pink_extension::http_post;
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
