use crate::std::collections::BTreeMap;
use crate::std::string::{String, ToString};

use anyhow::Result;
use core::fmt;
use log::info;
use parity_scale_codec::{Decode, Encode};
use phala_mq::MessageOrigin;

use super::{TransactionResult, TransactionError};
use crate::contracts;
use crate::contracts::{AccountId, NativeContext};
extern crate runtime as chain;

use phala_types::messaging::{GeolocationCommand, Coordinate};

type Command = GeolocationCommand<chain::AccountId>;


pub struct Geolocation {
    geolocations: BTreeMap<AccountId, Coordinate>,
}

#[derive(Encode, Decode, Debug)]
pub enum Error {
    NotAuthorized,
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {
    GetGeolocation { account: AccountId }
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Response {
    GetGeolocation { geolocation: Coordinate },
    Error(String),
}

impl Geolocation {
    pub fn new() -> Self {
        Geolocation {
            geolocations: BTreeMap::new()
        }
    }
}

impl contracts::NativeContract for Geolocation {
    type Cmd = Command;
    type QReq = Request;
    type QResp = Response;

    fn id(&self) -> contracts::ContractId32 {
        contracts::GEOLOCATION
    }

    fn handle_command(
        &mut self,
        context: &NativeContext,
        origin: MessageOrigin,
        cmd: Command,
    ) -> TransactionResult {
        match cmd {
            Command::UpdateGeolocation { sender, geolocation } => {
                if !origin.is_pallet() {
                    error!("Received event from unexpected origin: {:?}", origin);
                    return Err(TransactionError::BadOrigin);
                }
                info!(
                    "UpdateGeolocation: [{}] -> <{}, {}>",
                    hex::encode(&sender),
                    geolocation.latitude,
                    geolocation.longitude
                );
                if let Some(geo_data) = self.geolocations.get_mut(&sender) {
                    *geo_data = geolocation;
                } else {
                    self.geolocations.insert(sender, geolocation);
                };

                Ok(())
            }
        }
    }

    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response> {
            match req {
                Request::GetGeolocation { account } => {
                    if origin == None || origin.unwrap() != &account {
                        return Err(anyhow::Error::msg(Error::NotAuthorized));
                    }
                    let mut geolocation = Coordinate { latitude: 0, longitude: 0 };
                    if let Some(data) = self.geolocations.get(&account) {
                        geolocation = data.clone();
                    } else {
                        return Err(anyhow::Error::msg(Error::Other("no record".to_string())));
                    }
                    Ok(Response::GetGeolocation { geolocation })
                }
            }
        };
        match inner() {
            Err(error) => Response::Error(error.to_string()),
            Ok(resp) => resp,
        }
    }
}
