use frame_support::weights::Weight;
use pallet_contracts::Determinism;
use pallet_contracts_primitives::StorageDeposit;
use scale::{Decode, Encode};
use sp_runtime::DispatchError;

use crate::{
    runtime::{
        BoxedEventCallbacks, Contracts, ExecSideEffects, Pink as PalletPink, System, Timestamp,
    },
    storage,
    types::{AccountId, Balance, BlockNumber, Hash},
};

type Storage = storage::in_memory_backend::InMemoryStorage;

type ContractExecResult = pallet_contracts_primitives::ContractExecResult<Balance>;
type ContractInstantiateResult =
    pallet_contracts_primitives::ContractInstantiateResult<AccountId, Balance>;

type ContractResult<T> =
    pallet_contracts_primitives::ContractResult<Result<T, DispatchError>, Balance>;

macro_rules! define_mask_fn {
    ($name: ident, $bits: expr, $typ: ty) => {
        /// Mask given number's lowest bits.
        ///
        /// Given a number 0x1000beef, in binary representation:
        ///     0b_10000_00000000_10111110_11101111
        /// We want to mask it to 0x100fffff.
        /// Rough steps:
        ///     0b_10000_00000000_10111110_11101111
        ///        ^
        ///      1. we find the most left bit position here
        ///     0b_10000_00000000_10111110_11101111
        ///                  ^^^^^^^^^^^^^^^^^^^^^^
        ///               2. than calculate these bits need to be mask
        ///     0b_10000_00001111_11111111_11111111
        ///                  ^^^^^^^^^^^^^^^^^^^^^^
        ///               3. mask it
        fn $name(v: $typ, min_mask_bits: u32) -> $typ {
            // Given v = 0x1000_beef
            // Suppose we have:
            // bits = 64
            // v =0b010000_00000000_10111110_11101111
            let pos = $bits - v.leading_zeros();
            //    0b010000_00000000_10111110_11101111
            //      ^
            //     here, pos=30
            // shift right by 9
            let pos = pos.max(9) - 9;
            // Now pos =  0b_10000_00000000_00000000
            //               ^
            //              now here, pos=21
            // If min_mask_bits = 16
            //                  0b_10000000_00000000
            //                     ^
            //                  min_mask_bits here
            let pos = pos.clamp(min_mask_bits, $bits - 1);
            let cursor: $typ = 1 << pos;
            //            0b_10000_00000000_00000000
            //               ^
            //               cursor here
            let mask = cursor.saturating_sub(1);
            // mask =  0b_00001111_11111111_11111111
            // v | mask =
            //    0b10000_00001111_11111111_11111111
            //  = 0x100fffff
            v | mask
        }
    };
}

define_mask_fn!(mask_low_bits64, 64, u64);
define_mask_fn!(mask_low_bits128, 128, u128);

fn mask_deposit(deposit: u128, deposit_per_byte: u128) -> u128 {
    let min_mask_bits = 128 - (deposit_per_byte * 1024).leading_zeros();
    mask_low_bits128(deposit, min_mask_bits)
}

fn mask_gas(weight: Weight) -> Weight {
    Weight::from_ref_time(mask_low_bits64(weight.ref_time(), 28))
}

#[test]
fn mask_low_bits_works() {
    let min_mask_bits = 24;
    assert_eq!(mask_low_bits64(0, min_mask_bits), 0xffffff);
    assert_eq!(mask_low_bits64(0x10, min_mask_bits), 0xffffff);
    assert_eq!(mask_low_bits64(0x1000_0000, min_mask_bits), 0x10ff_ffff);
    assert_eq!(
        mask_low_bits64(0x10_0000_0000, min_mask_bits),
        0x10_0fff_ffff
    );
    assert_eq!(
        mask_low_bits64(0x10_0000_0000_0000, min_mask_bits),
        0x10_0fff_ffff_ffff
    );
    assert_eq!(
        mask_low_bits64(0xffff_ffff_0000_0000, min_mask_bits),
        0xffff_ffff_ffff_ffff
    );

    let price = 10;
    assert_eq!(mask_deposit(0, price), 0x3fff);
    assert_eq!(mask_deposit(0x10, price), 0x3fff);
    assert_eq!(mask_deposit(0x10_0000, price), 0x10_3fff);
    assert_eq!(mask_deposit(0x10_0000_0000, price), 0x10_0fff_ffff);
    assert_eq!(
        mask_deposit(0x10_0000_0000_0000, price),
        0x10_0fff_ffff_ffff
    );
}

fn coarse_grained<T>(mut result: ContractResult<T>, deposit_per_byte: u128) -> ContractResult<T> {
    result.gas_consumed = mask_gas(result.gas_consumed);
    result.gas_required = mask_gas(result.gas_required);

    match &mut result.storage_deposit {
        StorageDeposit::Charge(v) => {
            *v = mask_deposit(*v, deposit_per_byte);
        }
        StorageDeposit::Refund(v) => {
            *v = mask_deposit(*v, deposit_per_byte);
        }
    }
    result
}

#[derive(Debug, Default, Encode, Decode, Clone)]
struct HookSelectors {
    on_block_end: Option<(u32, Weight)>,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct Contract {
    pub address: AccountId,
    hooks: HookSelectors,
}

pub struct TransactionArguments<'a> {
    pub origin: AccountId,
    pub now: u64,
    pub block_number: BlockNumber,
    pub storage: &'a mut Storage,
    pub transfer: Balance,
    pub gas_limit: Weight,
    pub gas_free: bool,
    pub storage_deposit_limit: Option<Balance>,
    pub callbacks: Option<BoxedEventCallbacks>,
}

impl Contract {
    /// Create a new contract instance from existing address.
    pub fn from_address(address: AccountId) -> Self {
        Contract {
            address,
            hooks: Default::default(),
        }
    }

    /// Create a new contract instance.
    ///
    /// # Parameters
    ///
    /// * `origin`: The owner of the created contract instance.
    /// * `code_hash`: The hash of contract code which has been uploaded.
    /// * `input_data`: The input data to pass to the contract constructor.
    /// * `salt`: Used for the address derivation.
    pub fn new(
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        in_query: bool,
        args: TransactionArguments,
    ) -> Result<(Self, ExecSideEffects), DispatchError> {
        let (result, effects) = Self::instantiate(code_hash, input_data, salt, in_query, args);
        let result = result.result?;
        if result.result.did_revert() {
            return Err(DispatchError::Other("Exec reverted"));
        }
        Ok((Self::from_address(result.account_id), effects))
    }

    pub fn instantiate(
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        in_query: bool,
        args: TransactionArguments,
    ) -> (ContractInstantiateResult, ExecSideEffects) {
        let TransactionArguments {
            origin,
            block_number,
            now,
            storage,
            transfer,
            gas_limit,
            storage_deposit_limit,
            callbacks,
            gas_free,
        } = args;
        let gas_limit = gas_limit.set_proof_size(u64::MAX);
        storage.execute_mut(in_query, callbacks, move || {
            let result = contract_tx(
                origin.clone(),
                block_number,
                now,
                gas_limit,
                gas_free,
                move || {
                    Contracts::bare_instantiate(
                        origin,
                        transfer,
                        gas_limit,
                        storage_deposit_limit,
                        pallet_contracts_primitives::Code::Existing(code_hash),
                        input_data,
                        salt,
                        false,
                    )
                },
            );
            log::info!("Contract instantiation result: {:?}", &result.result);
            if in_query {
                coarse_grained(result, PalletPink::deposit_per_byte())
            } else {
                result
            }
        })
    }

    pub fn new_with_selector(
        code_hash: Hash,
        selector: [u8; 4],
        args: impl Encode,
        salt: Vec<u8>,
        tx_args: TransactionArguments,
    ) -> Result<(Self, ExecSideEffects), DispatchError> {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        Self::new(code_hash, input_data, salt, false, tx_args)
    }

    /// Call a contract method
    ///
    /// # Parameters
    /// * `input_data`: The SCALE encoded arguments including the 4-bytes selector as prefix.
    /// # Return
    /// Returns the SCALE encoded method return value.
    pub fn bare_call(
        &self,
        input_data: Vec<u8>,
        in_query: bool,
        tx_args: TransactionArguments,
    ) -> (ContractExecResult, ExecSideEffects) {
        let TransactionArguments {
            origin,
            now,
            block_number,
            storage,
            transfer,
            gas_limit,
            gas_free,
            callbacks,
            storage_deposit_limit,
        } = tx_args;
        let addr = self.address.clone();
        let gas_limit = gas_limit.set_proof_size(u64::MAX);
        let determinism = if in_query {
            Determinism::AllowIndeterminism
        } else {
            Determinism::Deterministic
        };
        storage.execute_mut(in_query, callbacks, move || {
            let result = contract_tx(
                origin.clone(),
                block_number,
                now,
                gas_limit,
                gas_free,
                move || {
                    Contracts::bare_call(
                        origin,
                        addr,
                        transfer,
                        gas_limit,
                        storage_deposit_limit,
                        input_data,
                        true,
                        determinism,
                    )
                },
            );
            if in_query {
                coarse_grained(result, PalletPink::deposit_per_byte())
            } else {
                result
            }
        })
    }

    /// Call a contract method given it's selector
    pub fn call_with_selector<RV: Decode>(
        &self,
        selector: [u8; 4],
        args: impl Encode,
        in_query: bool,
        tx_args: TransactionArguments,
    ) -> (Result<RV, DispatchError>, ExecSideEffects) {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        let (result, effects) = self.bare_call(input_data, in_query, tx_args);
        let result = result.result.and_then(|ret| {
            Decode::decode(&mut &ret.data[..])
                .map_err(|_| DispatchError::Other("Decode result failed"))
        });

        (result, effects)
    }

    /// Called by on each block end by the runtime
    pub fn on_block_end(
        &self,
        storage: &mut Storage,
        block_number: BlockNumber,
        now: u64,
        callbacks: Option<BoxedEventCallbacks>,
    ) -> Result<ExecSideEffects, DispatchError> {
        if let Some((selector, gas_limit)) = self.hooks.on_block_end {
            let mut input_data = vec![];
            selector.to_be_bytes().encode_to(&mut input_data);

            let (result, effects) = self.bare_call(
                input_data,
                false,
                TransactionArguments {
                    origin: self.address.clone(),
                    now,
                    block_number,
                    storage,
                    transfer: 0,
                    gas_limit,
                    gas_free: false,
                    storage_deposit_limit: None,
                    callbacks,
                },
            );
            let _ = result.result?;
            Ok(effects)
        } else {
            Ok(ExecSideEffects::V1 {
                pink_events: vec![],
                ink_events: vec![],
                instantiated: vec![],
            })
        }
    }

    pub fn set_on_block_end_selector(&mut self, selector: u32, gas_limit: u64) {
        self.hooks.on_block_end = Some((selector, Weight::from_ref_time(gas_limit)));
    }
}

fn contract_tx<T>(
    origin: AccountId,
    block_number: BlockNumber,
    now: u64,
    gas_limit: Weight,
    gas_free: bool,
    tx_fn: impl FnOnce() -> ContractResult<T>,
) -> ContractResult<T> {
    System::set_block_number(block_number);
    Timestamp::set_timestamp(now);
    if !gas_free && PalletPink::pay_for_gas(&origin, gas_limit).is_err() {
        return ContractResult {
            gas_consumed: Weight::zero(),
            gas_required: Weight::zero(),
            storage_deposit: Default::default(),
            debug_message: Default::default(),
            result: Err(DispatchError::Other("InsufficientBalance")),
        };
    }
    let result = tx_fn();
    if !gas_free {
        let refund = gas_limit
            .checked_sub(&result.gas_consumed)
            .expect("BUG: consumed gas more than the gas limit");
        PalletPink::refund_gas(&origin, refund).expect("BUG: failed to refund gas");
    }
    result
}

pub use contract_file::ContractFile;

mod contract_file {
    use impl_serde::serialize as bytes;
    use serde::Deserialize;
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContractFile {
        pub metadata_version: String,
        pub source: Source,
        pub contract: Contract,
    }

    #[derive(Debug, Deserialize)]
    pub struct Source {
        #[serde(with = "bytes")]
        pub wasm: Vec<u8>,
        #[serde(with = "bytes")]
        pub hash: Vec<u8>,
        pub language: String,
        pub compiler: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Contract {
        pub name: String,
        pub version: String,
    }

    impl ContractFile {
        pub fn load(json_contract: &[u8]) -> serde_json::Result<Self> {
            serde_json::from_slice(json_contract)
        }
    }
}
