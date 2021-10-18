use alloc::vec::Vec;
use core::convert::TryFrom;
use parity_scale_codec::{Decode, Encode, FullCodec};
pub use sp_finality_grandpa::{AuthorityList, SetId};

use sp_core::U256;
use sp_runtime::{generic::Header, traits::Hash as HashT};
pub use phala_trie_storage::ser::StorageChanges;

pub type StorageProof = Vec<Vec<u8>>;
pub type StorageState = Vec<(Vec<u8>, Vec<u8>)>;

/// The GRNADPA authority set with the id
#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub struct AuthoritySet {
    pub list: AuthorityList,
    pub id: SetId,
}

/// AuthoritySet change with the storage proof (including both the authority set and the id)
#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub struct AuthoritySetChange {
    pub authority_set: AuthoritySet,
    pub authority_proof: StorageProof,
}

/// The genesis block initialization info.
///
/// The genesis block is the first block to start GRNADPA light validation tracking. It could
/// be block 0 or a later block on the relay chain. The authority set represents the validator
/// infomation at the selected block.
#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub struct GenesisBlockInfo {
    pub block_header: chain::Header,
    pub authority_set: AuthoritySet,
    pub proof: StorageProof,
}

pub type RuntimeHasher = <chain::Runtime as frame_system::Config>::Hashing;
pub type HeaderToSync = GenericHeaderToSync<chain::BlockNumber, RuntimeHasher>;
pub type BlockHeaderWithChanges =
    GenericBlockHeaderWithChanges<chain::BlockNumber, RuntimeHasher>;
pub type Headers = Vec<Header<chain::BlockNumber, RuntimeHasher>>;
pub type HeadersToSync = Vec<HeaderToSync>;

pub type RawStorageKey = Vec<u8>;

#[derive(Debug, Encode, Decode, Clone)]
pub struct StorageKV<T: FullCodec + Clone>(pub RawStorageKey, pub T);

impl<T: FullCodec + Clone> StorageKV<T> {
    pub fn key(&self) -> &RawStorageKey {
        &self.0
    }
    pub fn value(&self) -> &T {
        &self.1
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct GenericHeaderToSync<BlockNumber, Hash>
where
    BlockNumber: Copy + Into<U256> + TryFrom<U256> + Clone,
    Hash: HashT,
{
    pub header: Header<BlockNumber, Hash>,
    pub justification: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct GenericBlockHeaderWithChanges<BlockNumber, Hash>
where
    BlockNumber: Copy + Into<U256> + TryFrom<U256> + FullCodec + Clone,
    Hash: HashT,
{
    pub block_header: Header<BlockNumber, Hash>,
    pub storage_changes: StorageChanges,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct SyncHeaderReq {
    pub headers: Vec<HeaderToSync>,
    pub authority_set_change: Option<AuthoritySetChange>,
}

// TODO.kevin: import it from some other crate
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct ParaId(u32);

impl ParaId {
    pub fn new(n: u32) -> ParaId {
        ParaId(n)
    }
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct SyncParachainHeaderReq {
    pub headers: Headers,
    pub proof: StorageProof,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct SyncCombinedHeadersReq {
    pub relaychain_headers: Vec<HeaderToSync>,
    pub authority_set_change: Option<AuthoritySetChange>,
    pub parachain_headers: Headers,
    pub proof: StorageProof,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct DispatchBlockReq {
    pub blocks: Vec<BlockHeaderWithChanges>,
}

#[cfg(feature = "serde")]
pub mod compat {
    use alloc::string::String;
    use alloc::vec::Vec;
    use parity_scale_codec::Encode;
    use serde::Serialize;

    #[derive(Serialize, Debug)]
    pub struct SyncHeaderReq {
        pub headers_b64: Vec<String>,
        pub authority_set_change_b64: Option<String>,
    }

    impl From<super::SyncHeaderReq> for SyncHeaderReq {
        fn from(v: super::SyncHeaderReq) -> Self {
            let headers_b64: Vec<_> = v
                .headers
                .into_iter()
                .map(|x| base64::encode(x.encode()))
                .collect();
            let authority_set_change_b64 =
                v.authority_set_change.map(|x| base64::encode(x.encode()));
            Self {
                headers_b64,
                authority_set_change_b64,
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub struct DispatchBlockReq {
        pub blocks_b64: Vec<String>,
    }

    impl From<super::DispatchBlockReq> for DispatchBlockReq {
        fn from(v: super::DispatchBlockReq) -> Self {
            let blocks_b64: Vec<_> = v
                .blocks
                .into_iter()
                .map(|x| base64::encode(x.encode()))
                .collect();
            Self { blocks_b64 }
        }
    }

    #[derive(Serialize, Debug)]
    pub struct ContractInput<T> {
        pub input: T,
    }

    impl<T> ContractInput<T> {
        pub fn new(input: T) -> Self {
            Self { input }
        }
    }
}
