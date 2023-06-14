use alloc::borrow::Cow;
use alloc::vec::Vec;
use ink::ChainExtensionInstance;

pub use http_request::{HttpRequest, HttpRequestError, HttpResponse};
pub use ink::primitives::AccountId;
pub use signing::SigType;

use crate::{Balance, EcdsaPublicKey, EcdsaSignature, Hash};

mod http_request;
pub mod signing;

#[cfg(feature = "std")]
pub mod test;

#[derive(scale::Encode, scale::Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct StorageQuotaExceeded;

mod sealed {
    pub trait Sealed {}
}
pub trait CodableError: sealed::Sealed {
    fn encode(&self) -> u32;
    fn decode(code: u32) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! impl_codable_error_for {
    ($t: path) => {
        impl sealed::Sealed for $t {}
        impl CodableError for $t {
            fn encode(&self) -> u32 {
                1
            }

            fn decode(code: u32) -> Option<Self> {
                if code == 1 {
                    Some(Self)
                } else {
                    None
                }
            }
        }

        impl From<ErrorCode> for $t {
            fn from(value: ErrorCode) -> Self {
                match <Self as CodableError>::decode(value.0) {
                    None => crate::panic!("chain extension: invalid output"),
                    Some(err) => err,
                }
            }
        }

        impl From<scale::Error> for $t {
            fn from(_value: scale::Error) -> Self {
                crate::panic!("chain_ext: failed to decocde output")
            }
        }
    };
}
impl_codable_error_for!(StorageQuotaExceeded);

pub struct EncodeOutput<T>(pub T);

pub trait EncodeOutputFallbask {
    fn encode(self) -> (u32, Vec<u8>);
}

impl<T: scale::Encode, E: CodableError> EncodeOutput<Result<T, E>> {
    pub fn encode(self) -> (u32, Vec<u8>) {
        match self.0 {
            Ok(val) => (0, val.encode()),
            Err(err) => (err.encode(), Vec::new()),
        }
    }
}

impl<T: scale::Encode> EncodeOutputFallbask for EncodeOutput<T> {
    fn encode(self) -> (u32, Vec<u8>) {
        (0, self.0.encode())
    }
}

#[derive(scale::Encode, scale::Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ErrorCode(u32);

impl ink::env::chain_extension::FromStatusCode for ErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            _ => Err(ErrorCode(status_code)),
        }
    }
}

pub type BatchHttpResult = Result<Vec<Result<HttpResponse, HttpRequestError>>, HttpRequestError>;

/// Extensions for the ink runtime defined by phat contract.
#[pink_extension_macro::chain_extension]
pub trait PinkExt {
    type ErrorCode = ErrorCode;

    #[ink(extension = 1, handle_status = false)]
    fn http_request(request: HttpRequest) -> HttpResponse;

    #[ink(extension = 2, handle_status = false)]
    fn sign(sigtype: SigType, key: &[u8], message: &[u8]) -> Vec<u8>;

    #[ink(extension = 3, handle_status = false)]
    fn verify(sigtype: SigType, pubkey: &[u8], message: &[u8], signature: &[u8]) -> bool;

    #[ink(extension = 4, handle_status = false)]
    fn derive_sr25519_key(salt: Cow<[u8]>) -> Vec<u8>;

    #[ink(extension = 5, handle_status = false)]
    fn get_public_key(sigtype: SigType, key: &[u8]) -> Vec<u8>;

    /// Set a value in the local cache.
    ///
    /// The default expiration time is 7 days. Use `cache_set_expiration` to set a custom expiration
    /// time.
    /// Values stored in cache can only be read in query functions.
    ///
    /// Alwasy returns `Ok(())` if it is called from a command context.
    #[ink(extension = 6, handle_status = true)]
    fn cache_set(key: &[u8], value: &[u8]) -> Result<(), StorageQuotaExceeded>;

    /// Set the expiration time of a value in the local cache.
    ///
    /// Arguments:
    /// - `key`: The key of the value to set the expiration time for.
    /// - `expire`: The expiration time from now in seconds.
    #[ink(extension = 7, handle_status = false)]
    fn cache_set_expiration(key: &[u8], expire: u64);

    /// Get a value from the local cache.
    ///
    /// Only for query functions. Always returns `None` if it is called from a command context.
    #[ink(extension = 8, handle_status = false)]
    fn cache_get(key: &[u8]) -> Option<Vec<u8>>;

    /// Remove a value from the local cache.
    ///
    /// Returns the removed value if it existed. Always returns `None` if it is called from a
    /// command context.
    #[ink(extension = 9, handle_status = false)]
    fn cache_remove(args: &[u8]) -> Option<Vec<u8>>;

    /// Print log message.
    #[ink(extension = 10, handle_status = false)]
    fn log(level: u8, message: &str);

    /// Get random bytes, for query only
    #[ink(extension = 11, handle_status = false)]
    fn getrandom(length: u8) -> Vec<u8>;

    /// Check if it is running in a Command context.
    #[allow(clippy::wrong_self_convention)]
    #[ink(extension = 12, handle_status = false)]
    fn is_in_transaction() -> bool;

    #[ink(extension = 13, handle_status = false)]
    fn ecdsa_sign_prehashed(key: &[u8], message_hash: Hash) -> EcdsaSignature;

    #[ink(extension = 14, handle_status = false)]
    fn ecdsa_verify_prehashed(
        signature: EcdsaSignature,
        message_hash: Hash,
        pubkey: EcdsaPublicKey,
    ) -> bool;
    /// Get the contract id of the preinstalled pink-system
    #[ink(extension = 15, handle_status = false)]
    fn system_contract_id() -> AccountId;

    /// Get (total, free) balance of given contract
    #[ink(extension = 16, handle_status = false)]
    fn balance_of(account: AccountId) -> (Balance, Balance);

    /// Get worker public key. Query only.
    #[ink(extension = 17, handle_status = false)]
    fn worker_pubkey() -> crate::EcdhPublicKey;

    /// Get current millis since unix epoch from the OS. (Query only)
    #[ink(extension = 18, handle_status = false)]
    fn untrusted_millis_since_unix_epoch() -> u64;

    /// Check whether the code exists in the cluster storage.
    #[ink(extension = 19, handle_status = false)]
    fn code_exists(code_hash: Hash, sidevm: bool) -> bool;

    /// This loads the latest system contract code from chain storage to the cluster storage.
    ///
    /// Returns the code hash of the latest system contract code.
    #[ink(extension = 20, handle_status = false)]
    fn import_latest_system_code(payer: AccountId) -> Option<Hash>;

    /// Get the version of the current contract runtime in this cluster.
    #[ink(extension = 21, handle_status = false)]
    fn runtime_version() -> (u32, u32);

    /// Batch http request
    #[ink(extension = 22, handle_status = true)]
    fn batch_http_request(requests: Vec<HttpRequest>, timeout_ms: u64) -> BatchHttpResult;

    /// Get current event chain head info
    ///
    /// Returns (next event block number, last_event_block_hash)
    #[ink(extension = 23, handle_status = false)]
    fn current_event_chain_head() -> (u64, Hash);
}

pub fn pink_extension_instance() -> <PinkExt as ChainExtensionInstance>::Instance {
    <PinkExt as ChainExtensionInstance>::instantiate()
}
