use super::*;
pub use ext_types::*;
use sp_runtime::StateVersion;

/// State RPC errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Provided block range couldn't be resolved to a list of blocks.
    #[error("Cannot resolve a block range ['{from}' ... '{to}].")]
    InvalidBlockRange {
        /// Beginning of the block range.
        from: String,
        /// End of the block range.
        to: String,
    },

    /// Aborted due resource limiting such as MAX_NUMBER_OF_BLOCKS.
    #[error("Resource limited, {0}.")]
    ResourceLimited(String),

    /// Error occurred while processing some block.
    #[error("Error occurred while processing the block {0}.")]
    InvalidBlock(String),

    /// The RPC is unavailable.
    #[error("This RPC is unavailable. {0}")]
    Unavailable(String),
}

impl Error {
    fn invalid_block<Block: BlockT, E: Display>(id: BlockId<Block>, error: E) -> Self {
        Self::InvalidBlock(format!("{id}: {error}"))
    }
}

impl From<Error> for JsonRpseeError {
    fn from(e: Error) -> Self {
        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            CUSTOM_RPC_ERROR,
            e.to_string(),
            Option::<()>::None,
        )))
    }
}

pub(super) fn get_storage_changes<Client, BE, Block>(
    client: &Client,
    backend: &BE,
    from: Block::Hash,
    to: Block::Hash,
    with_roots: bool,
) -> Result<GetStorageChangesResponseWithRoot, Error>
where
    BE: Backend<Block>,
    Client: StorageProvider<Block, BE>
        + HeaderBackend<Block>
        + BlockBackend<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>
        + ProvideRuntimeApi<Block>,
    Block: BlockT + 'static,
    Client::Api:
        sp_api::Metadata<Block> + ApiExt<Block>,
    <<Block as BlockT>::Header as Header>::Number: Into<u64>,
{
    fn header<Client: HeaderBackend<Block>, Block: BlockT>(
        client: &Client,
        hash: Block::Hash,
    ) -> Result<Block::Header, Error> {
        client
            .header(hash)
            .map_err(|e| Error::invalid_block(BlockId::<Block>::Hash(hash), e))?
            .ok_or_else(|| Error::invalid_block(BlockId::<Block>::Hash(hash), "header not found"))
    }

    macro_rules! get_state_root {
        ($state:expr) => {
            if with_roots {
                $state
                    .storage_root(core::iter::empty(), StateVersion::V0)
                    .0
                    .as_ref()
                    .to_vec()
            } else {
                Vec::new()
            }
        };
    }

    let n_from: u64 = (*header(client, from)?.number()).into();
    let n_to: u64 = (*header(client, to)?.number()).into();

    if n_from > n_to {
        return Err(Error::InvalidBlockRange {
            from: format!("{from}({n_from})"),
            to: format!("{to}({n_to})"),
        });
    }

    // TODO: Set max_number_of_blocks properly.
    let max_number_of_blocks = 10000u64;
    if n_to - n_from > max_number_of_blocks {
        return Err(Error::ResourceLimited("Too large number of blocks".into()));
    }

    let mut headers = std::collections::VecDeque::new();

    let mut this_block = to;

    loop {
        let id = BlockId::Hash(this_block);
        let header = header(client, this_block)?;
        let parent = *header.parent_hash();
        headers.push_front((id, header));
        if this_block == from {
            break;
        }
        this_block = parent;
    }

    headers
        .into_iter()
        .map(|(id, mut header)| -> Result<_, Error> {
            let api = client.runtime_api();
            let hash = client
                .expect_block_hash_from_id(&id)
                .expect("Should get the block hash");
            if (*header.number()).into() == 0u64 {
                let state = backend
                    .state_at(hash)
                    .map_err(|e| Error::invalid_block(id, e))?;
                let state_root = get_state_root!(&state);
                return Ok(StorageChangesWithRoot {
                    changes: StorageChanges {
                        main_storage_changes: state
                            .pairs(Default::default())
                            .expect("Should get the pairs iter")
                            .map(|pair| {
                                let (k, v) = pair.expect("Should get the key and value");
                                (StorageKey(k), Some(StorageKey(v)))
                            })
                            .collect(),
                        child_storage_changes: vec![],
                    },
                    state_root,
                });
            }

            let extrinsics = client
                .block_body(hash)
                .map_err(|e| Error::invalid_block(id, e))?
                .ok_or_else(|| Error::invalid_block(id, "block body not found"))?;
            let parent_hash = *header.parent_hash();

            // Remove all `Seal`s as they are added by the consensus engines after building the block.
            // On import they are normally removed by the consensus engine.
            header.digest_mut().logs.retain(|d| d.as_seal().is_none());

            let block = Block::new(header, extrinsics);
            api.execute_block(parent_hash, block)
                .map_err(|e| Error::invalid_block(id, e))?;

            let state = backend
                .state_at(hash)
                .map_err(|e| Error::invalid_block(BlockId::<Block>::Hash(parent_hash), e))?;

            let storage_changes = api
                .into_storage_changes(&state, parent_hash)
                .map_err(|e| Error::invalid_block(BlockId::<Block>::Hash(parent_hash), e))?;

            let state_root = get_state_root!(&state);
            Ok(StorageChangesWithRoot {
                changes: StorageChanges {
                    main_storage_changes: storage_changes.main_storage_changes.into_(),
                    child_storage_changes: storage_changes.child_storage_changes.into_(),
                },
                state_root,
            })
        })
        .collect()
}
