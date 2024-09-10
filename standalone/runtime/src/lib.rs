// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! The Substrate runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "1024"]
#![allow(clippy::identity_op)]

mod msg_routing;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_election_provider_support::{
    bounds::{ElectionBounds, ElectionBoundsBuilder},
    onchain, BalancingConfig, ElectionDataProvider, SequentialPhragmen, VoteWeight,
};
use frame_support::{
    construct_runtime,
    dispatch::DispatchClass,
    pallet_prelude::Get,
    parameter_types,
    traits::{
        fungible::HoldConsideration,
        tokens::{PayFromAccount, UnityAssetBalanceConversion},
        AsEnsureOriginWithArg, ConstU128, ConstU32, Currency, EitherOfDiverse, EqualPrivilegeOnly,
        Everything, Imbalance, InstanceFilter, KeyOwnerProofSystem, LinearStoragePrice,
        LockIdentifier, OnUnbalanced, SortedMembers, WithdrawReasons,
    },
    weights::{
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
        IdentityFee, Weight,
    },
    PalletId,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot, EnsureSigned, EnsureSignedBy, EnsureWithSuccess,
};
pub use node_primitives::{
    AccountId, AccountIndex, Balance, BlockNumber, Hash, Moment, Nonce, Signature,
};
use pallet_election_provider_multi_phase::{GeometricDepositBase, SolutionAccuracyOf};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
pub use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::{
    create_runtime_str,
    curve::PiecewiseLinear,
    generic, impl_opaque_keys,
    traits::{
        self, BlakeTwo256, Block as BlockT, Bounded, ConvertInto, NumberFor, OpaqueKeys,
        SaturatedConversion, StaticLookup, TrailingZeroInput,
    },
    transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
    AccountId32, ApplyExtrinsicResult, FixedPointNumber, FixedU128, Perbill, Percent, Permill,
    Perquintill, RuntimeDebug,
};
use sp_std::prelude::*;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;

#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use pallet_staking::StakerStatus;
#[cfg(any(feature = "std", test))]
pub use pallet_sudo::Call as SudoCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;
use impls::Author;

/// Constant values used within the runtime.
pub mod constants;
use constants::{currency::*, time::*};
use sp_runtime::generic::Era;

/// Generated voter bag information.
mod voter_bags;

pub use phala_pallets::{
    pallet_base_pool, pallet_computation, pallet_mq, pallet_phat, pallet_phat_tokenomic,
    pallet_registry, pallet_stake_pool, pallet_stake_pool_v2, pallet_vault, pallet_wapod_workers,
    pallet_wrapped_balances, puppets,
};
use phat_offchain_rollup::{anchor as pallet_anchor, oracle as pallet_oracle};

// Make the WASM binary available.
#[cfg(all(feature = "std", feature = "include-wasm"))]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Max size for serialized extrinsic params for this testing runtime.
/// This is a quite arbitrary but empirically battle tested value.
#[cfg(test)]
pub const CALL_PARAMS_MAX_SIZE: usize = 300;

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(all(feature = "std", feature = "include-wasm"))]
pub fn wasm_binary_unwrap() -> &'static [u8] {
    WASM_BINARY.expect(
        "Development wasm binary is not available. This means the client is built with \
         `SKIP_WASM_BUILD` flag and it is only usable for production chains. Please rebuild with \
         the flag disabled.",
    )
}

/// Runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("phala-node"),
    impl_name: create_runtime_str!("phala-node"),
    authoring_version: 1,
    // Per convention: if the runtime behavior changes, increment spec_version
    // and set impl_version to 0. If only runtime
    // implementation changes and behavior does not, then leave spec_version as
    // is and increment impl_version.
    spec_version: 101,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 0,
};

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: PRIMARY_PROBABILITY,
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
    };

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            // for fees, 80% to treasury, 20% to author
            let mut split = fees.ration(80, 20);
            if let Some(tips) = fees_then_tips.next() {
                // for tips, if any, 80% to treasury, 20% to author (though this can be anything)
                tips.ration_merge_into(80, 20, &mut split);
            }
            Treasury::on_unbalanced(split.0);
            Author::on_unbalanced(split.1);
        }
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time, with maximum proof size.
const MAXIMUM_BLOCK_WEIGHT: Weight =
    Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), u64::MAX);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u16 = 30;
    pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

impl frame_system::Config for Runtime {
    type BaseCallFilter = Everything;
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
    type DbWeight = RocksDbWeight;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = Nonce;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = Indices;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl phala_pallets::PhalaConfig for Runtime {
    type Currency = Balances;
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // One storage item; key size 32, value size 8; .
    pub const ProxyDepositBase: Balance = deposit(1, 8);
    // Additional storage item size of 33 bytes.
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const MaxProxies: u16 = 32;
    pub const AnnouncementDepositBase: Balance = deposit(1, 8);
    pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
    pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    RuntimeDebug,
    MaxEncodedLen,
    scale_info::TypeInfo,
)]
pub enum ProxyType {
    Any,
    NonTransfer,
    Governance,
    Staking,
    StakePoolManager,
}
impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
}
impl InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, c: &RuntimeCall) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::NonTransfer => !matches!(
                c,
                RuntimeCall::Balances(..)
                    | RuntimeCall::Vesting(pallet_vesting::Call::vested_transfer { .. })
                    | RuntimeCall::Indices(pallet_indices::Call::transfer { .. })
            ),
            ProxyType::Governance => matches!(
                c,
                RuntimeCall::Democracy(..)
                    | RuntimeCall::Council(..)
                    | RuntimeCall::Society(..)
                    | RuntimeCall::TechnicalCommittee(..)
                    | RuntimeCall::Elections(..)
                    | RuntimeCall::Treasury(..)
            ),
            ProxyType::Staking => {
                matches!(c, RuntimeCall::Staking(..) | RuntimeCall::FastUnstake(..))
            }
            ProxyType::StakePoolManager => matches!(
                c,
                RuntimeCall::Utility { .. }
                    | RuntimeCall::PhalaStakePoolv2(pallet_stake_pool_v2::Call::add_worker { .. })
                    | RuntimeCall::PhalaStakePoolv2(
                        pallet_stake_pool_v2::Call::remove_worker { .. }
                    )
                    | RuntimeCall::PhalaStakePoolv2(
                        pallet_stake_pool_v2::Call::start_computing { .. }
                    )
                    | RuntimeCall::PhalaStakePoolv2(
                        pallet_stake_pool_v2::Call::stop_computing { .. }
                    )
                    | RuntimeCall::PhalaStakePoolv2(
                        pallet_stake_pool_v2::Call::restart_computing { .. }
                    )
                    | RuntimeCall::PhalaStakePoolv2(
                        pallet_stake_pool_v2::Call::reclaim_pool_worker { .. }
                    )
                    | RuntimeCall::PhalaStakePoolv2(pallet_stake_pool_v2::Call::create { .. })
                    | RuntimeCall::PhalaRegistry(pallet_registry::Call::register_worker { .. })
                    | RuntimeCall::PhalaRegistry(pallet_registry::Call::register_worker_v2 { .. })
                    | RuntimeCall::PhalaMq(pallet_mq::Call::sync_offchain_message { .. })
            ),
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            (ProxyType::NonTransfer, _) => true,
            _ => false,
        }
    }
}

impl pallet_proxy::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        RuntimeBlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 512;
}

impl pallet_scheduler::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRootOrHalfCouncil;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type Preimages = Preimage;
}

parameter_types! {
    pub const PreimageBaseDeposit: Balance = 1 * DOLLARS;
    // One cent: $10,000 / MB
    pub const PreimageByteDeposit: Balance = 1 * CENTS;
    pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type Consideration = HoldConsideration<
        AccountId,
        Balances,
        PreimageHoldReason,
        LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
    >;
}

parameter_types! {
    // NOTE: Currently it is not possible to change the epoch duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub EpochDuration: u64 = EpochDurationInSlots::get();
    pub ExpectedBlockTime: Moment = MillisecsPerBlock::get();
    pub ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type DisabledValidators = Session;
    type WeightInfo = ();
    type MaxAuthorities = MaxAuthorities;
    type MaxNominators = MaxNominatorRewardedPerValidator;
    type KeyOwnerProof =
        <Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
    type EquivocationReportSystem =
        pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

parameter_types! {
    pub const IndexDeposit: Balance = 1 * DOLLARS;
}

impl pallet_indices::Config for Runtime {
    type AccountIndex = AccountIndex;
    type Currency = Balances;
    type Deposit = IndexDeposit;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1 * CENTS;
    // For weight estimation, we assume that the most locks on an individual account will be 50.
    // This number may need to be adjusted in the future if this assumption no longer holds true.
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Pallet<Runtime>;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = ConstU32<3>;
    type MaxHolds = ConstU32<5>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
}

parameter_types! {
    pub const OperationalFeeMultiplier: u8 = 5;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
    pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = TargetedFeeAdjustment<
        Self,
        TargetBlockFullness,
        AdjustmentVariable,
        MinimumMultiplier,
        MaximumMultiplier,
    >;
}

parameter_types! {
    pub MinimumPeriod: Moment = SlotDuration::get() / 2;
}

impl pallet_timestamp::Config for Runtime {
    type Moment = Moment;
    type OnTimestampSet = Babe;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type EventHandler = (Staking, ImOnline);
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub babe: Babe,
        pub im_online: ImOnline,
        pub authority_discovery: AuthorityDiscovery,
    }
}

impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    pub const BondingDuration: sp_staking::EraIndex = 24 * 28;
    pub const SlashDeferDuration: sp_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 256;
    pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
    pub const MaxControllersInDeprecationBatch: u32 = 5900;
    pub OffchainRepeat: BlockNumber = 5;
    pub HistoryDepth: u32 = 84;
}

/// Upper limit on the number of NPOS nominations.
const MAX_QUOTA_NOMINATIONS: u32 = 16;

pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
    type MaxNominators = ConstU32<1000>;
    type MaxValidators = ConstU32<1000>;
}

impl pallet_staking::Config for Runtime {
    type Currency = Balances;
    type CurrencyBalance = Balance;
    type UnixTime = Timestamp;
    type CurrencyToVote = sp_staking::currency_to_vote::U128CurrencyToVote;
    type RewardRemainder = Treasury;
    type RuntimeEvent = RuntimeEvent;
    type Slash = Treasury; // send the slashed funds to the treasury.
    type Reward = (); // rewards are minted from the void
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    /// A super-majority of the council can cancel the slash.
    type AdminOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
    >;
    type SessionInterface = Self;
    type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
    type NextNewSession = Session;
    type MaxExposurePageSize = ConstU32<256>;
    type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
    type ElectionProvider = ElectionProviderMultiPhase;
    type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type VoterList = VoterList;
    type NominationsQuota = pallet_staking::FixedNominationsQuota<MAX_QUOTA_NOMINATIONS>;
    // This a placeholder, to be introduced in the next PR as an instance of bags-list
    type TargetList = pallet_staking::UseValidatorsMap<Self>;
    type MaxUnlockingChunks = ConstU32<32>;
    type HistoryDepth = HistoryDepth;
    type EventListeners = NominationPools;
    type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
    type BenchmarkingConfig = StakingBenchmarkingConfig;
}

impl pallet_fast_unstake::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ControlOrigin = frame_system::EnsureRoot<AccountId>;
    type BatchSize = ConstU32<64>;
    type Deposit = ConstU128<{ DOLLARS }>;
    type Currency = Balances;
    type Staking = Staking;
    type MaxErasToCheckPerBlock = ConstU32<1>;
    #[cfg(feature = "runtime-benchmarks")]
    type MaxBackersPerValidator = MaxNominatorRewardedPerValidator;
    type WeightInfo = ();
}

parameter_types! {
    // phase durations. 1/4 of the last session for each.
    pub const SignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;
    pub const UnsignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;

    // signed config
    pub const SignedRewardBase: Balance = 1 * DOLLARS;
    pub const SignedFixedDeposit: Balance = 1 * DOLLARS;
    pub const SignedDepositIncreaseFactor: Percent = Percent::from_percent(10);
    pub const SignedDepositByte: Balance = 1 * CENTS;

    pub BetterUnsignedThreshold: Perbill = Perbill::from_rational(1u32, 10_000);

    // miner configs
    pub const MultiPhaseUnsignedPriority: TransactionPriority = StakingUnsignedPriority::get() - 1u64;
    pub MinerMaxWeight: Weight = RuntimeBlockWeights::get()
        .get(DispatchClass::Normal)
        .max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
        .saturating_sub(BlockExecutionWeight::get());
    // Solution can occupy 90% of normal block size
    pub MinerMaxLength: u32 = Perbill::from_rational(9u32, 10) *
        *RuntimeBlockLength::get()
        .max
        .get(DispatchClass::Normal);
}

frame_election_provider_support::generate_solution_type!(
    #[compact]
    pub struct NposSolution16::<
        VoterIndex = u32,
        TargetIndex = u16,
        Accuracy = sp_runtime::PerU16,
        MaxVoters = MaxElectingVotersSolution,
    >(16)
);

parameter_types! {
    // Note: the EPM in this runtime runs the election on-chain. The election bounds must be
    // carefully set so that an election round fits in one block.
    pub ElectionBoundsMultiPhase: ElectionBounds = ElectionBoundsBuilder::default()
        .voters_count(10_000.into()).targets_count(1_500.into()).build();
    pub ElectionBoundsOnChain: ElectionBounds = ElectionBoundsBuilder::default()
        .voters_count(5_000.into()).targets_count(1_250.into()).build();

    pub MaxNominations: u32 = <NposSolution16 as frame_election_provider_support::NposSolution>::LIMIT as u32;
    pub MaxElectingVotersSolution: u32 = 40_000;
    // The maximum winners that can be elected by the Election pallet which is equivalent to the
    // maximum active validators the staking pallet can have.
    pub MaxActiveValidators: u32 = 1000;
}

/// The numbers configured here could always be more than the the maximum limits of staking pallet
/// to ensure election snapshot will not run out of memory. For now, we set them to smaller values
/// since the staking is bounded and the weight pipeline takes hours for this single pallet.
pub struct ElectionProviderBenchmarkConfig;
impl pallet_election_provider_multi_phase::BenchmarkingConfig for ElectionProviderBenchmarkConfig {
    const VOTERS: [u32; 2] = [1000, 2000];
    const TARGETS: [u32; 2] = [500, 1000];
    const ACTIVE_VOTERS: [u32; 2] = [500, 800];
    const DESIRED_TARGETS: [u32; 2] = [200, 400];
    const SNAPSHOT_MAXIMUM_VOTERS: u32 = 1000;
    const MINER_MAXIMUM_VOTERS: u32 = 1000;
    const MAXIMUM_TARGETS: u32 = 300;
}

/// Maximum number of iterations for balancing that will be executed in the embedded OCW
/// miner of election provider multi phase.
pub const MINER_MAX_ITERATIONS: u32 = 10;

/// A source of random balance for NposSolver, which is meant to be run by the OCW election miner.
pub struct OffchainRandomBalancing;
impl Get<Option<BalancingConfig>> for OffchainRandomBalancing {
    fn get() -> Option<BalancingConfig> {
        let iterations = match MINER_MAX_ITERATIONS {
            0 => 0,
            max => {
                let seed = sp_io::offchain::random_seed();
                let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
                    .expect("input is padded with zeroes; qed")
                    % max.saturating_add(1);
                random as usize
            }
        };

        let config = BalancingConfig {
            iterations,
            tolerance: 0,
        };
        Some(config)
    }
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
    type System = Runtime;
    type Solver = SequentialPhragmen<
        AccountId,
        pallet_election_provider_multi_phase::SolutionAccuracyOf<Runtime>,
    >;
    type DataProvider = <Runtime as pallet_election_provider_multi_phase::Config>::DataProvider;
    type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
    type MaxWinners = <Runtime as pallet_election_provider_multi_phase::Config>::MaxWinners;
    type Bounds = ElectionBoundsOnChain;
}

impl pallet_election_provider_multi_phase::MinerConfig for Runtime {
    type AccountId = AccountId;
    type MaxLength = MinerMaxLength;
    type MaxWeight = MinerMaxWeight;
    type Solution = NposSolution16;
    type MaxVotesPerVoter =
    <<Self as pallet_election_provider_multi_phase::Config>::DataProvider as ElectionDataProvider>::MaxVotesPerVoter;
    type MaxWinners = MaxActiveValidators;

    // The unsigned submissions have to respect the weight of the submit_unsigned call, thus their
    // weight estimate function is wired to this call's weight.
    fn solution_weight(v: u32, t: u32, a: u32, d: u32) -> Weight {
        <
        <Self as pallet_election_provider_multi_phase::Config>::WeightInfo
        as
        pallet_election_provider_multi_phase::WeightInfo
        >::submit_unsigned(v, t, a, d)
    }
}

impl pallet_election_provider_multi_phase::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EstimateCallFee = TransactionPayment;
    type SignedPhase = SignedPhase;
    type UnsignedPhase = UnsignedPhase;
    type BetterUnsignedThreshold = BetterUnsignedThreshold;
    type BetterSignedThreshold = ();
    type OffchainRepeat = OffchainRepeat;
    type MinerTxPriority = MultiPhaseUnsignedPriority;
    type MinerConfig = Self;
    type SignedMaxSubmissions = ConstU32<10>;
    type SignedRewardBase = SignedRewardBase;
    type SignedDepositBase =
        GeometricDepositBase<Balance, SignedFixedDeposit, SignedDepositIncreaseFactor>;
    type SignedDepositByte = SignedDepositByte;
    type SignedMaxRefunds = ConstU32<3>;
    type SignedDepositWeight = ();
    type SignedMaxWeight = MinerMaxWeight;
    type SlashHandler = (); // burn slashes
    type RewardHandler = (); // nothing to do upon rewards
    type DataProvider = Staking;
    type Fallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type GovernanceFallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type Solver = SequentialPhragmen<AccountId, SolutionAccuracyOf<Self>, OffchainRandomBalancing>;
    type ForceOrigin = EnsureRootOrHalfCouncil;
    type MaxWinners = MaxActiveValidators;
    type ElectionBounds = ElectionBoundsMultiPhase;
    type BenchmarkingConfig = ElectionProviderBenchmarkConfig;
    type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Self>;
}

parameter_types! {
    pub const BagThresholds: &'static [u64] = &voter_bags::THRESHOLDS;
}

type VoterBagsListInstance = pallet_bags_list::Instance1;
impl pallet_bags_list::Config<VoterBagsListInstance> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    /// The voter bags-list is loosely kept up to date, and the real source of truth for the score
    /// of each node is the staking pallet.
    type ScoreProvider = Staking;
    type BagThresholds = BagThresholds;
    type Score = VoteWeight;
    type WeightInfo = pallet_bags_list::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const PostUnbondPoolsWindow: u32 = 4;
    pub const NominationPoolsPalletId: PalletId = PalletId(*b"py/nopls");
    pub const MaxPointsToBalance: u8 = 10;
}

use sp_runtime::traits::Convert;
pub struct BalanceToU256;
impl Convert<Balance, sp_core::U256> for BalanceToU256 {
    fn convert(balance: Balance) -> sp_core::U256 {
        sp_core::U256::from(balance)
    }
}
pub struct U256ToBalance;
impl Convert<sp_core::U256, Balance> for U256ToBalance {
    fn convert(n: sp_core::U256) -> Balance {
        n.try_into().unwrap_or(Balance::max_value())
    }
}

impl pallet_nomination_pools::Config for Runtime {
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type RewardCounter = FixedU128;
    type BalanceToU256 = BalanceToU256;
    type U256ToBalance = U256ToBalance;
    type Staking = Staking;
    type PostUnbondingPoolsWindow = PostUnbondPoolsWindow;
    type MaxMetadataLen = ConstU32<256>;
    type MaxUnbonding = ConstU32<8>;
    type PalletId = NominationPoolsPalletId;
    type MaxPointsToBalance = MaxPointsToBalance;
}

parameter_types! {
    pub LaunchPeriod: BlockNumber = 28 * 24 * 60 * Minutes::get();
    pub VotingPeriod: BlockNumber = 28 * 24 * 60 * Minutes::get();
    pub FastTrackVotingPeriod: BlockNumber = 3 * 24 * 60 * Minutes::get();
    pub const InstantAllowed: bool = true;
    pub const MinimumDeposit: Balance = 100 * DOLLARS;
    pub EnactmentPeriod: BlockNumber = 30 * 24 * 60 * Minutes::get();
    pub CooloffPeriod: BlockNumber = 28 * 24 * 60 * Minutes::get();
    pub const MaxVotes: u32 = 100;
    pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
    type MinimumDeposit = MinimumDeposit;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
    /// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
    type SubmitOrigin = EnsureSigned<AccountId>;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>;
    type InstantOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;
    type InstantAllowed = InstantAllowed;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
    type CancellationOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
    // To cancel a proposal before it has been passed, the technical committee must be unanimous or
    // Root must agree.
    type CancelProposalOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
    >;
    type BlacklistOrigin = EnsureRoot<AccountId>;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cool-off period.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
    type CooloffPeriod = CooloffPeriod;
    type Slash = Treasury;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = MaxVotes;
    type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
    type MaxProposals = MaxProposals;
    type Preimages = Preimage;
    type MaxDeposits = ConstU32<100>;
    type MaxBlacklisted = ConstU32<100>;
}

parameter_types! {
    pub CouncilMotionDuration: BlockNumber = 5 * Days::get();
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    type SetMembersOrigin = EnsureRoot<Self::AccountId>;
    type MaxProposalWeight = MaxCollectivesProposalWeight;
}

parameter_types! {
    pub const CandidacyBond: Balance = 10 * DOLLARS;
    // 1 storage item created, key size is 32 bytes, value size is 16+16.
    pub const VotingBondBase: Balance = deposit(1, 64);
    // additional data per vote is 32 bytes (account id).
    pub const VotingBondFactor: Balance = deposit(0, 32);
    pub TermDuration: BlockNumber = 7 * Days::get();
    pub const DesiredMembers: u32 = 13;
    pub const DesiredRunnersUp: u32 = 7;
    pub const MaxVotesPerVoter: u32 = 16;
    pub const MaxVoters: u32 = 512;
    pub const MaxCandidates: u32 = 64;
    pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = ElectionsPhragmenPalletId;
    type Currency = Balances;
    type ChangeMembers = Council;
    // NOTE: this implies that council's genesis members cannot be set directly and must come from
    // this module.
    type InitializeMembers = Council;
    type CurrencyToVote = sp_staking::currency_to_vote::U128CurrencyToVote;
    type CandidacyBond = CandidacyBond;
    type VotingBondBase = VotingBondBase;
    type VotingBondFactor = VotingBondFactor;
    type LoserCandidate = ();
    type KickedMember = ();
    type DesiredMembers = DesiredMembers;
    type DesiredRunnersUp = DesiredRunnersUp;
    type TermDuration = TermDuration;
    type MaxVoters = MaxVoters;
    type MaxVotesPerVoter = MaxVotesPerVoter;
    type MaxCandidates = MaxCandidates;
    type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub TechnicalMotionDuration: BlockNumber = 5 * Days::get();
    pub const TechnicalMaxProposals: u32 = 100;
    pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = TechnicalMotionDuration;
    type MaxProposals = TechnicalMaxProposals;
    type MaxMembers = TechnicalMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    type SetMembersOrigin = EnsureRoot<Self::AccountId>;
    type MaxProposalWeight = MaxCollectivesProposalWeight;
}

type EnsureRootOrHalfCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;
impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = EnsureRootOrHalfCouncil;
    type RemoveOrigin = EnsureRootOrHalfCouncil;
    type SwapOrigin = EnsureRootOrHalfCouncil;
    type ResetOrigin = EnsureRootOrHalfCouncil;
    type PrimeOrigin = EnsureRootOrHalfCouncil;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
    type MaxMembers = TechnicalMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
    pub const SpendPeriod: BlockNumber = 2 * MINUTES;
    pub const Burn: Permill = Permill::from_percent(1);
    pub const TipCountdown: BlockNumber = 1 * DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = 1 * DOLLARS;
    pub const DataDepositPerByte: Balance = 1 * CENTS;
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const MaximumReasonLength: u32 = 300;
    pub const MaxApprovals: u32 = 100;
    pub const MaxBalance: Balance = Balance::max_value();
    pub const SpendPayoutPeriod: BlockNumber = 1 * DAYS;
    pub TreasuryAccount: AccountId = Treasury::account_id();
}

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
    >;
    type RejectOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
    >;
    type RuntimeEvent = RuntimeEvent;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type ProposalBondMaximum = ();
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    type SpendFunds = Bounties;
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type MaxApprovals = MaxApprovals;
    type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
    type AssetKind = ();
    type Beneficiary = AccountId;
    type BeneficiaryLookup = Indices;
    type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
    type BalanceConverter = UnityAssetBalanceConversion;
    type PayoutPeriod = SpendPayoutPeriod;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

parameter_types! {
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: Balance = 5 * DOLLARS;
    pub const BountyDepositBase: Balance = 1 * DOLLARS;
    pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
    pub const CuratorDepositMin: Balance = 1 * DOLLARS;
    pub const CuratorDepositMax: Balance = 100 * DOLLARS;
    pub const BountyDepositPayoutDelay: BlockNumber = 2 * MINUTES;
    pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
}

impl pallet_bounties::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type CuratorDepositMultiplier = CuratorDepositMultiplier;
    type CuratorDepositMin = CuratorDepositMin;
    type CuratorDepositMax = CuratorDepositMax;
    type BountyValueMinimum = BountyValueMinimum;
    type DataDepositPerByte = DataDepositPerByte;
    type MaximumReasonLength = MaximumReasonLength;
    type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
    type ChildBountyManager = ChildBounties;
}

parameter_types! {
    pub const ChildBountyValueMinimum: Balance = 1 * DOLLARS;
}

impl pallet_child_bounties::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxActiveChildBountyCount = ConstU32<5>;
    type ChildBountyValueMinimum = ChildBountyValueMinimum;
    type WeightInfo = pallet_child_bounties::weights::SubstrateWeight<Runtime>;
}

impl pallet_tips::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type DataDepositPerByte = DataDepositPerByte;
    type MaximumReasonLength = MaximumReasonLength;
    type Tippers = Elections;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type MaxTipAmount = ConstU128<{ 500 * DOLLARS }>;
    type WeightInfo = pallet_tips::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    /// We prioritize im-online heartbeats over election solution submission.
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
    pub const MaxAuthorities: u32 = 100;
    pub const MaxKeys: u32 = 10_000;
    pub const MaxPeerInHeartbeats: u32 = 10_000;
    pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    RuntimeCall: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: RuntimeCall,
        public: <Signature as traits::Verify>::Signer,
        account: AccountId,
        nonce: Nonce,
    ) -> Option<(
        RuntimeCall,
        <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
    )> {
        let tip = 0;
        // take the biggest period possible.
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;
        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);
        let era = Era::mortal(period, current_block);
        let extra = (
            frame_system::CheckNonZeroSender::<Runtime>::new(),
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(era),
            frame_system::CheckNonce::<Runtime>::from(nonce),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_mq::CheckMqSequence::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                log::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let address = Indices::unlookup(account);
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (address, signature, extra)))
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = RuntimeCall;
}

impl pallet_im_online::Config for Runtime {
    type AuthorityId = ImOnlineId;
    type RuntimeEvent = RuntimeEvent;
    type NextSessionRotation = Babe;
    type ValidatorSet = Historical;
    type ReportUnresponsiveness = Offences;
    type UnsignedPriority = ImOnlineUnsignedPriority;
    type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
    type MaxKeys = MaxKeys;
    type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
}

impl pallet_offences::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
}

impl pallet_authority_discovery::Config for Runtime {
    type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
    pub const MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = MaxAuthorities;
    type MaxNominators = MaxNominatorRewardedPerValidator;
    type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
    type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type EquivocationReportSystem =
        pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

parameter_types! {
    pub const BasicDeposit: Balance = 10 * DOLLARS;       // 258 bytes on-chain
    pub const ByteDeposit: Balance = 250 * CENTS;        // 66 bytes on-chain
    pub const SubAccountDeposit: Balance = 2 * DOLLARS;   // 53 bytes on-chain
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 100;
    pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type BasicDeposit = BasicDeposit;
    type ByteDeposit = ByteDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type MaxSubAccounts = MaxSubAccounts;
    type IdentityInformation = pallet_identity::legacy::IdentityInfo<MaxAdditionalFields>;
    type MaxRegistrars = MaxRegistrars;
    type Slashed = Treasury;
    type ForceOrigin = EnsureRootOrHalfCouncil;
    type RegistrarOrigin = EnsureRootOrHalfCouncil;
    type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ConfigDepositBase: Balance = 5 * DOLLARS;
    pub const FriendDepositFactor: Balance = 50 * CENTS;
    pub const MaxFriends: u16 = 9;
    pub const RecoveryDeposit: Balance = 5 * DOLLARS;
}

impl pallet_recovery::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_recovery::weights::SubstrateWeight<Runtime>;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ConfigDepositBase = ConfigDepositBase;
    type FriendDepositFactor = FriendDepositFactor;
    type MaxFriends = MaxFriends;
    type RecoveryDeposit = RecoveryDeposit;
}

parameter_types! {
    pub const GraceStrikes: u32 = 10;
    pub const SocietyVotingPeriod: BlockNumber = 80 * HOURS;
    pub const ClaimPeriod: BlockNumber = 80 * HOURS;
    pub const PeriodSpend: Balance = 500 * DOLLARS;
    pub const MaxLockDuration: BlockNumber = 36 * 30 * DAYS;
    pub const ChallengePeriod: BlockNumber = 7 * DAYS;
    pub const MaxPayouts: u32 = 10;
    pub const MaxBids: u32 = 10;
    pub const SocietyPalletId: PalletId = PalletId(*b"py/socie");
}

impl pallet_society::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = SocietyPalletId;
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type GraceStrikes = GraceStrikes;
    type PeriodSpend = PeriodSpend;
    type VotingPeriod = SocietyVotingPeriod;
    type ClaimPeriod = ClaimPeriod;
    type MaxLockDuration = MaxLockDuration;
    type FounderSetOrigin =
        pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>;
    type ChallengePeriod = ChallengePeriod;
    type MaxPayouts = MaxPayouts;
    type MaxBids = MaxBids;
    type WeightInfo = pallet_society::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const MinVestedTransfer: Balance = 1 * CENTS;
    pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
        WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type BlockNumberToBalance = ConvertInto;
    type MinVestedTransfer = MinVestedTransfer;
    type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
    type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
    // `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
    // highest number of schedules that encodes less than 2^10.
    const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
    pub const LotteryPalletId: PalletId = PalletId(*b"py/lotto");
    pub const MaxCalls: u32 = 10;
    pub const MaxGenerateRandom: u32 = 10;
}

impl pallet_lottery::Config for Runtime {
    type PalletId = LotteryPalletId;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type RuntimeEvent = RuntimeEvent;
    type ManagerOrigin = EnsureRootOrHalfCouncil;
    type MaxCalls = MaxCalls;
    type ValidateCall = Lottery;
    type MaxGenerateRandom = MaxGenerateRandom;
    type WeightInfo = pallet_lottery::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub ExpectedBlockTimeSec: u32 = SecsPerBlock::get() as u32;
    pub const MinWorkingStaking: Balance = 1 * DOLLARS;
    pub const MinContribution: Balance = 1 * CENTS;
    pub const WorkingGracePeriod: u64 = 7 * 24 * 3600;
    pub const VaultQueuePeriod: u64 = 21 * 24 * 3600;
    pub const MinInitP: u32 = 50;
    pub const MaxPoolWorkers: u32 = 200;
    pub const NoneAttestationEnabled: bool = true;
    pub const VerifyPRuntime: bool = false;
    pub const VerifyRelaychainGenesisBlockHash: bool = false;
    pub ParachainId: u32 = ParachainInfo::parachain_id().0;
    pub const CheckWorkerRegisterTime: bool = true;
}

impl pallet_registry::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type UnixTime = Timestamp;
    type LegacyAttestationValidator = pallet_registry::IasValidator;
    type NoneAttestationEnabled = NoneAttestationEnabled;
    type VerifyPRuntime = VerifyPRuntime;
    type VerifyRelaychainGenesisBlockHash = VerifyRelaychainGenesisBlockHash;
    type GovernanceOrigin = EnsureRootOrHalfCouncil;
    type ParachainId = ParachainId;
}
impl pallet_mq::Config for Runtime {
    type QueueNotifyConfig = msg_routing::MessageRouteConfig;
    type CallMatcher = MqCallMatcher;
}

pub struct SetBudgetMembers;

impl SortedMembers<AccountId> for SetBudgetMembers {
    fn sorted_members() -> Vec<AccountId> {
        [pallet_computation::pallet::ContractAccount::<Runtime>::get()].to_vec()
    }
}

impl pallet_computation::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ExpectedBlockTimeSec = ExpectedBlockTimeSec;
    type MinInitP = MinInitP;
    type Randomness = RandomnessCollectiveFlip;
    type OnReward = PhalaStakePoolv2;
    type OnUnbound = PhalaStakePoolv2;
    type OnStopped = PhalaStakePoolv2;
    type OnTreasurySettled = Treasury;
    type UpdateTokenomicOrigin = EnsureRootOrHalfCouncil;
    type SetBudgetOrigins = EnsureSignedBy<SetBudgetMembers, AccountId>;
    type SetContractRootOrigins = EnsureRootOrHalfCouncil;
    type CheckWorkerRegisterTime = CheckWorkerRegisterTime;
}
impl pallet_stake_pool_v2::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MinContribution = MinContribution;
    type GracePeriod = WorkingGracePeriod;
    type MaxPoolWorkers = MaxPoolWorkers;
}
impl pallet_stake_pool::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
}

parameter_types! {
    pub const InitialPriceCheckPoint: Balance = 1 * DOLLARS;
}

impl pallet_vault::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type InitialPriceCheckPoint = InitialPriceCheckPoint;
    type VaultQueuePeriod = VaultQueuePeriod;
}
parameter_types! {
    pub const CollectionDeposit: Balance = 0; // 1 UNIT deposit to create collection
    pub const ItemDeposit: Balance = 0; // 1/100 UNIT deposit to create item
    pub const StringLimit: u32 = 52100;
    pub const KeyLimit: u32 = 32000; // Max 32 bytes per key
    pub const ValueLimit: u32 = 512000; // Max 64 bytes per value
    pub const UniquesMetadataDepositBase: Balance = 0;
    pub const AttributeDepositBase: Balance = 0;
    pub const DepositPerByte: Balance = 0;
}
impl pallet_uniques::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = u32;
    type ItemId = u32;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
    type Locker = pallet_rmrk_core::Pallet<Runtime>;
    type CollectionDeposit = CollectionDeposit;
    type ItemDeposit = ItemDeposit;
    type MetadataDepositBase = UniquesMetadataDepositBase;
    type AttributeDepositBase = AttributeDepositBase;
    type DepositPerByte = DepositPerByte;
    type StringLimit = StringLimit;
    type KeyLimit = KeyLimit;
    type ValueLimit = ValueLimit;
    type WeightInfo = ();
}
parameter_types! {
    pub ClassBondAmount: Balance = 100;
    pub MaxMetadataLength: u32 = 256;
    pub const ResourceSymbolLimit: u32 = 10;
    pub const PartsLimit: u32 = 10;
    pub const MaxPriorities: u32 = 3;
    pub const PropertiesLimit: u32 = 15;
    pub const CollectionSymbolLimit: u32 = 100;
    pub const MaxResourcesOnMint: u32 = 100;
}
impl pallet_rmrk_core::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ProtocolOrigin = EnsureRoot<AccountId>;
    type NestingBudget = ConstU32<200>;
    type ResourceSymbolLimit = ResourceSymbolLimit;
    type PartsLimit = PartsLimit;
    type MaxPriorities = MaxPriorities;
    type PropertiesLimit = PropertiesLimit;
    type CollectionSymbolLimit = CollectionSymbolLimit;
    type MaxResourcesOnMint = MaxResourcesOnMint;
    type TransferHooks = PhalaWrappedBalances;
    type WeightInfo = pallet_rmrk_core::weights::SubstrateWeight<Runtime>;
    type CollectionId = u32;
    type ItemId = u32;
}
impl pallet_phat::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type InkCodeSizeLimit = ConstU32<{ 1024 * 1024 * 2 }>;
    type SidevmCodeSizeLimit = ConstU32<{ 1024 * 1024 * 8 }>;
    type Currency = Balances;
}

pub struct WrappedBalancesPalletAccount;

impl Get<AccountId32> for WrappedBalancesPalletAccount {
    fn get() -> AccountId32 {
        (b"wpha/")
            .using_encoded(|b| AccountId32::decode(&mut TrailingZeroInput::new(b)))
            .expect("Decoding zero-padded account id should always succeed; qed")
    }
}

impl pallet_wrapped_balances::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WPhaAssetId = ConstU32<10000>;
    type WrappedBalancesAccountId = WrappedBalancesPalletAccount;
    type OnSlashed = Treasury;
}

pub struct MigrationAccount;

impl Get<AccountId32> for MigrationAccount {
    fn get() -> AccountId32 {
        let account: [u8; 32] =
            hex_literal::hex!("5492cf1c4c446223e1e26b29c5017a8cd4da21799cd7f497fab075b5567efd6f");
        account.into()
    }
}

impl pallet_base_pool::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MigrationAccountId = MigrationAccount;
}

parameter_types! {
    pub const AssetDeposit: Balance = 1 * CENTS; // 1 CENTS deposit to create asset
    pub const ApprovalDeposit: Balance = 1 * CENTS;
    pub const AssetsStringLimit: u32 = 50;
    pub const AssetAccountDeposit: u128 = 1 * DOLLARS;
    /// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
    // https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
    pub const MetadataDepositBase: Balance = deposit(1, 68);
    pub const MetadataDepositPerByte: Balance = deposit(0, 1);
}

impl pallet_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = u32;
    type AssetIdParameter = codec::Compact<u32>;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = ConstU128<10>;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
    type Freezer = ();
    type Extra = ();
    type CallbackHandle = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
    type RemoveItemsLimit = ConstU32<1000>;
}

impl pallet_phat_tokenomic::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
}

impl pallet_wapod_workers::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Crypto = pallet_wapod_workers::SpCrypto;
}

parameter_types! {
    pub const QueuePrefix: &'static [u8] = b"_queue/";
    pub const QueueCapacity: u32 = 128;
}

impl pallet_anchor::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnResponse = PhatOracle;
    type QueuePrefix = QueuePrefix;
    type QueueCapacity = QueueCapacity;
}
impl pallet_oracle::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
}

impl puppets::parachain_info::Config for Runtime {}
impl puppets::parachain_system::Config for Runtime {}

pub struct DealWithServiceFee;
impl OnUnbalanced<NegativeImbalance> for DealWithServiceFee {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        Treasury::on_unbalanced(amount);
    }
}

parameter_types! {
    pub EIP712Name: Vec<u8> = b"PhalaNetwork".to_vec();
    pub EIP712Version: Vec<u8> = b"2".to_vec();
    pub EIP712ChainID: pallet_evm_account_mapping::EIP712ChainID = sp_core::U256::from(0);
    pub EIP712VerifyingContractAddress: pallet_evm_account_mapping::EIP712VerifyingContractAddress = sp_core::H160::from([0u8; 20]);
}

impl pallet_evm_account_mapping::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type AddressConverter = pallet_evm_account_mapping::EvmTransparentConverter;
    type ServiceFee = ConstU128<10000000000>; // 0.01 PHA
    type OnUnbalancedForServiceFee = DealWithServiceFee;
    type CallFilter = frame_support::traits::Everything;
    type EIP712Name = EIP712Name;
    type EIP712Version = EIP712Version;
    type EIP712ChainID = EIP712ChainID;
    type EIP712VerifyingContractAddress = EIP712VerifyingContractAddress;
    type WeightInfo = pallet_evm_account_mapping::weights::SubstrateWeight<Runtime>;
}

construct_runtime!(
    pub struct Runtime {
        Assets: pallet_assets,
        System: frame_system,
        Utility: pallet_utility,
        Babe: pallet_babe,
        Timestamp: pallet_timestamp,
        Authorship: pallet_authorship,
        Indices: pallet_indices,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        ElectionProviderMultiPhase: pallet_election_provider_multi_phase,
        Staking: pallet_staking,
        FastUnstake: pallet_fast_unstake,
        VoterList: pallet_bags_list::<Instance1>,
        NominationPools: pallet_nomination_pools,
        Session: pallet_session,
        Democracy: pallet_democracy,
        Council: pallet_collective::<Instance1>,
        TechnicalCommittee: pallet_collective::<Instance2>,
        Elections: pallet_elections_phragmen,
        TechnicalMembership: pallet_membership::<Instance1>,
        Grandpa: pallet_grandpa,
        Treasury: pallet_treasury,
        Sudo: pallet_sudo,
        ImOnline: pallet_im_online,
        AuthorityDiscovery: pallet_authority_discovery,
        Offences: pallet_offences,
        Historical: pallet_session_historical,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
        Identity: pallet_identity,
        Society: pallet_society,
        Recovery: pallet_recovery,
        Vesting: pallet_vesting,
        Scheduler: pallet_scheduler,
        Preimage: pallet_preimage,
        Proxy: pallet_proxy,
        Multisig: pallet_multisig,
        Bounties: pallet_bounties,
        Tips: pallet_tips,
        Lottery: pallet_lottery,
        ChildBounties: pallet_child_bounties,
        // Phala
        PhalaMq: pallet_mq,
        PhalaRegistry: pallet_registry,
        PhalaComputation: pallet_computation,
        PhalaStakePoolv2: pallet_stake_pool_v2,
        PhalaStakePool: pallet_stake_pool,
        PhalaVault: pallet_vault,
        PhalaWrappedBalances: pallet_wrapped_balances,
        PhalaBasePool: pallet_base_pool,
        PhalaPhatContracts: pallet_phat,
        PhalaPhatTokenomic: pallet_phat_tokenomic,
        PhalaWapodWorkers: pallet_wapod_workers,

        // Rollup and Oracles
        PhatRollupAnchor: pallet_anchor = 100,
        PhatOracle: pallet_oracle = 101,

        // Put them here to make sure pherry could be compiled with phala's metadata.
        ParachainInfo: puppets::parachain_info,
        ParachainSystem: puppets::parachain_system,

        // NFT
        Uniques: pallet_uniques::{Pallet, Storage, Event<T>},
        RmrkCore: pallet_rmrk_core::{Pallet, Call, Event<T>},

        EvmAccountMapping: pallet_evm_account_mapping,
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, AccountIndex>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
///
/// When you change this, you **MUST** modify [`sign`] in `bin/node/testing/src/keyring.rs`!
///
/// [`sign`]: <../../testing/src/keyring.rs.html>
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_mq::CheckMqSequence<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    Migrations,
>;

// All migrations executed on runtime upgrade as a nested tuple of types implementing
// `OnRuntimeUpgrade`.
type Migrations = ();

pub struct MqCallMatcher;
impl pallet_mq::CallMatcher<Runtime> for MqCallMatcher {
    fn match_call(call: &RuntimeCall) -> Option<&pallet_mq::Call<Runtime>> {
        match call {
            RuntimeCall::PhalaMq(mq_call) => Some(mq_call),
            _ => None,
        }
    }
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> sp_consensus_grandpa::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            authority_id: GrandpaId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_grandpa::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_grandpa::OpaqueKeyOwnershipProof::new)
        }
    }

    impl pallet_nomination_pools_runtime_api::NominationPoolsApi<Block, AccountId, Balance> for Runtime {
        fn pending_rewards(who: AccountId) -> Balance {
            NominationPools::api_pending_rewards(who).unwrap_or_default()
        }

        fn points_to_balance(pool_id: pallet_nomination_pools::PoolId, points: Balance) -> Balance {
            NominationPools::api_points_to_balance(pool_id, points)
        }

        fn balance_to_points(pool_id: pallet_nomination_pools::PoolId, new_funds: Balance) -> Balance {
            NominationPools::api_balance_to_points(pool_id, new_funds)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeConfiguration {
            let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
            sp_consensus_babe::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: epoch_config.c,
                authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: epoch_config.allowed_slots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }

    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block, Balance,
    > for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<
        Block, Balance, RuntimeCall
    > for Runtime
    {
        fn query_call_info(call: RuntimeCall, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(call: RuntimeCall, len: u32) -> FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl pallet_mq_runtime_api::MqApi<Block> for Runtime {
        fn sender_sequence(sender: &phala_types::messaging::MessageOrigin) -> Option<u64> {
            PhalaMq::offchain_ingress(sender)
        }
    }

    impl pallet_wapod_workers::WapodWorkersApi<Block, Balance> for Runtime {
        fn balance_of_ticket(ticket_id: pallet_wapod_workers::TicketId) -> Balance {
            PhalaWapodWorkers::balance_of_ticket(ticket_id)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade(checks).unwrap();
            (weight, RuntimeBlockWeights::get().max_block)
        }

        fn execute_block(
            block: Block,
            state_root_check: bool,
            signature_check: bool,
            select: frame_try_runtime::TryStateSelect
        ) -> Weight {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here.
            Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_election_provider_support::NposSolution;
    use frame_system::offchain::CreateSignedTransaction;
    use sp_runtime::UpperOf;

    #[test]
    fn validate_transaction_submitter_bounds() {
        fn is_submit_signed_transaction<T>()
        where
            T: CreateSignedTransaction<RuntimeCall>,
        {
        }

        is_submit_signed_transaction::<Runtime>();
    }

    #[test]
    fn perbill_as_onchain_accuracy() {
        type OnChainAccuracy =
        <<Runtime as pallet_election_provider_multi_phase::MinerConfig>::Solution as NposSolution>::Accuracy;
        let maximum_chain_accuracy: Vec<UpperOf<OnChainAccuracy>> = (0..MaxNominations::get())
            .map(|_| <UpperOf<OnChainAccuracy>>::from(OnChainAccuracy::one().deconstruct()))
            .collect();
        let _: UpperOf<OnChainAccuracy> = maximum_chain_accuracy
            .iter()
            .fold(0, |acc, x| acc.checked_add(*x).unwrap());
    }

    #[test]
    fn call_size() {
        let size = core::mem::size_of::<RuntimeCall>();
        assert!(
            size <= CALL_PARAMS_MAX_SIZE,
            "size of RuntimeCall {size} is more than {CALL_PARAMS_MAX_SIZE} bytes.
             Some calls have too big arguments, use Box to reduce the size of RuntimeCall.
             If the limit is too strong, maybe consider increase the limit.",
        );
    }
}
