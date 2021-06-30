#![no_std]
extern crate alloc;

pub mod actions {
    pub const ACTION_TEST: u8 = 0;
    pub const ACTION_INIT_RUNTIME: u8 = 1;
    pub const ACTION_GET_INFO: u8 = 2;
    pub const ACTION_DUMP_STATES: u8 = 3;
    pub const ACTION_LOAD_STATES: u8 = 4;
    pub const ACTION_SYNC_HEADER: u8 = 5;
    pub const ACTION_QUERY: u8 = 6;
    pub const ACTION_DISPATCH_BLOCK: u8 = 7;
    // Reserved: 8, 9
    pub const ACTION_GET_RUNTIME_INFO: u8 = 10;
    pub const ACTION_GET_EGRESS_MESSAGES: u8 = 23;
    pub const ACTION_TEST_INK: u8 = 100;
}

pub mod blocks {
    use alloc::vec::Vec;
    use core::convert::TryFrom;
    use parity_scale_codec::{Decode, Encode, FullCodec};
    use sp_finality_grandpa::{AuthorityList, SetId};

    use sp_core::U256;
    use sp_runtime::{generic::Header, traits::Hash as HashT};
    use trie_storage::ser::StorageChanges;

    pub type StorageProof = Vec<Vec<u8>>;

    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub struct AuthoritySet {
        pub authority_set: AuthorityList,
        pub set_id: SetId,
    }

    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub struct AuthoritySetChange {
        pub authority_set: AuthoritySet,
        pub authority_proof: StorageProof,
    }

    pub type HeaderToSync =
        GenericHeaderToSync<chain::BlockNumber, <chain::Runtime as frame_system::Config>::Hashing>;
    pub type BlockHeaderWithEvents = GenericBlockHeaderWithEvents<
        chain::BlockNumber,
        <chain::Runtime as frame_system::Config>::Hashing,
    >;

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
    pub struct GenericBlockHeaderWithEvents<BlockNumber, Hash>
    where
        BlockNumber: Copy + Into<U256> + TryFrom<U256> + FullCodec + Clone,
        Hash: HashT,
    {
        pub block_header: Header<BlockNumber, Hash>,
        pub storage_changes: StorageChanges,
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct SyncHeaderReq {
        pub headers_b64: Vec<HeaderToSync>,
        pub authority_set_change_b64: Option<AuthoritySetChange>,
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct DispatchBlockReq {
        pub blocks_b64: Vec<BlockHeaderWithEvents>,
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
                    .headers_b64
                    .into_iter()
                    .map(|x| base64::encode(x.encode()))
                    .collect();
                let authority_set_change_b64 = v
                    .authority_set_change_b64
                    .map(|x| base64::encode(x.encode()));
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
                    .blocks_b64
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
}
