mod extension;
mod pallet_pink;
mod weights;

use std::time::{Duration, Instant};

use crate::types::{AccountId, Balance, BlockNumber, Hash, Hashing, Index};
use frame_support::{parameter_types, weights::Weight};
use pallet_contracts::{Config, Frame, Schedule};
use sp_runtime::{generic::Header, traits::IdentityLookup};

pub use extension::{get_side_effects, ExecSideEffects};
pub use pink_extension::{HookPoint, Message, OspMessage, PinkEvent};

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
        Randomness: pallet_randomness_collective_flip,
        Contracts: pallet_contracts,
        Pink: pallet_pink,
    }
}

const WEIGHT_PER_SECOND: Weight = Weight::from_ref_time(1_000_000_000_000);

const UNIT: Balance = 1_000_000_000_000;
const DOLLARS: Balance = UNIT;
const CENTS: Balance = DOLLARS / 100;

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub RuntimeBlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(WEIGHT_PER_SECOND.saturating_mul(2));
    pub const ExistentialDeposit: Balance = CENTS;
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

impl pallet_randomness_collective_flip::Config for PinkRuntime {}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for PinkRuntime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub DepositPerStorageByte: Balance = Pink::deposit_per_byte();
    pub DepositPerStorageItem: Balance = Pink::deposit_per_item();
    pub const DeletionQueueDepth: u32 = 1024;
    pub const DeletionWeightLimit: Weight = Weight::from_ref_time(500_000_000_000);
    pub const MaxCodeLen: u32 = 2 * 1024 * 1024;
    pub const MaxStorageKeyLen: u32 = 128;

    pub DefaultSchedule: Schedule<PinkRuntime> = {
        let mut schedule = Schedule::<PinkRuntime>::default();
        const MB: u32 = 16;  // 64KiB * 16
        // Each concurrent query would create a VM instance to serve it. We couldn't
        // allocate too much here.
        schedule.limits.memory_pages = 4 * MB;
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
    type CallStack = [Frame<Self>; 31];
    type WeightPrice = Pink;
    type WeightInfo = weights::PinkWeights<Self>;
    type ChainExtension = extension::PinkExtension;
    type DeletionQueueDepth = DeletionQueueDepth;
    type DeletionWeightLimit = DeletionWeightLimit;
    type Schedule = DefaultSchedule;
    type DepositPerByte = DepositPerStorageByte;
    type DepositPerItem = DepositPerStorageItem;
    type AddressGenerator = Pink;
    type ContractAccessWeight = pallet_contracts::DefaultContractAccessWeight<RuntimeBlockWeights>;
    type MaxCodeLen = MaxCodeLen;
    type MaxStorageKeyLen = MaxStorageKeyLen;
}

#[test]
fn detect_parameter_changes() {
    use sp_core::Get;
    insta::assert_debug_snapshot!((
        <PinkRuntime as frame_system::Config>::BlockWeights::get(),
        <PinkRuntime as Config>::Schedule::get(),
        <PinkRuntime as Config>::ContractAccessWeight::get(),
        <PinkRuntime as Config>::DeletionQueueDepth::get(),
        <PinkRuntime as Config>::DeletionWeightLimit::get(),
        <PinkRuntime as Config>::MaxCodeLen::get(),
        <PinkRuntime as Config>::MaxStorageKeyLen::get(),
    ));
}

#[derive(Clone, Copy)]
pub enum CallMode {
    Query,
    Command,
}

pub trait EventCallbacks {
    fn emit_log(&self, contract: &AccountId, in_query: bool, level: u8, message: String);
}

pub type BoxedEventCallbacks = Box<dyn EventCallbacks>;

struct CallInfo {
    mode: CallMode,
    start_at: Instant,
    callbacks: Option<BoxedEventCallbacks>,
}

environmental::environmental!(call_info: CallInfo);

pub fn using_mode<T>(
    mode: CallMode,
    callbacks: Option<BoxedEventCallbacks>,
    f: impl FnOnce() -> T,
) -> T {
    let mut info = CallInfo {
        mode,
        start_at: Instant::now(),
        callbacks,
    };
    call_info::using(&mut info, f)
}

pub fn get_call_mode() -> Option<CallMode> {
    call_info::with(|info| info.mode)
}

pub fn get_call_elapsed() -> Option<Duration> {
    call_info::with(|info| info.start_at.elapsed())
}

pub fn emit_log(id: &AccountId, level: u8, msg: String) {
    call_info::with(|info| {
        if let Some(callbacks) = &info.callbacks {
            callbacks.emit_log(id, matches!(info.mode, CallMode::Query), level, msg);
        }
    });
}

#[cfg(test)]
mod tests {
    #![allow(clippy::type_complexity)]

    use frame_support::assert_ok;
    use pallet_contracts::Config;
    use sp_runtime::{traits::Hash, AccountId32};

    use crate::{
        runtime::{Contracts, PinkRuntime, RuntimeOrigin as Origin},
        types::ENOUGH,
    };
    pub use frame_support::weights::Weight;

    pub fn compile_wat<T>(wat_bytes: &[u8]) -> wat::Result<(Vec<u8>, <T::Hashing as Hash>::Output)>
    where
        T: frame_system::Config,
    {
        let wasm_binary = wat::parse_bytes(wat_bytes)?.into_owned();
        let code_hash = T::Hashing::hash(&wasm_binary);
        Ok((wasm_binary, code_hash))
    }

    #[test]
    pub fn contract_test() {
        use scale::Encode;
        pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

        let (wasm, code_hash) =
            compile_wat::<PinkRuntime>(include_bytes!("../tests/fixtures/event_size.wat")).unwrap();

        exec::execute_with(|| {
            Contracts::instantiate_with_code(
                Origin::signed(ALICE),
                ENOUGH,
                Weight::MAX,
                None,
                wasm,
                vec![],
                vec![],
            )
            .unwrap();
            let addr = Contracts::contract_address(&ALICE, &code_hash, &[]);

            Contracts::call(
                Origin::signed(ALICE),
                addr,
                0,
                Weight::MAX,
                None,
                <PinkRuntime as Config>::Schedule::get()
                    .limits
                    .payload_len
                    .encode(),
            )
            .unwrap();
        });
        log::info!("contract OK");
    }

    #[test]
    pub fn crypto_hashes_test() {
        pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
        const GAS_LIMIT: Weight = Weight::from_ref_time(1_000_000_000_000_000);

        let (wasm, code_hash) =
            compile_wat::<PinkRuntime>(include_bytes!("../tests/fixtures/crypto_hashes.wat"))
                .unwrap();

        exec::execute_with(|| {
            // Instantiate the CRYPTO_HASHES contract.
            assert_ok!(Contracts::instantiate_with_code(
                Origin::signed(ALICE),
                1_000_000_000_000_000,
                GAS_LIMIT,
                None,
                wasm,
                vec![],
                vec![],
            ));
            let addr = Contracts::contract_address(&ALICE, &code_hash, &[]);
            // Perform the call.
            let input = b"_DEAD_BEEF";
            use sp_io::hashing::*;
            // Wraps a hash function into a more dynamic form usable for testing.
            macro_rules! dyn_hash_fn {
                ($name:ident) => {
                    Box::new(|input| $name(input).as_ref().to_vec().into_boxed_slice())
                };
            }
            // All hash functions and their associated output byte lengths.
            let test_cases: &[(Box<dyn Fn(&[u8]) -> Box<[u8]>>, usize)] = &[
                (dyn_hash_fn!(sha2_256), 32),
                (dyn_hash_fn!(keccak_256), 32),
                (dyn_hash_fn!(blake2_256), 32),
                (dyn_hash_fn!(blake2_128), 16),
            ];
            // Test the given hash functions for the input: "_DEAD_BEEF"
            for (n, (hash_fn, expected_size)) in test_cases.iter().enumerate() {
                // We offset data in the contract tables by 1.
                let mut params = vec![(n + 1) as u8];
                params.extend_from_slice(input);
                let result =
                    Contracts::bare_call(ALICE, addr.clone(), 0, GAS_LIMIT, None, params, false)
                        .result
                        .unwrap();
                assert!(!result.did_revert());
                let expected = hash_fn(input.as_ref());
                assert_eq!(&result.data[..*expected_size], &*expected);
            }
        })
    }

    pub mod exec {
        use sp_runtime::traits::BlakeTwo256;
        use sp_state_machine::{
            backend::AsTrieBackend, Ext, OverlayedChanges, StorageTransactionCache,
        };

        pub type InMemoryBackend = sp_state_machine::InMemoryBackend<BlakeTwo256>;

        pub fn execute_with<R>(f: impl FnOnce() -> R) -> R {
            let state = InMemoryBackend::default();
            let backend = state.as_trie_backend();

            let mut overlay = OverlayedChanges::default();
            overlay.start_transaction();
            let mut cache = StorageTransactionCache::default();
            let mut ext = Ext::new(&mut overlay, &mut cache, backend, None);
            let r = sp_externalities::set_and_run_with_externalities(&mut ext, f);
            overlay
                .commit_transaction()
                .expect("BUG: mis-paired transaction");
            r
        }
    }
}
