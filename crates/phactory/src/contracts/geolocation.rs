use std::collections::BTreeMap;
use std::string::String;

use anyhow::Result;
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
    data: Geocoding,
    created_at: chain::BlockNumber,
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
    GetAvailableRegionName,
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

    pub fn purge(&mut self, current_blocknum: &chain::BlockNumber) {
        // purging expired region map
        for (k, v) in &self.geo_data {
            if v.created_at <= current_blocknum - GEOCODING_EXPIRED_BLOCKNUM {
                // Will be removed very soon, delete its corresponding region map.
                if let Some(workers) = self.region_map.get_mut(&v.data.region_name) {
                    if let Some(pos) = workers.iter().position(|x| *x == *k) {
                        workers.remove(pos);
                    }
                } else {
                    error!("Cannot locate previous city name. Something is wrong in the geolocation contract's UpdateGeolocation() function");
                }
            }
        }

        // Purging expired geo_data
        self.geo_data.retain(|_, v| {
                v.created_at > current_blocknum - GEOCODING_EXPIRED_BLOCKNUM
        });
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
            Command::UpdateGeolocation { geocoding } => {
                let sender = match &origin {
                    MessageOrigin::Worker(pubkey) => AccountId::from(*pubkey),
                    _ => return Err(TransactionError::BadOrigin),
                };

                // If no geocoding is coming, return
                if geocoding.is_none() {
                    return Err(TransactionError::BadInput);
                }

                // Insert data to geolocation info btreemap
                if let Some(geo_datum) = self.geo_data.get_mut(&sender) {
                    // geocoding has a different region name compared to geo_datum's,
                    // we need to clear the region info of the sender:
                    if geo_datum.data.region_name != geocoding.as_ref().unwrap().region_name {
                        // Remove account id to previous region
                        if let Some(workers) = self.region_map.get_mut(&geo_datum.data.region_name) {
                            if let Some(pos) = workers.iter().position(|x| *x == sender) {
                                workers.remove(pos);
                            }
                        } else {
                            error!("Cannot locate previous city name. Something is wrong in the geolocation contract's UpdateGeolocation() function");
                            return Err(TransactionError::UnknownError);
                        }
                        // Insert account id to new region
                        let workers = self
                            .region_map
                            .entry(geocoding.as_ref().unwrap().region_name.clone())
                            .or_default();
                        workers.push(sender);
                    }
                    *geo_datum = GeocodingWithBlockInfo {
                        data: geocoding.unwrap().clone(),
                        created_at: context.block.block_number,
                    };
                } else {
                    // newly arrived worker
                    self.geo_data.insert(
                        sender.clone(),
                        GeocodingWithBlockInfo {
                            data: geocoding.as_ref().unwrap().clone(),
                            created_at: context.block.block_number,
                        },
                    );
                    let workers = self
                        .region_map
                        .entry(geocoding.unwrap().region_name)
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
            Request::GetGeocoding { account } => {
                if origin != Some(&account) {
                    return Err(Error::NotAuthorized);
                }
                let geo_data = self.geo_data.get(&account).ok_or(Error::NoRecord)?;
                let geocoding = geo_data.data.clone();
                Ok(Response::GetGeocoding { geocoding })
            }
            Request::GetAvailableRegionName {} => {
                // let valid_region_map = self.region_map.clone()
                //     .into_iter()
                //     .filter(|(_, v)|
                //         v.len() == 0)
                //     .collect();
                // let region_names: Vec<String> = valid_region_map.keys().cloned().collect();

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

    fn on_block_end(
        &mut self,
        context: &NativeContext,
    ) {
        self.purge(&context.block.block_number);
    }
}
