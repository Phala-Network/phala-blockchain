use frame_support::weights::Weight;
use pallet_contracts::Determinism;
use pallet_contracts::StorageDeposit;
use pink_capi::{types::ExecutionMode, v1::ecall::TransactionArguments};
use sp_runtime::DispatchError;

use crate::{
    runtime::{Contracts, Pink as PalletPink, PinkRuntime},
    types::{AccountId, Balance, Hash},
};
use anyhow::Result;

type EventRecord = frame_system::EventRecord<
    <PinkRuntime as frame_system::Config>::RuntimeEvent,
    <PinkRuntime as frame_system::Config>::Hash,
>;

pub type ContractExecResult = pallet_contracts::ContractExecResult<Balance, EventRecord>;
pub type ContractInstantiateResult =
    pallet_contracts::ContractInstantiateResult<AccountId, Balance, EventRecord>;
pub type ContractResult<T> =
    pallet_contracts::ContractResult<Result<T, DispatchError>, Balance, EventRecord>;

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
    const MIN_MASKED_BYTES: u128 = 256;
    let min_masked_value = deposit_per_byte
        .saturating_mul(MIN_MASKED_BYTES)
        .saturating_sub(1);
    let min_mask_bits = 128 - min_masked_value.leading_zeros();
    mask_low_bits128(deposit, min_mask_bits)
}

fn mask_gas(weight: Weight) -> Weight {
    Weight::from_parts(mask_low_bits64(weight.ref_time(), 28), 0)
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
    assert_eq!(mask_deposit(0, 0), 0);
    assert_eq!(mask_deposit(0, 1), 255);
    assert_eq!(mask_deposit(0, price), 4095);
    assert_eq!(mask_deposit(0x10, price), 4095);
    assert_eq!(mask_deposit(0x10_0000, price), 0x10_0fff);
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

pub fn check_instantiate_result(result: &ContractInstantiateResult) -> Result<AccountId> {
    let ret = result
        .result
        .as_ref()
        .map_err(|err| anyhow::anyhow!("{err:?}"))?;
    if ret.result.did_revert() {
        anyhow::bail!("contract instantiation failed: {:?}", ret.result)
    }
    Ok(ret.account_id.clone())
}

/// Instantiate a contract with given code hash and input data.
pub fn instantiate(
    code_hash: Hash,
    input_data: Vec<u8>,
    salt: Vec<u8>,
    mode: ExecutionMode,
    args: TransactionArguments,
) -> ContractInstantiateResult {
    let TransactionArguments {
        origin,
        transfer,
        gas_limit,
        storage_deposit_limit,
        gas_free,
        deposit: _,
    } = args;
    let gas_limit = Weight::from_parts(gas_limit, 0).set_proof_size(u64::MAX);
    let result = contract_tx(origin.clone(), gas_limit, gas_free, move || {
        Contracts::bare_instantiate(
            origin,
            transfer,
            gas_limit,
            storage_deposit_limit,
            pallet_contracts::Code::Existing(code_hash),
            input_data,
            salt,
            pallet_contracts::DebugInfo::UnsafeDebug,
            pallet_contracts::CollectEvents::Skip,
        )
    });
    log::info!("Contract instantiation result: {:?}", &result.result);
    if mode.should_return_coarse_gas() {
        coarse_grained(result, PalletPink::deposit_per_byte())
    } else {
        result
    }
}

/// Call a contract method
///
/// # Parameters
/// * `input_data`: The SCALE encoded arguments including the 4-bytes selector as prefix.
/// # Return
/// Returns the SCALE encoded method return value.
pub fn bare_call(
    address: AccountId,
    input_data: Vec<u8>,
    mode: ExecutionMode,
    tx_args: TransactionArguments,
) -> ContractExecResult {
    let TransactionArguments {
        origin,
        transfer,
        gas_limit,
        gas_free,
        storage_deposit_limit,
        deposit: _,
    } = tx_args;
    let gas_limit = Weight::from_parts(gas_limit, 0).set_proof_size(u64::MAX);
    let determinism = if mode.deterministic_required() {
        Determinism::Enforced
    } else {
        Determinism::Relaxed
    };
    let result = contract_tx(origin.clone(), gas_limit, gas_free, move || {
        Contracts::bare_call(
            origin,
            address,
            transfer,
            gas_limit,
            storage_deposit_limit,
            input_data,
            pallet_contracts::DebugInfo::UnsafeDebug,
            pallet_contracts::CollectEvents::Skip,
            determinism,
        )
    });
    if mode.should_return_coarse_gas() {
        coarse_grained(result, PalletPink::deposit_per_byte())
    } else {
        result
    }
}

fn contract_tx<T>(
    origin: AccountId,
    gas_limit: Weight,
    gas_free: bool,
    tx_fn: impl FnOnce() -> ContractResult<T>,
) -> ContractResult<T> {
    if !gas_free {
        if let Err(err) = PalletPink::pay_for_gas(&origin, gas_limit) {
            return ContractResult {
                gas_consumed: Weight::zero(),
                gas_required: Weight::zero(),
                storage_deposit: Default::default(),
                debug_message: Default::default(),
                result: Err(err),
                events: None,
            };
        }
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
