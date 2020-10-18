use serde::{Serialize, Deserialize};

use crate::contracts;
use crate::types::TxRef;
use crate::TransactionStatus;
use crate::contracts::AccountIdWrapper;

use crate::std::collections::BTreeMap;
use crate::std::string::String;

/// SecretNote contract states.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SecretNote {
    notes: BTreeMap<AccountIdWrapper, String>,
}

/// The commands that the contract accepts from the blockchain. Also called transactions.
/// Commands are supposed to update the states of the contract.
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    /// Set the note for current user
    SetNote {
        note: String,
    },
}

/// The errors that the contract could throw for some queries
#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NotAuthorized,
}

/// Query requests. The end users can only query the contract states by sending requests.
/// Queries are not supposed to write to the contract states.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    /// Read the note for current user
    GetNote,
}

/// Query responses.
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// Return the note for current user
    GetNote {
        note: String,
    },
    /// Something wrong happened
    Error(Error)
}


impl SecretNote {
    /// Initializes the contract
    pub fn new() -> Self {
        Default::default()
    }
}

impl contracts::Contract<Command, Request, Response> for SecretNote {
    // Returns the contract id
    fn id(&self) -> contracts::ContractId { contracts::SECRET_NOTE }

    // Handles the commands from transactions on the blockchain. This method doesn't respond.
    fn handle_command(&mut self, _origin: &chain::AccountId, _txref: &TxRef, cmd: Command) -> TransactionStatus {
        match cmd {
            // Handle the `SetNote` command with one parameter
            Command::SetNote { note } => {
                // Simply increment the counter by some value
                let current_user = AccountIdWrapper(_origin.clone());
                // Insert the note, we only keep the latest note
                self.notes.insert(current_user, note);
                // Returns TransactionStatus::Ok to indicate a successful transaction
                TransactionStatus::Ok
            },
        }
    }

    // Handles a direct query and responds to the query. It shouldn't modify the contract states.
    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response, Error> {
            match req {
                // Handle the `GetNote` request
                Request::GetNote => {
                    // Unwrap the current user account
                    if let Some(account) = _origin {
                        let current_user = AccountIdWrapper(account.clone());
                        if self.notes.contains_key(&current_user) {
                            // Respond with the note in the notes
                            let note = self.notes.get(&current_user);
                            return Ok(Response::GetNote { note: note.unwrap().clone() })
                        }
                    }

                    // Respond NotAuthorized when no account is specified
                    Err(Error::NotAuthorized)
                },
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp
        }
    }
}
