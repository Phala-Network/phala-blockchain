use anyhow::Result;
use log::info;
use parity_scale_codec::{Decode, Encode};
use phala_mq::MessageOrigin;
use sp_core::hashing;
use std::convert::TryInto;

use super::{TransactionError, TransactionResult};
use crate::contracts;
use crate::contracts::{AccountId, NativeContext};
extern crate runtime as chain;

use phala_types::messaging::GuessNumberCommand;

/// Contract Overview
///
/// The contracts of Phala Network will handle two kinds of requests: Command and Query.
/// (we name Query as `Request` in the following code)
///
/// The Commands are allowed to update the state of contract. They are first sent to the blockchain, and then distributed
/// the according contract. Such design ensures the state consistency across multiple instances of the same contract, since
/// all the instances will reach the same state after replaying all the Commands.
/// Such property limits the use of random generator in our contracts: you can only generate random with on-chain entropy,
/// because off-chain random generation can break the state consistency. We will show an example in the following code.
///
/// The Queries are not allowed to change the state of contract. They are directly sent to contract through the local rpc
/// endpoint. Since they are off-chain requests, they can be sent and then real-time processed.
///
/// For the advanced usage of HTTP request in contract, refer to `btc_price_bot.rs`.

/// The Commands to this contract
///
/// Commands need to be first posted on chain then will be dispatched to the contract, that why we define the `GuessNumberCommand`
/// in phala_types to be used globally.
/// They can change the state of the contract, with no responses.
type Command = GuessNumberCommand;

type RandomNumber = u32;

/// Contract state
#[derive(Encode, Decode, Debug, Clone)]
pub struct GuessNumber {
    owner: AccountId,
    random_number: RandomNumber,
}

/// The Queries to this contract
///
/// End users query the contract state by directly sending Queries to the pRuntime without going on chain.
/// They should not change the contract state.
#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {
    /// Query the current owner of the contract
    QueryOwner,
    /// Make a guess on the number
    Guess { guess_number: RandomNumber },
    /// Peek random number (this should only be used by contract owner or root account)
    PeekRandomNumber,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum GuessResult {
    TooLarge,
    TooSmall,
    Correct,
}

/// The Query results
#[derive(Encode, Decode, Debug, Clone)]
pub enum Response {
    Owner(AccountId),
    GuessResult(GuessResult),
    RandomNumber(RandomNumber),
}

#[derive(Encode, Decode, Debug)]
pub enum Error {
    OriginUnavailable,
    NotAuthorized,
}

impl GuessNumber {
    pub fn new() -> Self {
        GuessNumber {
            owner: Default::default(),
            random_number: Default::default(),
        }
    }

    /// Generate random number using on-chain block height
    ///
    /// As mentioned above, off-chain random generation can break the state consistency across multiple instances of the
    /// same contract
    pub fn gen_random_number(context: &NativeContext) -> RandomNumber {
        let hash = hashing::blake2_256(&context.block.block_number.to_be_bytes());
        u32::from_be_bytes(
            hash[..4]
                .try_into()
                .expect("should never failed with corrent array length; qed."),
        )
    }
}

// Alice is the pre-defined root account in dev mode
const ALICE: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

impl contracts::NativeContract for GuessNumber {
    type Cmd = Command;
    type QReq = Request;
    type QResp = Result<Response, Error>;

    /// Handle the Commands from transactions on the blockchain. This method doesn't respond.
    ///
    /// # Arguments
    ///
    /// * `origin` - The sender of the Command, can be Pallet, pRuntime, Contract, Account or even entities from other chain
    /// * `cmd` - The on-chain Command to process
    /// * `context` - The current block info with the necessary egress channel
    fn handle_command(
        &mut self,
        origin: MessageOrigin,
        cmd: Command,
        context: &mut NativeContext,
    ) -> TransactionResult {
        info!("Command received: {:?}", &cmd);

        // we want to limit the sender who can use the Commands to the pre-define root account
        let sender = match &origin {
            MessageOrigin::AccountId(account) => AccountId::from(*account.as_fixed_bytes()),
            _ => return Err(TransactionError::BadOrigin),
        };
        let alice = contracts::account_id_from_hex(ALICE)
            .expect("should not failed with valid address; qed.");
        match cmd {
            Command::NextRandom => {
                if sender != alice && sender != self.owner {
                    return Err(TransactionError::BadOrigin);
                }
                self.random_number = GuessNumber::gen_random_number(context);
                Ok(Default::default())
            }
            Command::SetOwner { owner } => {
                if sender != alice {
                    return Err(TransactionError::BadOrigin);
                }
                self.owner = AccountId::from(*owner.as_fixed_bytes());
                Ok(Default::default())
            }
        }
    }

    /// Handle a direct Query and respond to it. It shouldn't modify the contract state.
    ///
    /// # Arguments
    ///
    /// * `origin` - For off-chain Query, the sender can only be AccountId
    /// * `req` — Off-chain Query to handle
    /// * `context` - The simplified current block info
    fn handle_query(
        &mut self,
        origin: Option<&chain::AccountId>,
        req: Request,
        _context: &mut contracts::QueryContext,
    ) -> Result<Response, Error> {
        info!("Query received: {:?}", &req);
        match req {
            Request::QueryOwner => Ok(Response::Owner(self.owner.clone())),
            Request::Guess { guess_number } => {
                if guess_number > self.random_number {
                    Ok(Response::GuessResult(GuessResult::TooLarge))
                } else if guess_number < self.random_number {
                    Ok(Response::GuessResult(GuessResult::TooSmall))
                } else {
                    Ok(Response::GuessResult(GuessResult::Correct))
                }
            }
            Request::PeekRandomNumber => {
                // also, we only allow Alice or contract owner to peek the number
                let sender = origin.ok_or(Error::OriginUnavailable)?;
                let alice = contracts::account_id_from_hex(ALICE)
                    .expect("should not failed with valid address; qed.");

                if sender != &alice && sender != &self.owner {
                    return Err(Error::NotAuthorized);
                }

                Ok(Response::RandomNumber(self.random_number))
            }
        }
    }
}
