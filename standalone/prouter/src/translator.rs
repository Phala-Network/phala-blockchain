use phaxt::ParachainApi;

use phaxt::parachain::runtime_types::{
    phala_types::{worker_endpoint_v1::WorkerEndpoint, VersionedWorkerEndpoints},
    sp_core::sr25519::Public,
};

use phala_types::EndpointType;

pub async fn get_endpoint_info_by_pubkey(
    api: &ParachainApi,
    pubkey: [u8; 32],
    endpoint_type: EndpointType,
) -> Option<Vec<u8>> {
    let query = phaxt::parachain::storage()
        .phala_registry()
        .endpoints(&Public(pubkey));
    let endpoint_storage = api
        .storage()
        .fetch(&query, None)
        .await
        .ok()?;

    if let Some(versioned_endpoint) = endpoint_storage {
        match versioned_endpoint {
            VersionedWorkerEndpoints::V1(endpoints_info) => {
                for endpoint_info in endpoints_info {
                    match endpoint_info.endpoint {
                        WorkerEndpoint::I2P(endpoint) => {
                            if matches!(endpoint_type, EndpointType::I2P) {
                                return Some(endpoint);
                            }
                        }
                        WorkerEndpoint::Http(endpoint) => {
                            if matches!(endpoint_type, EndpointType::I2P) {
                                return Some(endpoint);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn block_get_endpoint_info_by_pubkey(
    api: &ParachainApi,
    pubkey: [u8; 32],
    endpoint_type: EndpointType,
) -> Option<Vec<u8>> {
    return tokio::runtime::Runtime::new()
        .map(|r| r.block_on(get_endpoint_info_by_pubkey(api, pubkey, endpoint_type)))
        .expect("Failed to create runtime");
}
