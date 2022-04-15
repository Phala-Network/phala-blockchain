use phaxt::ParachainApi;

use phaxt::khala::runtime_types::phala_types::{
    VersionedWorkerEndpoint,
    worker_endpoint_v1::WorkerEndpoint,
};

use phala_types::EndpointType;

pub async fn get_endpoint_info_by_pubkey(api: &mut &ParachainApi, pubkey: [u8; 32]) -> Option<(EndpointType, Vec<u8>)> {
    let endpoint_storage_iter = &mut api
        .storage()
        .phala_registry()
        .endpoints_iter(None)
        .await
        .ok()?;

    while let Some((_, versioned_endpoint_info)) = endpoint_storage_iter.next().await.ok()? {
        match versioned_endpoint_info {
            VersionedWorkerEndpoint::V1(endpoint_info) => match endpoint_info {
                WorkerEndpoint::I2P(i2p_endpoint) => {
                    if i2p_endpoint.pubkey.0 == pubkey {
                        return Some((EndpointType::I2P, i2p_endpoint.endpoint));
                    }
                }
                WorkerEndpoint::Http(http_endpoint) => {
                    if http_endpoint.pubkey.0 == pubkey {
                        return Some((EndpointType::Http, http_endpoint.endpoint));
                    }
                }
            },
        }
    }

    None
}

pub fn block_get_endpoint_info_by_pubkey(api: &mut &ParachainApi, pubkey: [u8; 32]) -> Option<(EndpointType, Vec<u8>)> {
    return match tokio::runtime::Runtime::new() {
        Ok(r) => r.block_on(get_endpoint_info_by_pubkey(api, pubkey)),
        Err(_) => None,
    };
}
