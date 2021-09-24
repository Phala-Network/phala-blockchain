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

use phala_types::messaging::{Geocoding, GeolocationCommand};

type Command = GeolocationCommand;

const GEOCODING_EXPIRED_BLOCKNUM: u32 = 2400; // roughly 8 hours

#[derive(Encode, Decode, Debug, Clone)]
pub struct GeocodingWithBlockInfo {
    data: Option<Geocoding>,
    created_at: chain::BlockNumber,
    err_info: Option<String>,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct Geolocation {
    geo_data: BTreeMap<AccountId, GeocodingWithBlockInfo>,
    region_map: BTreeMap<String, Vec<AccountId>>,
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
    GetGeocoding { account: AccountId },
    GetAvailableRegionName {},
    GetAccountsInRegion { region_name: String },
    GetAccountCountInRegion { region_name: String },
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Response {
    GetGeocoding { geocoding: Geocoding },
    GetAvailableRegionName { region_names: Vec<String> },
    GetAccountsInRegion { workers: Vec<AccountId> },
    GetAccountCountInRegion { count: u32 },
    Error(String),
}

impl Geolocation {
    pub fn new() -> Self {
        Geolocation {
            geo_data: BTreeMap::new(),
            region_map: BTreeMap::new(),
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
            Command::UpdateGeolocation { geocoding, err_info } => {
                let sender = match &origin {
                    MessageOrigin::Worker(pubkey) => AccountId::from(*pubkey),
                    _ => return Err(TransactionError::BadOrigin),
                };

                // Perform geo_data cleaning
                // Purging expired geo_data
                self.geo_data = self.geo_data.clone()
                    .into_iter()
                    .filter(|(_, v)|
                        v.created_at <= context.block.block_number - GEOCODING_EXPIRED_BLOCKNUM)
                    .collect();

                // Insert data to geolocation info btreemap
                if let Some(geo_datum) = self.geo_data.get_mut(&sender) {
                    // Some cases that we need to clear the region info of the sender:
                    // 1. geo_datum has value while geocoding has not;
                    // 2. geocoding has a different region name compared to geo_datum's
                    if (geo_datum.data.is_some() && geocoding.is_none())
                        || (geo_datum.data.as_ref().unwrap().region_name != geocoding.as_ref().unwrap().region_name) {
                        // Remove account id to previous region
                        if let Some(workers) = self.region_map.get_mut(&geo_datum.data.as_ref().unwrap().region_name) {
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
                        // Insert account id to new region
                        if geocoding.is_some() {
                            let workers = self
                                .region_map
                                .entry(geocoding.as_ref().unwrap().region_name.clone())
                                .or_default();
                            workers.push(sender);
                        }
                    }
                    *geo_datum = GeocodingWithBlockInfo {
                        data: geocoding.clone(),
                        created_at: context.block.block_number,
                        err_info
                    };
                } else {
                    // newly arrived worker
                    self.geo_data
                        .insert(sender.clone(),
                                GeocodingWithBlockInfo {
                                    data: geocoding.clone(),
                                    created_at: context.block.block_number,
                                    err_info
                                });
                    if geocoding.is_some() {
                        let workers = self
                            .region_map
                            .entry(geocoding.unwrap().region_name)
                            .or_default();
                        workers.push(sender);
                    }
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
            Request::GetGeocoding { account } => {
                if origin != Some(&account) {
                    return Err(Error::NotAuthorized);
                }
                if let Some(geo_data) = self.geo_data.get(&account) {
                    if let Some(geocoding) = geo_data.data.clone() {
                        Ok(Response::GetGeocoding { geocoding })
                    } else {
                        Err(Error::NoRecord)
                    }
                } else {
                    Err(Error::NoRecord)
                }
            }
            Request::GetAvailableRegionName {} => {
                let region_names: Vec<String> = self.region_map.keys().cloned().collect();
                Ok(Response::GetAvailableRegionName { region_names })
            }
            Request::GetAccountsInRegion { region_name } => {
                // TODO(soptq): Authorization
                Err(Error::Unimplemented)
                // if let Some(workers) = self.city_distribution.get(&region_name) {
                //     Ok(Response::GetCityDistribution { workers: workers.clone() })
                // } else {
                //     error!("Unavailable city name provided");
                //     Err(anyhow::Error::msg(Error::InvalidRequest))
                // }
            }
            Request::GetAccountCountInRegion { region_name } => {
                let workers = self
                    .region_map
                    .get(&region_name)
                    .ok_or(Error::UnavailableCityName)?;
                let count = u32::try_from(workers.len()).unwrap_or(u32::MAX);
                Ok(Response::GetAccountCountInRegion { count })
            }
        }
    }
}