use std::borrow::Cow;

use pink_extension::chain_extension::mock::mock_all_with;
use pink_extension::chain_extension::SigType;
use pink_extension::{chain_extension as ext, EcdsaPublicKey, EcdsaSignature, Hash};
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
    ) -> Result<(pink_extension::Balance, pink_extension::Balance), Self::Error> {
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
