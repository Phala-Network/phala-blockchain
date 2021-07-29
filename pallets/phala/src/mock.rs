use crate::{mining, mq, registry, stakepool};

use frame_support::{parameter_types, traits::GenesisBuild};
use frame_support_test::TestRandomness;
use frame_system as system;
use phala_types::messaging::Message;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

pub(crate) type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;

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
		PhalaMq: mq::{Pallet},
		PhalaRegistry: registry::{Pallet, Event, Storage, Config<T>},
		PhalaMining: mining::{Pallet, Event<T>, Storage, Config},
		PhalaStakePool: stakepool::{Pallet, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 20;
	pub const MinimumPeriod: u64 = 1;
	pub const ExpectedBlockTimeSec: u32 = 12;
	pub const MinMiningStaking: Balance = 1 * DOLLARS;
	pub const MinContribution: Balance = 1 * CENTS;
	pub const MiningInsurancePeriod: u64 = 3 * DAYS;
}
impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::AllowAll;
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
pub const CENTS: Balance = DOLLARS / 100;

impl mq::Config for Test {
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
	type OnReward = PhalaStakePool;
	type OnUnbound = PhalaStakePool;
	type OnReclaim = PhalaStakePool;
}

impl stakepool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type MinContribution = MinContribution;
	type InsurancePeriod = MiningInsurancePeriod;
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	// Inject genesis storage
	let zero_pubkey = sp_core::sr25519::Public::from_raw([0u8; 32]);
	let zero_ecdh_pubkey = Vec::from(&[0u8; 32][..]);
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 1000 * DOLLARS),
			(2, 2000 * DOLLARS),
			(3, 1000 * DOLLARS),
			(99, 1_000_000 * DOLLARS),
			(PhalaMining::account_id(), 690_000_000 * DOLLARS),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	crate::registry::GenesisConfig::<Test> {
		workers: vec![(zero_pubkey.clone(), zero_ecdh_pubkey, None)],
		gatekeepers: vec![(zero_pubkey.clone())],
		benchmark_duration: 0u32,
	}
	.assimilate_storage(&mut t)
	.unwrap();
	GenesisBuild::<Test>::assimilate_storage(&crate::mining::GenesisConfig::default(), &mut t)
		.unwrap();
	sp_io::TestExternalities::new(t)
}

pub fn set_block_1() {
	System::set_block_number(1);
}

pub fn take_events() -> Vec<Event> {
	let evt = System::events()
		.into_iter()
		.map(|evt| evt.event)
		.collect::<Vec<_>>();
	println!("event(): {:?}", evt);
	System::reset_events();
	evt
}

pub fn take_messages() -> Vec<Message> {
	let messages = PhalaMq::messages();
	println!("messages(): {:?}", messages);
	mq::OutboundMessages::<Test>::kill();
	messages
}

use phala_types::{EcdhPublicKey, WorkerPublicKey};

pub fn worker_pubkey(i: u8) -> WorkerPublicKey {
	let mut raw = [0u8; 32];
	raw[31] = i;
	raw[30] = 1; // distinguish with the genesis config
	WorkerPublicKey::from_raw(raw)
}
pub fn ecdh_pubkey(i: u8) -> EcdhPublicKey {
	let mut raw = [0u8; 32];
	raw[31] = i;
	raw[30] = 1; // distinguish with the genesis config
	EcdhPublicKey(raw)
}

/// Sets up `n` workers starting from 1, registered and benchmarked. All owned by account1.
pub fn setup_workers(n: u8) {
	use frame_support::assert_ok;
	for i in 1..=n {
		let worker = worker_pubkey(i);
		assert_ok!(PhalaRegistry::force_register_worker(
			Origin::root(),
			worker.clone(),
			ecdh_pubkey(1),
			Some(1)
		));
		PhalaRegistry::internal_set_benchmark(&worker, Some(1));
	}
}

/// Sets up `n` workers starting from 1, registered and benchmarked, owned by the corresponding
/// accounts.
pub fn setup_workers_linked_operators(n: u8) {
	use frame_support::assert_ok;
	for i in 1..=n {
		let worker = worker_pubkey(i);
		assert_ok!(PhalaRegistry::force_register_worker(
			Origin::root(),
			worker.clone(),
			ecdh_pubkey(1),
			Some(i as _)
		));
		PhalaRegistry::internal_set_benchmark(&worker, Some(1));
	}
}

pub fn elapse_cool_down() {
	let now = Timestamp::get();
	Timestamp::set_timestamp(now + PhalaMining::cool_down_period());
}
