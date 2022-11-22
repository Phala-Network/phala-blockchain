use alloc::borrow::Cow;
use alloc::vec::Vec;
use ink::ChainExtensionInstance;
use ink_lang as ink;

pub use http_request::{HttpRequest, HttpResponse};
pub use ink_env::AccountId;
pub use signing::SigType;

use crate::{Balance, EcdsaPublicKey, EcdsaSignature, Hash};

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
    fn sign(sigtype: SigType, key: &[u8], message: &[u8]) -> Vec<u8>;

    #[ink(extension = 3, handle_status = false, returns_result = false)]
    fn verify(sigtype: SigType, pubkey: &[u8], message: &[u8], signature: &[u8]) -> bool;

    #[ink(extension = 4, handle_status = false, returns_result = false)]
    fn derive_sr25519_key(salt: Cow<[u8]>) -> Vec<u8>;

    #[ink(extension = 5, handle_status = false, returns_result = false)]
    fn get_public_key(sigtype: SigType, key: &[u8]) -> Vec<u8>;

    /// Set a value in the local cache.
    ///
    /// The default expiration time is 7 days. Use `cache_set_expiration` to set a custom expiration
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
    fn cache_set_expiration(key: &[u8], expire: u64);

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
    #[allow(clippy::wrong_self_convention)]
    #[ink(extension = 12, handle_status = false, returns_result = false)]
    fn is_in_transaction() -> bool;

    #[ink(extension = 13, handle_status = false, returns_result = false)]
    fn ecdsa_sign_prehashed(key: &[u8], message_hash: Hash) -> EcdsaSignature;

    #[ink(extension = 14, handle_status = false, returns_result = false)]
    fn ecdsa_verify_prehashed(
        signature: EcdsaSignature,
        message_hash: Hash,
        pubkey: EcdsaPublicKey,
    ) -> bool;
    /// Get the contract id of the preinstalled pink-system
    #[ink(extension = 15, handle_status = false, returns_result = false)]
    fn system_contract_id() -> AccountId;

    /// Get (total, free) balance of given contract
    #[ink(extension = 16, handle_status = false, returns_result = false)]
    fn balance_of(account: AccountId) -> (Balance, Balance);

    /// Get worker public key. Query only.
    #[ink(extension = 17, handle_status = false, returns_result = false)]
    fn worker_pubkey() -> crate::EcdhPublicKey;

    /// Get current millis since unix epoch from the OS. (Query only)
    #[ink(extension = 18, handle_status = false, returns_result = false)]
    fn untrusted_millis_since_unix_epoch() -> u64;
}

pub fn pink_extension_instance() -> <PinkExt as ChainExtensionInstance>::Instance {
    <PinkExt as ChainExtensionInstance>::instantiate()
}
