use pink_extension::chain_extension as ext;
use pink_extension::chain_extension::mock::mock_all_with;
use sp_core::crypto::AccountId32;

pub struct MockExtension;

impl super::PinkRuntimeEnv for MockExtension {
    type AccountId = AccountId32;

    fn address(&self) -> &Self::AccountId {
        // It's not important in mocking env
        static ADDRESS: AccountId32 = AccountId32::new([0; 32]);
        &ADDRESS
    }

    fn call_elapsed(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs(0))
    }
}

impl ext::PinkExtBackend for MockExtension {
    fn http_request(&self, request: ext::HttpRequest) -> Result<ext::HttpResponse, Self::Error> {
        super::DefaultPinkExtension::new(self).http_request(request)
    }

    fn sign(&self, args: ext::SignArgs) -> Result<Vec<u8>, Self::Error> {
        super::DefaultPinkExtension::new(self).sign(args)
    }

    fn verify(&self, args: ext::VerifyArgs) -> Result<bool, Self::Error> {
        super::DefaultPinkExtension::new(self).verify(args)
    }

    fn derive_sr25519_key(&self, salt: std::borrow::Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        super::DefaultPinkExtension::new(self).derive_sr25519_key(salt)
    }

    fn get_public_key(&self, args: ext::PublicKeyForArgs) -> Result<Vec<u8>, Self::Error> {
        super::DefaultPinkExtension::new(self).get_public_key(args)
    }

    fn cache_set(
        &self,
        _key: std::borrow::Cow<[u8]>,
        _value: std::borrow::Cow<[u8]>,
    ) -> Result<Result<(), ext::StorageQuotaExceeded>, Self::Error> {
        Ok(Ok(()))
    }

    fn cache_set_expire(
        &self,
        _key: std::borrow::Cow<[u8]>,
        _expire: u64,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn cache_get(&self, _key: std::borrow::Cow<[u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }

    fn cache_remove(&self, _args: std::borrow::Cow<[u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }

    fn log(&self, level: u8, message: std::borrow::Cow<str>) -> Result<(), Self::Error> {
        super::DefaultPinkExtension::new(self).log(level, message)
    }

    type Error = String;
}

pub fn mock_all_ext() {
    mock_all_with(&MockExtension)
}
