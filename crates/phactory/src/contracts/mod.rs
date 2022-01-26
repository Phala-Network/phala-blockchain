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

pub use support::*;
mod support;
