use log::info;
use phactory_api::{
    blocks::{GenesisBlockInfo, HeaderToSync},
    prpc as pb,
    pruntime_client::new_pruntime_client,
};
use pherry::headers_cache::{BlockInfo, Record};
use scale::Decode;

use super::Args;

pub async fn feed_pruntime(url: String, args: Args) {
    let pruntime = new_pruntime_client(url);

    let info = pruntime
        .get_info(())
        .await
        .expect("Failed to call get_info");

    if !info.initialized {
        let genesis = std::fs::read(&args.genesis).expect("Failed to read genesis file");
        let genesis_info =
            GenesisBlockInfo::decode(&mut &genesis[..]).expect("Failed to decode the genesis data");

        let _response = pruntime
            .init_runtime(pb::InitRuntimeRequest::new(
                true,
                genesis_info,
                None,
                vec![],
                None,
                true,
                None,
            ))
            .await
            .expect("Failed to init pruntime");
    }

    let mut headers_file = std::fs::File::open(&args.headers).expect("Failed to open headers.bin");

    let mut batch = vec![];
    let mut read_buffer = vec![0u8; 1024 * 100];

    loop {
        let Some(record) = Record::read(&mut headers_file, &mut read_buffer).unwrap() else {
            break;
        };

        let header = record.header().unwrap();
        if header.number < info.headernum {
            if header.number % 10000 == 0 {
                info!("Skipped {}", header.number);
            }
            continue;
        }
        if header.number > args.to {
            break;
        }
        let info = BlockInfo::decode(&mut record.payload())
            .unwrap_or_else(|err| panic!("Failed to decode header {}: {err:?}", header.number));

        let has_justification = info.justification.is_some();
        let block_number = info.header.number;

        batch.push(HeaderToSync {
            header: info.header,
            justification: info.justification,
        });

        if let Some(auth_change) = info.authority_set_change {
            info!("AuthSet changed at {}", block_number);
            let len = batch.len();
            for block in batch[..len - 1].iter_mut() {
                block.justification = None;
            }
            let _to = pruntime
                .sync_header(pb::HeadersToSync::new(
                    core::mem::take(&mut batch),
                    Some(auth_change),
                ))
                .await
                .unwrap_or_else(|err| {
                    panic!("Failed to sync header: {err:?}");
                });
            continue;
        }

        if has_justification && args.from <= block_number {
            info!("Feeding {}", block_number);
            let _to = pruntime
                .sync_header(pb::HeadersToSync::new(core::mem::take(&mut batch), None))
                .await
                .unwrap_or_else(|err| {
                    panic!("Failed to sync header: {err:?}");
                });
            continue;
        }
    }
}
