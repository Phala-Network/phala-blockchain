use alloc::borrow::Cow;
use alloc::vec::Vec;
use ink::ChainExtensionInstance;
use ink_lang as ink;

pub use http_request::{HttpRequest, HttpResponse};
pub use signing::{PublicKeyForArgs, SigType, SignArgs, VerifyArgs};

mod http_request;
pub mod signing;

#[cfg(feature = "std")]
pub mod test;

#[derive(scale::Encode, scale::Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct StorageQuotaExceeded;

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

    #[ink(extension = 1, handle_status = false, returns_result = false)]
    fn http_request(request: HttpRequest) -> HttpResponse;

    #[ink(extension = 2, handle_status = false, returns_result = false)]
    fn sign(args: SignArgs) -> Vec<u8>;

    #[ink(extension = 3, handle_status = false, returns_result = false)]
    fn verify(args: VerifyArgs) -> bool;

    #[ink(extension = 4, handle_status = false, returns_result = false)]
    fn derive_sr25519_key(salt: Cow<[u8]>) -> Vec<u8>;

    #[ink(extension = 5, handle_status = false, returns_result = false)]
    fn get_public_key(args: PublicKeyForArgs) -> Vec<u8>;

    /// Set a value in the local cache.
    ///
    /// The default expiration time is 7 days. Use `cache_set_expire` to set a custom expiration
    /// time.
    /// Values stored in cache can only be read in query functions.
    ///
    /// Alwasy returns `Ok(())` if it is called from a command context.
    #[ink(extension = 6, handle_status = false, returns_result = false)]
    fn cache_set(key: &[u8], value: &[u8]) -> Result<(), StorageQuotaExceeded>;

    /// Set the expiration time of a value in the local cache.
    ///
    /// Arguments:
    /// - `key`: The key of the value to set the expiration time for.
    /// - `expire`: The expiration time from now in seconds.
    #[ink(extension = 7, handle_status = false, returns_result = false)]
    fn cache_set_expire(key: &[u8], expire: u64) -> ();

    /// Get a value from the local cache.
    ///
    /// Only for query functions. Always returns `None` if it is called from a command context.
    #[ink(extension = 8, handle_status = false, returns_result = false)]
    fn cache_get(key: &[u8]) -> Option<Vec<u8>>;

    /// Remove a value from the local cache.
    ///
    /// Returns the removed value if it existed. Always returns `None` if it is called from a
    /// command context.
    #[ink(extension = 9, handle_status = false, returns_result = false)]
    fn cache_remove(args: &[u8]) -> Option<Vec<u8>>;

    /// Print log message.
    #[ink(extension = 10, handle_status = false, returns_result = false)]
    fn log(level: u8, message: &str);

    /// Get random bytes, for query only
    #[ink(extension = 11, handle_status = false, returns_result = false)]
    fn getrandom(length: u8) -> Vec<u8>;

    /// Check if it is running in a Command context.
    #[ink(extension = 12, handle_status = false, returns_result = false)]
    fn is_running_in_command() -> bool;
}

pub fn pink_extension_instance() -> <PinkExt as ChainExtensionInstance>::Instance {
    <PinkExt as ChainExtensionInstance>::instantiate()
}
