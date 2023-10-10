mod extension;
mod pallet_pink;

use crate::types::{AccountId, Balance, BlockNumber, Hash, Hashing, Nonce};
use frame_support::{
    parameter_types,
    traits::{ConstBool, ConstU32},
    weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
};
use log::info;
use pallet_contracts::{
    migration::{v11, v12},
    weights::SubstrateWeight,
    Config, Frame, Migration, Schedule,
};
use sp_runtime::{traits::IdentityLookup, Perbill};

pub use extension::get_side_effects;
pub use pink_capi::types::ExecSideEffects;
pub use pink_extension::{EcdhPublicKey, HookPoint, Message, OspMessage, PinkEvent};

type Block = sp_runtime::generic::Block<
    sp_runtime::generic::Header<BlockNumber, Hashing>,
    frame_system::mocking::MockUncheckedExtrinsic<PinkRuntime>,
>;

pub type SystemEvents = Vec<frame_system::EventRecord<RuntimeEvent, Hash>>;

frame_support::construct_runtime! {
    pub struct PinkRuntime {
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
    pub const MaxHolds: u32 = 10;
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
    type FreezeIdentifier = ();
    type MaxHolds = MaxHolds;
    type MaxFreezes = ();
    type RuntimeHoldReason = RuntimeHoldReason;
}

impl frame_system::Config for PinkRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Nonce = Nonce;
    type Hash = Hash;
    type RuntimeCall = RuntimeCall;
    type Hashing = Hashing;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
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
        schedule.instruction_weights.base = 8000;
        schedule.limits.runtime_memory = 2048 * 1024 * 1024; // For unittests
        schedule.limits.payload_len = 1024 * 1024; // Max size for storage value
        schedule
    };
    pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(30);
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
    type WeightInfo = SubstrateWeight<Self>;
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
    type Migrations = (v11::Migration<Self>, v12::Migration<Self, Balances>);
    type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
    type MaxDelegateDependencies = ConstU32<32>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type Debug = ();
    type Environment = ();
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

/// Call on_genesis for all pallets. This would put the each pallet's storage version into
/// the frame_support::StorageVersion.
pub fn on_genesis() {
    <AllPalletsWithSystem as frame_support::traits::OnGenesis>::on_genesis();
}

/// Call on_runtime_upgrade for all pallets
pub fn on_runtime_upgrade() {
    use frame_support::traits::OnRuntimeUpgrade;
    type Migrations = (Migration<PinkRuntime>, AllPalletsWithSystem);
    Migrations::on_runtime_upgrade();
    info!("Runtime database migration done");
}

/// Call on_idle for each pallet.
pub fn on_idle(n: BlockNumber) {
    <AllPalletsWithSystem as frame_support::traits::OnIdle<BlockNumber>>::on_idle(n, Weight::MAX);
}
