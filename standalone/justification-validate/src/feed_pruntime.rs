use log::info;
use phactory_api::{
    blocks::{GenesisBlockInfo, HeaderToSync},
    prpc as pb,
    pruntime_client::new_pruntime_client,
};
use scale::Decode;

use crate::fetcher;
use crate::fetcher::BlockInfo;

use super::Args;

pub async fn feed_pruntime(url: String, args: Args) {
    let pruntime = new_pruntime_client(url);
    let fetcher = fetcher::BlockFetcher::new(&args.cache_uri);
    let genesis = fetcher
        .get_genesis()
        .await
        .expect("Failed to connect to fetcher");

    let info = pruntime
        .get_info(())
        .await
        .expect("Failed to call get_info");

    if !info.initialized {
        let genesis_info = GenesisBlockInfo::decode(&mut genesis.as_slice())
            .expect("Failed to decode the genesis data");

        let _response = pruntime
            .init_runtime(pb::InitRuntimeRequest::new(
                true,
                genesis_info,
                None,
                vec![],
                None,
                true,
            ))
            .await
            .expect("Failed to init pruntime");
    }

    let mut next = info.headernum;

    loop {
        info!("Fetching headers from {}", next);
        let encoded_blocks = fetcher
            .fetch_headers(next)
            .await
            .expect("Failed to fetch headers");
        let mut blocks = Vec::<BlockInfo>::decode(&mut encoded_blocks.as_slice())
            .expect("Failed to decode the blocks");
        let authority_set_change =
            std::mem::take(&mut blocks.last_mut().expect("No blocks").authority_set_change);
        let headers_to_sync = blocks
            .into_iter()
            .map(|block| HeaderToSync {
                header: block.header,
                justification: block.justification,
            })
            .collect::<Vec<_>>();
        let to = pruntime
            .sync_header(pb::HeadersToSync::new(
                headers_to_sync,
                authority_set_change,
            ))
            .await
            .unwrap_or_else(|err| {
                panic!("Failed to sync header: {err:?}");
            });
        next = to.synced_to + 1;
    }
}
