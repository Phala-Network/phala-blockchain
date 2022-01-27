use std::convert::TryFrom;

use frame_support::log::error;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use pink_extension::PinkEvent;
use scale::{Decode, Encode};
use sp_runtime::DispatchError;

use crate::types::AccountId;

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
        let mut env = env.buf_in_buf_out();

        match func_id {
            // http_request
            0xff000001 => {
                use pink_extension::chain_extension::{HttpRequest, HttpResponse};
                // TODO.kevin.must: Forbid to call from command.
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
                        return Err(DispatchError::Other("Unsupported method".into()));
                    }
                };

                let mut body = Vec::new();
                const MAX_BODY_SIZE: usize = 1024 * 256;
                let mut writer = LimitedWriter::new(&mut body, MAX_BODY_SIZE);

                req.timeout(Some(std::time::Duration::from_secs(10)));

                let response = req
                    .send(&mut writer)
                    .or(Err(DispatchError::Other("Failed to send request".into())))?;

                let headers: Vec<_> = response
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_owned()))
                    .collect();
                let response = HttpResponse {
                    status_code: response.status_code().into(),
                    body,
                    headers,
                };
                env.write(&response.encode(), false, None).map_err(|_| {
                    DispatchError::Other("ChainExtension failed to return http_request")
                })?;
                Ok(RetVal::Converging(0))
            }
            _ => {
                error!(target: "pink", "Called an unregistered `func_id`: {:}", func_id);
                return Err(DispatchError::Other("Unimplemented func_id"));
            }
        }
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
