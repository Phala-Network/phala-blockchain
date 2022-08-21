use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

#[subxt::subxt(runtime_metadata_path = "metadata_files/khala_metadata.scale")]
pub mod khala {
    #[subxt(substitute_type = "phala_mq::types::SignedMessage")]
    pub use ::phala_types::messaging::SignedMessage;

    #[subxt(substitute_type = "polkadot_parachain::primitives::Id")]
    pub use crate::ParaId;
}
#[subxt::subxt(runtime_metadata_path = "metadata_files/kusama_metadata.scale")]
pub mod kusama {}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, PartialOrd, Ord, Debug)]
pub struct ParaId(pub u32);

pub mod rpc;

pub type ParachainApi = khala::RuntimeApi<DefaultConfig, ExtrinsicParams>;
pub type RelaychainApi = kusama::RuntimeApi<DefaultConfig, ExtrinsicParams>;
pub type ExtrinsicParams = DefaultExtrinsicParams<DefaultConfig>;
pub type ExtrinsicParamsBuilder = DefaultExtrinsicParamsBuilder<DefaultConfig>;
pub use subxt::DefaultConfig as Config;
pub type RpcClient = subxt::Client<Config>;

pub use subxt;
pub use subxt::sp_core::storage::{StorageData, StorageKey};

use subxt::{
    extrinsic::{
        PolkadotExtrinsicParams as DefaultExtrinsicParams,
        PolkadotExtrinsicParamsBuilder as DefaultExtrinsicParamsBuilder,
    },
    DefaultConfig,
};

pub fn storage_key<T: subxt::StorageEntry>(entry: T) -> StorageKey {
    let prefix = subxt::storage::StorageKeyPrefix::new::<T>();
    entry.key().final_key(prefix)
}

pub type Index = <DefaultConfig as subxt::Config>::Index;
pub type BlockNumber = <DefaultConfig as subxt::Config>::BlockNumber;
pub type Hash = <DefaultConfig as subxt::Config>::Hash;
pub type Hashing = <DefaultConfig as subxt::Config>::Hashing;
pub type AccountId = <DefaultConfig as subxt::Config>::AccountId;
pub type Address = <DefaultConfig as subxt::Config>::Address;
pub type Header = <DefaultConfig as subxt::Config>::Header;
pub type Signature = <DefaultConfig as subxt::Config>::Signature;
pub type Extrinsic = <DefaultConfig as subxt::Config>::Extrinsic;
