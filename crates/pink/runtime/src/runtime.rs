mod extension;
mod pallet_pink;
mod weights;

use crate::types::{AccountId, Balance, BlockNumber, Hash, Hashing, Index};
use frame_support::{
    parameter_types,
    traits::ConstBool,
    weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
};
use pallet_contracts::{Config, Frame, Schedule};
use sp_runtime::{generic::Header, traits::IdentityLookup, Perbill};

pub use extension::get_side_effects;
pub use pink_capi::types::ExecSideEffects;
pub use pink_extension::{EcdhPublicKey, HookPoint, Message, OspMessage, PinkEvent};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<PinkRuntime>;
type Block = frame_system::mocking::MockBlock<PinkRuntime>;

frame_support::construct_runtime! {
    pub enum PinkRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        Randomness: pallet_insecure_randomness_collective_flip,
        Contracts: pallet_contracts,
        Pink: pallet_pink,
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub RuntimeBlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
            NORMAL_DISPATCH_RATIO,
        );
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_pink::Config for PinkRuntime {
    type Currency = Balances;
}

impl pallet_balances::Config for PinkRuntime {
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Pallet<PinkRuntime>;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<PinkRuntime>;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type MaxHolds = ();
    type MaxFreezes = ();
}

impl frame_system::Config for PinkRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type RuntimeCall = RuntimeCall;
    type Hashing = Hashing;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header<Self::BlockNumber, Self::Hashing>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_insecure_randomness_collective_flip::Config for PinkRuntime {}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for PinkRuntime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// Workaround for the test failure in runtime_integrity_tests
// https://github.com/paritytech/substrate/pull/12993/files#diff-67684005af418e25ff88c2ae5b520f0c040371f1d817e03a3652e76b9485224aR1217
#[cfg(test)]
const MAX_CODE_LEN: u32 = 64 * 1024;
#[cfg(not(test))]
const MAX_CODE_LEN: u32 = 2 * 1024 * 1024;

parameter_types! {
    pub DepositPerStorageByte: Balance = Pink::deposit_per_byte();
    pub DepositPerStorageItem: Balance = Pink::deposit_per_item();
    pub const DefaultDepositLimit: Balance = Balance::max_value();
    pub const MaxCodeLen: u32 = MAX_CODE_LEN;
    pub const MaxStorageKeyLen: u32 = 128;
    pub const MaxDebugBufferLen: u32 = 128 * 1024;

    pub DefaultSchedule: Schedule<PinkRuntime> = {
        let mut schedule = Schedule::<PinkRuntime>::default();
        const MB: u32 = 16;  // 64KiB * 16
        // Each concurrent query would create a VM instance to serve it. We couldn't
        // allocate too much here.
        schedule.limits.memory_pages = 4 * MB;
        schedule.instruction_weights.fallback = 8000;
        schedule
    };
}

impl Config for PinkRuntime {
    type Time = Timestamp;
    type Randomness = Randomness;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type CallFilter = frame_support::traits::Nothing;
    type CallStack = [Frame<Self>; 5];
    type WeightPrice = Pink;
    type WeightInfo = weights::PinkWeights<Self>;
    type ChainExtension = extension::PinkExtension;
    type Schedule = DefaultSchedule;
    type DepositPerByte = DepositPerStorageByte;
    type DepositPerItem = DepositPerStorageItem;
    type DefaultDepositLimit = DefaultDepositLimit;
    type AddressGenerator = Pink;
    type MaxCodeLen = MaxCodeLen;
    type MaxStorageKeyLen = MaxStorageKeyLen;
    type UnsafeUnstableInterface = ConstBool<false>;
    type MaxDebugBufferLen = MaxDebugBufferLen;
}

#[test]
fn detect_parameter_changes() {
    insta::assert_debug_snapshot!((
        <PinkRuntime as frame_system::Config>::BlockWeights::get(),
        <PinkRuntime as Config>::Schedule::get(),
        <PinkRuntime as Config>::DefaultDepositLimit::get(),
        <PinkRuntime as Config>::MaxCodeLen::get(),
        <PinkRuntime as Config>::MaxStorageKeyLen::get(),
    ));
}
