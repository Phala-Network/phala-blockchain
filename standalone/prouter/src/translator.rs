use phaxt::ParachainApi;
use phala_types::{worker_endpoint_v1::WorkerEndpoint, VersionedWorkerEndpoints};
use phaxt::subxt::ext::sp_core::sr25519::Public;
use codec::Decode;

use phala_types::EndpointType;

pub async fn get_endpoint_info_by_pubkey(
    api: &ParachainApi,
    pubkey: [u8; 32],
    endpoint_type: EndpointType,
) -> Option<Vec<u8>> {

    let address = api.storage_key("PhalaRegistry", "Endpoints", &Public(pubkey)).ok()?;
    let endpoint_storage = api
        .rpc()
        .storage(&address, None)
        .await
        .ok()?;

    if let Some(data) = endpoint_storage {
        let versioned_endpoint = VersionedWorkerEndpoints::decode(&mut &data.0[..]).ok()?;
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
