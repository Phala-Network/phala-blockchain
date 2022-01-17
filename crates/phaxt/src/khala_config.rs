use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

use subxt::DefaultConfig;

pub type Index = <DefaultConfig as subxt::Config>::Index;
pub type BlockNumber = <DefaultConfig as subxt::Config>::BlockNumber;
pub type Hash = <DefaultConfig as subxt::Config>::Hash;
pub type Hashing = <DefaultConfig as subxt::Config>::Hashing;
pub type AccountId = <DefaultConfig as subxt::Config>::AccountId;
pub type Address = <DefaultConfig as subxt::Config>::Address;
pub type Header = <DefaultConfig as subxt::Config>::Header;
pub type Signature = <DefaultConfig as subxt::Config>::Signature;
pub type Extrinsic = <DefaultConfig as subxt::Config>::Extrinsic;

#[derive(Default, Clone, TypeInfo, Encode, Decode, PartialEq, Eq, Debug)]
pub struct KhalaConfig;

impl subxt::Config for KhalaConfig {
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = Hashing;
    type AccountId = AccountId;
    type Address = Address;
    type Header = Header;
    type Signature = Signature;
    type Extrinsic = Extrinsic;
}

impl subxt::AccountData<KhalaConfig> for crate::khala::system::storage::Account {
    fn nonce(
        result: &<Self as subxt::StorageEntry>::Value,
    ) -> <KhalaConfig as subxt::Config>::Index {
        result.nonce
    }
    fn storage_entry(account_id: <KhalaConfig as ::subxt::Config>::AccountId) -> Self {
        Self(account_id)
    }
}
