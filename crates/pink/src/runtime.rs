mod mock_types;

use frame_support::{parameter_types, weights::constants::WEIGHT_PER_SECOND};
use pallet_contracts::{
    chain_extension::{
        ChainExtension, Environment, Ext, InitState, Result as ExtensionResult, RetVal, SysConfig,
        UncheckedFrom,
    },
    Config, Frame, Schedule,
};
use sp_runtime::{Perbill, generic::Header, traits::{Convert, IdentityLookup}};
use frame_support::weights::Weight;
use crate::types::{Balance, Hash, BlockNumber, Hashing, AccountId, Index};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<PinkRuntime>;
type Block = frame_system::mocking::MockBlock<PinkRuntime>;

frame_support::construct_runtime! {
    pub enum PinkRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Randomness: pallet_randomness_collective_flip::{Pallet, Storage},
        Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>},
    }
}

pub struct NoExtension;

impl ChainExtension<PinkRuntime> for NoExtension {
    fn call<E>(func_id: u32, _env: Environment<E, InitState>) -> ExtensionResult<RetVal>
    where
        E: Ext<T = PinkRuntime>,
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        panic!("Unknown func_id: {}", func_id);
    }
}

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(2 * WEIGHT_PER_SECOND);
    pub static ExistentialDeposit: u64 = 0;
}

impl frame_system::Config for PinkRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = BlockWeights;
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Call = Call;
    type Hashing = Hashing;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header<Self::BlockNumber, Self::Hashing>;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
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
    pub const SignedClaimHandicap: u32 = 2;
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
    pub DefaultSchedule: Schedule<PinkRuntime> = <Schedule<PinkRuntime>>::default();
    pub const TransactionByteFee: u64 = 0;
}

impl Convert<Weight, Balance> for PinkRuntime {
    fn convert(w: Weight) -> Balance {
        w as _
    }
}

impl Config for PinkRuntime {
    type Time = Timestamp;
    type Randomness = Randomness;
    type Currency = mock_types::NoCurrency;
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
    type ChainExtension = NoExtension;
    type DeletionQueueDepth = DeletionQueueDepth;
    type DeletionWeightLimit = DeletionWeightLimit;
    type Schedule = DefaultSchedule;
}

#[cfg(test)]
mod tests {
    use pallet_contracts::Config;
    use sp_runtime::{traits::Hash, AccountId32};

    use crate::runtime::{Balance, Contracts, Origin, PinkRuntime};
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
        use codec::Encode;
        pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
        const GAS_LIMIT: Weight = 10_000_000_000;
        const ENOUGH: Balance = Balance::MAX / 2;

        let (wasm, code_hash) =
            compile_wat::<PinkRuntime>(include_bytes!("../fixtures/event_size.wat")).unwrap();

        exec::execute_with(|| {
            Contracts::instantiate_with_code(
                Origin::signed(ALICE),
                ENOUGH,
                GAS_LIMIT,
                wasm,
                vec![],
                vec![],
            )
            .unwrap();
            let addr = Contracts::contract_address(&ALICE, &code_hash, &[]);

            Contracts::call(
                Origin::signed(ALICE),
                addr.clone(),
                0,
                GAS_LIMIT * 2,
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
        const GAS_LIMIT: Weight = 10_000_000_000;

        let (wasm, code_hash) =
            compile_wat::<PinkRuntime>(include_bytes!("../fixtures/crypto_hashes.wat")).unwrap();

        exec::execute_with(|| {
            // Instantiate the CRYPTO_HASHES contract.
            assert!(Contracts::instantiate_with_code(
                Origin::signed(ALICE),
                1_000_000_000_000_000,
                GAS_LIMIT,
                wasm,
                vec![],
                vec![],
            )
            .is_ok());
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
                let result = Contracts::bare_call(ALICE, addr.clone(), 0, GAS_LIMIT, params, false)
                    .result
                    .unwrap();
                assert!(result.is_success());
                let expected = hash_fn(input.as_ref());
                assert_eq!(&result.data[..*expected_size], &*expected);
            }
        })
    }

    pub mod exec {
        use sp_runtime::traits::BlakeTwo256;
        use sp_state_machine::{
            disabled_changes_trie_state, Backend, Ext, OverlayedChanges, StorageTransactionCache,
        };
        pub type InMemoryBackend = sp_state_machine::InMemoryBackend<BlakeTwo256>;

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
            overlay
                .commit_transaction()
                .expect("BUG: mis-paired transaction");
            r
        }
    }
}
