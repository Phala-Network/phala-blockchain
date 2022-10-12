use crate::{
	attestation::{Attestation, AttestationValidator, Error as AttestationError, IasFields},
	fat, fat_tokenomic, mq, registry,
};

use frame_support::{pallet_prelude::ConstU32, parameter_types, traits::GenesisBuild};
use frame_system as system;
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
		// Phala pallets
		PhalaMq: mq::{Pallet, Call},
		PhalaRegistry: registry::{Pallet, Event<T>, Storage, Config<T>},
		FatContracts: fat,
		FatTokenomic: fat_tokenomic,
	}
);

parameter_types! {
	pub const ExistentialDeposit: u64 = 2;
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 20;
	pub const MinimumPeriod: u64 = 1;
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
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sp_core::crypto::AccountId32;
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

pub const DOLLARS: Balance = 1_000_000_000_000;

impl mq::Config for Test {
	type QueueNotifyConfig = ();
	type CallMatcher = MqCallMatcher;
}

pub struct MqCallMatcher;
impl mq::CallMatcher<Test> for MqCallMatcher {
	fn match_call(call: &RuntimeCall) -> Option<&mq::Call<Test>> {
		match call {
			RuntimeCall::PhalaMq(mq_call) => Some(mq_call),
			_ => None,
		}
	}
}

impl registry::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AttestationValidator = MockValidator;
	type UnixTime = Timestamp;
	type VerifyPRuntime = VerifyPRuntime;
	type VerifyRelaychainGenesisBlockHash = VerifyRelaychainGenesisBlockHash;
	type GovernanceOrigin = frame_system::EnsureRoot<Self::AccountId>;
}

impl fat::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type InkCodeSizeLimit = ConstU32<{ 1024 * 1024 }>;
	type SidevmCodeSizeLimit = ConstU32<{ 1024 * 1024 }>;
}

impl fat_tokenomic::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
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

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let zero_pubkey = sp_core::sr25519::Public::from_raw([0u8; 32]);
	let zero_ecdh_pubkey = Vec::from(&[0u8; 32][..]);
	crate::registry::GenesisConfig::<Test> {
		workers: vec![(zero_pubkey.clone(), zero_ecdh_pubkey, None)],
		gatekeepers: vec![(zero_pubkey.clone())],
		benchmark_duration: 0u32,
	}
	.assimilate_storage(&mut t)
	.unwrap();
	sp_io::TestExternalities::new(t)
}

pub fn take_events() -> Vec<RuntimeEvent> {
	let evt = System::events()
		.into_iter()
		.map(|evt| evt.event)
		.collect();
	System::reset_events();
	evt
}
