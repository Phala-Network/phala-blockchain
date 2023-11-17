use log::{error, info};
use phactory::PRuntimeLightValidation;
use phactory_api::{
    blocks::{AuthoritySetChange, GenesisBlockInfo, HeaderToSync},
    storage_sync::BlockSyncState,
};
use scale::Decode;

use crate::fetcher;
use crate::fetcher::BlockInfo;

use super::Args;

pub struct Validator {
    client: BlockSyncState<PRuntimeLightValidation>,
    fetcher: fetcher::BlockFetcher,
    genesis_block: u32,
}

impl Validator {
    pub async fn new(args: Args) -> Self {
        let fetcher = fetcher::BlockFetcher::new(&args.cache_uri);
        let genesis = fetcher.get_genesis().await.expect("Failed to get genesis");
        let genesis_info =
            GenesisBlockInfo::decode(&mut &genesis[..]).expect("Failed to decode the genesis data");
        let genesis_block = genesis_info.block_header.number;
        let mut validator = PRuntimeLightValidation::new();
        let main_bridge = validator
            .initialize_bridge(
                genesis_info.block_header,
                genesis_info.authority_set,
                genesis_info.proof,
            )
            .expect("Failed to init bridge");
        let client = BlockSyncState::new(validator, main_bridge, genesis_block + 1, 0);
        Self {
            client,
            fetcher,
            genesis_block,
        }
    }

    fn validate_batch(
        &mut self,
        headers: Vec<HeaderToSync>,
        auth_change: Option<AuthoritySetChange>,
    ) {
        do_validate(&mut self.client, headers, auth_change);
    }

    pub async fn run(mut self) {
        let mut next = self.genesis_block + 1;
        loop {
            info!("Checking headers start at {}", next);
            let encoded_blocks = self.fetcher.fetch_headers(next).await;
            let encoded_blocks = match encoded_blocks {
                Ok(b) => b,
                Err(err) => {
                    error!("Failed to fetch headers: {err:?}");
                    break;
                }
            };
            let mut blocks = Vec::<BlockInfo>::decode(&mut encoded_blocks.as_slice())
                .expect("Failed to decode the blocks");
            let last = blocks.last().expect("No blocks").header.number;
            next = last + 1;
            let authority_set_change =
                std::mem::take(&mut blocks.last_mut().expect("No blocks").authority_set_change);
            let headers_to_sync = blocks
                .into_iter()
                .map(|block| HeaderToSync {
                    header: block.header,
                    justification: block.justification,
                })
                .collect::<Vec<_>>();
            self.validate_batch(headers_to_sync, authority_set_change);
        }
    }
}

fn do_validate(
    client: &mut BlockSyncState<PRuntimeLightValidation>,
    headers: Vec<HeaderToSync>,
    auth_change: Option<AuthoritySetChange>,
) {
    if headers.is_empty() {
        return;
    }
    let from = headers[0].header.number;
    let to = headers.last().unwrap().header.number;

    let mut state_roots = Default::default();
    let result = client.sync_header(headers, auth_change, &mut state_roots);
    if let Err(err) = result {
        error!("Failed to sync header from={from:?} to={to:?}: {err}");
        std::process::exit(1);
    }
}
