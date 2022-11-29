use super::blocks::{
    AuthoritySetChange, BlockHeaderWithChanges, HeaderToSync, RuntimeHasher, StorageProof,
};

use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use chain::Hash;
use derive_more::Display;
use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};

type Storage = phala_trie_storage::TrieStorage<RuntimeHasher>;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Display, Debug)]
pub enum Error {
    /// No header or block data in the request
    EmptyRequest,
    /// No Justification found in the last header
    MissingJustification,
    /// Header validation failed
    #[display(fmt = "HeaderValidateFailed({_0})")]
    HeaderValidateFailed(String),
    /// Storage proof failed
    #[display(fmt = "StorageProofFailed({_0})")]
    StorageProofFailed(String),
    /// Relay chain header not synced before syncing parachain header
    RelaychainHeaderNotSynced,
    /// Some header parent hash mismatches it's parent header
    HeaderHashMismatch,
    /// The end block number mismatch the expecting next block number
    BlockNumberMismatch,
    /// No state root to validate the storage changes
    NoStateRoot,
    /// Invalid storage changes that cause the state root mismatch
    #[display(
        fmt = "StateRootMismatch block={:?} expected={:?} actual={:?}",
        block,
        expected,
        actual
    )]
    StateRootMismatch {
        block: chain::BlockNumber,
        expected: chain::Hash,
        actual: chain::Hash,
    },
    /// Solo/Para mode mismatch
    ChainModeMismatch,
    CanNotLoadStateAfterSynced,
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
        proof: StorageProof,
        items: &[(&[u8], &[u8])],
    ) -> Result<()>;
}

pub trait StorageSynchronizer {
    /// Return the next block numbers to sync.
    fn counters(&self) -> Counters;

    /// Given chain headers in sequence, validate it and output the state_roots
    fn sync_header(
        &mut self,
        headers: Vec<HeaderToSync>,
        authority_set_change: Option<AuthoritySetChange>,
    ) -> Result<chain::BlockNumber>;

    /// Given the parachain headers in sequence, validate it and cached the state_roots for block validation
    fn sync_parachain_header(
        &mut self,
        headers: Vec<chain::Header>,
        proof: StorageProof,
        storage_key: &[u8],
    ) -> Result<chain::BlockNumber>;

    /// Feed in a block of storage changes
    fn feed_block(&mut self, block: &BlockHeaderWithChanges, storage: &mut Storage) -> Result<()>;

    /// Assume synced to given block.
    fn assume_at_block(&mut self, block_number: chain::BlockNumber) -> Result<()>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockSyncState<Validator> {
    validator: Validator,
    main_bridge: u64,
    /// The next block number of header to be sync.
    header_number_next: chain::BlockNumber,
    /// The next block number of storage to be sync.
    /// Note: in parachain mode the `block_number_next` is for the parachain and the
    /// header_number_next is for the relaychain.
    block_number_next: chain::BlockNumber,
    genesis_state_validated: bool,
}

impl<Validator> BlockSyncState<Validator>
where
    Validator: BlockValidator,
{
    pub fn new(
        validator: Validator,
        main_bridge: u64,
        header_number_next: chain::BlockNumber,
        block_number_next: chain::BlockNumber,
    ) -> Self {
        Self {
            validator,
            main_bridge,
            header_number_next,
            block_number_next,
            genesis_state_validated: true,
        }
    }

    /// Given chain headers in sequence, validate it and output the state_roots
    pub fn sync_header(
        &mut self,
        mut headers: Vec<HeaderToSync>,
        authority_set_change: Option<AuthoritySetChange>,
        state_roots: &mut VecDeque<Hash>,
        min_block: chain::BlockNumber,
    ) -> Result<chain::BlockNumber> {
        headers.retain(|header| header.header.number >= self.header_number_next);

        let first_header = match headers.first() {
            Some(header) => header,
            None => return Ok(self.header_number_next - 1),
        };
        if first_header.header.number != self.header_number_next {
            return Err(Error::BlockNumberMismatch);
        }

        // Light validation when possible
        let last_header = headers.last().ok_or(Error::EmptyRequest)?;

        {
            // 1. the last header must has justification
            let justification = last_header
                .justification
                .as_ref()
                .ok_or(Error::MissingJustification)?
                .clone();
            let last_header = last_header.header.clone();
            // 2. check header sequence
            for (i, header) in headers.iter().enumerate() {
                if i > 0 && headers[i - 1].header.hash() != header.header.parent_hash {
                    log::error!(
                        "Parent hash of {} mismatch: actual={:?} expected={:?}",
                        header.header.number,
                        headers[i - 1].header.hash(),
                        header.header.parent_hash
                    );
                    return Err(Error::HeaderHashMismatch);
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

        // Save the block hashes for future dispatch
        for header in headers.iter() {
            if header.header.number < min_block {
                continue;
            }
            state_roots.push_back(header.header.state_root);
        }

        self.header_number_next = last_header.header.number + 1;

        Ok(last_header.header.number)
    }

    /// Feed a block and apply changes to storage if it's valid.
    pub fn feed_block(
        &mut self,
        block: &BlockHeaderWithChanges,
        state_roots: &mut VecDeque<Hash>,
        storage: &mut Storage,
    ) -> Result<()> {
        if block.block_header.number != self.block_number_next {
            return Err(Error::BlockNumberMismatch);
        }

        if !self.genesis_state_validated {
            let genesis_state_root = state_roots.pop_front().ok_or(Error::NoStateRoot)?;
            if storage.root() != &genesis_state_root {
                panic!(
                    "Genesis state root mismatch, expacted: {:?}, actual: {:?}",
                    genesis_state_root,
                    storage.root()
                );
            }
            self.genesis_state_validated = true;
        }

        let expected_root = state_roots.get(0).ok_or(Error::NoStateRoot)?;

        let changes = &block.storage_changes;

        log::debug!(
            "calc root ({}, {})",
            changes.main_storage_changes.len(),
            changes.child_storage_changes.len()
        );
        let (state_root, transaction) = storage.calc_root_if_changes(
            &changes.main_storage_changes,
            &changes.child_storage_changes,
        );

        if expected_root != &state_root {
            return Err(Error::StateRootMismatch {
                block: block.block_header.number,
                expected: *expected_root,
                actual: state_root,
            });
        }

        log::debug!("apply changes");
        storage.apply_changes(state_root, transaction);
        log::debug!("applied");

        self.block_number_next += 1;
        state_roots.pop_front();
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct Counters {
    pub next_header_number: chain::BlockNumber,
    pub next_para_header_number: chain::BlockNumber,
    pub next_block_number: chain::BlockNumber,
    pub waiting_for_paraheaders: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SolochainSynchronizer<Validator> {
    sync_state: BlockSyncState<Validator>,
    state_roots: VecDeque<Hash>,
}

impl<Validator: BlockValidator> SolochainSynchronizer<Validator> {
    pub fn new(validator: Validator, main_bridge: u64) -> Self {
        Self {
            sync_state: BlockSyncState::new(validator, main_bridge, 1, 1),
            state_roots: Default::default(),
        }
    }
}

impl<Validator: BlockValidator> StorageSynchronizer for SolochainSynchronizer<Validator> {
    fn counters(&self) -> Counters {
        Counters {
            next_block_number: self.sync_state.block_number_next,
            next_header_number: self.sync_state.header_number_next,
            next_para_header_number: 0,
            waiting_for_paraheaders: false,
        }
    }

    fn sync_header(
        &mut self,
        headers: Vec<HeaderToSync>,
        authority_set_change: Option<AuthoritySetChange>,
    ) -> Result<chain::BlockNumber> {
        let min_block = self.sync_state.block_number_next.saturating_sub(1);
        self.sync_state.sync_header(
            headers,
            authority_set_change,
            &mut self.state_roots,
            min_block,
        )
    }

    fn feed_block(&mut self, block: &BlockHeaderWithChanges, storage: &mut Storage) -> Result<()> {
        self.sync_state
            .feed_block(block, &mut self.state_roots, storage)
    }

    fn sync_parachain_header(
        &mut self,
        _headers: Vec<chain::Header>,
        _proof: StorageProof,
        _storage_key: &[u8],
    ) -> Result<chain::BlockNumber> {
        Err(Error::ChainModeMismatch)
    }

    fn assume_at_block(&mut self, block_number: chain::BlockNumber) -> Result<()> {
        if self.sync_state.block_number_next > 1 || self.sync_state.header_number_next > 1 {
            return Err(Error::CanNotLoadStateAfterSynced);
        }

        self.sync_state.block_number_next = block_number + 1;
        self.sync_state.genesis_state_validated = false;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ParachainSynchronizer<Validator> {
    sync_state: BlockSyncState<Validator>,
    last_relaychain_state_root: Option<Hash>,
    para_header_number_next: chain::BlockNumber,
    para_state_roots: VecDeque<Hash>,
}

impl<Validator: BlockValidator> ParachainSynchronizer<Validator> {
    pub fn new(validator: Validator, main_bridge: u64, headernum_next: chain::BlockNumber) -> Self {
        Self {
            sync_state: BlockSyncState::new(validator, main_bridge, headernum_next, 1),
            last_relaychain_state_root: None,
            para_header_number_next: 1,
            para_state_roots: Default::default(),
        }
    }
}

impl<Validator: BlockValidator> StorageSynchronizer for ParachainSynchronizer<Validator> {
    fn counters(&self) -> Counters {
        Counters {
            next_block_number: self.sync_state.block_number_next,
            next_header_number: self.sync_state.header_number_next,
            next_para_header_number: self.para_header_number_next,
            waiting_for_paraheaders: self.last_relaychain_state_root.is_some(),
        }
    }

    /// Given the relaychain headers in sequence, validate it and cached the last state_root for parachain header validation
    fn sync_header(
        &mut self,
        headers: Vec<HeaderToSync>,
        authority_set_change: Option<AuthoritySetChange>,
    ) -> Result<chain::BlockNumber> {
        let mut state_roots = Default::default();
        let last_header =
            self.sync_state
                .sync_header(headers, authority_set_change, &mut state_roots, 0)?;
        // Don't overwrite the last state_root if given headers is empty
        if let Some(last_root) = state_roots.pop_back() {
            self.last_relaychain_state_root = Some(last_root);
        }
        Ok(last_header)
    }

    /// Given the parachain headers in sequence, validate it and cached the state_roots for block validation
    fn sync_parachain_header(
        &mut self,
        mut headers: Vec<chain::Header>,
        proof: StorageProof,
        storage_key: &[u8],
    ) -> Result<chain::BlockNumber> {
        headers.retain(|header| header.number >= self.para_header_number_next);

        let first_hdr = match headers.first() {
            Some(hdr) => hdr,
            None => return Ok(self.para_header_number_next - 1),
        };
        if self.para_header_number_next != first_hdr.number {
            return Err(Error::BlockNumberMismatch);
        }

        let last_hdr = headers.last().ok_or(Error::EmptyRequest)?;

        let state_root = self
            .last_relaychain_state_root
            .as_ref()
            .cloned()
            .ok_or(Error::RelaychainHeaderNotSynced)?;

        // 1. validate storage proof
        self.sync_state.validator.validate_storage_proof(
            state_root,
            proof,
            &[(storage_key, last_hdr.encode().encode().as_slice())],
        )?;

        // 2. check header sequence
        for (i, header) in headers.iter().enumerate() {
            if i > 0 && headers[i - 1].hash() != header.parent_hash {
                log::error!(
                    "Parent hash of {} mismatch: actual={:?} expected={:?}",
                    header.number,
                    headers[i - 1].hash(),
                    header.parent_hash
                );
                return Err(Error::HeaderHashMismatch);
            }
        }

        // All checks passed, enqueue the state roots for storage validation.
        for hdr in headers.iter() {
            self.para_state_roots.push_back(hdr.state_root);
        }

        self.last_relaychain_state_root = None;
        self.para_header_number_next = last_hdr.number + 1;

        Ok(last_hdr.number)
    }

    /// Feed in a block of storage changes
    fn feed_block(&mut self, block: &BlockHeaderWithChanges, storage: &mut Storage) -> Result<()> {
        self.sync_state
            .feed_block(block, &mut self.para_state_roots, storage)
    }

    fn assume_at_block(&mut self, block_number: chain::BlockNumber) -> Result<()> {
        if self.sync_state.block_number_next > 1 || self.para_header_number_next > 1 {
            return Err(Error::CanNotLoadStateAfterSynced);
        }
        self.sync_state.block_number_next = block_number + 1;
        self.para_header_number_next = block_number;
        self.sync_state.genesis_state_validated = false;
        Ok(())
    }
}

// We create this new type to help serialize the original dyn StorageSynchronizer.
// Because it it impossible to impl Serialize/Deserialize for dyn StorageSynchronizer.
#[derive(Serialize, Deserialize)]
pub enum Synchronizer<Validator> {
    Solo(SolochainSynchronizer<Validator>),
    Para(ParachainSynchronizer<Validator>),
}

impl<Validator: BlockValidator> Synchronizer<Validator> {
    pub fn new_parachain(
        validator: Validator,
        main_bridge: u64,
        headernum_next: chain::BlockNumber,
    ) -> Self {
        Self::Para(ParachainSynchronizer::new(
            validator,
            main_bridge,
            headernum_next,
        ))
    }

    pub fn new_solochain(validator: Validator, main_bridge: u64) -> Self {
        Self::Solo(SolochainSynchronizer::new(validator, main_bridge))
    }

    pub fn as_dyn(&self) -> &dyn StorageSynchronizer {
        match self {
            Self::Solo(s) => s,
            Self::Para(p) => p,
        }
    }

    pub fn as_dyn_mut(&mut self) -> &mut dyn StorageSynchronizer {
        match self {
            Self::Solo(s) => s,
            Self::Para(p) => p,
        }
    }
}

impl<Validator: BlockValidator> StorageSynchronizer for Synchronizer<Validator> {
    fn counters(&self) -> Counters {
        self.as_dyn().counters()
    }

    fn sync_header(
        &mut self,
        headers: Vec<HeaderToSync>,
        authority_set_change: Option<AuthoritySetChange>,
    ) -> Result<chain::BlockNumber> {
        self.as_dyn_mut().sync_header(headers, authority_set_change)
    }

    fn sync_parachain_header(
        &mut self,
        headers: Vec<chain::Header>,
        proof: StorageProof,
        storage_key: &[u8],
    ) -> Result<chain::BlockNumber> {
        self.as_dyn_mut()
            .sync_parachain_header(headers, proof, storage_key)
    }

    fn feed_block(&mut self, block: &BlockHeaderWithChanges, storage: &mut Storage) -> Result<()> {
        self.as_dyn_mut().feed_block(block, storage)
    }

    fn assume_at_block(&mut self, block_number: chain::BlockNumber) -> Result<()> {
        self.as_dyn_mut().assume_at_block(block_number)
    }
}
