use std::borrow::Cow;

use log::error;
use frame_support::traits::Currency;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, Result as ExtResult, RetVal,
};
use phala_crypto::sr25519::{Persistence, KDF};
use phala_types::contract::ConvertTo;
use pink_extension::{
    chain_extension::{
        self as ext, HttpRequest, HttpResponse, PinkExtBackend, SigType, StorageQuotaExceeded,
    },
    dispatch_ext_call, CacheOp, EcdhPublicKey, EcdsaPublicKey, EcdsaSignature, Hash, PinkEvent,
};
use pink_extension_runtime::{DefaultPinkExtension, PinkRuntimeEnv};
use scale::{Decode, Encode};
use sp_runtime::{AccountId32, DispatchError};

use crate::{capi::OCallImpl, types::AccountId};

use pink_capi::{types::ExecSideEffects, v1::ocall::OCallsRo};

use super::{pallet_pink, PinkRuntime, SystemEvents};
use crate::runtime::Pink as PalletPink;
type Error = pallet_pink::Error<PinkRuntime>;

fn deposit_pink_event(contract: AccountId, event: PinkEvent) {
    let topics = [pink_extension::PinkEvent::event_topic().into()];
    let event = super::RuntimeEvent::Contracts(pallet_contracts::Event::ContractEmitted {
        contract,
        data: event.encode(),
    });
    super::System::deposit_event_indexed(&topics[..], event);
}

pub fn get_side_effects() -> (SystemEvents, ExecSideEffects) {
    let mut pink_events = Vec::default();
    let mut ink_events = Vec::default();
    let mut instantiated = Vec::default();
    let mut system_events = vec![];
    for event in super::System::events() {
        let mut is_private_event = false;
        if let super::RuntimeEvent::Contracts(ink_event) = &event.event {
            use pallet_contracts::Event as ContractEvent;
            match ink_event {
                ContractEvent::Instantiated {
                    deployer,
                    contract: address,
                } => instantiated.push((deployer.clone(), address.clone())),
                ContractEvent::ContractEmitted {
                    contract: address,
                    data,
                } => {
                    if event.topics.len() == 1
                        && event.topics[0].0 == pink_extension::PinkEvent::event_topic()
                    {
                        match pink_extension::PinkEvent::decode(&mut &data[..]) {
                            Ok(event) => {
                                pink_events.push((address.clone(), event.clone()));
                                is_private_event = event.is_private();
                            }
                            Err(_) => {
                                error!("Contract emitted an invalid pink event");
                            }
                        }
                    } else {
                        ink_events.push((address.clone(), event.topics.clone(), data.clone()));
                    }
                }
                _ => (),
            }
        }
        if !is_private_event {
            system_events.push(event);
        }
    }
    (
        system_events,
        ExecSideEffects::V1 {
            pink_events,
            ink_events,
            instantiated,
        },
    )
}

/// Contract extension for `pink contracts`
#[derive(Default)]
pub struct PinkExtension;

impl ChainExtension<PinkRuntime> for PinkExtension {
    fn call<E: Ext<T = PinkRuntime>>(
        &mut self,
        env: Environment<E, InitState>,
    ) -> ExtResult<RetVal> {
        let mut env = env.buf_in_buf_out();
        if env.ext_id() != 0 {
            error!(target: "pink", "Unknown extension id: {:}", env.ext_id());
            return Err(Error::UnknownChainExtensionId.into());
        }

        let address = env.ext().address().clone();
        let call_in_query = CallInQuery { address };
        let mode = OCallImpl.exec_context().mode;
        let (ret, output) = if mode.is_query() {
            dispatch_ext_call!(env.func_id(), call_in_query, env)
        } else {
            let call = CallInCommand {
                as_in_query: call_in_query,
            };
            dispatch_ext_call!(env.func_id(), call, env)
        }
        .ok_or(Error::UnknownChainExtensionFunction)
        .map_err(|err| {
            error!(target: "pink", "Called an unregistered `func_id`: {:}", env.func_id());
            err
        })?;
        env.write(&output, false, None)
            .or(Err(Error::ContractIoBufferOverflow))?;
        Ok(RetVal::Converging(ret))
    }

    fn enabled() -> bool {
        true
    }
}

struct CallInQuery {
    address: AccountId,
}

impl PinkRuntimeEnv for CallInQuery {
    type AccountId = AccountId;

    fn address(&self) -> &Self::AccountId {
        &self.address
    }
}

impl CallInQuery {
    fn ensure_system(&self) -> Result<(), DispatchError> {
        let contract: AccountId32 = self.address.convert_to();
        if Some(contract) != PalletPink::system_contract() {
            return Err(DispatchError::BadOrigin);
        }
        Ok(())
    }
    fn address_bytes(&self) -> Vec<u8> {
        let slice: &[u8] = self.address.as_ref();
        slice.to_vec()
    }
}

impl PinkExtBackend for CallInQuery {
    type Error = DispatchError;
    fn http_request(&self, request: HttpRequest) -> Result<HttpResponse, Self::Error> {
        OCallImpl
            .http_request(self.address.clone(), request)
            .map_err(|err| DispatchError::Other(err.display()))
    }

    fn batch_http_request(
        &self,
        requests: Vec<ext::HttpRequest>,
        timeout_ms: u64,
    ) -> Result<ext::BatchHttpResult, Self::Error> {
        Ok(OCallImpl.batch_http_request(self.address.clone(), requests, timeout_ms))
    }

    fn sign(
        &self,
        sigtype: SigType,
        key: Cow<[u8]>,
        message: Cow<[u8]>,
    ) -> Result<Vec<u8>, Self::Error> {
        DefaultPinkExtension::new(self).sign(sigtype, key, message)
    }

    fn verify(
        &self,
        sigtype: SigType,
        pubkey: Cow<[u8]>,
        message: Cow<[u8]>,
        signature: Cow<[u8]>,
    ) -> Result<bool, Self::Error> {
        DefaultPinkExtension::new(self).verify(sigtype, pubkey, message, signature)
    }

    fn derive_sr25519_key(&self, salt: Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        let privkey = PalletPink::key().ok_or(Error::KeySeedMissing)?;
        let privkey = sp_core::sr25519::Pair::restore_from_secret_key(&privkey);
        let contract_address: &[u8] = self.address.as_ref();
        let derived_pair = privkey
            .derive_sr25519_pair(&[contract_address, &salt, b"keygen"])
            .or(Err(Error::DeriveKeyFailed))?;
        let priviate_key = derived_pair.dump_secret_key();
        let priviate_key: &[u8] = priviate_key.as_ref();
        Ok(priviate_key.to_vec())
    }

    fn get_public_key(&self, sigtype: SigType, key: Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        DefaultPinkExtension::new(self).get_public_key(sigtype, key)
    }

    fn cache_set(
        &self,
        key: Cow<[u8]>,
        value: Cow<[u8]>,
    ) -> Result<Result<(), StorageQuotaExceeded>, Self::Error> {
        Ok(OCallImpl.cache_set(self.address_bytes(), key.into_owned(), value.into_owned()))
    }

    fn cache_set_expiration(&self, key: Cow<[u8]>, expire: u64) -> Result<(), Self::Error> {
        OCallImpl.cache_set_expiration(self.address_bytes(), key.into_owned(), expire);
        Ok(())
    }

    fn cache_get(&self, key: Cow<'_, [u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(OCallImpl.cache_get(self.address_bytes(), key.into_owned()))
    }

    fn cache_remove(&self, key: Cow<'_, [u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(OCallImpl.cache_remove(self.address_bytes(), key.into_owned()))
    }

    fn log(&self, level: u8, message: Cow<str>) -> Result<(), Self::Error> {
        OCallImpl.log_to_server(self.address.clone(), level, message.as_ref().into());
        DefaultPinkExtension::new(self).log(level, message)
    }

    fn getrandom(&self, length: u8) -> Result<Vec<u8>, Self::Error> {
        DefaultPinkExtension::new(self).getrandom(length)
    }

    fn is_in_transaction(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn ecdsa_sign_prehashed(
        &self,
        key: Cow<[u8]>,
        message_hash: Hash,
    ) -> Result<EcdsaSignature, Self::Error> {
        DefaultPinkExtension::new(self).ecdsa_sign_prehashed(key, message_hash)
    }

    fn ecdsa_verify_prehashed(
        &self,
        signature: EcdsaSignature,
        message_hash: Hash,
        pubkey: EcdsaPublicKey,
    ) -> Result<bool, Self::Error> {
        DefaultPinkExtension::new(self).ecdsa_verify_prehashed(signature, message_hash, pubkey)
    }

    fn system_contract_id(&self) -> Result<ext::AccountId, Self::Error> {
        PalletPink::system_contract()
            .map(|address| address.convert_to())
            .ok_or(Error::SystemContractMissing.into())
    }

    fn balance_of(
        &self,
        account: ext::AccountId,
    ) -> Result<(pink_extension::Balance, pink_extension::Balance), Self::Error> {
        self.ensure_system()?;
        let account: AccountId32 = account.convert_to();
        let total = crate::runtime::Balances::total_balance(&account);
        let free = crate::runtime::Balances::free_balance(&account);
        Ok((total, free))
    }

    fn untrusted_millis_since_unix_epoch(&self) -> Result<u64, Self::Error> {
        DefaultPinkExtension::new(self).untrusted_millis_since_unix_epoch()
    }

    fn worker_pubkey(&self) -> Result<EcdhPublicKey, Self::Error> {
        Ok(OCallImpl.worker_pubkey())
    }

    fn code_exists(&self, code_hash: Hash, sidevm: bool) -> Result<bool, Self::Error> {
        if sidevm {
            Ok(PalletPink::sidevm_code_exists(&code_hash.into()))
        } else {
            Ok(crate::storage::external_backend::code_exists(
                &code_hash.into(),
            ))
        }
    }

    fn import_latest_system_code(
        &self,
        payer: ext::AccountId,
    ) -> Result<Option<Hash>, Self::Error> {
        self.ensure_system()?;
        let system_code = OCallImpl.latest_system_code();
        if system_code.is_empty() {
            return Ok(None);
        }
        let code_hash = sp_core::blake2_256(&system_code);
        if !self.code_exists(code_hash, false)? {
            crate::runtime::Contracts::bare_upload_code(
                payer.convert_to(),
                system_code,
                None,
                pallet_contracts::Determinism::Enforced,
            )?;
        };
        Ok(Some(code_hash))
    }

    fn runtime_version(&self) -> Result<(u32, u32), Self::Error> {
        Ok(crate::version())
    }

    fn current_event_chain_head(&self) -> Result<(u64, Hash), Self::Error> {
        Ok((
            PalletPink::next_event_block_number(),
            PalletPink::last_event_block_hash().into(),
        ))
    }
}

struct CallInCommand {
    as_in_query: CallInQuery,
}

/// This implementation is used when calling the extension in a command.
/// # NOTE FOR IMPLEMENTORS
/// Make sure the return values are deterministic.
impl PinkExtBackend for CallInCommand {
    type Error = DispatchError;

    fn http_request(&self, _request: HttpRequest) -> Result<HttpResponse, Self::Error> {
        Ok(HttpResponse {
            status_code: 523,
            reason_phrase: "API Unavailable".into(),
            headers: vec![],
            body: vec![],
        })
    }
    fn batch_http_request(
        &self,
        _requests: Vec<ext::HttpRequest>,
        _timeout_ms: u64,
    ) -> Result<ext::BatchHttpResult, Self::Error> {
        Ok(Err(ext::HttpRequestError::NotAllowed))
    }
    fn sign(
        &self,
        sigtype: SigType,
        key: Cow<[u8]>,
        message: Cow<[u8]>,
    ) -> Result<Vec<u8>, Self::Error> {
        if matches!(sigtype, SigType::Sr25519) {
            return Ok(vec![]);
        }
        self.as_in_query.sign(sigtype, key, message)
    }

    fn verify(
        &self,
        sigtype: SigType,
        pubkey: Cow<[u8]>,
        message: Cow<[u8]>,
        signature: Cow<[u8]>,
    ) -> Result<bool, Self::Error> {
        self.as_in_query.verify(sigtype, pubkey, message, signature)
    }

    fn derive_sr25519_key(&self, salt: Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        self.as_in_query.derive_sr25519_key(salt)
    }

    fn get_public_key(&self, sigtype: SigType, key: Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        self.as_in_query.get_public_key(sigtype, key)
    }

    fn cache_set(
        &self,
        key: Cow<[u8]>,
        value: Cow<[u8]>,
    ) -> Result<Result<(), StorageQuotaExceeded>, Self::Error> {
        deposit_pink_event(
            self.as_in_query.address.clone(),
            PinkEvent::CacheOp(CacheOp::Set {
                key: key.into_owned(),
                value: value.into_owned(),
            }),
        );
        Ok(Ok(()))
    }

    fn cache_set_expiration(&self, key: Cow<[u8]>, expiration: u64) -> Result<(), Self::Error> {
        deposit_pink_event(
            self.as_in_query.address.clone(),
            PinkEvent::CacheOp(CacheOp::SetExpiration {
                key: key.into_owned(),
                expiration,
            }),
        );
        Ok(())
    }

    fn cache_get(&self, _key: Cow<[u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }

    fn cache_remove(&self, key: Cow<[u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        deposit_pink_event(
            self.as_in_query.address.clone(),
            PinkEvent::CacheOp(CacheOp::Remove {
                key: key.into_owned(),
            }),
        );
        Ok(None)
    }

    fn log(&self, level: u8, message: Cow<str>) -> Result<(), Self::Error> {
        self.as_in_query.log(level, message)
    }

    fn getrandom(&self, _length: u8) -> Result<Vec<u8>, Self::Error> {
        Ok(vec![])
    }

    fn is_in_transaction(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn ecdsa_sign_prehashed(
        &self,
        key: Cow<[u8]>,
        message_hash: Hash,
    ) -> Result<EcdsaSignature, Self::Error> {
        self.as_in_query.ecdsa_sign_prehashed(key, message_hash)
    }

    fn ecdsa_verify_prehashed(
        &self,
        signature: EcdsaSignature,
        message_hash: Hash,
        pubkey: EcdsaPublicKey,
    ) -> Result<bool, Self::Error> {
        self.as_in_query
            .ecdsa_verify_prehashed(signature, message_hash, pubkey)
    }

    fn system_contract_id(&self) -> Result<ext::AccountId, Self::Error> {
        self.as_in_query.system_contract_id()
    }

    fn balance_of(
        &self,
        account: ext::AccountId,
    ) -> Result<(pink_extension::Balance, pink_extension::Balance), Self::Error> {
        self.as_in_query.balance_of(account)
    }

    fn untrusted_millis_since_unix_epoch(&self) -> Result<u64, Self::Error> {
        Ok(0)
    }

    fn worker_pubkey(&self) -> Result<EcdhPublicKey, Self::Error> {
        Ok(Default::default())
    }

    fn code_exists(&self, code_hash: Hash, sidevm: bool) -> Result<bool, Self::Error> {
        self.as_in_query.code_exists(code_hash, sidevm)
    }

    fn import_latest_system_code(
        &self,
        payer: ext::AccountId,
    ) -> Result<Option<Hash>, Self::Error> {
        self.as_in_query.import_latest_system_code(payer)
    }

    fn runtime_version(&self) -> Result<(u32, u32), Self::Error> {
        self.as_in_query.runtime_version()
    }

    fn current_event_chain_head(&self) -> Result<(u64, Hash), Self::Error> {
        self.as_in_query.current_event_chain_head()
    }
}
