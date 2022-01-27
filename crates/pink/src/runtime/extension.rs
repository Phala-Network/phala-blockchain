use frame_support::log::{error, info};
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

                let mut body = Vec::new();

                const MAX_BODY_SIZE: usize = 1024 * 256;
                let mut writer = LimitedWriter::new(&mut body, MAX_BODY_SIZE);

                let response = match request.method.as_str() {
                    "GET" => http_req::request::get(&request.url, &mut writer).map_err(|e| {
                        error!("http_get error: {:?}", e);
                        DispatchError::Other("http_get error")
                    })?,
                    "POST" => http_req::request::post(&request.url, &request.body, &mut writer)
                        .map_err(|e| {
                            error!("http_post error: {:?}", e);
                            DispatchError::Other("http_post error")
                        })?,
                    _ => {
                        return Err(DispatchError::Other("Unsupported method".into()));
                    }
                };

                info!(
                    "HTTP response status: {} {}",
                    response.status_code(),
                    response.reason()
                );

                let response = HttpResponse {
                    status_code: response.status_code().into(),
                    body,
                    headers: Default::default(),
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
