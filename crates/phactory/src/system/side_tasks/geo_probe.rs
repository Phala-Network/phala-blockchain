use chain::BlockNumber;
use phala_mq::Sr25519MessageChannel;

use crate::side_task::async_side_task::AsyncSideTask;
use crate::side_task::SideTaskManager;

use std::{error, fmt};
use std::str::FromStr;

use phala_types::contract;
use phala_types::messaging::{Geocoding, GeolocationCommand};

use maxminddb::geoip2;
use std::net::IpAddr;

use sp_core::{hashing::blake2_256, sr25519, Pair};
use phala_crypto::sr25519::KDF;
use crate::secret_channel::SecretMessageChannel;
use std::convert::TryInto;

const BLOCK_INTERVAL: BlockNumber = 2400;   // 8 hours
const PROBE_DURATION: BlockNumber = 5;      // 1 min

// For detecting the public IP address of worker, we now use the service provided by ipify.org
// Why we use this service:
// 1. It is one of the largest and most popular IP address API services on the internet. ipify serves over 30 billion requests per month!
// 2. It is accessible in both Chinese mainland and other regions.
// 3. It is open sourced at https://github.com/rdegges/ipify-api.
const IP_PROBE_URL: &str = "api.ipify.org";   // ipinfo is reported to be baned in China

#[derive(Debug)]
pub enum GeoProbeError {
    // geo_probe
    FailedToGetPublicIPAddress,
    DBNotFound,
    DBNotValid,
    IPNotValid,
    NoRecord,
    UnknownError,
}

impl fmt::Display for GeoProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeoProbeError::FailedToGetPublicIPAddress => {
                write!(f, "network error, failed to get public IP address")
            },
            GeoProbeError::DBNotFound => write!(f, "geolite DB not found"),
            GeoProbeError::DBNotValid => write!(f, "geolite DB is probably broken"),
            GeoProbeError::IPNotValid => write!(f, "fetched IP address not valid for parsing"),
            GeoProbeError::NoRecord => write!(f, "no record found in DB"),
            GeoProbeError::UnknownError => write!(f, "unknown error"),
        }
    }
}

impl error::Error for GeoProbeError {}

pub fn db_query_region_name<'a>(city_general_data: &'a geoip2::City) -> &'a str {
    if let Some(city) = &city_general_data.city {
        return *city.names.as_ref().unwrap().get(&"en").unwrap();
    }

    if let Some(subdivisions) = &city_general_data.subdivisions {
        return *subdivisions[0].names.as_ref().unwrap().get(&"en").unwrap();
    }

    if let Some(country) = &city_general_data.country {
        return *country.names.as_ref().unwrap().get(&"en").unwrap();
    } else {
        return &"NA";
    }
}

pub fn process_block(
    block_number: BlockNumber,
    egress: &Sr25519MessageChannel,
    side_task_man: &mut SideTaskManager,
    identity_key: &sr25519::Pair,
    geoip_city_db: String,
) {
    let identity_key = identity_key.clone();
    let worker_pubkey = identity_key.public();
    let raw_pubkey: &[u8] = worker_pubkey.as_ref();
    let pkh = blake2_256(raw_pubkey);
    let (pkh_first_32_bits, _) = pkh.split_at(std::mem::size_of::<u32>());
    let worker_magic = u32::from_be_bytes(
        match pkh_first_32_bits.try_into() {
            Ok(data) => data,
            Err(e) => {
                info!("failed to init geo_probe side task");
                return;
            }
        }
    ) % BLOCK_INTERVAL;
    if block_number % BLOCK_INTERVAL == 1 {
        log::info!("start geolocation probing at block {}, worker magic {}",
            block_number, worker_magic);

        let egress = egress.clone();
        let duration = PROBE_DURATION;
        let task = AsyncSideTask::spawn(
            block_number,
            duration,
            async {
                // 1. check if db exists, if not then return
                if !std::path::Path::new(&geoip_city_db).exists() {
                    return Err(GeoProbeError::DBNotFound)
                }

                // 1. get IP address.
                let mut resp = surf::get(IP_PROBE_URL).send().await.map_err(|_| GeoProbeError::FailedToGetPublicIPAddress)?;
                let pub_ip = resp.body_string().await.map_err(|_| GeoProbeError::FailedToGetPublicIPAddress)?;
                log::info!("public IP address: {}", pub_ip);

                // 2. Look up geolocation info in maxmind database.
                let geo_db_buf = std::fs::read(geoip_city_db).map_err(|_| GeoProbeError::DBNotFound)?;
                let reader =
                    maxminddb::Reader::from_source(geo_db_buf).map_err(|_| GeoProbeError::DBNotValid)?;
                let ip: IpAddr = FromStr::from_str(&pub_ip).map_err(|_| GeoProbeError::IPNotValid)?;

                let city_general_data: geoip2::City = reader.lookup(ip).map_err(|_| GeoProbeError::NoRecord)?;
                let region_name = db_query_region_name(&city_general_data).parse::<String>().map_err(|_| GeoProbeError::NoRecord)?;

                let location = city_general_data.location.clone().ok_or(GeoProbeError::NoRecord)?;
                let latitude = location.latitude.ok_or(GeoProbeError::NoRecord)?;
                let longitude = location.longitude.ok_or(GeoProbeError::NoRecord)?;

                info!(
                    "look-up geolocation: {}, {}, {}",
                    latitude, longitude, region_name
                );

                let geocoding = Geocoding {
                    latitude: (latitude * 10000f64) as i32,
                    longitude: (longitude * 10000f64) as i32,
                    region_name,
                };

                Ok(geocoding)
            },
            move |result, _context| {
                let result = result
                    .unwrap_or(Err(GeoProbeError::UnknownError));
                match result {
                    Err(ref e) => {
                        info!("geo_probe sidetask error: {}", e);
                    }
                    _ => {}
                };
                let msg = GeolocationCommand::update_geolocation(result.ok());

                let my_ecdh_key = identity_key
                    .derive_ecdh_key()
                    .expect("Should never failed with valid identity key; qed.");
                // TODO: currently assume contract key equals to local ecdh key
                let public_contract_ecdh_key = my_ecdh_key.clone().public();
                // TODO: currently is a fake key map.
                let key_map = |topic: &[u8]| {
                    Some(public_contract_ecdh_key)
                };
                let secret_egress = SecretMessageChannel::new(&my_ecdh_key,
                                                              &egress,
                                                              &key_map);
                let topic = contract::command_topic(contract::id256(contract::GEOLOCATION));
                log::info!("send msg [{:?}] to topic [{:?}]", &msg, String::from_utf8_lossy(&topic));

                secret_egress.sendto(topic, &msg, Some(&public_contract_ecdh_key));
            },
        );
        side_task_man.add_task(task);
    }
}