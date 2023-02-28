use anyhow::{Context, Result};
use std::{ops::Deref, sync::Arc};

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use subxt::config::polkadot::{PolkadotExtrinsicParams, PolkadotExtrinsicParamsBuilder};

mod chain_api;
pub mod dynamic;
pub mod rpc;

pub use sp_core;

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, PartialOrd, Ord, Debug)]
pub struct ParaId(pub u32);

pub type StorageProof = Vec<Vec<u8>>;
pub type StorageState = Vec<(Vec<u8>, Vec<u8>)>;
pub type ExtrinsicParams = PolkadotExtrinsicParams<subxt::SubstrateConfig>;
pub type ExtrinsicParamsBuilder = PolkadotExtrinsicParamsBuilder<subxt::SubstrateConfig>;
pub use subxt::PolkadotConfig as Config;
pub type RpcClient = subxt::OnlineClient<Config>;

/// A wrapper for subxt::tx::PairSigner to make it compatible with older API.
pub struct PairSigner {
    pub signer: subxt::tx::PairSigner<Config, sp_core::sr25519::Pair>,
    nonce: Index,
}
impl PairSigner {
    pub fn new(pair: sp_core::sr25519::Pair) -> Self {
        Self {
            signer: subxt::tx::PairSigner::new(pair),
            nonce: 0,
        }
    }
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }
    pub fn nonce(&self) -> Index {
        self.nonce
    }
    pub fn set_nonce(&mut self, nonce: Index) {
        self.nonce = nonce;
    }
    pub fn account_id(&self) -> &AccountId {
        self.signer.account_id()
    }
}

#[derive(Clone)]
pub struct ChainApi(pub RpcClient);
pub type ParachainApi = ChainApi;
pub type RelaychainApi = ChainApi;

impl Deref for ChainApi {
    type Target = RpcClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub use subxt;
pub type BlockNumber = u32;
pub type Hash = primitive_types::H256;
pub type AccountId = <Config as subxt::Config>::AccountId;
pub type Index = <Config as subxt::Config>::Index;

use jsonrpsee::{
    async_client::ClientBuilder,
    client_transport::ws::{Uri, WsTransportClientBuilder},
};

pub async fn connect(uri: &str) -> Result<ChainApi> {
    let rpc_client = ws_client(uri).await?;
    let client = RpcClient::from_rpc_client(Arc::new(rpc_client))
        .await
        .context("Failed to connect to substrate")?;
    let update_client = client.updater();
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
