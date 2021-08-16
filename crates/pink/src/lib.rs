use frame_support::{
    parameter_types,
    traits::Currency,
    weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use pallet_contracts::{
    chain_extension::{
        ChainExtension, Environment, Ext, InitState, Result as ExtensionResult, RetVal, SysConfig,
        UncheckedFrom,
    },
    Config, Frame, Schedule,
};
use sp_runtime::{
    testing::{Header, H256},
    traits::{BlakeTwo256, Convert, Hash, IdentityLookup},
    AccountId32, Perbill,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Pink>;
type Block = frame_system::mocking::MockBlock<Pink>;
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

frame_support::construct_runtime!(
    pub enum Pink where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Randomness: pallet_randomness_collective_flip::{Pallet, Storage},
        Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>},
    }
);

pub struct NullExtension;

impl ChainExtension<Pink> for NullExtension {
    fn call<E>(func_id: u32, _env: Environment<E, InitState>) -> ExtensionResult<RetVal>
    where
        E: Ext<T = Pink>,
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        panic!("Unknown func_id: {}", func_id);
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(2 * WEIGHT_PER_SECOND);
    pub static ExistentialDeposit: u64 = 0;
}
impl frame_system::Config for Pink {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = BlockWeights;
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Call = Call;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

impl pallet_randomness_collective_flip::Config for Pink {}
impl pallet_balances::Config for Pink {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}
impl pallet_timestamp::Config for Pink {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const SignedClaimHandicap: u64 = 2;
    pub const TombstoneDeposit: u64 = 16;
    pub const DepositPerContract: u64 = 8 * DepositPerStorageByte::get();
    pub const DepositPerStorageByte: u64 = 10_000;
    pub const DepositPerStorageItem: u64 = 10_000;
    pub RentFraction: Perbill = Perbill::from_rational(4u32, 10_000u32);
    pub const SurchargeReward: u64 = 500_000;
    pub const MaxValueSize: u32 = 16_384;
    pub const DeletionQueueDepth: u32 = 1024;
    pub const DeletionWeightLimit: Weight = 500_000_000_000;
    pub const MaxCodeSize: u32 = 2 * 1024;
    pub MySchedule: Schedule<Pink> = <Schedule<Pink>>::default();
    pub const TransactionByteFee: u64 = 0;
}

impl Convert<Weight, BalanceOf<Self>> for Pink {
    fn convert(w: Weight) -> BalanceOf<Self> {
        w
    }
}

impl Config for Pink {
    type Time = Timestamp;
    type Randomness = Randomness;
    type Currency = Balances;
    type Event = Event;
    type Call = Call;
    type CallFilter = frame_support::traits::Everything;
    type RentPayment = ();
    type SignedClaimHandicap = SignedClaimHandicap;
    type TombstoneDeposit = TombstoneDeposit;
    type DepositPerContract = DepositPerContract;
    type DepositPerStorageByte = DepositPerStorageByte;
    type DepositPerStorageItem = DepositPerStorageItem;
    type RentFraction = RentFraction;
    type SurchargeReward = SurchargeReward;
    type CallStack = [Frame<Self>; 31];
    type WeightPrice = Self;
    type WeightInfo = ();
    type ChainExtension = NullExtension;
    type DeletionQueueDepth = DeletionQueueDepth;
    type DeletionWeightLimit = DeletionWeightLimit;
    type Schedule = MySchedule;
}

pub fn compile_wat<T>(wat_bytes: &[u8]) -> wat::Result<(Vec<u8>, <T::Hashing as Hash>::Output)>
where
    T: frame_system::Config,
{
    let wasm_binary = wat::parse_bytes(wat_bytes)?.into_owned();
    let code_hash = T::Hashing::hash(&wasm_binary);
    Ok((wasm_binary, code_hash))
}

pub mod exec {
    use sp_runtime::traits::BlakeTwo256;
    use sp_state_machine::{
        disabled_changes_trie_state, Backend, Ext, OverlayedChanges,
        StorageTransactionCache,
    };
    pub type InMemoryBackend = sp_state_machine::InMemoryBackend::<BlakeTwo256>;

    pub fn execute_with<R>(f: impl FnOnce() -> R) -> R {
        let state = InMemoryBackend::default();
        let backend = state.as_trie_backend().unwrap();

        let mut overlay = OverlayedChanges::default();
        overlay.start_transaction();
        let mut cache = StorageTransactionCache::default();
        let mut ext = Ext::new(
            &mut overlay,
            &mut cache,
            backend,
            disabled_changes_trie_state::<_, u64>(),
            None,
        );
        let r = sp_externalities::set_and_run_with_externalities(&mut ext, f);
        overlay.commit_transaction().expect("BUG: mis-paired transaction");
        r
    }
}
