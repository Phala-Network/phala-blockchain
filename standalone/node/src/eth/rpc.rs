// pub fn extend_rpc<Client, BE, Block, P>(
//     io: &mut RpcModule<()>,
//     client: Arc<Client>,
//     backend: Arc<BE>,
//     is_archive_mode: bool,
//     pool: Arc<P>,
// ) where
//     BE: Backend<Block> + 'static,
//     Client: StorageProvider<Block, BE>
//         + HeaderBackend<Block>
//         + BlockBackend<Block>
//         + HeaderMetadata<Block, Error = sp_blockchain::Error>
//         + ProvideRuntimeApi<Block>
//         + 'static,
//     Block: BlockT + 'static,
//     Client::Api: sp_api::Metadata<Block> + ApiExt<Block>,
//     Client::Api: MqApi<Block>,
//     <<Block as BlockT>::Header as Header>::Number: Into<u64>,
//     P: TransactionPool + 'static,
// {
// }
