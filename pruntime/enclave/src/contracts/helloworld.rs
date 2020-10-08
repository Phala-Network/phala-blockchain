use serde::{Serialize, Deserialize};

use crate::contracts;
use crate::types::TxRef;
use crate::TransactionStatus;

/// HelloWorld contract states.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HelloWorld {
    counter: u32,
}

/// The commands that the contract accepts from the blockchain. Also called transactions.
/// Commands are supposed to update the states of the contract.
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    /// Increments the counter in the contract by some number
    Increment {
        value: u32,
    },
}

/// The errors that the contract could throw for some queries
#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NotAuthorized,
    SomeOtherError,
}

/// Query requests. The end users can only query the contract states by sending requests.
/// Queries are not supposed to write to the contract states.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    /// Ask for the value of the counter
    GetCount,
}

/// Query responses.
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// Returns the value of the counter
    GetCount {
        count: u32,
    },
    /// Something wrong happened
    Error(Error)
}


impl HelloWorld {
    /// Initializes the contract
    pub fn new() -> Self {
        Default::default()
    }
}

impl contracts::Contract<Command, Request, Response> for HelloWorld {
    // Returns the contract id
    fn id(&self) -> contracts::ContractId { contracts::HELLO_WORLD }

    // Handles the commands from transactions on the blockchain. This method doesn't respond.
    fn handle_command(&mut self, _origin: &chain::AccountId, _txref: &TxRef, cmd: Command) -> TransactionStatus {
        match cmd {
            // Handle the `Increment` command with one parameter
            Command::Increment { value } => {
                // Simply increment the counter by some value.
                self.counter += value;
                // Returns TransactionStatus::Ok to indicate a successful transaction
                TransactionStatus::Ok
            },
        }
    }

    // Handles a direct query and responds to the query. It shouldn't modify the contract states.
    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response, Error> {
            match req {
                // Hanlde the `GetCount` request.
                Request::GetCount => {
                    // Respond with the counter in the contract states.
                    Ok(Response::GetCount { count: self.counter })
                },
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp
        }
    }
}

