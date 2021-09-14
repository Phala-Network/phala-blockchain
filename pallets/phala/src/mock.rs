use crate::{
	attestation::{Attestation, AttestationValidator, Error as AttestationError, IasFields},
	mining, mq, registry, stakepool,
};

use frame_support::{
	parameter_types,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
};
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
pub(crate) type BlockNumber = u64;

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
		PhalaMq: mq::{Pallet, Call},
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
	pub const MiningGracePeriod: u64 = 7 * 24 * 3600;
	pub const MinInitP: u32 = 1;
	pub const MiningEnabledByDefault: bool = true;
	pub const MaxPoolWorkers: u32 = 10;
	pub const VerifyPRuntime: bool = false;
	pub const VerifyRelaychainGenesisBlockHash: bool = true;
}
impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
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

pub const DOLLARS: Balance = 1_000_000_000_000;
pub const CENTS: Balance = DOLLARS / 100;

impl mq::Config for Test {
	type QueueNotifyConfig = ();
	type CallMatcher = MqCallMatcher;
}

pub struct MqCallMatcher;
impl mq::CallMatcher<Test> for MqCallMatcher {
	fn match_call(call: &Call) -> Option<&mq::Call<Test>> {
		match call {
			Call::PhalaMq(mq_call) => Some(mq_call),
			_ => None,
		}
	}
}

impl registry::Config for Test {
	type Event = Event;
	type AttestationValidator = MockValidator;
	type UnixTime = Timestamp;
	type VerifyPRuntime = VerifyPRuntime;
	type VerifyRelaychainGenesisBlockHash = VerifyRelaychainGenesisBlockHash;
	type GovernanceOrigin = frame_system::EnsureRoot<Self::AccountId>;
}

impl mining::Config for Test {
	type Event = Event;
	type ExpectedBlockTimeSec = ExpectedBlockTimeSec;
	type MinInitP = MinInitP;
	type Currency = Balances;
	type Randomness = TestRandomness<Self>;
	type OnReward = PhalaStakePool;
	type OnUnbound = PhalaStakePool;
	type OnReclaim = PhalaStakePool;
	type OnStopped = PhalaStakePool;
	type OnTreasurySettled = ();
	type UpdateTokenomicOrigin = frame_system::EnsureRoot<Self::AccountId>;
}

impl stakepool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type MinContribution = MinContribution;
	type GracePeriod = MiningGracePeriod;
	type MiningEnabledByDefault = MiningEnabledByDefault;
	type MaxPoolWorkers = MaxPoolWorkers;
	type OnSlashed = ();
	type MiningSwitchOrigin = frame_system::EnsureRoot<Self::AccountId>;
}

pub struct MockValidator;
impl AttestationValidator for MockValidator {
	fn validate(
		_attestation: &Attestation,
		_user_data_hash: &[u8; 32],
		_now: u64,
		_verify_pruntime: bool,
		_pruntime_allowlist: Vec<Vec<u8>>,
	) -> Result<IasFields, AttestationError> {
		Ok(IasFields {
			mr_enclave: [0u8; 32],
			mr_signer: [0u8; 32],
			isv_prod_id: [0u8; 2],
			isv_svn: [0u8; 2],
			report_data: [0u8; 64],
			confidence_level: 128u8,
		})
	}
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

pub fn setup_relaychain_genesis_allowlist() {
	use frame_support::assert_ok;
	let sample: H256 = H256::repeat_byte(1);
	assert_ok!(PhalaRegistry::add_relaychain_genesis_block_hash(
		Origin::root(),
		sample
	));
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

pub fn elapse_seconds(sec: u64) {
	let now = Timestamp::get();
	Timestamp::set_timestamp(now + sec * 1000);
}

pub fn elapse_cool_down() {
	let now = Timestamp::get();
	Timestamp::set_timestamp(now + PhalaMining::cool_down_period() * 1000);
}

pub fn teleport_to_block(n: u64) {
	let now = System::block_number();
	PhalaStakePool::on_finalize(now);
	PhalaMining::on_finalize(now);
	PhalaRegistry::on_finalize(now);
	PhalaMq::on_finalize(now);
	System::on_finalize(now);
	System::set_block_number(n);
	System::on_initialize(System::block_number());
	PhalaMq::on_initialize(System::block_number());
	PhalaRegistry::on_initialize(System::block_number());
	PhalaMining::on_initialize(System::block_number());
	PhalaStakePool::on_initialize(System::block_number());
}
