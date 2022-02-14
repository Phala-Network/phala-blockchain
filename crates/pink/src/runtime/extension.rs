use std::{convert::TryFrom, time::Duration};

use frame_support::log::error;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use phala_crypto::sr25519::{Persistence, KDF};
use pink_extension::PinkEvent;
use scale::{Decode, Encode};
use sp_core::Pair;
use sp_runtime::DispatchError;

use crate::{
    runtime::{get_call_elapsed, get_call_mode, CallMode},
    types::AccountId,
};

#[derive(Default, Debug)]
pub struct ExecSideEffects {
    pub pink_events: Vec<(AccountId, PinkEvent)>,
    pub instantiated: Vec<(AccountId, AccountId)>,
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
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        let call = Call { env };
        // func_id refer to https://github.com/patractlabs/PIPs/blob/main/PIPs/pip-100.md
        match func_id {
            0xff000001 => call.http_request(),
            0xff000002 => call.sign(),
            0xff000003 => call.verify(),
            0xff000004 => call.derive_sr25519_pair(),
            _ => {
                error!(target: "pink", "Called an unregistered `func_id`: {:}", func_id);
                Err(DispatchError::Other("Unimplemented func_id"))
            }
        }
    }
}

struct Call<'a, 'b, E: Ext> {
    env: Environment<'a, 'b, E, InitState>,
}

impl<'a, 'b, E: Ext> Call<'a, 'b, E>
where
    <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
{
    fn http_request(self) -> Result<RetVal, DispatchError> {
        use pink_extension::chain_extension::{HttpRequest, HttpResponse};
        if !matches!(get_call_mode(), Some(CallMode::Query)) {
            return Err(DispatchError::Other(
                "http_request can only be called in query mode",
            ));
        }

        let mut env = self.env.buf_in_buf_out();
        let request: HttpRequest = env.read_as_unbounded(env.in_len())?;
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
        env.write(&response.encode(), false, None)
            .map_err(|_| DispatchError::Other("ChainExtension failed to return http_request"))?;
        Ok(RetVal::Converging(0))
    }

    fn sign(self) -> Result<RetVal, DispatchError> {
        use pink_extension::chain_extension::{SigType, SignArgs};
        let mut env = self.env.buf_in_buf_out();
        let args: SignArgs = env.read_as_unbounded(env.in_len())?;

        macro_rules! sign_with {
            ($sigtype:ident) => {{
                let pair = sp_core::$sigtype::Pair::from_seed_slice(&args.key)
                    .or(Err(DispatchError::Other("Invalid key")))?;
                let signature = pair.sign(&args.message);
                let signature: &[u8] = signature.as_ref();
                env.write(&signature.encode(), false, None)
                    .map_err(|_| DispatchError::Other("ChainExtension failed to return sign"))?;
            }};
        }

        match args.sigtype {
            SigType::Sr25519 => sign_with!(sr25519),
            SigType::Ed25519 => sign_with!(ed25519),
            SigType::Ecdsa => sign_with!(ecdsa),
        }
        Ok(RetVal::Converging(0))
    }

    fn verify(self) -> Result<RetVal, DispatchError> {
        use pink_extension::chain_extension::{SigType, VerifyArgs};
        let mut env = self.env.buf_in_buf_out();
        let args: VerifyArgs = env.read_as_unbounded(env.in_len())?;

        let result = match args.sigtype {
            SigType::Sr25519 => {
                sp_core::sr25519::Pair::verify_weak(&args.signature, &args.message, &args.pubkey)
            }
            SigType::Ed25519 => {
                sp_core::ed25519::Pair::verify_weak(&args.signature, &args.message, &args.pubkey)
            }
            SigType::Ecdsa => {
                sp_core::ecdsa::Pair::verify_weak(&args.signature, &args.message, &args.pubkey)
            }
        };
        env.write(&result.encode(), false, None)
            .map_err(|_| DispatchError::Other("ChainExtension failed to return verify"))?;
        Ok(RetVal::Converging(0))
    }

    fn derive_sr25519_pair(self) -> Result<RetVal, DispatchError> {
        let mut env = self.env.buf_in_buf_out();
        let salt: Vec<u8> = env.read_as_unbounded(env.in_len())?;
        let seed =
            crate::runtime::Pink::key_seed().ok_or(DispatchError::Other("Key seed missing"))?;
        let seed_key = sp_core::sr25519::Pair::restore_from_secret_key(&seed);
        let contract_address = env.ext().address();
        let contract_address: &[u8] = contract_address.as_ref();
        let derived_pair = seed_key
            .derive_sr25519_pair(&[contract_address, &salt, b"keygen"])
            .or(Err(DispatchError::Other("Failed to derive sr25519 pair")))?;
        let priviate_key = derived_pair.dump_secret_key();
        let priviate_key: &[u8] = priviate_key.as_ref();
        let public_key = derived_pair.public();
        let public_key: &[u8] = public_key.as_ref();

        env.write(&(priviate_key, public_key).encode(), false, None)
            .map_err(|_| {
                DispatchError::Other("ChainExtension failed to return derive_sr25519_pair")
            })?;
        Ok(RetVal::Converging(0))
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
