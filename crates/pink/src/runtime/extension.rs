use std::borrow::Cow;
use std::{convert::TryFrom, time::Duration};

use frame_support::log::error;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use phala_crypto::sr25519::{Persistence, KDF};
use pink_extension::CacheOp;
use pink_extension::{
    chain_extension::{
        HttpRequest, HttpResponse, PinkExtBackend, PublicKeyForArgs, SigType, SignArgs,
        StorageQuotaExceeded, VerifyArgs,
    },
    dispatch_ext_call, PinkEvent,
};
use scale::{Decode, Encode};
use sp_core::{ByteArray, Pair};
use sp_runtime::DispatchError;

use crate::{
    runtime::{get_call_elapsed, get_call_mode, CallMode},
    types::AccountId,
};

use crate::local_cache::GLOBAL_CACHE;

#[derive(Default, Debug)]
pub struct ExecSideEffects {
    pub pink_events: Vec<(AccountId, PinkEvent)>,
    pub instantiated: Vec<(AccountId, AccountId)>,
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
                    if event.topics.len() != 1 {
                        continue;
                    }
                    if event.topics[0].0 == pink_extension::PinkEvent::event_topic() {
                        match pink_extension::PinkEvent::decode(&mut &data[..]) {
                            Ok(event) => {
                                result.pink_events.push((address, event));
                            }
                            Err(_) => {
                                error!("Contract emitted an invalid pink event");
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }
    result
}

/// Contract extension for `pink contracts`
pub struct PinkExtension;

impl ChainExtension<super::PinkRuntime> for PinkExtension {
    fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        <E::T as SysConfig>::AccountId:
            UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]> + Clone,
    {
        let mut env = env.buf_in_buf_out();
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
            dispatch_ext_call!(func_id, call, env)
        } else {
            dispatch_ext_call!(func_id, call_in_query, env)
        };
        let output = match result {
            Some(output) => output,
            None => {
                error!(target: "pink", "Called an unregistered `func_id`: {:}", func_id);
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
}

struct CallInQuery {
    address: AccountId,
}

impl PinkExtBackend for CallInQuery {
    type Error = DispatchError;
    fn http_request(&self, request: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let uri = http_req::uri::Uri::try_from(request.url.as_str())
            .or(Err(DispatchError::Other("Invalid URL")))?;

        let mut req = http_req::request::Request::new(&uri);
        for (key, value) in &request.headers {
            req.header(key, value);
        }

        match request.method.as_str() {
            "GET" => {
                req.method(http_req::request::Method::GET);
            }
            "POST" => {
                req.method(http_req::request::Method::POST)
                    .body(request.body.as_slice());
                req.header("Content-Length", &request.body.len());
            }
            _ => {
                return Err(DispatchError::Other("Unsupported method"));
            }
        };

        // Hardcoded limitations for now
        const MAX_QUERY_TIME: u64 = 10; // seconds
        const MAX_BODY_SIZE: usize = 1024 * 256; // 256KB

        let elapsed = get_call_elapsed().ok_or(DispatchError::Other("Invalid exec env"))?;
        let timeout = Duration::from_secs(MAX_QUERY_TIME) - elapsed;
        req.timeout(Some(timeout));

        let mut body = Vec::new();
        let mut writer = LimitedWriter::new(&mut body, MAX_BODY_SIZE);

        let response = req
            .send(&mut writer)
            .or(Err(DispatchError::Other("Failed to send request")))?;

        let headers: Vec<_> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_owned()))
            .collect();
        let response = HttpResponse {
            status_code: response.status_code().into(),
            reason_phrase: response.reason().into(),
            body,
            headers,
        };
        Ok(response)
    }

    fn sign(&self, args: SignArgs) -> Result<Vec<u8>, Self::Error> {
        macro_rules! sign_with {
            ($sigtype:ident) => {{
                let pair = sp_core::$sigtype::Pair::from_seed_slice(&args.key)
                    .or(Err(DispatchError::Other("Invalid key")))?;
                let signature = pair.sign(&args.message);
                let signature: &[u8] = signature.as_ref();
                signature.to_vec()
            }};
        }

        Ok(match args.sigtype {
            SigType::Sr25519 => sign_with!(sr25519),
            SigType::Ed25519 => sign_with!(ed25519),
            SigType::Ecdsa => sign_with!(ecdsa),
        })
    }

    fn verify(&self, args: VerifyArgs) -> Result<bool, Self::Error> {
        macro_rules! verify_with {
            ($sigtype:ident) => {{
                sp_core::$sigtype::Pair::verify_weak(&args.signature, &args.message, &args.pubkey)
            }};
        }
        Ok(match args.sigtype {
            SigType::Sr25519 => verify_with!(sr25519),
            SigType::Ed25519 => verify_with!(ed25519),
            SigType::Ecdsa => verify_with!(ecdsa),
        })
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

    fn get_public_key(&self, args: PublicKeyForArgs) -> Result<Vec<u8>, Self::Error> {
        macro_rules! public_key_with {
            ($sigtype:ident) => {{
                sp_core::$sigtype::Pair::from_seed_slice(&args.key)
                    .or(Err(DispatchError::Other("Invalid key")))?
                    .public()
                    .to_raw_vec()
            }};
        }
        let pubkey = match args.sigtype {
            SigType::Ed25519 => public_key_with!(ed25519),
            SigType::Sr25519 => public_key_with!(sr25519),
            SigType::Ecdsa => public_key_with!(ecdsa),
        };
        Ok(pubkey)
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
        let value = GLOBAL_CACHE
            .read()
            .unwrap()
            .get(contract, key.as_ref());
        Ok(value)
    }

    fn cache_remove(&self, key: Cow<'_, [u8]>) -> Result<Option<Vec<u8>>, Self::Error> {
        let contract: &[u8] = self.address.as_ref();
        let value = GLOBAL_CACHE
            .write()
            .unwrap()
            .remove(contract, key.as_ref());
        Ok(value)
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

    fn sign(&self, args: SignArgs) -> Result<Vec<u8>, Self::Error> {
        self.as_in_query.sign(args)
    }

    fn verify(&self, args: VerifyArgs) -> Result<bool, Self::Error> {
        self.as_in_query.verify(args)
    }

    fn derive_sr25519_key(&self, salt: Cow<[u8]>) -> Result<Vec<u8>, Self::Error> {
        self.as_in_query.derive_sr25519_key(salt)
    }

    fn get_public_key(&self, args: PublicKeyForArgs) -> Result<Vec<u8>, Self::Error> {
        self.as_in_query.get_public_key(args)
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
}

struct LimitedWriter<W> {
    writer: W,
    written: usize,
    limit: usize,
}

impl<W> LimitedWriter<W> {
    fn new(writer: W, limit: usize) -> Self {
        Self {
            writer,
            written: 0,
            limit,
        }
    }
}

impl<W: std::io::Write> std::io::Write for LimitedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.written + buf.len() > self.limit {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Buffer limit exceeded",
            ));
        }
        let wlen = self.writer.write(buf)?;
        self.written += wlen;
        Ok(wlen)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
