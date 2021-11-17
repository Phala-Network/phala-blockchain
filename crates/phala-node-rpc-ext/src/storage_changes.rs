use super::*;
pub use ext_types::*;

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
        Self::InvalidBlock(format!("{}: {}", id, error))
    }
}

impl From<Error> for jsonrpc_core::Error {
    fn from(e: Error) -> Self {
        jsonrpc_core::Error {
            code: jsonrpc_core::ErrorCode::ServerError(CUSTOM_RPC_ERROR),
            message: e.to_string(),
            data: None,
        }
    }
}

pub(super) fn get_storage_changes<Client, BE, Block>(
    client: &Client,
    backend: &BE,
    from: Block::Hash,
    to: Block::Hash,
) -> Result<GetStorageChangesResponse, Error>
where
    BE: Backend<Block>,
    Client: StorageProvider<Block, BE>
        + HeaderBackend<Block>
        + BlockBackend<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>
        + ProvideRuntimeApi<Block>,
    Block: BlockT + 'static,
    Client::Api:
        sp_api::Metadata<Block> + ApiExt<Block, StateBackend = backend::StateBackendFor<BE, Block>>,
    <<Block as BlockT>::Header as Header>::Number: Into<u64>,
{
    fn header<Client: HeaderBackend<Block>, Block: BlockT>(
        client: &Client,
        id: BlockId<Block>,
    ) -> Result<Block::Header, Error> {
        client
            .header(id)
            .map_err(|e| Error::invalid_block(id, e))?
            .ok_or_else(|| Error::invalid_block(id, "header not found"))
    }

    let n_from: u64 = (*header(client, BlockId::Hash(from))?.number()).into();
    let n_to: u64 = (*header(client, BlockId::Hash(to))?.number()).into();

    if n_from > n_to {
        return Err(Error::InvalidBlockRange {
            from: format!("{}({})", from, n_from),
            to: format!("{}({})", to, n_to),
        });
    }

    // TODO: Set max_number_of_blocks properly.
    let max_number_of_blocks = 10000u64;
    if n_to - n_from > max_number_of_blocks {
        return Err(Error::ResourceLimited("Too large number of blocks".into()));
    }

    let api = client.runtime_api();
    let mut changes = vec![];
    let mut this_block = to;

    loop {
        let id = BlockId::Hash(this_block);
        let mut header = header(client, id)?;
        let extrinsics = client
            .block_body(&id)
            .map_err(|e| Error::invalid_block(id, e))?
            .ok_or_else(|| Error::invalid_block(id, "block body not found"))?;
        let parent_hash = *header.parent_hash();
        let parent_id = BlockId::Hash(parent_hash);

        if (*header.number()).into() == 0u64 {
            let state = backend
                .state_at(id)
                .map_err(|e| Error::invalid_block(parent_id, e))?;
            changes.push(StorageChanges {
                main_storage_changes: state
                    .pairs()
                    .into_iter()
                    .map(|(k, v)| (StorageKey(k), Some(StorageKey(v))))
                    .collect(),
                child_storage_changes: vec![],
            });
            break;
        }

        // Remove all `Seal`s as they are added by the consensus engines after building the block.
        // On import they are normally removed by the consensus engine.
        header.digest_mut().logs.retain(|d| d.as_seal().is_none());

        let block = Block::new(header, extrinsics);
        api.execute_block(&parent_id, block)
            .map_err(|e| Error::invalid_block(id, e))?;

        let state = backend
            .state_at(parent_id)
            .map_err(|e| Error::invalid_block(parent_id, e))?;

        let storage_changes = api
            .into_storage_changes(&state, parent_hash)
            .map_err(|e| Error::invalid_block(parent_id, e))?;

        changes.push(StorageChanges {
            main_storage_changes: storage_changes.main_storage_changes.into_(),
            child_storage_changes: storage_changes.child_storage_changes.into_(),
        });
        if this_block == from {
            break;
        } else {
            this_block = parent_hash;
        }
    }
    changes.reverse();
    Ok(changes)
}

// Stuffs to convert ChildStorageCollection and StorageCollection types,
// in order to dump the keys values into hex strings instead of list of dec numbers.
pub trait MakeInto<T>: Sized {
    fn into_(self) -> T;
}

impl MakeInto<StorageKey> for Vec<u8> {
    fn into_(self) -> StorageKey {
        StorageKey(self)
    }
}

impl MakeInto<Vec<u8>> for StorageKey {
    fn into_(self) -> Vec<u8> {
        self.0
    }
}

impl<F: MakeInto<T>, T> MakeInto<Option<T>> for Option<F> {
    fn into_(self) -> Option<T> {
        self.map(|v| v.into_())
    }
}

impl<T1, T2, F1, F2> MakeInto<(T1, T2)> for (F1, F2)
where
    F1: MakeInto<T1>,
    F2: MakeInto<T2>,
{
    fn into_(self) -> (T1, T2) {
        (self.0.into_(), self.1.into_())
    }
}

impl<F: MakeInto<T>, T> MakeInto<Vec<T>> for Vec<F> {
    fn into_(self) -> Vec<T> {
        self.into_iter().map(|v| v.into_()).collect()
    }
}
