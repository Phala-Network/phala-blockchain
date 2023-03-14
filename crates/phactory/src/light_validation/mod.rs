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

use im::OrdMap as BTreeMap;
use std::fmt;

use anyhow::Result;
use error::JustificationError;
use justification::GrandpaJustification;
use log::{error, info};
use phala_serde_more as more;
use serde::{Deserialize, Serialize};
use storage_proof::{StorageProof, StorageProofChecker};

use finality_grandpa::voter_set::VoterSet;
use num::AsPrimitive;
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_finality_grandpa::{AuthorityId, AuthorityWeight, SetId};
use sp_runtime::traits::{Block as BlockT, Header, NumberFor};
use sp_runtime::EncodedJustification;

pub use types::{AuthoritySet, AuthoritySetChange};

#[derive(Encode, Decode, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeInfo<T: Config> {
    #[serde(bound(
        serialize = "T::Header: ::serde::Serialize",
        deserialize = "T::Header: ::serde::de::DeserializeOwned"
    ))]
    last_finalized_block_header: T::Header,
    #[serde(with = "more::scale_bytes")]
    current_set: AuthoritySet,
}

impl<T: Config> BridgeInfo<T> {
    pub fn new(block_header: T::Header, validator_set: AuthoritySet) -> Self {
        BridgeInfo {
            last_finalized_block_header: block_header,
            current_set: validator_set,
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

#[derive(Encode, Decode, Clone, Serialize, Deserialize)]
pub struct LightValidation<T: Config> {
    num_bridges: BridgeId,
    #[serde(bound(
        serialize = "T::Header: ::serde::Serialize",
        deserialize = "T::Header: ::serde::de::DeserializeOwned"
    ))]
    tracked_bridges: BTreeMap<BridgeId, BridgeInfo<T>>,
}

impl<T: Config> LightValidation<T>
where
    NumberFor<T::Block>: AsPrimitive<usize>,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        LightValidation {
            num_bridges: 0,
            tracked_bridges: BTreeMap::new(),
        }
    }

    pub fn initialize_bridge(
        &mut self,
        block_header: T::Header,
        validator_set: AuthoritySet,
        proof: StorageProof,
    ) -> Result<BridgeId> {
        let state_root = block_header.state_root();

        Self::check_validator_set_proof(state_root, proof, &validator_set.list, validator_set.id)
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
        let voter_set = VoterSet::new(voters.list.clone()).unwrap();
        let voter_set_id = voters.id;
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
                    if change.authority_set.id != voter_set_id + 1 {
                        return Err(anyhow::Error::msg(Error::UnexpectedValidatorSetId));
                    }
                    // Check validator set change proof
                    let state_root = bridge_info.last_finalized_block_header.state_root();
                    Self::check_validator_set_proof(
                        state_root,
                        change.authority_proof,
                        &change.authority_set.list,
                        change.authority_set.id,
                    )?;
                    // Commit
                    bridge_info.current_set = AuthoritySet {
                        list: change.authority_set.list,
                        id: change.authority_set.id,
                    }
                }
            }
            _ => panic!("We succesfully got this bridge earlier, therefore it exists; qed"),
        };

        Ok(())
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
    // StorageRootMismatch,
    StorageValueUnavailable,
    // InvalidValidatorSetProof,
    ValidatorSetMismatch,
    InvalidAncestryProof,
    NoSuchBridgeExists,
    InvalidFinalityProof,
    // UnknownClientError,
    // HeaderAncestryMismatch,
    UnexpectedValidatorSetId,
    StorageValueMismatch,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Error::StorageRootMismatch => write!(f, "storage root mismatch"),
            Error::StorageValueUnavailable => write!(f, "storage value unavailable"),
            Error::ValidatorSetMismatch => write!(f, "validator set mismatch"),
            Error::InvalidAncestryProof => write!(f, "invalid ancestry proof"),
            Error::NoSuchBridgeExists => write!(f, "no such bridge exists"),
            Error::InvalidFinalityProof => write!(f, "invalid finality proof"),
            // Error::HeaderAncestryMismatch => write!(f, "header ancestry mismatch"),
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
        validator_set: &[(AuthorityId, AuthorityWeight)],
        _set_id: SetId,
    ) -> Result<()> {
        let checker = <StorageProofChecker<T::Hashing>>::new(*state_root, proof)?;

        // By encoding the given set we should have an easy way to compare
        // with the stuff we get out of storage via `read_value`
        let mut encoded_validator_set = validator_set.encode();
        encoded_validator_set.insert(0, 1); // Add AUTHORITIES_VERISON == 1
        let actual_validator_set = checker
            .read_value(b":grandpa_authorities")?
            .ok_or_else(|| anyhow::Error::msg(Error::StorageValueUnavailable))?;

        // TODO: check set_id
        // checker.read_value(grandpa::CurrentSetId.key())

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
			self.last_finalized_block_header, self.current_set.list, self.current_set.id)
    }
}

pub mod utils {
    use parity_scale_codec::Encode;

    /// Gets the prefix of a storage item
    pub fn storage_prefix(module: &str, storage: &str) -> Vec<u8> {
        let mut bytes = sp_core::twox_128(module.as_bytes()).to_vec();
        bytes.extend(&sp_core::twox_128(storage.as_bytes())[..]);
        bytes
    }

    /// Calculates the Substrate storage key prefix for a StorageMap
    pub fn storage_map_prefix_twox_64_concat(
        module: &[u8],
        storage_item: &[u8],
        key: &(impl Encode + ?Sized),
    ) -> Vec<u8> {
        let mut bytes = sp_core::twox_128(module).to_vec();
        bytes.extend(&sp_core::twox_128(storage_item)[..]);
        let encoded = key.encode();
        bytes.extend(sp_core::twox_64(&encoded));
        bytes.extend(&encoded);
        bytes
    }

    #[test]
    #[ignore = "for debug"]
    fn show_keys() {
        let modules = [
            "System",
            "Timestamp",
            "RandomnessCollectiveFlip",
            "Utility",
            "Multisig",
            "Proxy",
            "Vesting",
            "Scheduler",
            "Preimage",
            "ParachainInfo",
            "ParachainSystem",
            "XcmpQueue",
            "CumulusXcm",
            "DmpQueue",
            "PolkadotXcm",
            "Balances",
            "TransactionPayment",
            "Authorship",
            "CollatorSelection",
            "Session",
            "Aura",
            "AuraExt",
            "Identity",
            "Democracy",
            "Council",
            "Treasury",
            "Bounties",
            "Lottery",
            "TechnicalCommittee",
            "TechnicalMembership",
            "PhragmenElection",
            "Tips",
            "ChildBounties",
            "ChainBridge",
            "XcmBridge",
            "XTransfer",
            "PhalaMq",
            "PhalaRegistry",
            "PhalaComputation",
            "PhalaStakePool",
            "Assets",
            "AssetsRegistry",
            "PhalaStakePoolv2",
            "PhalaVault",
            "PhalaWrappedBalances",
            "PhalaBasePool",
            "Uniques",
            "RmrkCore",
            "RmrkEquip",
            "RmrkMarket",
            "PWNftSale",
            "PWIncubation",
        ];
        for module in modules.iter() {
            let key = storage_prefix(module, "");
            println!("{module}: 0x{}", hex::encode(key));
        }

        let storage_keys = [
            "Collections",
            "Nfts",
            "Priorities",
            "Children",
            "Resources",
            "EquippableBases",
            "EquippableSlots",
            "Properties",
            "Lock",
            "DummyStorage",
        ];
        for key in storage_keys.iter() {
            let prefix = storage_prefix("RmrkCore", key);
            println!("RmrkCore::{key}: 0x{}", hex::encode(prefix));
        }
        /*
        System: 0x26aa394eea5630e07c48ae0c9558cef799e9d85137db46ef4bbea33613baafd5
        Timestamp: 0xf0c365c3cf59d671eb72da0e7a4113c499e9d85137db46ef4bbea33613baafd5
        RandomnessCollectiveFlip: 0xbd2a529379475088d3e29a918cd4787299e9d85137db46ef4bbea33613baafd5
        Utility: 0xd5e1a2fa16732ce6906189438c0a82c699e9d85137db46ef4bbea33613baafd5
        Multisig: 0x7474449cca95dc5d0c00e71735a6d17d99e9d85137db46ef4bbea33613baafd5
        Proxy: 0x1809d78346727a0ef58c0fa03bafa32399e9d85137db46ef4bbea33613baafd5
        Vesting: 0x5f27b51b5ec208ee9cb25b55d872824399e9d85137db46ef4bbea33613baafd5
        Scheduler: 0x3db7a24cfdc9de785974746c14a99df999e9d85137db46ef4bbea33613baafd5
        Preimage: 0xd8f314b7f4e6b095f0f8ee4656a4482599e9d85137db46ef4bbea33613baafd5
        ParachainInfo: 0x0d715f2646c8f85767b5d2764bb2782699e9d85137db46ef4bbea33613baafd5
        ParachainSystem: 0x45323df7cc47150b3930e2666b0aa31399e9d85137db46ef4bbea33613baafd5
        XcmpQueue: 0x7b3237373ffdfeb1cab4222e3b520d6b99e9d85137db46ef4bbea33613baafd5
        CumulusXcm: 0x79e2fe5d327165001f8232643023ed8b99e9d85137db46ef4bbea33613baafd5
        DmpQueue: 0xcd5c1f6df63bc97f4a8ce37f14a50ca799e9d85137db46ef4bbea33613baafd5
        PolkadotXcm: 0xe38f185207498abb5c213d0fb059b3d899e9d85137db46ef4bbea33613baafd5
        Balances: 0xc2261276cc9d1f8598ea4b6a74b15c2f99e9d85137db46ef4bbea33613baafd5
        TransactionPayment: 0x3f1467a096bcd71a5b6a0c8155e2081099e9d85137db46ef4bbea33613baafd5
        Authorship: 0xd57bce545fb382c34570e5dfbf338f5e99e9d85137db46ef4bbea33613baafd5
        CollatorSelection: 0x15464cac3378d46f113cd5b7a4d71c8499e9d85137db46ef4bbea33613baafd5
        Session: 0xcec5070d609dd3497f72bde07fc96ba099e9d85137db46ef4bbea33613baafd5
        Aura: 0x57f8dc2f5ab09467896f47300f04243899e9d85137db46ef4bbea33613baafd5
        AuraExt: 0x3c311d57d4daf52904616cf69648081e99e9d85137db46ef4bbea33613baafd5
        Identity: 0x2aeddc77fe58c98d50bd37f1b90840f999e9d85137db46ef4bbea33613baafd5
        Democracy: 0xf2794c22e353e9a839f12faab03a911b99e9d85137db46ef4bbea33613baafd5
        Council: 0xaebd463ed9925c488c112434d61debc099e9d85137db46ef4bbea33613baafd5
        Treasury: 0x89d139e01a5eb2256f222e5fc5dbe6b399e9d85137db46ef4bbea33613baafd5
        Bounties: 0xa37f719efab16103103a0c8c2c784ce199e9d85137db46ef4bbea33613baafd5
        Lottery: 0xfbc9f53700f75f681f234e70fb7241eb99e9d85137db46ef4bbea33613baafd5
        TechnicalCommittee: 0xed25f63942de25ac5253ba64b5eb64d199e9d85137db46ef4bbea33613baafd5
        TechnicalMembership: 0x3a2d6c9353500637d8f8e3e0fa0bb1c599e9d85137db46ef4bbea33613baafd5
        PhragmenElection: 0xe2e62dd81c48a88f73b6f6463555fd8e99e9d85137db46ef4bbea33613baafd5
        Tips: 0x2c5de123c468aef7f3ac2ab3a76f87ce99e9d85137db46ef4bbea33613baafd5
        ChildBounties: 0xedfb05b766f199ce00df85317e33050e99e9d85137db46ef4bbea33613baafd5
        ChainBridge: 0x43cdcd39d5edb1d16e24fa028edde0de99e9d85137db46ef4bbea33613baafd5
        XcmBridge: 0x9d0cdc3697970df81fa5fabe88fa03ea99e9d85137db46ef4bbea33613baafd5
        XTransfer: 0xc0cf946351a2b7b37cc8f3086b3674a199e9d85137db46ef4bbea33613baafd5
        PhalaMq: 0x2f039a6a7f13e94b9545257e54062a0499e9d85137db46ef4bbea33613baafd5
        PhalaRegistry: 0x0d746931e7a6bfd47fbcccfd71984aef99e9d85137db46ef4bbea33613baafd5
        PhalaComputation: 0xb71c310d8c830d345ee1c1b84566a8d199e9d85137db46ef4bbea33613baafd5
        PhalaStakePool: 0x9708ddcf89326bf4f4428dd135287d5199e9d85137db46ef4bbea33613baafd5
        Assets: 0x682a59d51ab9e48a8c8cc418ff9708d299e9d85137db46ef4bbea33613baafd5
        AssetsRegistry: 0xf7860e52b3d3660de35c808455ec483699e9d85137db46ef4bbea33613baafd5
        PhalaStakePoolv2: 0x75e3ed3f59e45643ed1149ad80929c1b99e9d85137db46ef4bbea33613baafd5
        PhalaVault: 0xa61c43efbf2367eb32ddcf956fc97dd499e9d85137db46ef4bbea33613baafd5
        PhalaWrappedBalances: 0x1466de7f00add77dbab4df042b8c4a8499e9d85137db46ef4bbea33613baafd5
        PhalaBasePool: 0x00f8eafbad3b4a32114491ad7e12491499e9d85137db46ef4bbea33613baafd5
        Uniques: 0x5e8a19e3cd1b7c148b33880c479c028199e9d85137db46ef4bbea33613baafd5
        RmrkCore: 0x5bef2c5471aa9e955551dc810f5abb3999e9d85137db46ef4bbea33613baafd5
        RmrkEquip: 0x8c2ffe3a0b5892f363d8b9e374b9e9fc99e9d85137db46ef4bbea33613baafd5
        RmrkMarket: 0x826a25a29a1da02112a6b8390475706699e9d85137db46ef4bbea33613baafd5
        PWNftSale: 0x04d3f224a1307398074171146ffc417299e9d85137db46ef4bbea33613baafd5
        PWIncubation: 0x66b8232d707a1e10cc0cc5a75b738ad299e9d85137db46ef4bbea33613baafd5
        RmrkCore::Collections: 0x5bef2c5471aa9e955551dc810f5abb399200647b8c99af7b8b52752114831bdb
        RmrkCore::Nfts: 0x5bef2c5471aa9e955551dc810f5abb39e8d49389c2e23e152fdd6364daadd2cc
        RmrkCore::Priorities: 0x5bef2c5471aa9e955551dc810f5abb397f6749268d89e15586d82478e7290431
        RmrkCore::Children: 0x5bef2c5471aa9e955551dc810f5abb39261f5a952a31d4199096219bbfd87740
        RmrkCore::Resources: 0x5bef2c5471aa9e955551dc810f5abb392111e0df19de9563b58301e5f7e00743
        RmrkCore::EquippableBases: 0x5bef2c5471aa9e955551dc810f5abb39ef660df27389f71e45f1741b554773fb
        RmrkCore::EquippableSlots: 0x5bef2c5471aa9e955551dc810f5abb393e26973064c5f9a17e8bfaa18aee3013
        RmrkCore::Properties: 0x5bef2c5471aa9e955551dc810f5abb39a436740684271e6e2985d7bb452fdf99
        RmrkCore::Lock: 0x5bef2c5471aa9e955551dc810f5abb39fb1ef94455cc6b5a3840206754686d98
        RmrkCore::DummyStorage: 0x5bef2c5471aa9e955551dc810f5abb399439307cc9be85229487820a36657c35
        */
    }
}
