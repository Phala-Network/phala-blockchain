use std::borrow::Cow;

use pink::{
    chain_extension::{self as ext, mock::mock_all_with, JsCode, JsValue, SigType},
    types::sgx::{AttestationType, SgxQuote},
    EcdsaPublicKey, EcdsaSignature, Hash,
};
use sp_core::crypto::AccountId32;

use crate::local_cache;

pub struct MockExtension;

impl super::PinkRuntimeEnv for MockExtension {
    type AccountId = AccountId32;

    fn address(&self) -> &Self::AccountId {
        // It's not important in mocking env
        static ADDRESS: AccountId32 = AccountId32::new([0; 32]);
        &ADDRESS
    }
}

impl ext::PinkExtBackend for MockExtension {
    type Error = String;

    fn http_request(&self, request: ext::HttpRequest) -> Result<ext::HttpResponse, Self::Error> {
        super::DefaultPinkExtension::new(self).http_request(request)
    }

    fn batch_http_request(
        &self,
        requests: Vec<ext::HttpRequest>,
        timeout_ms: u64,
    ) -> Result<ext::BatchHttpResult, Self::Error> {
        super::DefaultPinkExtension::new(self).batch_http_request(requests, timeout_ms)
    }

    fn sign(
        &self,
        sigtype: SigType,
        key: Cow<[u8]>,
        message: Cow<[u8]>,
    ) -> Result<Vec<u8>, Self::Error> {
        super::DefaultPinkExtension::new(self).sign(sigtype, key, message)
    }

    fn verify(
        &self,
        sigtype: SigType,
        pubkey: Cow<[u8]>,
        message: Cow<[u8]>,
        signature: Cow<[u8]>,
    ) -> Result<bool, Self::Error> {
        super::DefaultPinkExtension::new(self).verify(sigtype, pubkey, message, signature)
    }

    fn derive_sr25519_key(&self, salt: std::borrow::Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        super::DefaultPinkExtension::new(self).derive_sr25519_key(salt)
    }

    fn get_public_key(&self, sigtype: SigType, key: Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        super::DefaultPinkExtension::new(self).get_public_key(sigtype, key)
    }

    fn cache_set(
        &self,
        key: Cow<[u8]>,
        value: Cow<[u8]>,
    ) -> Result<Result<(), ext::StorageQuotaExceeded>, Self::Error> {
        Ok(local_cache::set(&[], &key, &value))
    }

    fn cache_set_expiration(&self, key: Cow<[u8]>, expire: u64) -> Result<(), Self::Error> {
        local_cache::set_expiration(&[], &key, expire);
        Ok(())
    }

    fn cache_get(&self, key: Cow<'_, [u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(local_cache::get(&[], &key))
    }

    fn cache_remove(&self, key: Cow<'_, [u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(local_cache::remove(&[], &key))
    }

    fn log(&self, level: u8, message: std::borrow::Cow<str>) -> Result<(), Self::Error> {
        super::DefaultPinkExtension::new(self).log(level, message)
    }

    fn getrandom(&self, length: u8) -> Result<Vec<u8>, Self::Error> {
        super::DefaultPinkExtension::new(self).getrandom(length)
    }

    fn is_in_transaction(&self) -> Result<bool, Self::Error> {
        Ok(IS_COMMAND_MODE.with(|mode| mode.get()))
    }

    fn ecdsa_sign_prehashed(
        &self,
        key: Cow<[u8]>,
        message_hash: Hash,
    ) -> Result<EcdsaSignature, Self::Error> {
        super::DefaultPinkExtension::new(self).ecdsa_sign_prehashed(key, message_hash)
    }

    fn ecdsa_verify_prehashed(
        &self,
        signature: EcdsaSignature,
        message_hash: Hash,
        pubkey: EcdsaPublicKey,
    ) -> Result<bool, Self::Error> {
        super::DefaultPinkExtension::new(self).ecdsa_verify_prehashed(
            signature,
            message_hash,
            pubkey,
        )
    }

    fn system_contract_id(&self) -> Result<ext::AccountId, Self::Error> {
        Err("No default system contract id".into())
    }

    fn balance_of(
        &self,
        _account: ext::AccountId,
    ) -> Result<(pink::Balance, pink::Balance), Self::Error> {
        Ok((0, 0))
    }

    fn untrusted_millis_since_unix_epoch(&self) -> Result<u64, Self::Error> {
        super::DefaultPinkExtension::new(self).untrusted_millis_since_unix_epoch()
    }

    fn worker_pubkey(&self) -> Result<crate::EcdhPublicKey, Self::Error> {
        Ok(Default::default())
    }

    fn code_exists(&self, _code_hash: Hash, _sidevm: bool) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn import_latest_system_code(
        &self,
        _payer: ext::AccountId,
    ) -> Result<Option<Hash>, Self::Error> {
        Ok(None)
    }

    fn runtime_version(&self) -> Result<(u32, u32), Self::Error> {
        Ok((1, 0))
    }

    fn current_event_chain_head(&self) -> Result<(u64, Hash), Self::Error> {
        Ok((0, Default::default()))
    }

    fn js_eval(&self, codes: Vec<JsCode>, args: Vec<String>) -> Result<JsValue, Self::Error> {
        super::DefaultPinkExtension::new(self).js_eval(codes, args)
    }

    fn worker_sgx_quote(&self) -> Result<Option<SgxQuote>, Self::Error> {
        let quote = include_bytes!("mock-quote.bin").to_vec();
        Ok(Some(SgxQuote {
            attestation_type: AttestationType::Dcap,
            quote,
        }))
    }
}

thread_local! {
    static IS_COMMAND_MODE: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

pub fn set_mode(is_command: bool) {
    IS_COMMAND_MODE.with(|cell| cell.set(is_command));
}

pub fn mock_all_ext() {
    local_cache::enable_test_mode();
    let default_caller: &[u8] = &[];
    local_cache::apply_quotas([(default_caller, 1024 * 1024 * 20)]);
    mock_all_with(&MockExtension)
}

#[cfg(test)]
mod tests {
    use crate::PinkRuntimeEnv;
    use pink::chain_extension::{HttpRequest, PinkExtBackend};

    use super::*;

    #[test]
    fn http_request_works() {
        mock_all_ext();
        let ext = MockExtension;
        assert_eq!(ext.address(), &AccountId32::new([0; 32]));
        let response = ext.http_request(HttpRequest {
            method: "GET".into(),
            url: "https://httpbin.org/get".into(),
            body: Default::default(),
            headers: Default::default(),
        });
        assert!(response.is_ok());
        let responses = ext
            .batch_http_request(
                vec![
                    HttpRequest {
                        method: "GET".into(),
                        url: "https://httpbin.org/get".into(),
                        body: Default::default(),
                        headers: Default::default(),
                    },
                    HttpRequest {
                        method: "GET".into(),
                        url: "https://httpbin.org/get".into(),
                        body: Default::default(),
                        headers: Default::default(),
                    },
                ],
                1000,
            )
            .unwrap()
            .unwrap();
        assert_eq!(responses.len(), 2);
        for response in responses {
            assert!(response.is_ok());
        }
    }

    #[test]
    fn sign_works() {
        mock_all_ext();
        let ext = MockExtension;
        let key = ext.derive_sr25519_key(Default::default()).unwrap();
        let message = b"hello world";
        let signature = ext
            .sign(
                SigType::Sr25519,
                Cow::Borrowed(&key),
                Cow::Borrowed(message),
            )
            .unwrap();
        let pubkey = ext
            .get_public_key(SigType::Sr25519, Cow::Borrowed(&key))
            .unwrap();
        assert_eq!(signature.len(), 64);
        let ok = ext
            .verify(
                SigType::Sr25519,
                Cow::Borrowed(&pubkey),
                Cow::Borrowed(message),
                Cow::Borrowed(&signature),
            )
            .unwrap();
        assert!(ok);
    }

    #[test]
    fn cache_works() {
        mock_all_ext();
        let ext = MockExtension;
        let key = b"hello";
        let value = b"world";
        let result = ext.cache_set(Cow::Borrowed(key), Cow::Borrowed(value));
        assert!(result.is_ok());
        let result = ext.cache_get(Cow::Borrowed(key));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), value);
        let result = ext.cache_remove(Cow::Borrowed(key));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), value);
        let result = ext.cache_get(Cow::Borrowed(key));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let result = ext.cache_set(Cow::Borrowed(key), Cow::Borrowed(value));
        assert!(result.is_ok());
        ext.cache_set_expiration(Cow::Borrowed(key), 0).unwrap();
        let result = ext.cache_get(Cow::Borrowed(key));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn log_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.log(0, "hello world".into());
        assert!(result.is_ok());
    }

    #[test]
    fn getrandom_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.getrandom(32);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn ecdsa_works() {
        mock_all_ext();
        let ext = MockExtension;
        let key = [1u8; 32];
        let message = b"hello world";
        let signature = ext
            .ecdsa_sign_prehashed(Cow::Borrowed(&key), sp_core::blake2_256(message))
            .unwrap();
        let pubkey = ext
            .get_public_key(SigType::Ecdsa, Cow::Borrowed(&key))
            .unwrap();
        let ok = ext
            .ecdsa_verify_prehashed(
                signature,
                sp_core::blake2_256(message),
                pubkey.try_into().unwrap(),
            )
            .unwrap();
        assert!(ok);
    }

    #[test]
    fn system_contract_id_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.system_contract_id();
        assert!(result.is_err());
    }

    #[test]
    fn balance_of_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.balance_of([0u8; 32].into());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (0, 0));
    }

    #[test]
    fn untrusted_millis_since_unix_epoch_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.untrusted_millis_since_unix_epoch();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn worker_pubkey_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.worker_pubkey();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0u8; 32]);
    }

    #[test]
    fn code_exists_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.code_exists(Default::default(), false);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn import_latest_system_code_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.import_latest_system_code([0u8; 32].into());
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn runtime_version_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.runtime_version();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (1, 0));
    }

    #[test]
    fn current_event_chain_head_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.current_event_chain_head();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (0, Default::default()));
    }

    #[test]
    fn js_eval_works() {
        mock_all_ext();
        let ext = MockExtension;
        let result = ext.js_eval(
            vec![JsCode::Source("1 + 1".into())],
            vec!["".into(), "".into()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn exec_mode_works() {
        mock_all_ext();
        let ext = MockExtension;
        set_mode(true);
        assert!(ext.is_in_transaction().unwrap());
    }
}
