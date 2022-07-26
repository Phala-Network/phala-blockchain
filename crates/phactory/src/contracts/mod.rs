use crate::secret_channel::{KeyPair, SecretMessageChannel};
use std::convert::TryFrom as _;
use std::fmt::Debug;

use crate::system::{TransactionError, TransactionResult};
use anyhow::{Context, Error, Result};
use chain::AccountId;
use parity_scale_codec::{Decode, Encode};
use phala_mq::{MessageOrigin, SignedMessageChannel};

pub mod assets;
pub mod balances;
pub mod btc_lottery;
pub mod geolocation;
pub mod pink;
// pub mod substrate_kitties;

// Disabled due to requiring &mut self in query
// pub mod web3analytics;
// pub mod data_plaza;

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

pub fn contract_address_to_id(address: &AccountId) -> ContractId {
    let inner: &[u8; 32] = address.as_ref();
    inner.into()
}

pub use support::*;
mod support;
