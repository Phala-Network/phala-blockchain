use crate::transaction::MultiAddress;
use alloc::vec::Vec;
use scale::{Compact, Decode, Encode};

pub(crate) fn build_contract_call<AccountId, AccountIndex, Balance, ARGS: Encode>(
    contract_id: AccountId,
    contract_method: [u8; 4],
    contract_args: Option<&ARGS>,
    value: Balance,
    gas_limit: WeightV2,
) -> ContractCall<AccountId, AccountIndex, Balance> {
    let storage_deposit_limit = None;

    let mut data = Vec::new();
    contract_method.encode_to(&mut data);
    if let Some(args) = contract_args {
        let mut encoded_contract_args = args.encode();
        data.append(&mut encoded_contract_args);
    }

    ContractCall {
        dest: MultiAddress::Id(contract_id),
        value,
        gas_limit,
        storage_deposit_limit,
        data,
    }
}

pub(crate) fn build_contract_query<AccountId, Balance, ARGS: Encode>(
    origin: AccountId,
    contract_id: AccountId,
    contract_method: [u8; 4],
    contract_args: Option<&ARGS>,
    value: Balance,
) -> ContractQuery<AccountId, Balance> {
    let mut data = Vec::new();
    contract_method.encode_to(&mut data);
    if let Some(args) = contract_args {
        let mut encoded_contract_args = args.encode();
        data.append(&mut encoded_contract_args);
    }

    ContractQuery {
        origin,
        dest: contract_id,
        value,
        gas_limit: None,
        storage_deposit_limit: None,
        data,
    }
}

/// Struct used to send an encoded transaction to the contract
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub(crate) struct ContractCall<AccountId, AccountIndex, Balance> {
    /// Contract address
    dest: MultiAddress<AccountId, AccountIndex>,
    /// Only for payable messages, call will fail otherwise
    #[codec(compact)]
    value: Balance,
    /// Maximum gas to be consumed. If it is too small the extrinsic will fail
    gas_limit: WeightV2,
    /// A limit to how much Balance to be used to pay for the storage created by the contract call.
    /// if None is passed, unlimited balance can be used
    storage_deposit_limit: Option<Compact<Balance>>,
    /// data: method name + args
    data: Vec<u8>,
}

/// Gas to be consumed: gaz = ref_time * proof_size
#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct WeightV2 {
    #[codec(compact)]
    pub ref_time: u64,
    #[codec(compact)]
    pub proof_size: u64,
}

/// Struct used to query a wasm contract
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub(crate) struct ContractQuery<AccountId, Balance> {
    origin: AccountId,
    dest: AccountId,
    value: Balance,
    gas_limit: Option<WeightV2>,
    storage_deposit_limit: Option<Balance>,
    data: Vec<u8>,
}

/// Result when we query a wasm contract
#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ContractQueryResult<Error, Balance> {
    pub(crate) gas_consumed: WeightV2,
    pub(crate) gas_required: WeightV2,
    pub(crate) storage_deposit: StorageDeposit<Balance>,
    pub(crate) debug_message: Vec<u8>,
    pub(crate) result: ExecReturnValue<Error>,
}

#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub(crate) struct ExecReturnValue<Error> {
    pub(crate) flags: u32,
    pub(crate) data: Result<Vec<u8>, Error>,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub(crate) enum StorageDeposit<Balance> {
    Refund(Balance),
    Charge(Balance),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_contract_call() {
        let contract_id: [u8; 32] =
            hex_literal::hex!("f77bfd16d61d39dcd8c4413ac88642354f5726bb5915bf52bc4f502a671f1aa5");
        let contract_method = hex_literal::hex!("1d32619f");
        let contract_args = Some(&2i32);

        let call: ContractCall<[u8; 32], u32, u128> = build_contract_call(
            contract_id,
            contract_method,
            contract_args,
            0u128,
            WeightV2 {
                ref_time: 3991666688u64,
                proof_size: 131072u64,
            },
        );

        let encoded_call = Encode::encode(&call);
        let expected =  hex_literal::hex!("00f77bfd16d61d39dcd8c4413ac88642354f5726bb5915bf52bc4f502a671f1aa500030000eced0200080000201d32619f02000000");
        assert_eq!(&expected, encoded_call.as_slice());
    }

    #[test]
    fn test_encode_contract_query() {
        let origin: [u8; 32] =
            hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
        let contract_id: [u8; 32] =
            hex_literal::hex!("f77bfd16d61d39dcd8c4413ac88642354f5726bb5915bf52bc4f502a671f1aa5");
        let contract_method = hex_literal::hex!("2f865bd9");
        let contract_args: Option<&()> = None; // no args

        let call = build_contract_query(origin, contract_id, contract_method, contract_args, 0u128);

        let encoded_call = Encode::encode(&call);
        let expected =  hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27df77bfd16d61d39dcd8c4413ac88642354f5726bb5915bf52bc4f502a671f1aa5000000000000000000000000000000000000102f865bd9");
        assert_eq!(&expected, encoded_call.as_slice());
    }

    /// this struct should match with the error returned by the contract
    #[derive(Decode, Debug)]
    enum Error {
        Error1,
        Error2,
    }

    #[test]
    fn test_decode_contract_query_result() {
        let result =  hex_literal::hex!("d6b2469e3a3d0100030000eced020008000100000000000000000000000000000000000000000000140003000000");

        let contract_query_result =
            <ContractQueryResult<Error, u128>>::decode(&mut result.as_slice()).unwrap();

        assert_eq!(663858357u64, contract_query_result.gas_consumed.ref_time);
        assert_eq!(20302u64, contract_query_result.gas_consumed.proof_size);
        assert_eq!(3991666688u64, contract_query_result.gas_required.ref_time);
        assert_eq!(131072u64, contract_query_result.gas_required.proof_size);
        assert_eq!(
            StorageDeposit::Charge(0),
            contract_query_result.storage_deposit
        );
        assert_eq!(0u32, contract_query_result.result.flags);
        assert!(contract_query_result.result.data.is_ok());
        let result = contract_query_result.result.data.unwrap();
        let data = <Result<i32, Error>>::decode(&mut result.as_slice()).unwrap();

        assert!(data.is_ok());
        assert_eq!(3i32, data.unwrap());
    }
}
