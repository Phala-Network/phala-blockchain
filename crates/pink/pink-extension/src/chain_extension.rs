use alloc::vec::Vec;
use ink_lang as ink;

pub use http_request::{HttpRequest, HttpResponse};
pub use signing::{SigType, SignArgs, VerifyArgs};

mod http_request;
mod signing;

pub mod test;
pub mod func_ids {
    pub const HTTP_REQUEST: u32 = 0xff000001;
    pub const SIGN: u32 = 0xff000002;
    pub const VERIFY: u32 = 0xff000003;
    pub const DERIVE_SR25519_PAIR: u32 = 0xff000004;
}

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

/// Extensions for the ink runtime defined by fat contract.
#[ink::chain_extension]
pub trait PinkExt {
    type ErrorCode = ErrorCode;

    // func_id refer to https://github.com/patractlabs/PIPs/blob/main/PIPs/pip-100.md
    #[ink(extension = 0xff000001, handle_status = false, returns_result = false)]
    fn http_request(request: HttpRequest) -> HttpResponse;

    #[ink(extension = 0xff000002, handle_status = false, returns_result = false)]
    fn sign(args: SignArgs) -> Vec<u8>;

    #[ink(extension = 0xff000003, handle_status = false, returns_result = false)]
    fn verify(args: VerifyArgs) -> bool;

    #[ink(extension = 0xff000004, handle_status = false, returns_result = false)]
    fn derive_sr25519_pair(salt: &[u8]) -> (Vec<u8>, Vec<u8>);
}
