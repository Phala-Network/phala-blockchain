use chain::BlockNumber;
use phala_mq::traits::MessagePrepareChannel;
use phala_mq::SignedMessageChannel;

use crate::side_task::SideTaskManager;

use std::str::FromStr;
use std::{error, fmt};

use phala_types::contract;
use phala_types::messaging::{Geocoding, GeolocationCommand};

use maxminddb::geoip2;
use std::net::IpAddr;

use crate::secret_channel;
use phala_crypto::sr25519::KDF;
use sp_core::{hashing::blake2_256, sr25519, Pair};
use std::convert::TryInto;

const BLOCK_INTERVAL: BlockNumber = 2400;
// 8 hours
const PROBE_DURATION: BlockNumber = 5; // 1 min

// For detecting the public IP address of worker, we now use the service provided by ipify.org
// Why we use this service:
// 1. It is one of the largest and most popular IP address API services on the internet. ipify serves over 30 billion requests per month!
// 2. It is accessible in both Chinese mainland and other regions.
// 3. It is open sourced at https://github.com/rdegges/ipify-api.
const IP_PROBE_URL: &str = "https://api.ipify.org"; // ipinfo is reported to be baned in China

#[derive(Debug)]
pub enum GeoProbeError {
    // geo_probe
    FailedToGetPublicIPAddress,
    DBNotFound,
    DBNotValid,
    IPNotValid,
    NoRecord,
}

impl fmt::Display for GeoProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeoProbeError::FailedToGetPublicIPAddress => {
                write!(f, "network error, failed to get public IP address")
            }
            GeoProbeError::DBNotFound => write!(f, "geolite DB not found"),
            GeoProbeError::DBNotValid => write!(f, "geolite DB is probably broken"),
            GeoProbeError::IPNotValid => write!(f, "fetched IP address not valid for parsing"),
            GeoProbeError::NoRecord => write!(f, "no record found in DB"),
        }
    }
}

impl error::Error for GeoProbeError {}

pub fn db_query_region_name<'a>(city_general_data: &'a geoip2::City) -> Option<&'a str> {
    if let Some(city) = &city_general_data.city {
        return Some(*city.names.as_ref()?.get(&"en")?);
    }

    if let Some(subdivisions) = &city_general_data.subdivisions {
        return Some(*subdivisions[0].names.as_ref()?.get(&"en")?);
    }

    if let Some(country) = &city_general_data.country {
        return Some(*country.names.as_ref()?.get(&"en")?);
    }

    Some(&"N/A")
}

pub fn process_block(
    block_number: BlockNumber,
    egress: &SignedMessageChannel,
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
        pkh_first_32_bits
            .try_into()
            .expect("Should never fail with valid worker pubkey; qed."),
    ) % BLOCK_INTERVAL;
    if block_number % BLOCK_INTERVAL == worker_magic {
        log::info!(
            "start geolocation probing at block {}, worker magic {}",
            block_number,
            worker_magic
        );

        let egress = egress.clone();
        let duration = PROBE_DURATION;

        let topic = contract::command_topic(contract::id256(contract::GEOLOCATION));
        let my_ecdh_key = identity_key
            .derive_ecdh_key()
            .expect("Should never failed with valid identity key; qed.");
        // TODO: currently assume contract key equals to local ecdh key
        let remote_pubkey = my_ecdh_key.clone().public();

        let default_messages = {
            let message = GeolocationCommand::update_geolocation(None);
            let secret_channel =
                secret_channel::bind_remote(&egress, &my_ecdh_key, Some(&remote_pubkey));
            [secret_channel.prepare_message_to(&message, &topic[..])]
        };

        side_task_man.add_async_task(block_number, duration, default_messages, async move {
            // 1. we load the database first, so that in case where the database not exists,
            // we can just return an error without emits any http request.
            let geo_db_buf = std::fs::read(geoip_city_db).or(Err(GeoProbeError::DBNotFound))?;

            // 2. get IP address.
            let mut resp = surf::get(IP_PROBE_URL)
                .send()
                .await
                .or(Err(GeoProbeError::FailedToGetPublicIPAddress))?;
            let pub_ip = resp
                .body_string()
                .await
                .or(Err(GeoProbeError::FailedToGetPublicIPAddress))?;
            log::info!("public IP address: {}", pub_ip);

            // 3. Look up geolocation info in maxmind database.
            let reader =
                maxminddb::Reader::from_source(geo_db_buf).or(Err(GeoProbeError::DBNotValid))?;
            let ip: IpAddr = FromStr::from_str(&pub_ip).or(Err(GeoProbeError::IPNotValid))?;

            let city_general_data: geoip2::City =
                reader.lookup(ip).or(Err(GeoProbeError::NoRecord))?;
            let region_name =
                db_query_region_name(&city_general_data).ok_or(GeoProbeError::NoRecord)?;

            let location = city_general_data
                .location
                .clone()
                .ok_or(GeoProbeError::NoRecord)?;
            let latitude = location.latitude.ok_or(GeoProbeError::NoRecord)?;
            let longitude = location.longitude.ok_or(GeoProbeError::NoRecord)?;

            info!(
                "look-up geolocation: {}, {}, {}",
                latitude, longitude, region_name
            );

            let geocoding = Geocoding {
                latitude: (latitude * 10000f64) as i32,
                longitude: (longitude * 10000f64) as i32,
                region_name: region_name.to_string(),
            };

            // 4. construct the confidential contract command.
            let msg = GeolocationCommand::update_geolocation(Some(geocoding));

            // 5. construct the secret message channel
            let secret_channel =
                secret_channel::bind_remote(&egress, &my_ecdh_key, Some(&remote_pubkey));
            let topic = contract::command_topic(contract::id256(contract::GEOLOCATION));
            //6. send the command
            Ok([secret_channel.prepare_message_to(&msg, topic)])
        });
    }
}
