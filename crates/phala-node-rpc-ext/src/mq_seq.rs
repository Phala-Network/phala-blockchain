use super::*;
use codec::Decode;
use pallet_mq_runtime_api::MqApi;
use phala_mq::MessageOrigin;
use phala_pallets::mq::tag;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid sender")]
    InvalidSender,
    #[error("{0}")]
    ApiError(#[from] sp_api::ApiError),
}

impl From<Error> for JsonRpseeError {
    fn from(e: Error) -> Self {
        JsonRpseeError::Call(
            CallError::Custom(
                ErrorObject::owned(
                    CUSTOM_RPC_ERROR,
                    e.to_string(),
                    Option::<()>::None
                )
            )
        )
    }
}

pub(super) fn get_mq_seq<Client, BE, Block, P>(
    client: &Client,
    pool: &Arc<P>,
    sender_hex: String,
) -> Result<u64, Error>
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
    Client::Api: MqApi<Block>,
    <<Block as BlockT>::Header as Header>::Number: Into<u64>,
    P: TransactionPool,
{
    let sender_scl = hex::decode(sender_hex).map_err(|_| Error::InvalidSender)?;
    let sender = MessageOrigin::decode(&mut &sender_scl[..]).map_err(|_| Error::InvalidSender)?;

    let api = client.runtime_api();
    let best_hash = client.info().best_hash;
    let at = BlockId::hash(best_hash);

    let seq = api.sender_sequence(&at, &sender)?.unwrap_or(0);

    log::debug!(target: "rpc-ext", "State seq for {}: {}", sender, seq);

    // Now we need to query the transaction pool
    // and find transactions originating from the same sender.
    //
    // Since extrinsics are opaque to us, we look for them using
    // `provides` tag. And increment the sequence if we find a transaction
    // that matches the current one.
    let mut current_seq = seq.clone();
    let mut current_tag = tag(&sender, seq);
    for tx in pool.ready() {
        log::debug!(
            target: "rpc-ext",
            "Current seq to {}, checking {} vs {:?}",
            current_seq,
            hex::encode(&current_tag),
            tx.provides().iter().map(|x| format!("{}", hex::encode(x))).collect::<Vec<_>>(),
        );
        // since transactions in `ready()` need to be ordered by sequence
        // it's fine to continue with current iterator.
        for tg in tx.provides() {
            if tg == &current_tag {
                current_seq += 1;
                current_tag = tag(&sender, current_seq);
                break;
            }
        }
    }

    log::debug!(target: "rpc-ext", "return seq {}", current_seq);

    Ok(current_seq)
}
