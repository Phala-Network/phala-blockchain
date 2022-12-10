use crate::{anchor, oracle};

use frame_support::{pallet_prelude::ConstU32, parameter_types};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

pub(crate) type Balance = u128;

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
		// Pallets to test
		Anchor: anchor::{Pallet, Call, Storage, Event<T>},
		Oracle: oracle::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const ExistentialDeposit: u64 = 2;
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 20;
	pub const MinimumPeriod: u64 = 1;
	pub const ExpectedBlockTimeSec: u32 = 12;
	pub const MinMiningStaking: Balance = DOLLARS;
	pub const MinContribution: Balance = CENTS;
	pub const MiningGracePeriod: u64 = 7 * 24 * 3600;
	pub const MinInitP: u32 = 1;
	pub const MiningEnabledByDefault: bool = true;
	pub const MaxPoolWorkers: u32 = 10;
	pub const NoneAttestationEnabled: bool = true;
	pub const VerifyPRuntime: bool = false;
	pub const VerifyRelaychainGenesisBlockHash: bool = true;
}
impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
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
	type MaxConsumers = ConstU32<2>;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
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

impl anchor::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OnResponse = Oracle;
}

impl oracle::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

pub const DOLLARS: Balance = 1_000_000_000_000;
pub const CENTS: Balance = DOLLARS / 100;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	// Inject genesis storage
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 1000 * DOLLARS),
			(2, 2000 * DOLLARS),
			(3, 1000 * DOLLARS),
			(99, 1_000_000 * DOLLARS),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	sp_io::TestExternalities::new(t)
}

pub fn set_block_1() {
	System::set_block_number(1);
}

pub fn take_events() -> Vec<RuntimeEvent> {
	let evt = System::events()
		.into_iter()
		.map(|evt| evt.event)
		.collect::<Vec<_>>();
	println!("event(): {evt:?}");
	System::reset_events();
	evt
}

use frame_support::BoundedVec;
use sp_runtime::traits::Get;
pub(crate) fn bvec<S: Get<u32>>(raw: &[u8]) -> BoundedVec<u8, S> {
	BoundedVec::<u8, S>::truncate_from(raw.to_owned())
}
