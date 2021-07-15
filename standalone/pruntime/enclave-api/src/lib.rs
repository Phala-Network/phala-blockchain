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
        pub headers: Vec<HeaderToSync>,
        pub authority_set_change: Option<AuthoritySetChange>,
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct DispatchBlockReq {
        pub blocks: Vec<BlockHeaderWithEvents>,
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
}

pub mod block_feeders {
    use super::blocks::{AuthoritySetChange, BlockHeaderWithEvents, HeaderToSync};

    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use chain::Hash;
    use derive_more::Display;
    use parity_scale_codec::{Decode, Encode};

    type RuntimeHasher = <chain::Runtime as frame_system::Config>::Hashing;
    type Storage = trie_storage::TrieStorage<RuntimeHasher>;
    type Result<T> = core::result::Result<T, Error>;

    #[derive(Display)]
    pub enum Error {
        EmptyRequest,
        MissingJustification,
        IncorrectHeaderOrder,
        HeaderValidateFailed,
        UnexpectedHeader,
        NoRelaychainHeader,
        HeaderHashMismatch,
        BlockNumberMismatch,
        BlockHeaderNotFound,
        StateRootMismatch,
    }

    pub trait BlockValidator {
        fn submit_finalized_headers(
            &mut self,
            bridge_id: u64,
            header: chain::Header,
            ancestry_proof: Vec<chain::Header>,
            grandpa_proof: Vec<u8>,
            auhtority_set_change: Option<AuthoritySetChange>,
        ) -> Result<()>;

        fn validate_storage_proof(
            &self,
            state_root: Hash,
            proof: &[u8],
            items: &[(&[u8], &[u8])],
        ) -> Result<()>;
    }

    pub struct HeaderFeeder<Validator> {
        validator: Validator,
        main_bridge: u64,
        headernum: chain::BlockNumber,
    }

    impl<Validator> HeaderFeeder<Validator>
    where
        Validator: BlockValidator,
    {
        pub fn sync_header(
            &mut self,
            headers: Vec<HeaderToSync>,
            authority_set_change: Option<AuthoritySetChange>,
            state_roots: &mut VecDeque<Hash>,
        ) -> Result<chain::BlockNumber> {
            // Light validation when possible
            let last_header = headers.last().ok_or_else(|| Error::EmptyRequest)?;

            {
                // 1. the last header must has justification
                let justification = last_header
                    .justification
                    .as_ref()
                    .ok_or_else(|| Error::MissingJustification)?
                    .clone();
                let last_header = last_header.header.clone();
                // 2. check header sequence
                for (i, header) in headers.iter().enumerate() {
                    if i > 0 && headers[i - 1].header.hash() != header.header.parent_hash {
                        return Err(Error::IncorrectHeaderOrder);
                    }
                }
                // 3. generate accenstor proof
                let mut accenstor_proof: Vec<_> = headers[0..headers.len() - 1]
                    .iter()
                    .map(|h| h.header.clone())
                    .collect();
                accenstor_proof.reverse(); // from high to low
                                           // 4. submit to light client
                let bridge_id = self.main_bridge;
                self.validator.submit_finalized_headers(
                    bridge_id,
                    last_header,
                    accenstor_proof,
                    justification,
                    authority_set_change,
                )?;
            }

            let mut last_header = 0;
            for header_with_events in headers.iter() {
                let header = &header_with_events.header;
                if header.number != self.headernum {
                    return Err(Error::UnexpectedHeader);
                }

                // move forward
                last_header = header.number;
                self.headernum = last_header + 1;
            }

            // Save the block hashes for future dispatch
            for header in headers.iter() {
                state_roots.push_back(header.header.state_root);
            }

            Ok(last_header)
        }
    }

    pub struct BlockFeeder {
        state_roots: VecDeque<Hash>,
        next_block_number: chain::BlockNumber,
    }

    impl BlockFeeder {

        /// Feed a block and apply changes to storage if it's valid.
        fn feed_block(
            &mut self,
            block: &BlockHeaderWithEvents,
            storage: &mut Storage,
        ) -> Result<()> {
            if block.block_header.number != self.next_block_number {
                return Err(Error::BlockNumberMismatch);
            }

            let expected_root = self.state_roots.get(0).ok_or(Error::BlockHeaderNotFound)?;

            let changes = &block.storage_changes;

            let (state_root, transaction) = storage.calc_root_if_changes(
                &changes.main_storage_changes,
                &changes.child_storage_changes,
            );

            if expected_root != &state_root {
                return Err(Error::StateRootMismatch);
            }

            storage.apply_changes(state_root, transaction);

            self.next_block_number += 1;
            self.state_roots.pop_front();
            Ok(())
        }
    }

    struct SolochainFeeder<Validator> {
        header_feeder: HeaderFeeder<Validator>,
        block_feeder: BlockFeeder,
    }

    impl<Validator: BlockValidator> SolochainFeeder<Validator> {
        pub fn sync_header(
            &mut self,
            headers: Vec<HeaderToSync>,
            authority_set_change: Option<AuthoritySetChange>,
        ) -> Result<chain::BlockNumber> {
            self.header_feeder.sync_header(
                headers,
                authority_set_change,
                &mut self.block_feeder.state_roots,
            )
        }

        pub fn feed_block(
            &mut self,
            block: &BlockHeaderWithEvents,
            storage: &mut Storage,
        ) -> Result<()> {
            self.block_feeder.feed_block(block, storage)
        }
    }

    struct ParachainFeeder<Validator> {
        relaychain_header_feeder: HeaderFeeder<Validator>,
        last_relaychain_state_root: Option<Hash>,
        block_feeder: BlockFeeder,
    }

    impl<Validator: BlockValidator> ParachainFeeder<Validator> {
        pub fn sync_relaychain_header(
            &mut self,
            headers: Vec<HeaderToSync>,
            authority_set_change: Option<AuthoritySetChange>,
        ) -> Result<chain::BlockNumber> {
            let mut state_roots = Default::default();
            let last_header = self.relaychain_header_feeder.sync_header(
                headers,
                authority_set_change,
                &mut state_roots,
            )?;
            self.last_relaychain_state_root = state_roots.pop_back();
            Ok(last_header)
        }

        pub fn sync_parachain_header(
            &mut self,
            headers: Vec<chain::Header>,
            proof: &[u8],
            storage_key: &[u8],
        ) -> Result<()> {
            let last_raw_header = if let Some(hdr) = headers.last() {
                hdr.encode()
            } else {
                return Err(Error::EmptyRequest);
            };

            let state_root = if let Some(root) = &self.last_relaychain_state_root {
                root.clone()
            } else {
                return Err(Error::NoRelaychainHeader);
            };

            self.relaychain_header_feeder
                .validator
                .validate_storage_proof(
                    state_root,
                    proof,
                    &[(storage_key, last_raw_header.as_slice())],
                )?;

            check_headers_hash(&headers)?;

            // All checks passed, enqueue the state roots for storage validation.
            for hdr in headers.iter().rev() {
                self.block_feeder.state_roots.push_back(hdr.state_root);
            }

            self.last_relaychain_state_root = None;

            Ok(())
        }

        pub fn feed_block(
            &mut self,
            block: &BlockHeaderWithEvents,
            storage: &mut Storage,
        ) -> Result<()> {
            self.block_feeder.feed_block(block, storage)
        }
    }

    fn check_headers_hash(headers: &[chain::Header]) -> Result<()> {
        let mut expected_hash = headers.last().ok_or(Error::EmptyRequest)?.hash();

        for hdr in headers.iter().rev() {
            if hdr.hash() != expected_hash {
                return Err(Error::HeaderHashMismatch);
            }
            expected_hash = hdr.parent_hash;
        }
        Ok(())
    }
}
