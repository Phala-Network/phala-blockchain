use anyhow::{Context, Result};
use std::{ops::Deref, sync::Arc};

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use subxt::{
    ext::sp_core::sr25519,
    tx::{PolkadotExtrinsicParams, PolkadotExtrinsicParamsBuilder},
};

mod chain_api;
pub mod dynamic;
pub mod rpc;

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, PartialOrd, Ord, Debug)]
pub struct ParaId(pub u32);

pub type StorageProof = Vec<Vec<u8>>;
pub type StorageState = Vec<(Vec<u8>, Vec<u8>)>;
pub type PairSigner = subxt::tx::PairSigner<Config, sr25519::Pair>;
pub type ExtrinsicParams = PolkadotExtrinsicParams<subxt::SubstrateConfig>;
pub type ExtrinsicParamsBuilder = PolkadotExtrinsicParamsBuilder<subxt::SubstrateConfig>;
pub use subxt::PolkadotConfig as Config;
pub type RpcClient = subxt::OnlineClient<Config>;
pub type Client<T> = subxt::OnlineClient<T>;

#[derive(Clone)]
pub struct ChainApi(RpcClient);
pub type ParachainApi = ChainApi;
pub type RelaychainApi = ChainApi;

impl Deref for ParachainApi {
    type Target = RpcClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub use subxt;
pub type Index = <Config as subxt::Config>::Index;
pub type BlockNumber = <Config as subxt::Config>::BlockNumber;
pub type Hash = <Config as subxt::Config>::Hash;
pub type Hashing = <Config as subxt::Config>::Hashing;
pub type AccountId = <Config as subxt::Config>::AccountId;
pub type Address = <Config as subxt::Config>::Address;
pub type Header = <Config as subxt::Config>::Header;
pub type Signature = <Config as subxt::Config>::Signature;
pub type Extrinsic = <Config as subxt::Config>::ExtrinsicParams;

use jsonrpsee::{
    async_client::ClientBuilder,
    client_transport::ws::{Uri, WsTransportClientBuilder},
};

pub async fn connect(uri: &str) -> Result<ChainApi> {
    let rpc_client = ws_client(uri).await?;
    let client = Client::from_rpc_client(Arc::new(rpc_client))
        .await
        .context("Failed to connect to substrate")?;
    let update_client = client.subscribe_to_updates();
    tokio::spawn(async move {
        let result = update_client.perform_runtime_updates().await;
        eprintln!("Runtime update failed with result={result:?}");
    });
    Ok(ChainApi(client))
}

async fn ws_client(url: &str) -> Result<jsonrpsee::async_client::Client> {
    let url: Uri = url.parse().context("Invalid websocket url")?;
    let (sender, receiver) = WsTransportClientBuilder::default()
        .max_request_body_size(u32::MAX)
        .build(url)
        .await
        .context("Failed to build ws transport")?;
    Ok(ClientBuilder::default().build_with_tokio(sender, receiver))
}
