use codec::Decode;
use phala_types::VersionedWorkerEndpoints;
use phaxt::subxt::ext::sp_core::sr25519::Public;
use phaxt::ParachainApi;

use phactory_api::endpoints::EndpointType;

pub async fn get_endpoint_info_by_pubkey(
    api: &ParachainApi,
    pubkey: [u8; 32],
    endpoint_type: EndpointType,
) -> Option<String> {
    let address = api
        .storage_key("PhalaRegistry", "Endpoints", &Public(pubkey))
        .ok()?;
    let storage_data = api.rpc().storage(&address, None).await.ok().flatten()?;
    let VersionedWorkerEndpoints::V1(endpoints_info) = Decode::decode(&mut &storage_data.0[..]).ok()?;
    return endpoints_info.into_iter().nth(0);
}

pub fn block_get_endpoint_info_by_pubkey(
    api: &ParachainApi,
    pubkey: [u8; 32],
    endpoint_type: EndpointType,
) -> Option<String> {
    return tokio::runtime::Runtime::new()
        .map(|r| r.block_on(get_endpoint_info_by_pubkey(api, pubkey, endpoint_type)))
        .expect("Failed to create runtime");
}
