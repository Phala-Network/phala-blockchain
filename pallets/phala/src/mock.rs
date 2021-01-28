// Creating mock runtime here

use crate as phala;
use sp_core::H256;
use sp_runtime::{
	Permill,
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;
use frame_support::{parameter_types, weights::Weight, traits::TestRandomness};
use frame_support::traits::Randomness;

pub(crate) type Balance = u128;
pub(crate) type BlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		PhalaModule: phala::{Module, Call, Config<T>, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const MinimumPeriod: u64 = 1;
}
impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ();
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

pub const HOURS: BlockNumber = 600;
pub const DAYS: BlockNumber = HOURS * 24;
pub const DOLLARS: Balance = 1_000_000_000_000;

parameter_types! {
	pub const MaxHeartbeatPerWorkerPerHour: u32 = 2;
	pub const RoundInterval: BlockNumber = 1 * HOURS;
	pub const DecayInterval: BlockNumber = 180 * DAYS;
	pub const DecayFactor: Permill = Permill::from_percent(75);
	pub const InitialReward: Balance = 129600000 * DOLLARS;
	pub const TreasuryRation: u32 = 20_000;
	pub const RewardRation: u32 = 80_000;
	pub const OnlineRewardPercentage: Permill = Permill::from_parts(375_000);
}

impl phala::Config for Test {
	type Event = Event;
	type Randomness = TestRandomness;
	type TEECurrency = Balances;
	type UnixTime = Timestamp;
	type Treasury = ();
	type ModuleWeightInfo = ();

	// Parameters
	type MaxHeartbeatPerWorkerPerHour = MaxHeartbeatPerWorkerPerHour;
	type RoundInterval = RoundInterval;
	type DecayInterval = DecayInterval;
	type DecayFactor = DecayFactor;
	type InitialReward = InitialReward;
	type TreasuryRation = TreasuryRation;
	type RewardRation = RewardRation;
	type OnlineRewardPercentage = OnlineRewardPercentage;
}

// mod test_events {
// 	pub use crate::Event;
// }

// pub type System = frame_system::Module<Test>;
// pub type Balances = pallet_balances::Module<Test>;
// pub type Randomness = pallet_randomness_collective_flip::Module<Test>;
// pub type PhalaModule = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	crate::GenesisConfig::<Test> {
		stakers: Default::default(),
		contract_keys: Default::default()
	}.assimilate_storage(&mut t).unwrap();
	sp_io::TestExternalities::new(t)
}
