use chain::BlockNumber;
use phala_mq::Sr25519MessageChannel;

use crate::side_task::async_side_task::AsyncSideTask;
use crate::side_task::SideTaskManager;

use std::str::FromStr;
use phala_types::contract;
use phala_types::messaging::{Geocoding, GeolocationCommand};

use maxminddb::geoip2;
use std::net::IpAddr;

use sp_core::sr25519;
use phala_crypto::sr25519::KDF;
use crate::secret_channel::SecretMessageChannel;

const BLOCK_INTERVAL: BlockNumber = 2400;   // 8 hours
const PROBE_DURATION: BlockNumber = 5;      // 1 min
const IP_PROBE_URL: &str = "https://ip.kvin.wang";   // ipinfo is reported to be baned in China

pub enum GeoProbeSideTaskResult {
    Success(Geocoding),
    Failed(String)
}

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
    identity_key: &sr25519::Pair
) {
    if block_number % BLOCK_INTERVAL == 1 {
        log::info!("start geolocation probing at block {}",
            block_number.clone());

        let identity_key = identity_key.clone();
        let egress = egress.clone();
        let duration = PROBE_DURATION;
        let task = AsyncSideTask::spawn(
            block_number,
            duration,
            async {
                // 1. get IP address.
                let mut resp = match surf::get(IP_PROBE_URL).send().await {
                    Ok(r) => r,
                    Err(err) => {
                        return GeoProbeSideTaskResult::Failed(format!("network error: {:?}", err));
                    }
                };
                let pub_ip = match resp.body_string().await {
                    Ok(body) => body,
                    Err(err) => {
                        return GeoProbeSideTaskResult::Failed(format!("network error: {:?}", err));
                    }
                };
                log::info!("public IP address: {}", pub_ip);

                // 2. Look up geolocation info in maxmind database.
                let geo_db_buf = match std::fs::read("./GeoLite2-City.mmdb") {
                    Ok(data) => data,
                    Err(err) => {
                        return GeoProbeSideTaskResult::Failed(format!("cannot open mmdb file: {:?}", err));
                    }
                };
                let reader =
                    maxminddb::Reader::from_source(geo_db_buf).expect("Geolite2 database is not loaded");
                let ip: IpAddr = FromStr::from_str(&pub_ip).unwrap();

                let city_general_data: geoip2::City = reader.lookup(ip).unwrap();
                let region_name = match db_query_region_name(&city_general_data).parse::<String>() {
                    Ok(data) => data,
                    Err(err) => {
                        return GeoProbeSideTaskResult::Failed(format!("fail to parse region name: {:?}", err));
                    }
                };

                if city_general_data.location.is_none() {
                    return GeoProbeSideTaskResult::Failed("location is not existed in database".into());
                }
                let location = city_general_data.location.clone().unwrap(); // This should never fail.

                if location.latitude.is_none() {
                    return GeoProbeSideTaskResult::Failed("latitude is not existed in database".into());
                }
                if location.longitude.is_none() {
                    return GeoProbeSideTaskResult::Failed("longitude is not existed in database".into());
                }
                let latitude = location.latitude.unwrap();  // This should never fail.
                let longitude = location.longitude.unwrap();    // This should never fail.

                info!(
                    "look-up geolocation: {}, {}, {}",
                    latitude, longitude, region_name
                );

                let geocoding = Geocoding {
                    latitude: (latitude * 10000f64) as i32,
                    longitude: (longitude * 10000f64) as i32,
                    region_name,
                };

                GeoProbeSideTaskResult::Success(geocoding)
            },
            move |result, _context| {
                let result = result
                    .unwrap_or(GeoProbeSideTaskResult::Failed("failed to probe geolocation.".into()));
                let msg = match result {
                    GeoProbeSideTaskResult::Failed(err_info) => {
                        GeolocationCommand::update_geolocation (
                            None, Some(err_info)
                        )
                    },
                    GeoProbeSideTaskResult::Success(geocoding) => {
                        GeolocationCommand::update_geolocation (
                            Some(geocoding), None
                        )
                    }
                };

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
                log::info!("send msg [{:?}] to topic [{:?}]", &msg, &topic);

                secret_egress.sendto(topic, &msg, Some(&public_contract_ecdh_key));
            },
        );
        side_task_man.add_task(task);
    }
}