use alloc::vec::Vec;
use objects::*;
use pink_extension::chain_extension::signing;
use primitive_types::H256;
use scale::{Decode, Encode};

use crate::contracts::objects::{ContractCall, WeightV2};

pub mod objects;

pub type ContractId = [u8; 32];
pub type Balance = u128;

#[derive(Encode, Decode, Debug)]
#[repr(u8)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    FailedToDryRunContract(crate::Error),
    FailedToQueryContract(crate::Error),
    FailedToCreateTransaction(crate::Error),
    FailedToSendTransaction(crate::Error),
    FailedToDecode,
    InvalidAddressLength,
    NoResult,
    FailedToReadResult,
}

#[derive(Encode, Decode, Debug)]
#[repr(u8)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    Error,
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct InkContract<'a> {
    rpc: &'a str,
    pallet_id: u8,
    call_id: u8,
    contract_id: &'a ContractId,
}

impl<'a> InkContract<'a> {
    pub fn new(rpc: &'a str, pallet_id: u8, call_id: u8, contract_id: &'a ContractId) -> Self {
        InkContract {
            rpc,
            pallet_id,
            call_id,
            contract_id,
        }
    }

    pub fn query<A: Encode, R: Decode>(
        &self,
        origin: [u8; 32],
        contract_method: [u8; 4],
        contract_args: Option<&A>,
        value: Balance,
    ) -> Result<R> {
        self.query_at(origin, contract_method, contract_args, value, None)
    }

    pub fn query_at<A: Encode, R: Decode>(
        &self,
        origin: [u8; 32],
        contract_method: [u8; 4],
        contract_args: Option<&A>,
        value: Balance,
        at: Option<H256>,
    ) -> Result<R> {
        /*
        let origin : [u8;32] = signing::get_public_key(signer, signing::SigType::Sr25519)
            .try_into()
            .map_err(|_| Error::InvalidAddressLength)?;

         */

        let call = build_contract_query(
            origin,
            *self.contract_id,
            contract_method,
            contract_args,
            value,
        );

        let encoded_call = Encode::encode(&call);

        let contract_query_result: ContractQueryResult<ContractError, Balance> =
            crate::query_contract(self.rpc, &encoded_call, at)
                .map_err(Error::FailedToQueryContract)?;

        let result = contract_query_result
            .result
            .data
            .map_err(|_| Error::NoResult)?;
        let result = <core::result::Result<R, Error>>::decode(&mut result.as_slice())
            .map_err(|_| Error::FailedToDecode)?;
        let result = result.map_err(|_| Error::FailedToReadResult)?;
        Ok(result)
    }

    pub fn send_transaction<A: Encode>(
        &self,
        contract_method: [u8; 4],
        contract_args: Option<&A>,
        value: Balance,
        gas_limit: WeightV2,
        signer: &[u8; 32],
    ) -> Result<Vec<u8>> {
        let call: ContractCall<ContractId, u32, Balance> = build_contract_call(
            *self.contract_id,
            contract_method,
            contract_args,
            value,
            gas_limit,
        );

        let signed_tx = crate::create_transaction(
            signer,
            "astar",
            self.rpc,
            self.pallet_id,
            self.call_id,
            call,
            crate::ExtraParam::default(),
        )
        .map_err(|e| Error::FailedToCreateTransaction(e))?;

        let result = crate::send_transaction(self.rpc, &signed_tx)
            .map_err(|e| Error::FailedToSendTransaction(e))?;

        return Ok(result);
    }

    pub fn dry_run_and_send_transaction<A: Encode>(
        &self,
        contract_method: [u8; 4],
        contract_args: Option<&A>,
        value: Balance,
        signer: &[u8; 32],
    ) -> Result<Vec<u8>> {
        let origin: [u8; 32] = signing::get_public_key(signer, signing::SigType::Sr25519)
            .try_into()
            .map_err(|_| Error::InvalidAddressLength)?;

        let call = build_contract_query(
            origin,
            *self.contract_id,
            contract_method,
            contract_args,
            value,
        );

        let encoded_call = Encode::encode(&call);

        let contract_query_result: ContractQueryResult<ContractError, Balance> =
            crate::query_contract(self.rpc, &encoded_call, None)
                .map_err(|e| Error::FailedToQueryContract(e))?;

        self.send_transaction(
            contract_method,
            contract_args,
            value,
            contract_query_result.gas_required,
            signer,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ///  Here the contract deployed on Shibuya (Astar testnet) , contract Id :
    ///
    /// #[ink::contract]
    /// mod incrementer {
    ///
    ///     #[ink(storage)]
    ///     pub struct Incrementer {
    ///         value: i32,
    ///     }
    ///
    ///     #[ink(event)]
    ///     pub struct Incremented {
    ///         by: i32,
    ///         new_value: i32,
    ///         who: AccountId,
    ///     }
    ///
    ///     /// Errors occurred in the contract
    ///     #[derive(scale::Encode, scale::Decode)]
    ///     #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    ///     pub enum Error {
    ///         WillfulError
    ///     }
    ///
    ///     impl Incrementer {
    ///
    ///         #[ink(constructor)]
    ///         pub fn new(init_value: i32) -> Self {
    ///             Self { value: init_value }
    ///         }
    ///
    ///         #[ink(message)]
    ///         pub fn inc_by_1(&mut self) -> Result<(), Error> {
    ///             self.inc(1);
    ///             Ok(())
    ///         }
    ///
    ///         #[ink(message)]
    ///         pub fn inc(&mut self, by: i32) {
    ///             self.value += by;
    ///             ink::env::debug_println!("increment by {}, new value {}", by, self.value);
    ///             let signer = self.env().caller();
    ///             self.env().emit_event(Incremented{ by, new_value: self.value, who: signer });
    ///         }
    ///
    ///         #[ink(message)]
    ///         pub fn get(&self) -> i32 {
    ///             self.value
    ///         }
    ///
    ///         #[ink(message)]
    ///         pub fn get_with_result(&self) -> Result<i32, Error> {
    ///             Ok(self.value)
    ///         }
    ///
    ///         #[ink(message)]
    ///         pub fn get_error(&self) -> Result<(), Error> {
    ///             Err(Error::WillfulError)
    ///         }
    ///
    ///     }
    /// }
    ///
    struct EnvVars {
        /// The RPC endpoint of the target blockchain
        rpc: String,
        pallet_id: u8,
        call_id: u8,
        contract_id: ContractId,
    }

    fn env() -> EnvVars {
        // local node
        /*
        EnvVars {
            rpc: "http://127.0.0.1:9944".to_string(),
            pallet_id: 07u8,
            call_id: 06u8,
            contract_id: hex_literal::hex!("14dca26ea5e235f71373a6b44752ee3e63b3bed2b68e8e5cce0ec9b486d59dab"),
        }
         */
        // shibuya
        EnvVars {
            rpc: "https://shibuya.public.blastapi.io".to_string(),
            pallet_id: 70u8,
            call_id: 06u8,
            contract_id: hex_literal::hex!(
                "f5836caf1c1956afca4527b43f31b7ef6c37345df4539a5091088fbf975a70a9"
            ),
        }
    }

    #[test]
    #[ignore = "this is expensive so we don't test it often"]
    fn test_query_with_primitive_result() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        // get the environment variables
        let EnvVars {
            rpc,
            pallet_id,
            call_id,
            contract_id,
        } = env();

        // create the struct to interact with the smart contract
        let contract = InkContract::new(&rpc, pallet_id, call_id, &contract_id);

        // address who performs the query
        let origin =
            hex_literal::hex!("189dac29296d31814dc8c56cf3d36a0543372bba7538fa322a4aebfebc39e056");
        // method to call:
        // "label": "get",
        // "selector": "0x2f865bd9"
        let method_get = hex_literal::hex!("2f865bd9");
        // no argument
        let params: Option<&()> = None;

        // call the method
        let value: i32 = contract
            .query(origin, method_get, params, 0)
            .expect("Error when call the method 'get'");
        // display the result
        println!("Query the method get, result : {}", value);
        assert!(value > 0);
    }

    #[test]
    #[ignore = "this is expensive so we don't test it often"]
    fn test_query_with_object_result() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        // get the environment variables
        let EnvVars {
            rpc,
            pallet_id,
            call_id,
            contract_id,
        } = env();

        // create the struct to interact with the smart contract
        let contract = InkContract::new(&rpc, pallet_id, call_id, &contract_id);

        // address who performs the query
        let origin =
            hex_literal::hex!("189dac29296d31814dc8c56cf3d36a0543372bba7538fa322a4aebfebc39e056");
        // method to call:
        //  "label": "get_with_result",
        //  "selector": "0xf21dd3cb"
        let method_get_with_result = hex_literal::hex!("f21dd3cb");
        // no argument
        let params: Option<&()> = None;

        // result of the query
        type Result = core::result::Result<i32, Error>;
        // call the method
        let value: Result = contract
            .query(origin, method_get_with_result, params, 0)
            .expect("Error when call the method 'get_with_result'");

        // display the result
        println!("Query the method get, result : {:?}", value);
        assert!(value.unwrap() > 0);
    }

    #[test]
    #[ignore = "this is expensive so we don't test it often"]
    fn test_call_without_params() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        // get the environment variables
        let EnvVars {
            rpc,
            pallet_id,
            call_id,
            contract_id,
        } = env();

        // create the struct to interact with the smart contract
        let contract = InkContract::new(&rpc, pallet_id, call_id, &contract_id);

        // Secret key of test account `//Alice`
        let alice_pk: [u8; 32] =
            hex_literal::hex!("e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a");

        // method to call:
        //         "label": "inc_by_1",
        //         "selector": "0xb5d14a10"
        let method_inc_by_1 = hex_literal::hex!("b5d14a10");
        // no argument
        let params: Option<&()> = None;

        // call the method
        contract
            .dry_run_and_send_transaction(method_inc_by_1, params, 0, &alice_pk)
            .expect("Error when call the method 'inc'");
    }

    #[test]
    #[ignore = "this is expensive so we don't test it often"]
    fn test_call_with_params() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        // get the environment variables
        let EnvVars {
            rpc,
            pallet_id,
            call_id,
            contract_id,
        } = env();

        // create the struct to interact with the smart contract
        let contract = InkContract::new(&rpc, pallet_id, call_id, &contract_id);

        // Secret key of test account `//Alice`
        let alice_pk: [u8; 32] =
            hex_literal::hex!("e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a");

        // method to call:
        //         "label": "inc",
        //         "selector": "0x1d32619f"
        let method_inc = hex_literal::hex!("1d32619f");
        // argument
        let params: Option<&i32> = Some(&3);

        // call the method
        contract
            .dry_run_and_send_transaction(method_inc, params, 0, &alice_pk)
            .expect("Error when call the method 'inc'");
    }

    #[test]
    #[ignore = "this is expensive so we don't test it often"]
    fn test_query_with_error() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        // get the environment variables
        let EnvVars {
            rpc,
            pallet_id,
            call_id,
            contract_id,
        } = env();

        // create the struct to interact with the smart contract
        let contract = InkContract::new(&rpc, pallet_id, call_id, &contract_id);

        // address who performs the query
        let origin =
            hex_literal::hex!("189dac29296d31814dc8c56cf3d36a0543372bba7538fa322a4aebfebc39e056");
        // method to call:
        //         "label": "get_error",
        //         "selector": "0x6baa1eed"
        let method_get_error = hex_literal::hex!("6baa1eed");
        // no argument
        let params: Option<&()> = None;
        // result of the query
        type Result = core::result::Result<i32, Error>;
        // call the method
        let result: Result = contract.query(origin, method_get_error, params, 0);
        match result {
            Err(e) => println!("Expected error {:?}", e),
            r => {
                println!("We expect to receive an error but we receive that {:?}", r);
                panic!("we expect to receive an error");
            }
        }
    }

    #[test]
    #[ignore = "this is expensive so we don't test it often"]
    fn test_call_with_error() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        // get the environment variables
        let EnvVars {
            rpc,
            pallet_id,
            call_id,
            contract_id,
        } = env();

        // create the struct to interact with the smart contract
        let contract = InkContract::new(&rpc, pallet_id, call_id, &contract_id);

        // address who performs the query
        let alice_pk: [u8; 32] =
            hex_literal::hex!("e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a");
        // method to call:
        //         "label": "get_error",
        //         "selector": "0x6baa1eed"
        let method_get_error = hex_literal::hex!("6baa1eed");
        // no argument
        let params: Option<&()> = None;

        // call the method
        let result = contract.dry_run_and_send_transaction(method_get_error, params, 0, &alice_pk);
        match result {
            Err(e) => println!("Expected error {:?}", e),
            r => {
                println!("We expect to receive an error but we receive that {:?}", r);
                panic!("we expect to receive an error");
            }
        }
    }
}
