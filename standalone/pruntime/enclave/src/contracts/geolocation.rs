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

use phala_types::messaging::{GeolocationCommand, CoordinateInfo};

type Command = GeolocationCommand<chain::AccountId>;


pub struct Geolocation {
    geolocation_info: BTreeMap<AccountId, CoordinateInfo>,
    city_distribution: BTreeMap<String, Vec<AccountId>>,
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
    GetGeolocationInfo { account: AccountId },
    GetAvailableCityName {},
    GetCityDistribution { city_name: String },
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Response {
    GetGeolocationInfo { geolocation_info: CoordinateInfo },
    GetAvailableCityName { city_names: Vec<String> },
    GetCityDistribution { workers: Vec<AccountId> },
    Error(String),
}

impl Geolocation {
    pub fn new() -> Self {
        Geolocation {
            geolocation_info: BTreeMap::new(),
            city_distribution: BTreeMap::new(),
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
            Command::UpdateGeolocation { sender, encrypted_geolocation_info } => {
                if !origin.is_pallet() {
                    error!("Received event from unexpected origin: {:?}", origin);
                    return Err(TransactionError::BadOrigin);
                }
                // Decrypt
                let geolocation_info = match context.ecdh_decrypt::<CoordinateInfo>(encrypted_geolocation_info) {
                    Ok(e) => e,
                    Err(_) => return Err(TransactionError::BadInput),
                };

                info!(
                    "UpdateGeolocation: [{}] -> <{}, {}>, city: {}",
                    hex::encode(&sender),
                    geolocation_info.latitude,
                    geolocation_info.longitude,
                    geolocation_info.city_name,
                );

                // Insert data to geolocation info btreemap
                if let Some(geo_data) = self.geolocation_info.get_mut(&sender) {
                    *geo_data = geolocation_info.clone();
                    if geo_data.city_name != geolocation_info.city_name {
                        // Remove account id to previous city
                        if let Some(workers) = self.city_distribution.get_mut(&geo_data.city_name) {
                            if let Some(pos) = workers.iter().position(|x| *x == sender) {
                                info!("Remove {} from city {}",
                                    hex::encode(&sender), geolocation_info.city_name);
                                workers.remove(pos);
                            } else {
                                error!("Cannot locate AccountId. Something is wrong in the geolocation contract's UpdateGeolocation() function");
                                return Err(TransactionError::UnknownError);
                            }
                        } else {
                            error!("Cannot locate previous city name. Something is wrong in the geolocation contract's UpdateGeolocation() function");
                            return Err(TransactionError::UnknownError);
                        }
                        // Insert account id to new city
                        info!("Push {} to city {}",
                            hex::encode(&sender), geolocation_info.city_name);
                        let workers = self.city_distribution.entry(geolocation_info.city_name).or_default();
                        workers.push(sender);
                    }
                } else {
                    // newly arrived worker
                    info!("Push {} to city {}",
                            hex::encode(&sender), geolocation_info.city_name);
                    self.geolocation_info.insert(sender.clone(), geolocation_info.clone());
                    let workers = self.city_distribution.entry(geolocation_info.city_name).or_default();
                    workers.push(sender);
                };

                Ok(())
            }
        }
    }

    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response> {
            match req {
                Request::GetGeolocationInfo { account } => {
                    if origin == None || origin.unwrap() != &account {
                        return Err(anyhow::Error::msg(Error::NotAuthorized));
                    }
                    if let Some(data) = self.geolocation_info.get(&account) {
                        let geolocation_info = data.clone();
                        Ok(Response::GetGeolocationInfo { geolocation_info })
                    } else {
                        error!("no record");
                        return Err(anyhow::Error::msg(Error::Other("no record".to_string())));
                    }
                },
                Request::GetAvailableCityName {} => {
                    let city_names: Vec<String> = self.city_distribution.keys().cloned().collect();
                    Ok(Response::GetAvailableCityName { city_names })
                },
                Request::GetCityDistribution { city_name } => {
                    if let Some(workers) = self.city_distribution.get(&city_name) {
                        Ok(Response::GetCityDistribution { workers: workers.clone() })
                    } else {
                        error!("Unavailable city name provided");
                        return Err(anyhow::Error::msg(Error::Other("Unavailable city name provided".to_string())));
                    }
                }
            }
        };
        match inner() {
            Err(error) => Response::Error(error.to_string()),
            Ok(resp) => resp,
        }
    }
}
