use pallet_evm::{
    runner::stack::Runner as EvmRunner, BalanceOf, Config as EvmConfig, Error as EvmError,
};
use sp_core::{H160, U256};

pub struct NoCreateRunner<T>(core::marker::PhantomData<T>);

impl<T: EvmConfig> pallet_evm::Runner<T> for NoCreateRunner<T>
where
    BalanceOf<T>: TryFrom<U256> + Into<U256>,
{
    type Error = EvmError<T>;

    fn validate(
        source: H160,
        target: Option<H160>,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<sp_core::H256>)>,
        is_transactional: bool,
        weight_limit: Option<frame_election_provider_support::Weight>,
        proof_size_base_cost: Option<u64>,
        evm_config: &evm::Config,
    ) -> Result<(), pallet_evm::RunnerError<Self::Error>> {
        EvmRunner::<T>::validate(
            source,
            target,
            input,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
            is_transactional,
            weight_limit,
            proof_size_base_cost,
            evm_config,
        )
    }

    fn call(
        source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<sp_core::H256>)>,
        is_transactional: bool,
        validate: bool,
        weight_limit: Option<frame_election_provider_support::Weight>,
        proof_size_base_cost: Option<u64>,
        config: &evm::Config,
    ) -> Result<fp_evm::CallInfo, pallet_evm::RunnerError<Self::Error>> {
        EvmRunner::<T>::call(
            source,
            target,
            input,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
            is_transactional,
            validate,
            weight_limit,
            proof_size_base_cost,
            config,
        )
    }

    fn create(
        _source: H160,
        _init: Vec<u8>,
        _value: U256,
        _gas_limit: u64,
        _max_fee_per_gas: Option<U256>,
        _max_priority_fee_per_gas: Option<U256>,
        _nonce: Option<U256>,
        _access_list: Vec<(H160, Vec<sp_core::H256>)>,
        _is_transactional: bool,
        _validate: bool,
        _weight_limit: Option<frame_election_provider_support::Weight>,
        _proof_size_base_cost: Option<u64>,
        _config: &evm::Config,
    ) -> Result<fp_evm::CreateInfo, pallet_evm::RunnerError<Self::Error>> {
        Err(pallet_evm::RunnerError {
            error: EvmError::Undefined,
            weight: Default::default(),
        })
    }

    fn create2(
        _source: H160,
        _init: Vec<u8>,
        _salt: sp_core::H256,
        _value: U256,
        _gas_limit: u64,
        _max_fee_per_gas: Option<U256>,
        _max_priority_fee_per_gas: Option<U256>,
        _nonce: Option<U256>,
        _access_list: Vec<(H160, Vec<sp_core::H256>)>,
        _is_transactional: bool,
        _validate: bool,
        _weight_limit: Option<frame_election_provider_support::Weight>,
        _proof_size_base_cost: Option<u64>,
        _config: &evm::Config,
    ) -> Result<fp_evm::CreateInfo, pallet_evm::RunnerError<Self::Error>> {
        Err(pallet_evm::RunnerError {
            error: EvmError::Undefined,
            weight: Default::default(),
        })
    }
}
