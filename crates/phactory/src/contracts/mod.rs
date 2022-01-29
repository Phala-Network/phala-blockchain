use crate::secret_channel::{KeyPair, SecretMessageChannel};
use std::convert::TryFrom as _;
use std::fmt::Debug;

use crate::system::{TransactionError, TransactionResult};
use crate::types::{deopaque_query, OpaqueError, OpaqueQuery, OpaqueReply};
use anyhow::{Context, Error, Result};
use chain::AccountId;
use parity_scale_codec::{Decode, Encode};
use phala_mq::{MessageOrigin, SignedMessageChannel};

pub mod assets;
pub mod balances;
pub mod btc_lottery;
pub mod data_plaza;
// pub mod diem;
pub mod geolocation;
pub mod pink;
// pub mod substrate_kitties;
pub mod web3analytics;

pub mod btc_price_bot;
pub mod guess_number;

pub use phala_types::contract::*;

fn account_id_from_hex(s: &str) -> Result<AccountId> {
    let bytes = hex::decode(s)
        .map_err(Error::msg)
        .context("Failed to decode AccountId hex")?;
    AccountId::try_from(&bytes[..])
        .map_err(|err| anyhow::anyhow!("Failed to convert AccountId: {:?}", err))
}

extern crate pink as pink_utils;
pub use pink_utils::contract_address;

pub fn contract_address_to_id(address: &AccountId) -> ContractId {
    let inner: &[u8; 32] = address.as_ref();
    inner.into()
}

// keep syncing with get_contract_id() in pallets/phala/src/fat.rs
pub fn get_contract_id(contract_info: &ContractInfo<chain::Hash, chain::AccountId>) -> ContractId {
    let contract_address = pink_utils::contract_address(
        &contract_info.deployer,
        contract_info.code_index.code_hash().as_ref(),
        contract_info.cluster_id.as_ref(),
        contract_info.salt.as_ref(),
    );
    contract_address_to_id(&contract_address)
}

pub use support::*;
mod support;
