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

    /// Make a HTTP request.
    ///
    /// # Arguments
    ///
    /// * `request`: A `HttpRequest` struct containing all the details for the HTTP request.
    ///
    /// # Returns
    ///
    /// * `HttpResponse` - The response from the HTTP request which contains the status code, headers, and body.
    ///
    /// # Example
    ///
    /// ```
    /// let request = HttpRequest::new("https://httpbin.org/get", "GET", Defualt::default(), Defualt::default());
    /// let response = pink::ext().http_request(request);
    /// ```
    ///
    /// There are also some shortcut macros for this function:
    /// - [`crate::http_get!`]
    /// - [`crate::http_post!`]
    /// - [`crate::http_put!`]
    ///
    /// # Availability
    /// any contract | query only
    #[ink(extension = 1, handle_status = false)]
    fn http_request(request: HttpRequest) -> HttpResponse;

    /// Sign a message with a given key.
    ///
    /// # Arguments
    ///
    /// * `sigtype`: The signature type to use for signing the message.
    /// * `key`: The private key used for signing the message.
    /// * `message`: The message to be signed.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The signed message as a byte vector.
    ///
    /// # Example
    ///
    /// ```
    /// let derived_key = pink::ext().derive_sr25519_key(b"some salt".into());
    /// let message = b"Hello, world!";
    /// let signature = pink::ext().sign(SigType::Sr25519, &key, message);
    /// let pubkey = pink::ext().get_public_key(SigType::Sr25519, &derived_key);
    /// let is_valid = pink::ext().verify(SigType::Sr25519, &pubkey, message, &signature);
    /// ```
    ///
    /// # Availability
    /// For SigType::Sr25519:
    ///     any contract | query only
    ///
    /// For Others:
    ///     any contract | query | transaction
    #[ink(extension = 2, handle_status = false)]
    fn sign(sigtype: SigType, key: &[u8], message: &[u8]) -> Vec<u8>;

    /// Verify a signature.
    ///
    /// This method verifies a digital signature given the signature type, public key, message, and signature.
    ///
    /// # Arguments
    ///
    /// * `sigtype`: The type of signature to verify.
    /// * `pubkey`: The public key associated with the private key that signed the message.
    /// * `message`: The original message that was signed.
    /// * `signature`: The digital signature to verify.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the signature is valid, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// let derived_key = pink::ext().derive_sr25519_key(b"some salt".into());
    /// let message = b"Hello, world!";
    /// let signature = pink::ext().sign(SigType::Sr25519, &key, message);
    /// let pubkey = pink::ext().get_public_key(SigType::Sr25519, &derived_key);
    /// let is_valid = pink::ext().verify(SigType::Sr25519, &pubkey, message, &signature);
    /// ```
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 3, handle_status = false)]
    fn verify(sigtype: SigType, pubkey: &[u8], message: &[u8], signature: &[u8]) -> bool;

    /// Derive a sr25519 key.
    ///
    /// This method derives a sr25519 key using the provided salt and the contract private key.
    /// The derived key is deterministic so it could be used in transactions to sign messages.
    ///
    /// # Arguments
    ///
    /// * `salt`: The salt to use in the key derivation function.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The derived sr25519 key as a byte vector.
    ///
    /// # Example
    ///
    /// ```
    /// let derived_key = pink::ext().derive_sr25519_key(b"some salt".into());
    /// ```
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 4, handle_status = false)]
    fn derive_sr25519_key(salt: Cow<[u8]>) -> Vec<u8>;

    /// Get the public key associated with a private key.
    ///
    /// This method takes a signature type and private key and returns the associated public key.
    ///
    /// # Arguments
    ///
    /// * `sigtype`: The type of signature to generate the public key for.
    /// * `key`: The private key used to generate the public key.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The public key associated with the given private key as a byte vector.
    ///
    /// # Example
    ///
    /// ```
    /// let derived_key = pink::ext().derive_sr25519_key(b"some salt".into());
    /// let pubkey = pink::ext().get_public_key(SigType::Sr25519, &derived_key);
    /// ```
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 5, handle_status = false)]
    fn get_public_key(sigtype: SigType, key: &[u8]) -> Vec<u8>;

    /// Set a value in the local cache.
    ///
    /// This method sets a value in the local cache with the default expiration time of 7 days.
    /// To set a custom expiration time, use `cache_set_expiration`.
    /// Values stored in cache can only be read in query functions.
    /// Always returns `Ok(())` if it is called from a transaction context.
    ///
    /// # Arguments
    ///
    /// * `key`: The key used to identify the value in the cache.
    /// * `value`: The value to be stored in the cache.
    ///
    /// # Returns
    ///
    /// * `Result<(), StorageQuotaExceeded>` - `Ok(())` or `Err(StorageQuotaExceeded)` if the storage quota is exceeded.
    ///
    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>Warning:</strong>
    /// The cache is not guaranteed to be persistent. It may be cleared at any time due
    ///     to various reasons:
    ///
    /// - The cached item is expired.
    /// - The entire cache in pRuntime is full and a new value needs to be stored (either from the contract itself or
    ///   other contracts).
    /// - The worker is restarted.
    /// </p>
    ///
    /// In order to use cache, the contract need to be staked via the phala on-chain API `PhatTokenomic::adjust_stake`.
    /// All contracts will share the 20MB cache storage by the ratio of stake.
    ///
    /// # Example
    ///
    /// ```
    /// let key = b"my key";
    /// let value = b"my value";
    /// let result = pink::ext().cache_set(key, value);
    /// ```
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 6, handle_status = true)]
    fn cache_set(key: &[u8], value: &[u8]) -> Result<(), StorageQuotaExceeded>;

    /// Set the expiration time of a value in the local cache.
    ///
    /// This method sets the expiration time for a given key in the local cache.
    ///
    /// # Arguments
    ///
    /// * `key`: The key of the value to set the expiration time for.
    /// * `expire`: The expiration time from now in seconds.
    ///
    /// # Example
    ///
    /// ```
    /// let key = b"my key";
    /// let expire = 60; // 1 minute
    /// pink::ext().cache_set_expiration(key, expire);
    /// ```
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 7, handle_status = false)]
    fn cache_set_expiration(key: &[u8], expire: u64);

    /// Get a value from the local cache.
    ///
    /// This method retrieves a value from the local cache. It can only be used in query functions.
    /// If called from a command context, it will always return `None`.
    ///
    /// # Arguments
    ///
    /// * `key`: The key used to identify the value in the cache.
    ///
    /// # Returns
    ///
    /// * `Option<Vec<u8>>` - The value from the cache as a byte vector wrapped in an Option,
    ///     or `None` if the value does not exist or called in transaction.
    ///
    /// # Example
    ///
    /// ```
    /// let key = b"my key";
    /// let value = pink::ext().cache_get(key);
    /// ```
    ///
    /// # Availability
    /// any contract | query
    #[ink(extension = 8, handle_status = false)]
    fn cache_get(key: &[u8]) -> Option<Vec<u8>>;

    /// Remove a value from the local cache.
    ///
    /// This method removes a value from the local cache and returns the removed value if it existed.
    /// If called from a command context, it will always return `None`.
    ///
    /// # Arguments
    ///
    /// * `args`: The key used to identify the value in the cache.
    ///
    /// # Returns
    ///
    /// * `Option<Vec<u8>>` - The removed value as a byte vector wrapped in an Option
    ///     or `None` if the value did not exist or called in transaction.
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 9, handle_status = false)]
    fn cache_remove(args: &[u8]) -> Option<Vec<u8>>;

    /// Log a message.
    ///
    /// This method logs a message at a given level.
    ///
    /// # Arguments
    ///
    /// * `level`: The level of the log message.
    /// * `message`: The message to be logged.
    ///
    /// # Example
    ///
    /// ```
    /// let level = 1;
    /// let message = "Hello, world!";
    /// pink::ext().log(level, message);
    /// ```
    ///
    /// # Note
    /// This is the low-level method for logging. It is recommended to use shortcuts macros below instead:
    ///
    /// - [`crate::debug!`]
    /// - [`crate::info!`]
    /// - [`crate::warn!`]
    /// - [`crate::error!`]
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 10, handle_status = false)]
    fn log(level: u8, message: &str);

    /// Get random bytes.
    ///
    /// This method generates a vector of random bytes of a given length. It returns random bytes
    /// generated by hardware RNG. So it is not deterministic and only available in a query context.
    ///
    /// # Note
    /// It always returns an empty vec![] when called in a transaction.
    ///
    ///
    /// # Arguments
    ///
    /// * `length`: The length of the random bytes vector.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - A vector of random bytes of the given length.
    ///
    /// # Example
    ///
    /// ```
    /// let length = 32;
    /// let random_bytes = pink::ext().getrandom(length);
    /// ```
    ///
    /// # Availability
    /// any contract | query only
    #[ink(extension = 11, handle_status = false)]
    fn getrandom(length: u8) -> Vec<u8>;

    /// Check if it is running in a Command context.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if it is running in a Command context, `false` if in query.
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 12, handle_status = false)]
    fn is_in_transaction() -> bool;

    /// Sign a prehashed message with a given key.
    ///
    /// This method uses the given key and prehashed message to create a ECDSA signature.
    ///
    /// # Arguments
    ///
    /// * `key`: The private key used for signing the message.
    /// * `message_hash`: The prehashed message to be signed.
    ///
    /// # Returns
    ///
    /// * `EcdsaSignature` - The signature of the message.
    ///
    /// # Example
    ///
    /// ```
    /// let key = [0u8; 32]; // replace with actual key
    /// let message_hash = Hash::zero(); // replace with actual hash
    /// let signature = pink::ext().ecdsa_sign_prehashed(&key, message_hash);
    /// ```
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 13, handle_status = false)]
    fn ecdsa_sign_prehashed(key: &[u8], message_hash: Hash) -> EcdsaSignature;

    /// Verify a prehashed ECDSA signature.
    ///
    /// This method verifies a prehashed ECDSA signature given the signature, prehashed message, and public key.
    ///
    /// # Arguments
    ///
    /// * `signature`: The ECDSA digital signature to verify.
    /// * `message_hash`: The prehashed original message that was signed.
    /// * `pubkey`: The public key associated with the private key that signed the message.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the signature is valid, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// let signature = EcdsaSignature::default(); // replace with actual signature
    /// let message_hash = Hash::zero(); // replace with actual hash
    /// let pubkey = EcdsaPublicKey::default(); // replace with actual pubkey
    /// let is_valid = pink::ext().ecdsa_verify_prehashed(signature, message_hash, pubkey);
    /// ```
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 14, handle_status = false)]
    fn ecdsa_verify_prehashed(
        signature: EcdsaSignature,
        message_hash: Hash,
        pubkey: EcdsaPublicKey,
    ) -> bool;

    /// Get the contract id of the preinstalled system contract.
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 15, handle_status = false)]
    fn system_contract_id() -> AccountId;

    /// Get balance of a given contract.
    ///
    /// # Arguments
    ///
    /// * `account`: The `AccountId` of the contract.
    ///
    /// # Returns
    ///
    /// * `(Balance, Balance)` - The total and free balance of a given contract.
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 16, handle_status = false)]
    fn balance_of(account: AccountId) -> (Balance, Balance);

    /// Get the worker public key.
    ///
    /// # Returns
    ///
    /// * `crate::EcdhPublicKey` - The public key of the worker.
    ///
    /// # Availability
    /// any contract | query only
    #[ink(extension = 17, handle_status = false)]
    fn worker_pubkey() -> crate::EcdhPublicKey;

    /// Get current millis since Unix epoch.
    ///
    /// This method returns the current time as milliseconds since the Unix epoch from the OS.
    ///
    /// # Returns
    ///
    /// * `u64` - The current time as milliseconds since the Unix epoch from the OS.
    ///
    /// # Note
    /// Because this method uses the OS time, it is not deterministic and may be manipulated by compromised OS.
    ///
    /// # Example
    ///
    /// ```
    /// let current_millis = pink::ext().untrusted_millis_since_unix_epoch();
    /// ```
    ///
    /// # Availability
    /// any contract | query only
    #[ink(extension = 18, handle_status = false)]
    fn untrusted_millis_since_unix_epoch() -> u64;

    /// Check whether the code exists in the cluster storage.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the code exists, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// let code_hash = Hash::zero(); // replace with actual code hash
    /// let exists = pink::ext().code_exists(code_hash, false);
    /// ```
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 19, handle_status = false)]
    fn code_exists(code_hash: Hash, sidevm: bool) -> bool;

    /// Import the latest system contract code from chain storage to the cluster storage.
    ///
    /// # Returns
    ///
    /// * `Option<Hash>` - The code hash of the latest system contract code, or `None` if the import failed.
    ///
    /// # Example
    ///
    /// ```
    /// let payer = AccountId::default(); // replace with actual payer id
    /// let code_hash = pink::ext().import_latest_system_code(payer);
    /// ```
    ///
    /// # Availability
    /// system only | query | transaction
    #[ink(extension = 20, handle_status = false)]
    fn import_latest_system_code(payer: AccountId) -> Option<Hash>;

    /// Get the version of the current contract runtime.
    ///
    /// # Returns
    ///
    /// * `(u32, u32)` - The version of the current contract runtime in this cluster as a tuple (major, minor).
    ///
    /// # Example
    ///
    /// ```
    /// let (major, minor) = pink::ext().runtime_version();
    /// ```
    ///
    /// # Availability
    /// any contract | query | transaction
    #[ink(extension = 21, handle_status = false)]
    fn runtime_version() -> (u32, u32);

    /// Batch HTTP request.
    ///
    /// This method sends a batch of HTTP requests with a given timeout and returns the results.
    ///
    /// # Arguments
    ///
    /// * `requests`: A vector of `HttpRequest` structs containing all the details for the HTTP requests.
    /// * `timeout_ms`: The timeout for the batch request in milliseconds.
    ///
    /// # Returns
    ///
    /// * `BatchHttpResult` - A vector of response to eahch HTTP requests.
    ///
    /// # Example
    ///
    /// ```
    /// let requests = vec![
    ///     HttpRequest::new("https://httpbin.org/get",
    ///         "GET",
    ///         Default::default(),
    ///         Default::default(),
    ///     ),
    ///     HttpRequest::new("https://httpbin.org/post",
    ///         "POST",
    ///         Default::default(),
    ///         b"Hello, world!".to_vec(),
    ///     ),
    /// ];
    /// let result = pink::ext().batch_http_request(requests, 5000);
    /// ```
    #[ink(extension = 22, handle_status = true)]
    fn batch_http_request(requests: Vec<HttpRequest>, timeout_ms: u64) -> BatchHttpResult;
}

pub fn pink_extension_instance() -> <PinkExt as ChainExtensionInstance>::Instance {
    <PinkExt as ChainExtensionInstance>::instantiate()
}
