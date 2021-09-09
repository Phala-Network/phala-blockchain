use std::collections::BTreeMap;
use std::string::{String, ToString};

use anyhow::Result;
use core::fmt;
use log::info;
use parity_scale_codec::{Decode, Encode};
use phala_mq::MessageOrigin;
use std::convert::TryFrom;

use super::{TransactionError, TransactionResult};
use crate::contracts;
use crate::contracts::{AccountId, NativeContext};
extern crate runtime as chain;

use phala_types::messaging::{CoordinateInfo, GeolocationCommand};

type Command = GeolocationCommand;

pub struct Geolocation {
    geolocation_info: BTreeMap<AccountId, CoordinateInfo>,
    city_distribution: BTreeMap<String, Vec<AccountId>>,
}

#[derive(Encode, Decode, Debug)]
pub enum Error {
    // InvalidRequest,
    NoRecord,
    NotAuthorized,
    UnavailableCityName,
    Unimplemented,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {
    GetGeolocationInfo { account: AccountId },
    GetAvailableCityName {},
    GetCityDistribution { city_name: String },
    GetCityDistributionCount { city_name: String },
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Response {
    GetGeolocationInfo { geolocation_info: CoordinateInfo },
    GetAvailableCityName { city_names: Vec<String> },
    GetCityDistribution { workers: Vec<AccountId> },
    GetCityDistributionCount { count: u32 },
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
    type QResp = Result<Response, Error>;

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
            Command::UpdateGeolocation { geolocation_info } => {
                let sender = match &origin {
                    MessageOrigin::Worker(pubkey) => AccountId::from(*pubkey),
                    _ => return Err(TransactionError::BadOrigin),
                };

                // Insert data to geolocation info btreemap
                if let Some(geo_data) = self.geolocation_info.get_mut(&sender) {
                    *geo_data = geolocation_info.clone();
                    if geo_data.city_name != geolocation_info.city_name {
                        // Remove account id to previous city
                        if let Some(workers) = self.city_distribution.get_mut(&geo_data.city_name) {
                            if let Some(pos) = workers.iter().position(|x| *x == sender) {
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
                        let workers = self
                            .city_distribution
                            .entry(geolocation_info.city_name)
                            .or_default();
                        workers.push(sender);
                    }
                } else {
                    // newly arrived worker
                    self.geolocation_info
                        .insert(sender.clone(), geolocation_info.clone());
                    let workers = self
                        .city_distribution
                        .entry(geolocation_info.city_name)
                        .or_default();
                    workers.push(sender);
                };

                Ok(())
            }
        }
    }

    fn handle_query(
        &mut self,
        origin: Option<&chain::AccountId>,
        req: Request,
    ) -> Result<Response, Error> {
        match req {
            Request::GetGeolocationInfo { account } => {
                if origin != Some(&account) {
                    return Err(Error::NotAuthorized);
                }
                if let Some(data) = self.geolocation_info.get(&account) {
                    let geolocation_info = data.clone();
                    Ok(Response::GetGeolocationInfo { geolocation_info })
                } else {
                    Err(Error::NoRecord)
                }
            }
            Request::GetAvailableCityName {} => {
                let city_names: Vec<String> = self.city_distribution.keys().cloned().collect();
                Ok(Response::GetAvailableCityName { city_names })
            }
            Request::GetCityDistribution { city_name } => {
                // TODO(soptq): Authorization
                Err(Error::Unimplemented)
                // if let Some(workers) = self.city_distribution.get(&city_name) {
                //     Ok(Response::GetCityDistribution { workers: workers.clone() })
                // } else {
                //     error!("Unavailable city name provided");
                //     Err(anyhow::Error::msg(Error::InvalidRequest))
                // }
            }
            Request::GetCityDistributionCount { city_name } => {
                let workers = self
                    .city_distribution
                    .get(&city_name)
                    .ok_or(Error::UnavailableCityName)?;
                let count = u32::try_from(workers.len()).unwrap_or(u32::MAX);
                Ok(Response::GetCityDistributionCount { count })
            }
        }
    }
}
