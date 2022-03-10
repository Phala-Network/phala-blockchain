use alloc::vec::Vec;
use alloc::borrow::Cow;
use ink_lang as ink;
use ink::ChainExtensionInstance;

pub use http_request::{HttpRequest, HttpResponse};
pub use signing::{SigType, SignArgs, VerifyArgs, PublicKeyForArgs};
pub use local_cache::CacheSetArgs;

mod http_request;
mod signing;
mod local_cache;

#[cfg(feature = "std")]
pub mod test;

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
#[pink_extension_macro::chain_extension]
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
    fn derive_sr25519_key(salt: Cow<[u8]>) -> Vec<u8>;

    #[ink(extension = 0xff000005, handle_status = false, returns_result = false)]
    fn get_public_key(args: PublicKeyForArgs) -> Vec<u8>;

    #[ink(extension = 0xff000006, handle_status = false, returns_result = false)]
    fn cache_set(args: CacheSetArgs) -> ();

    #[ink(extension = 0xff000007, handle_status = false, returns_result = false)]
    fn cache_get(key: Cow<[u8]>) -> Option<Vec<u8>>;

    #[ink(extension = 0xff000008, handle_status = false, returns_result = false)]
    fn cache_remove(args: Cow<[u8]>) -> Option<Vec<u8>>;
}

pub fn pink_extension_instance() -> <PinkExt as ChainExtensionInstance>::Instance {
    <PinkExt as ChainExtensionInstance>::instantiate()
}
