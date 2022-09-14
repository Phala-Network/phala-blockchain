use std::borrow::Cow;
use std::time::Duration;

use frame_support::log::error;
use pallet_contracts::chain_extension::Result as ExtResult;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use phala_crypto::sr25519::{Persistence, KDF};
use pink_extension::{
    chain_extension::{HttpRequest, HttpResponse, PinkExtBackend, SigType, StorageQuotaExceeded},
    dispatch_ext_call, CacheOp, EcdsaPublicKey, EcdsaSignature, Hash, PinkEvent,
};
use pink_extension_runtime::{DefaultPinkExtension, PinkRuntimeEnv};
use scale::{Decode, Encode};
use sp_core::H256;
use sp_runtime::DispatchError;

use crate::{
    runtime::{get_call_elapsed, get_call_mode, CallMode},
    types::AccountId,
};

use crate::local_cache::GLOBAL_CACHE;

#[derive(Default, Debug)]
pub struct ExecSideEffects {
    pub pink_events: Vec<(AccountId, PinkEvent)>,
    pub ink_events: Vec<(AccountId, Vec<H256>, Vec<u8>)>,
    pub instantiated: Vec<(AccountId, AccountId)>,
}

impl ExecSideEffects {
    pub fn into_query_only_effects(mut self) -> Self {
        self.pink_events
            .retain(|(_, event)| event.allowed_in_query());
        self.ink_events.clear();
        self.instantiated.clear();
        self
    }
}

fn deposit_pink_event(contract: AccountId, event: PinkEvent) {
    let topics = [pink_extension::PinkEvent::event_topic().into()];
    let event = super::Event::Contracts(pallet_contracts::Event::ContractEmitted {
        contract,
        data: event.encode(),
    });
    super::System::deposit_event_indexed(&topics[..], event);
}

pub fn get_side_effects() -> ExecSideEffects {
    let mut result = ExecSideEffects::default();
    for event in super::System::events() {
        if let super::Event::Contracts(ink_event) = event.event {
            use pallet_contracts::Event as ContractEvent;
            match ink_event {
                ContractEvent::Instantiated {
                    deployer,
                    contract: address,
                } => result.instantiated.push((deployer, address)),
                ContractEvent::ContractEmitted {
                    contract: address,
                    data,
                } => {
                    if event.topics.len() == 1
                        && event.topics[0].0 == pink_extension::PinkEvent::event_topic()
                    {
                        match pink_extension::PinkEvent::decode(&mut &data[..]) {
                            Ok(event) => {
                                result.pink_events.push((address, event));
                            }
                            Err(_) => {
                                error!("Contract emitted an invalid pink event");
                            }
                        }
                    } else {
                        result.ink_events.push((address, event.topics, data));
                    }
                }
                _ => (),
            }
        }
    }
    result
}

/// Contract extension for `pink contracts`
#[derive(Default)]
pub struct PinkExtension;

impl ChainExtension<super::PinkRuntime> for PinkExtension {
    fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> ExtResult<RetVal>
    where
        <E::T as SysConfig>::AccountId:
            UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]> + Clone,
    {
        let mut env = env.buf_in_buf_out();
        if env.ext_id() != 0 {
            error!(target: "pink", "Unknown extension id: {:}", env.ext_id());
            return Err(DispatchError::Other(
                "PinkExtension::call: unknown extension id",
            ));
        }

        let address = env
            .ext()
            .address()
            .as_ref()
            .try_into()
            .expect("Address should be valid");
        let call_in_query = CallInQuery {
            address: AccountId::new(address),
        };
        let result = if matches!(get_call_mode(), Some(CallMode::Command)) {
            let call = CallInCommand {
                as_in_query: call_in_query,
            };
            dispatch_ext_call!(env.func_id(), call, env)
        } else {
            dispatch_ext_call!(env.func_id(), call_in_query, env)
        };
        let output = match result {
            Some(output) => output,
            None => {
                error!(target: "pink", "Called an unregistered `func_id`: {:}", env.func_id());
                return Err(DispatchError::Other(
                    "PinkExtension::call: unknown function",
                ));
            }
        };
        env.write(&output, false, None)
            .or(Err(DispatchError::Other(
                "PinkExtension::call: failed to write output",
            )))?;
        Ok(RetVal::Converging(0))
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

    fn call_elapsed(&self) -> Option<Duration> {
        get_call_elapsed()
    }
}

impl PinkExtBackend for CallInQuery {
    type Error = DispatchError;
    fn http_request(&self, request: HttpRequest) -> Result<HttpResponse, Self::Error> {
        DefaultPinkExtension::new(self).http_request(request)
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
        let seed =
            crate::runtime::Pink::key_seed().ok_or(DispatchError::Other("Key seed missing"))?;
        let seed_key = sp_core::sr25519::Pair::restore_from_secret_key(&seed);
        let contract_address: &[u8] = self.address.as_ref();
        let derived_pair = seed_key
            .derive_sr25519_pair(&[contract_address, &salt, b"keygen"])
            .or(Err(DispatchError::Other("Failed to derive sr25519 pair")))?;
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
        let contract: &[u8] = self.address.as_ref();
        let result = GLOBAL_CACHE
            .write()
            .unwrap()
            .set(contract.into(), key, value);
        Ok(result)
    }

    fn cache_set_expire(&self, key: Cow<[u8]>, expire: u64) -> Result<(), Self::Error> {
        let contract: &[u8] = self.address.as_ref();
        GLOBAL_CACHE
            .write()
            .unwrap()
            .set_expire(contract.into(), key, expire);
        Ok(())
    }

    fn cache_get(&self, key: Cow<'_, [u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        let contract: &[u8] = self.address.as_ref();
        let value = GLOBAL_CACHE.read().unwrap().get(contract, key.as_ref());
        Ok(value)
    }

    fn cache_remove(&self, key: Cow<'_, [u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        let contract: &[u8] = self.address.as_ref();
        let value = GLOBAL_CACHE.write().unwrap().remove(contract, key.as_ref());
        Ok(value)
    }

    fn log(&self, level: u8, message: Cow<str>) -> Result<(), Self::Error> {
        super::emit_log(&self.address, level, message.as_ref().into());
        DefaultPinkExtension::new(self).log(level, message)
    }

    fn getrandom(&self, length: u8) -> Result<Vec<u8>, Self::Error> {
        DefaultPinkExtension::new(self).getrandom(length)
    }

    fn is_running_in_command(&self) -> Result<bool, Self::Error> {
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
        return Err(DispatchError::Other(
            "http_request can only be called in query mode",
        ));
    }
    fn sign(
        &self,
        sigtype: SigType,
        key: Cow<[u8]>,
        message: Cow<[u8]>,
    ) -> Result<Vec<u8>, Self::Error> {
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

    fn cache_set_expire(&self, key: Cow<[u8]>, expiration: u64) -> Result<(), Self::Error> {
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
        Err("getrandom is not allowed in command".into())
    }

    fn is_running_in_command(&self) -> Result<bool, Self::Error> {
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
}
