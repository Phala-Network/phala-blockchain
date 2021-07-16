// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! # Bridge Module
//!
//! This will eventually have some useful documentation.
//! For now though, enjoy this cow's wisdom.
//!
//!```ignore
//!________________________________________
//! / You are only young once, but you can  \
//! \ stay immature indefinitely.           /
//! ----------------------------------------
//!        \   ^__^
//!         \  (oo)\_______
//!            (__)\       )\/\
//!                ||----w |
//!                ||     ||
//!```

// // Ensure we're `no_std` when compiling for Wasm.
// #![cfg_attr(not(feature = "std"), no_std)]

mod error;
mod justification;
pub mod storage_proof;
mod types;

use crate::std::collections::BTreeMap;
use crate::std::fmt;
use crate::std::vec::Vec;

use anyhow::Result;
use error::JustificationError;
use justification::GrandpaJustification;
use log::{error, info};
use storage_proof::{StorageProof, StorageProofChecker};

use finality_grandpa::voter_set::VoterSet;
use num::AsPrimitive;
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_finality_grandpa::{AuthorityId, AuthorityList, AuthorityWeight};
use sp_runtime::traits::{Block as BlockT, Header, NumberFor};
use sp_runtime::EncodedJustification;

pub use types::{AuthoritySet, AuthoritySetChange};

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct BridgeInitInfo<T: Config> {
    pub block_header: T::Header,
    pub validator_set: AuthorityList,
    pub validator_set_proof: StorageProof,
}

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct BridgeInfo<T: Config> {
    last_finalized_block_header: T::Header,
    current_set: AuthoritySet,
}

impl<T: Config> BridgeInfo<T> {
    pub fn new(block_header: T::Header, validator_set: AuthorityList) -> Self {
        BridgeInfo {
            last_finalized_block_header: block_header,
            current_set: AuthoritySet {
                authority_set: validator_set,
                set_id: 0,
            },
        }
    }
}

type BridgeId = u64;

pub trait Config: frame_system::Config<Hash = H256> {
    type Block: BlockT<Hash = H256, Header = Self::Header>;
}

impl Config for chain::Runtime {
    type Block = chain::Block;
}

#[derive(Encode, Decode, Clone)]
pub struct LightValidation<T: Config> {
    num_bridges: BridgeId,
    tracked_bridges: BTreeMap<BridgeId, BridgeInfo<T>>,
}

impl<T: Config> LightValidation<T>
where
    NumberFor<T::Block>: AsPrimitive<usize>,
{
    pub fn new() -> Self {
        LightValidation {
            num_bridges: 0,
            tracked_bridges: BTreeMap::new(),
        }
    }

    pub fn initialize_bridge(
        &mut self,
        block_header: T::Header,
        validator_set: AuthorityList,
        validator_set_proof: StorageProof,
    ) -> Result<BridgeId> {
        let state_root = block_header.state_root();

        Self::check_validator_set_proof(state_root, validator_set_proof, &validator_set)
            .map_err(anyhow::Error::msg)?;

        let bridge_info = BridgeInfo::new(block_header, validator_set);

        let new_bridge_id = self.num_bridges + 1;
        self.tracked_bridges.insert(new_bridge_id, bridge_info);

        self.num_bridges = new_bridge_id;

        Ok(new_bridge_id)
    }

    /// Submits a sequence of block headers to the light client to validate
    ///
    /// The light client accepts a sequence of block headers, optionally with an authority set change
    /// in the last block. Without the authority set change, it assumes the authority set and the set
    /// id remains the same after submitting the blocks. One submission can have at most one authortiy
    /// set change (change.set_id == last_set_id + 1).
    pub fn submit_finalized_headers(
        &mut self,
        bridge_id: BridgeId,
        header: T::Header,
        ancestry_proof: Vec<T::Header>,
        grandpa_proof: EncodedJustification,
        auhtority_set_change: Option<AuthoritySetChange>,
    ) -> Result<()> {
        let bridge = self
            .tracked_bridges
            .get(&bridge_id)
            .ok_or_else(|| anyhow::Error::msg(Error::NoSuchBridgeExists))?;

        // Check that the new header is a decendent of the old header
        let last_header = &bridge.last_finalized_block_header;
        verify_ancestry(ancestry_proof, last_header.hash(), &header)?;

        let block_hash = header.hash();
        let block_num = *header.number();

        // Check that the header has been finalized
        let voters = &bridge.current_set;
        let voter_set = VoterSet::new(voters.authority_set.clone()).unwrap();
        let voter_set_id = voters.set_id;
        verify_grandpa_proof::<T::Block>(
            grandpa_proof,
            block_hash,
            block_num,
            voter_set_id,
            &voter_set,
        )?;

        match self.tracked_bridges.get_mut(&bridge_id) {
            Some(bridge_info) => {
                bridge_info.last_finalized_block_header = header;
                if let Some(change) = auhtority_set_change {
                    // Check the validator set increment
                    if change.authority_set.set_id != voter_set_id + 1 {
                        return Err(anyhow::Error::msg(Error::UnexpectedValidatorSetId));
                    }
                    // Check validator set change proof
                    let state_root = bridge_info.last_finalized_block_header.state_root();
                    Self::check_validator_set_proof(
                        state_root,
                        change.authority_proof,
                        &change.authority_set.authority_set,
                    )?;
                    // Commit
                    bridge_info.current_set = AuthoritySet {
                        authority_set: change.authority_set.authority_set,
                        set_id: change.authority_set.set_id,
                    }
                }
            }
            _ => panic!("We succesfully got this bridge earlier, therefore it exists; qed"),
        };

        Ok(())
    }

    pub fn submit_simple_header(
        &mut self,
        bridge_id: BridgeId,
        header: T::Header,
        grandpa_proof: EncodedJustification,
    ) -> Result<()> {
        let bridge = self
            .tracked_bridges
            .get(&bridge_id)
            .ok_or_else(|| anyhow::Error::msg(Error::NoSuchBridgeExists))?;
        if bridge.last_finalized_block_header.hash() != *header.parent_hash() {
            return Err(anyhow::Error::msg(Error::HeaderAncestryMismatch));
        }
        let ancestry_proof = vec![];
        self.submit_finalized_headers(bridge_id, header, ancestry_proof, grandpa_proof, None)
    }

    pub fn validate_storage_proof(
        &self,
        state_root: T::Hash,
        proof: StorageProof,
        items: &[(&[u8], &[u8])], // &[(key, value)]
    ) -> Result<()> {
        let checker = StorageProofChecker::<T::Hashing>::new(state_root, proof)?;
        for (k, v) in items {
            let actual_value = checker
                .read_value(k)?
                .ok_or_else(|| anyhow::Error::msg(Error::StorageValueUnavailable))?;
            if actual_value.as_slice() != *v {
                return Err(anyhow::Error::msg(Error::StorageValueMismatch));
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    // InvalidStorageProof,
    StorageRootMismatch,
    StorageValueUnavailable,
    // InvalidValidatorSetProof,
    ValidatorSetMismatch,
    InvalidAncestryProof,
    NoSuchBridgeExists,
    InvalidFinalityProof,
    // UnknownClientError,
    HeaderAncestryMismatch,
    UnexpectedValidatorSetId,
    StorageValueMismatch,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::StorageRootMismatch => write!(f, "storage root mismatch"),
            Error::StorageValueUnavailable => write!(f, "storage value unavailable"),
            Error::ValidatorSetMismatch => write!(f, "validator set mismatch"),
            Error::InvalidAncestryProof => write!(f, "invalid ancestry proof"),
            Error::NoSuchBridgeExists => write!(f, "no such bridge exists"),
            Error::InvalidFinalityProof => write!(f, "invalid finality proof"),
            Error::HeaderAncestryMismatch => write!(f, "header ancestry mismatch"),
            Error::UnexpectedValidatorSetId => write!(f, "unexpected validator set id"),
            Error::StorageValueMismatch => write!(f, "storage value mismatch"),
        }
    }
}

impl From<JustificationError> for Error {
    fn from(e: JustificationError) -> Self {
        match e {
            JustificationError::BadJustification(msg) => {
                error!("InvalidFinalityProof(BadJustification({}))", msg);
                Error::InvalidFinalityProof
            }
            JustificationError::JustificationDecode => {
                error!("InvalidFinalityProof(JustificationDecode)");
                Error::InvalidFinalityProof
            }
        }
    }
}

impl<T: Config> LightValidation<T>
where
    NumberFor<T::Block>: AsPrimitive<usize>,
{
    fn check_validator_set_proof(
        state_root: &T::Hash,
        proof: StorageProof,
        validator_set: &Vec<(AuthorityId, AuthorityWeight)>,
    ) -> Result<()> {
        let checker = <StorageProofChecker<T::Hashing>>::new(*state_root, proof.clone())?;

        // By encoding the given set we should have an easy way to compare
        // with the stuff we get out of storage via `read_value`
        let mut encoded_validator_set = validator_set.encode();
        encoded_validator_set.insert(0, 1); // Add AUTHORITIES_VERISON == 1
        let actual_validator_set = checker
            .read_value(b":grandpa_authorities")?
            .ok_or_else(|| anyhow::Error::msg(Error::StorageValueUnavailable))?;

        if encoded_validator_set == actual_validator_set {
            Ok(())
        } else {
            Err(anyhow::Error::msg(Error::ValidatorSetMismatch))
        }
    }
}

// A naive way to check whether a `child` header is a decendent
// of an `ancestor` header. For this it requires a proof which
// is a chain of headers between (but not including) the `child`
// and `ancestor`. This could be updated to use something like
// Log2 Ancestors (#2053) in the future.
fn verify_ancestry<H>(proof: Vec<H>, ancestor_hash: H::Hash, child: &H) -> Result<()>
where
    H: Header<Hash = H256>,
{
    {
        info!("ancestor_hash: {}", ancestor_hash);
        for h in proof.iter() {
            info!(
                "block {:?} - hash: {} parent: {}",
                h.number(),
                h.hash(),
                h.parent_hash()
            );
        }
        info!(
            "child block {:?} - hash: {} parent: {}",
            child.number(),
            child.hash(),
            child.parent_hash()
        );
    }

    let mut parent_hash = child.parent_hash();
    if *parent_hash == ancestor_hash {
        return Ok(());
    }

    // If we find that the header's parent hash matches our ancestor's hash we're done
    for header in proof.iter() {
        // Need to check that blocks are actually related
        if header.hash() != *parent_hash {
            break;
        }

        parent_hash = header.parent_hash();
        if *parent_hash == ancestor_hash {
            return Ok(());
        }
    }

    Err(anyhow::Error::msg(Error::InvalidAncestryProof))
}

fn verify_grandpa_proof<B>(
    justification: EncodedJustification,
    hash: B::Hash,
    number: NumberFor<B>,
    set_id: u64,
    voters: &VoterSet<AuthorityId>,
) -> Result<()>
where
    B: BlockT<Hash = H256>,
    NumberFor<B>: finality_grandpa::BlockNumberOps,
{
    // We don't really care about the justification, as long as it's valid
    let _ = GrandpaJustification::<B>::decode_and_verify_finalizes(
        &justification,
        (hash, number),
        set_id,
        voters,
    )
    .map_err(anyhow::Error::msg)?;

    Ok(())
}

impl<T: Config> fmt::Debug for LightValidation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LightValidationTest {{ num_bridges: {}, tracked_bridges: {:?} }}",
            self.num_bridges, self.tracked_bridges
        )
    }
}

impl<T: Config> fmt::Debug for BridgeInfo<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BridgeInfo {{ last_finalized_block_header: {:?}, current_validator_set: {:?}, current_validator_set_id: {} }}",
			self.last_finalized_block_header, self.current_set.authority_set, self.current_set.set_id)
    }
}

impl<T: Config> fmt::Debug for BridgeInitInfo<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BridgeInfo {{ block_header: {:?}, validator_set: {:?}, validator_set_proof: {:?} }}",
            self.block_header, self.validator_set, self.validator_set_proof
        )
    }
}

pub mod utils {
    use crate::std::vec::Vec;
    use anyhow::{Context, Result};
    use parity_scale_codec::Encode;

    /// Gets the prefix of a storage item
    pub fn storage_prefix(module: &str, storage: &str) -> Vec<u8> {
        let mut bytes = sp_core::twox_128(module.as_bytes()).to_vec();
        bytes.extend(&sp_core::twox_128(storage.as_bytes())[..]);
        bytes
    }

    /// Calculates the Substrate storage key prefix for a StorageMap
    pub fn storage_map_prefix(module: &str, storage_item: &str, item_key: &[u8]) -> Vec<u8> {
        let mut bytes = storage_prefix(module, storage_item);
        let hash = sp_core::twox_64(&item_key);
        bytes.extend(&hash);
        bytes.extend(item_key);
        bytes
    }

    /// Calculates the Substrate storage key prefix for a StorageMap
    pub fn storage_map_prefix_twox_64_concat(module: &[u8], storage_item: &[u8], key: &impl Encode) -> Vec<u8> {
        let mut bytes = sp_core::twox_128(module).to_vec();
        bytes.extend(&sp_core::twox_128(storage_item)[..]);
        let encoded = key.encode();
        bytes.extend(&sp_core::twox_64(&encoded));
        bytes.extend(&encoded);
        bytes
    }

    /// Calculates the Substrate storage key prefix for a StorageMap
    pub fn storage_map_prefix_blake2_128_concat(module: &[u8], storage_item: &[u8], key: &impl Encode) -> Vec<u8> {
        let mut bytes = sp_core::twox_128(module).to_vec();
        bytes.extend(&sp_core::twox_128(storage_item)[..]);
        let encoded = key.encode();
        bytes.extend(&sp_core::blake2_128(&encoded));
        bytes.extend(&encoded);
        bytes
    }

    /// Gets the last 32 bytes as the account key (`storage_key` must be longer than that)
    pub fn extract_account_id_key_unsafe<'a>(storage_key: &'a [u8]) -> &'a [u8] {
        if storage_key.len() < 32 {
            panic!("storage_key is too short (len={})", storage_key.len());
        }
        &storage_key[storage_key.len() - 32..]
    }
}
