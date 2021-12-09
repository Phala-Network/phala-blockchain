use anyhow::{anyhow, Result};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use phaxt::ParachainApi;

use phaxt::khala::runtime_types::phala_pallets::registry::pallet::{
    VersionedWorkerEndpoint,
    WorkerEndpointV1::{PhalaEndpointInfo, WorkerEndpoint},
};

pub async fn get_endpoint_by_pubkey(api: &mut &ParachainApi, pubkey: [u8; 32]) -> Option<Vec<u8>> {
    let phala_endpoint_storage_iter = &mut api
        .storage()
        .phala_registry()
        .phala_endpoints_iter(None)
        .await
        .ok()?;

    while let Some((_, versioned_endpoint_info)) = phala_endpoint_storage_iter.next().await.ok()? {
        match versioned_endpoint_info {
            VersionedWorkerEndpoint::V1(endpoint_info) => match endpoint_info {
                WorkerEndpoint::I2P(i2p_endpoint) => {
                    if i2p_endpoint.pubkey.0 == pubkey {
                        return Some(i2p_endpoint.endpoint);
                    }
                }
                WorkerEndpoint::Http(http_endpoint) => {
                    if http_endpoint.pubkey.0 == pubkey {
                        return Some(http_endpoint.endpoint);
                    }
                }
            },
        }
    }

    None
}

pub fn block_get_endpoint_by_pubkey(api: &mut &ParachainApi, pubkey: [u8; 32]) -> Option<Vec<u8>> {
    return match tokio::runtime::Runtime::new() {
        Ok(r) => r.block_on(get_endpoint_by_pubkey(api, pubkey)),
        Err(_) => None,
    };
}
