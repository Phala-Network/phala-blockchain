mod runtime;

use frame_support::traits::Currency;
use pallet_contracts::Config;
use sp_runtime::{traits::Hash, AccountId32};

pub use frame_support::weights::Weight;
use runtime::{Balances, Contracts, Origin, PinkRuntime};

pub fn compile_wat<T>(wat_bytes: &[u8]) -> wat::Result<(Vec<u8>, <T::Hashing as Hash>::Output)>
where
    T: frame_system::Config,
{
    let wasm_binary = wat::parse_bytes(wat_bytes)?.into_owned();
    let code_hash = T::Hashing::hash(&wasm_binary);
    Ok((wasm_binary, code_hash))
}

pub fn contract_test() {
    use codec::Encode;
    pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
    const GAS_LIMIT: Weight = 10_000_000_000;

    let (wasm, code_hash) =
        compile_wat::<PinkRuntime>(include_bytes!("../fixtures/event_size.wat")).unwrap();

    exec::execute_with(|| {
        let _ = Balances::deposit_creating(&ALICE, 1_000_000);
        Contracts::instantiate_with_code(
            Origin::signed(ALICE),
            30_000,
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

pub fn crypto_hashes_test() {
    pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
    const GAS_LIMIT: Weight = 10_000_000_000;

    let (wasm, code_hash) =
        compile_wat::<PinkRuntime>(include_bytes!("../fixtures/crypto_hashes.wat")).unwrap();

    exec::execute_with(|| {
        let _ = Balances::deposit_creating(&ALICE, 1_000_000);

        // Instantiate the CRYPTO_HASHES contract.
        assert!(Contracts::instantiate_with_code(
            Origin::signed(ALICE),
            100_000,
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
