use phaxt::ParachainApi;

use phaxt::khala::runtime_types::phala_types::{
    worker_endpoint_v1::WorkerEndpoint, VersionedWorkerEndpoint,
};

use phala_types::EndpointType;

pub async fn get_endpoint_info_by_pubkey(
    api: &ParachainApi,
    pubkey: [u8; 32],
    endpoint_type: EndpointType,
) -> Option<Vec<u8>> {
    let mut endpoint_storage_iter = api
        .storage()
        .phala_registry()
        .endpoints_iter(None)
        .await
        .ok()?;

    while let Some((key, versioned_endpoint_info)) = endpoint_storage_iter.next().await.ok()? {
        match versioned_endpoint_info {
            VersionedWorkerEndpoint::V1(endpoints_info) => {
                for endpoint_info in endpoints_info {
                    match endpoint_info {
                        WorkerEndpoint::I2P(endpoint_info) => {
                            if matches!(endpoint_type, EndpointType::I2P) && key.0 == pubkey {
                                return Some(endpoint_info.endpoint);
                            }
                        }
                        WorkerEndpoint::Http(endpoint_info) => {
                            if matches!(endpoint_type, EndpointType::I2P) && key.0 == pubkey {
                                return Some(endpoint_info.endpoint);
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
