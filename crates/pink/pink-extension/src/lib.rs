#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_env::Environment;
use ink_lang as ink;
use alloc::vec::Vec;

pub type EcdhPublicKey = [u8; 32];

#[ink::chain_extension]
pub trait PinkExtension {
    type ErrorCode = ErrorCode;

    #[ink(extension = 1, returns_result = false)]
    fn push_message(message: Vec<u8>, topic: Vec<u8>);

    #[ink(extension = 2, returns_result = false)]
    fn push_osp_message(message: Vec<u8>, topic: Vec<u8>, remote_pubkey: Option<EcdhPublicKey>);
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PinkEnvironment {}

impl Environment for PinkEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;
    type RentFraction = <ink_env::DefaultEnvironment as Environment>::RentFraction;

    type ChainExtension = PinkExtension;
}

#[derive(Debug)]
pub struct ErrorCode(u32);

impl ink_env::chain_extension::FromStatusCode for ErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        if status_code == 0 {
            Ok(())
        } else {
            Err(ErrorCode(status_code))
        }
    }
}
