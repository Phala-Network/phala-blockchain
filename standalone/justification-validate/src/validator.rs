use std::thread::JoinHandle;

use crossbeam::channel::{bounded, Sender};
use log::{error, warn};
use phactory::PRuntimeLightValidation;
use phactory_api::{
    blocks::{AuthoritySetChange, GenesisBlockInfo, HeaderToSync},
    storage_sync::BlockSyncState,
};
use pherry::headers_cache::{BlockInfo, Record};
use scale::Decode;

use super::Args;

type BatchData = (
    BlockSyncState<PRuntimeLightValidation>,
    Vec<HeaderToSync>,
    AuthoritySetChange,
);

pub struct Validator {
    args: Args,
    client: BlockSyncState<PRuntimeLightValidation>,
    threads: Vec<JoinHandle<()>>,
    sender: Sender<BatchData>,
}

impl Validator {
    pub fn new(args: Args) -> Self {
        let genesis = std::fs::read(&args.genesis).expect("Failed to read genesis file");
        let genesis_info =
            GenesisBlockInfo::decode(&mut &genesis[..]).expect("Failed to decode the genesis data");
        let header_number_next = genesis_info.block_header.number + 1;
        let mut validator = PRuntimeLightValidation::new();
        let main_bridge = validator
            .initialize_bridge(
                genesis_info.block_header,
                genesis_info.authority_set,
                genesis_info.proof,
            )
            .expect("Failed to init bridge");
        let (sender, threads) = create_thread_pool(args.threads);
        let client = BlockSyncState::new(validator, main_bridge, header_number_next, 0);
        Self {
            args,
            client,
            threads,
            sender,
        }
    }

    fn validate_batch(&mut self, headers: Vec<HeaderToSync>, auth_change: AuthoritySetChange) {
        if headers[0].header.number >= self.args.from && headers[0].header.number <= self.args.to {
            self.sender
                .send((self.client.clone(), headers.clone(), auth_change.clone()))
                .expect("Failed to send task");
        }
        do_validate(&mut self.client, headers, Some(auth_change));
    }

    pub fn run(mut self) {
        let mut headers_file =
            std::fs::File::open(&self.args.headers).expect("Failed to open headers.bin");
        let mut batch = vec![];
        let mut read_buffer = vec![0u8; 1024 * 100];

        loop {
            let Some(record) = Record::read(&mut headers_file, &mut read_buffer).unwrap() else {
                break;
            };

            let header = record.header().unwrap();
            if header.number > self.args.to {
                break;
            }
            let info = BlockInfo::decode(&mut record.payload())
                .unwrap_or_else(|err| panic!("Failed to decode header {}: {err:?}", header.number));

            batch.push(HeaderToSync {
                header: info.header,
                justification: info.justification,
            });

            if let Some(auth_change) = info.authority_set_change {
                warn!("auth changed at {}", header.number);
                self.validate_batch(core::mem::take(&mut batch), auth_change);
            }
        }

        drop(self.sender);
        warn!("Waiting for worker threads");
        for handle in self.threads.drain(..) {
            let _ = handle.join();
        }
    }
}

fn create_thread_pool(n: usize) -> (Sender<BatchData>, Vec<JoinHandle<()>>) {
    let (tx, rx) = bounded::<BatchData>(1);

    let mut threads = vec![];
    for _ in 0..n {
        let rx = rx.clone();
        let handle = std::thread::spawn(move || loop {
            let Ok((mut state, headers, auth)) = rx.recv() else {
                break;
            };
            let from = headers.first().unwrap().header.number;
            let to = headers.last().unwrap().header.number;
            let len = headers.len();
            let mut blocks = vec![];
            for (n, b) in headers.into_iter().enumerate() {
                let just = b.justification.is_some();
                blocks.push(b);
                if just {
                    let blocks = core::mem::take(&mut blocks);
                    if n + 1 == len {
                        do_validate(&mut state, blocks, Some(auth));
                        break;
                    } else {
                        do_validate(&mut state, blocks, None);
                    }
                }
            }
            warn!("validated ({from:?},{to:?}) len={len}");
        });
        threads.push(handle);
    }
    (tx, threads)
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
    let result = client.sync_header(headers, auth_change, &mut state_roots, 0);
    if let Err(err) = result {
        error!("Failed to sync header from={from:?} to={to:?}: {err:?}");
        std::process::exit(1);
    }
}
