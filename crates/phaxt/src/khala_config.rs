use crate::khala::DefaultConfig as KhalaConfig;

pub type Index = <KhalaConfig as subxt::Config>::Index;
pub type AccountId = <KhalaConfig as subxt::Config>::AccountId;
pub type Hash = <KhalaConfig as subxt::Config>::Hash;
pub type Hashing = <KhalaConfig as subxt::Config>::Hashing;
pub type BlockNumber = <KhalaConfig as subxt::Config>::BlockNumber;
pub type Header = <KhalaConfig as subxt::Config>::Header;
