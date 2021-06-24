// Creating mock runtime here

use crate::phala_legacy as phala;
use frame_support::parameter_types;
use frame_support_test::TestRandomness;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Permill,
};

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
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Phala: phala::{Pallet, Call, Config<T>, Storage, Event<T>},
		PhalaMq: super::mq::{Pallet, Event},
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
	type OnSetCode = ();
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ();
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
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
	pub const ComputeRewardPercentage: Permill = Permill::from_parts(625_000);
	pub const OfflineOffenseSlash: Balance = 100 * DOLLARS;
	pub const OfflineReportReward: Balance = 50 * DOLLARS;
}

impl crate::mq::Config for Test {
    type Event = Event;
    type QueueNotifyConfig = ();
}

impl phala::Config for Test {
	type Event = Event;
	type Randomness = TestRandomness<Self>;
	type TEECurrency = Balances;
	type UnixTime = Timestamp;
	type Treasury = ();
	type WeightInfo = ();
	type OnRoundEnd = ();

	// Parameters
	type MaxHeartbeatPerWorkerPerHour = MaxHeartbeatPerWorkerPerHour;
	type RoundInterval = RoundInterval;
	type DecayInterval = DecayInterval;
	type DecayFactor = DecayFactor;
	type InitialReward = InitialReward;
	type TreasuryRation = TreasuryRation;
	type RewardRation = RewardRation;
	type OnlineRewardPercentage = OnlineRewardPercentage;
	type ComputeRewardPercentage = ComputeRewardPercentage;
	type OfflineOffenseSlash = OfflineOffenseSlash;
	type OfflineReportReward = OfflineReportReward;
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	super::GenesisConfig::<Test> {
		stakers: Default::default(),
		contract_keys: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();
	sp_io::TestExternalities::new(t)
}
