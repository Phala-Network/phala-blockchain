use std::collections::{BTreeMap};
use serde::{Serialize, Deserialize};
use crate::std::string::String;
use core::{fmt,str};
use core::cmp::Ord;
use parity_scale_codec::{Encode, Decode};
use crate::std::vec::Vec;
use crate::contracts;
use crate::types::TxRef;
use crate::contracts::AccountIdWrapper;
use super::TransactionStatus;

extern crate runtime as chain;

const ALICE: &'static str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

type SequenceType = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct Balance {
    accounts: BTreeMap<AccountIdWrapper, chain::Balance>,
    sequence: SequenceType,
    queue: Vec<TransferData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NotAuthorized,
    Other(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Transfer {
        dest: AccountIdWrapper,
        #[serde(with = "super::serde_balance")]
        value: chain::Balance,
    },
    TransferToChain {
        dest: AccountIdWrapper,
        #[serde(with = "super::serde_balance")]
        value: chain::Balance,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    FreeBalance {
        account: AccountIdWrapper
    },
    TotalIssuance,
    PendingChainTransfer { sequence: SequenceType },
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub struct TransferData {
    dest: AccountIdWrapper,
    amount: chain::Balance,
    signature: Option<Vec<u8>>,
    sequence: SequenceType,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    FreeBalance {
        #[serde(with = "super::serde_balance")]
        balance: chain::Balance
    },
    TotalIssuance {
        #[serde(with = "super::serde_balance")]
        total_issuance: chain::Balance
    },
    PendingChainTransfer {
        transfer_queue_b64: String,
    },
    Error(Error)
}

const SUPPLY: u128 = 0;

impl Balance {
    pub fn new() -> Self{
        let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
        accounts.insert(AccountIdWrapper::from_hex(ALICE), SUPPLY);
        Balance {
            accounts,
            sequence: 0,
            queue: Vec::new(),
        }
    }
}

impl contracts::Contract<Command, Request, Response> for Balance {
    fn id(&self) -> contracts::ContractId { contracts::BALANCE }

    fn handle_command(&mut self, origin: &chain::AccountId, _txref: &TxRef, cmd: Command) -> TransactionStatus {
        let status = match cmd {
            Command::Transfer {dest, value} => {
                let o = AccountIdWrapper(origin.clone());
                println!("Transfer: [{}] -> [{}]: {}", o.to_string(), dest.to_string(), value);
                if let Some(src_amount) = self.accounts.get_mut(&o) {
                    if *src_amount >= value {
                        let src0 = *src_amount;
                        let mut dest0 = 0;

                        *src_amount -= value;
                        if let Some(dest_amount) = self.accounts.get_mut(&dest) {
                            dest0 = *dest_amount;
                            *dest_amount += value;
                        } else {
                            self.accounts.insert(dest, value);
                        }

                        println!("   src: {:>20} -> {:>20}", src0, src0 - value);
                        println!("  dest: {:>20} -> {:>20}", dest0, dest0 + value);

                        TransactionStatus::Ok
                    } else {
                        TransactionStatus::InsufficientBalance
                    }
                } else {
                    TransactionStatus::NoBalance
                }
            },
            Command::TransferToChain {dest, value} => {
                let o = AccountIdWrapper(origin.clone());
                println!("Transfer to chain: [{}] -> [{}]: {}", o.to_string(), dest.to_string(), value);
                if let Some(src_amount) = self.accounts.get_mut(&o) {
                    if *src_amount >= value {
                        let src0 = *src_amount;
                        *src_amount -= value;
                        println!("   src: {:>20} -> {:>20}", src0, src0 - value);

                        let sequence = self.sequence + 1;
                        let transfer_data = TransferData {
                            dest,
                            amount: value,
                            signature: None,
                            sequence,
                        };
                        self.queue.push(transfer_data);

                        self.sequence = sequence;

                        TransactionStatus::Ok
                    } else {
                        TransactionStatus::InsufficientBalance
                    }
                } else {
                    TransactionStatus::NoBalance
                }
            }
        };

        status
    }

    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response, Error> {
            match req {
                Request::FreeBalance {account} => {
                    if origin == None || origin.unwrap() != &account.0 {
                        return Err(Error::NotAuthorized)
                    }
                    let mut balance: chain::Balance = 0;
                    if let Some(ba) = self.accounts.get(&account) {
                        balance = *ba;
                    }
                    Ok(Response::FreeBalance { balance })
                },
                Request::PendingChainTransfer {sequence} => {
                    println!("PendingChainTransfer");
                    let transfer_queue: Vec<&TransferData> = self.queue.iter().filter(|x| x.sequence > sequence).collect::<_>();

                    Ok(Response::PendingChainTransfer { transfer_queue_b64: base64::encode(&transfer_queue.encode()) } )
                },
                Request::TotalIssuance => {
                    Ok(Response::TotalIssuance { total_issuance: SUPPLY })
                }
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp
        }
    }

    fn handle_event(&mut self, ce: chain::Event) {
        if let chain::Event::pallet_phala(pe) = ce {
            if let phala::RawEvent::TransferToTee(who, amount) = pe {
                println!("TransferToTee from :{:?}, {:}", who, amount);
                let account_id = chain::AccountId::decode(&mut who.as_slice()).expect("Bad account id");
                let dest = AccountIdWrapper(account_id);
                println!("dest:{:?}", dest.clone());
                if let Some(dest_amount) = self.accounts.get_mut(&dest) {
                    *dest_amount += amount;
                    println!("{:?}'s balance:{}", dest, *dest_amount);
                } else {
                    self.accounts.insert(dest, amount);
                }
            } else if let phala::RawEvent::TransferToChain(who, amount, sequence) = pe {
                println!("TransferToChain who: {:?}, amount: {:}", who, amount);
                let account_id = chain::AccountId::decode(&mut who.as_slice()).expect("Bad account id");
                let transfer_data = TransferData { dest: AccountIdWrapper(account_id), amount, signature: None, sequence };
                println!("transfer data:{:?}", transfer_data);
                self.queue.retain(|x| x.sequence > transfer_data.sequence);
                println!("queue len: {:}", self.queue.len());
            }
        }
    }
}
