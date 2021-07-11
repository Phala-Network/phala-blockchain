use crate::{mining, mq, registry};

use frame_support::{parameter_types, traits::GenesisBuild};
use frame_support_test::TestRandomness;
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
		PhalaMq: mq::{Pallet, Event},
		PhalaRegistry: registry::{Pallet, Event, Storage, Config<T>},
		PhalaMining: mining::{Pallet, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 20;
	pub const MinimumPeriod: u64 = 1;
	pub const ExpectedBlockTimeSec: u32 = 12;
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

// pub const HOURS: BlockNumber = 600;
// pub const DAYS: BlockNumber = HOURS * 24;
// pub const DOLLARS: Balance = 1_000_000_000_000;

impl mq::Config for Test {
	type Event = Event;
	type QueueNotifyConfig = ();
}

impl registry::Config for Test {
	type Event = Event;
	type UnixTime = Timestamp;
}

impl mining::Config for Test {
	type Event = Event;
	type ExpectedBlockTimeSec = ExpectedBlockTimeSec;
	type Currency = Balances;
	type Randomness = TestRandomness<Self>;
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	// Inject genesis storage
	let zero_pubkey = sp_core::ecdsa::Public::from_raw([0u8; 33]);
	let zero_ecdh_pubkey = Vec::from(&[0u8; 65][..]);
	crate::registry::GenesisConfig::<Test> {
		workers: vec![(zero_pubkey.clone(), zero_ecdh_pubkey, None)],
		gatekeepers: vec![(zero_pubkey.clone())],
		benchmark_duration: 0u32,
	}
	.assimilate_storage(&mut t)
	.unwrap();
	sp_io::TestExternalities::new(t)
}

pub fn set_block_1() {
	System::set_block_number(1);
}

pub fn events() -> Vec<Event> {
	let evt = System::events()
		.into_iter()
		.map(|evt| evt.event)
		.collect::<Vec<_>>();
	println!("event(): {:?}", evt);
	System::reset_events();
	evt
}
