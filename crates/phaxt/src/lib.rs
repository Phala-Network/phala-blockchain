use anyhow::{Context, Result};
use std::ops::Deref;

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

mod chain_api;
pub mod dynamic;
pub mod rpc;

#[subxt::subxt(runtime_metadata_path = "metadata_files/khala_metadata.scale")]
pub mod parachain {
    #[subxt(substitute_type = "phala_mq::types::SignedMessage")]
    pub use ::phala_types::messaging::SignedMessage;

    #[subxt(substitute_type = "polkadot_parachain::primitives::Id")]
    pub use crate::ParaId;
}
// #[subxt::subxt(runtime_metadata_path = "metadata_files/kusama_metadata.scale")]
// pub mod relaychain {}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, PartialOrd, Ord, Debug)]
pub struct ParaId(pub u32);

pub type StorageProof = Vec<Vec<u8>>;
pub type StorageState = Vec<(Vec<u8>, Vec<u8>)>;
pub type PairSigner = subxt::tx::PairSigner<Config, sr25519::Pair>;

use subxt::{
    ext::sp_core::sr25519,
    tx::{PolkadotExtrinsicParams, PolkadotExtrinsicParamsBuilder},
};
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
pub type Extrinsic = <Config as subxt::Config>::Extrinsic;

pub async fn connect(uri: &str) -> Result<ChainApi> {
    let client = Client::from_url(uri)
        .await
        .context("Failed to connect to substrate")?;
    Ok(ChainApi(client))
}
