use std::collections::{BTreeMap};
use serde::{Serialize, Deserialize};
use crate::std::string::String;
use crate::std::vec::Vec;
use core::{fmt,str};
use core::cmp::Ord;

use crate::contracts;
use crate::types::TxRef;
use crate::contracts::{AccountIdWrapper};
use super::TransactionStatus;

extern crate runtime as chain;

pub type AssetId = u32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetMetadata {
    owner: AccountIdWrapper,
    #[serde(with = "super::serde_balance")]
    total_supply: u128,
    symbol: String,
    id: u32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Assets {
    assets: BTreeMap<u32, BTreeMap<AccountIdWrapper, chain::Balance>>,
    metadata: Vec<AssetMetadata>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NotAuthorized,
    Other(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Issue {
        symbol: String,
        #[serde(with = "super::serde_balance")]
        total: chain::Balance
    },
    Destroy {
        id: AssetId,
    },
    Transfer {
        id: AssetId,
        dest: AccountIdWrapper,
        #[serde(with = "super::serde_balance")]
        value: chain::Balance,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    Balance {
        id: AssetId,
        account: AccountIdWrapper
    },
    TotalSupply {
        id: AssetId
    },
    Metadata
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Balance {
        #[serde(with = "super::serde_balance")]
        balance: chain::Balance
    },
    TotalSupply {
        #[serde(with = "super::serde_balance")]
        total_issuance: chain::Balance
    },
    Metadata {
        metadata: Vec<AssetMetadata>
    },
    Error(Error)
}

const ALICE: &'static str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const SUPPLY: u128 = 1_024_000_000_000_000;
const SYMBOL: &'static str = "TTT";

impl Assets {
    pub fn new() -> Self{
        let mut assets = BTreeMap::<u32, BTreeMap::<AccountIdWrapper, chain::Balance>>::new();
        let mut metadata = Vec::<AssetMetadata>::new();

        let owner = AccountIdWrapper::from_hex(ALICE);
        let symbol = String::from(SYMBOL);
        let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
        accounts.insert(owner.clone(), SUPPLY);

        let metadatum = AssetMetadata {
            owner: owner.clone(),
            total_supply: SUPPLY,
            symbol,
            id: 0
        };

        metadata.push(metadatum);
        assets.insert(0, accounts);

        Assets { assets, metadata }
    }
}

impl contracts::Contract<Command, Request, Response> for Assets {
    fn id(&self) -> contracts::ContractId { contracts::ASSETS }

    fn handle_command(&mut self, origin: &chain::AccountId, _txref: &TxRef, cmd: Command) -> TransactionStatus {
        match cmd {
            Command::Issue {symbol, total} => {
                let o = AccountIdWrapper(origin.clone());
                println!("Issue: [{}] -> [{}]: {}", o.to_string(), symbol, total);

                if let None = self.metadata.iter().find(|metadatum| metadatum.symbol == symbol) {
                    let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
                    accounts.insert(o.clone(), total);

                    let id = match self.metadata.last() {
                        Some(m) => m.id + 1,
                        None => 0
                    };

                    let metadatum = AssetMetadata {
                        owner: o.clone(),
                        total_supply: total,
                        symbol,
                        id
                    };

                    self.metadata.push(metadatum);
                    self.assets.insert(id.clone(), accounts);

					TransactionStatus::Ok
                } else {
					TransactionStatus::SymbolExist
				}
            },
            Command::Destroy {id} => {
                let o = AccountIdWrapper(origin.clone());

                if let Some(position) = self.metadata.iter().position(|metadatum| metadatum.id == id) {
                    let metadatum = self.metadata.get(position).unwrap();
                    if metadatum.owner.to_string() == o.to_string() {
                        self.metadata.remove(position.clone());
                        self.assets.remove(&id);

						TransactionStatus::Ok
                    } else {
						TransactionStatus::NotAssetOwner
					}
                } else {
					TransactionStatus::AssetIdNotFound
				}
            },
            Command::Transfer {id, dest, value} => {
                let o = AccountIdWrapper(origin.clone());

                if let Some(position) = self.metadata.iter().position(|metadatum| metadatum.id == id) {
                    let metadatum = self.metadata.get(position).unwrap();
                    let accounts = self.assets.get_mut(&metadatum.id).unwrap();

                    println!("Transfer: [{}] -> [{}]: {}", o.to_string(), dest.to_string(), value);
                    if let Some(src_amount) = accounts.get_mut(&o) {
                        if *src_amount >= value {
                            let src0 = *src_amount;
                            let mut dest0 = 0;

                            *src_amount -= value;
                            if let Some(dest_amount) = accounts.get_mut(&dest) {
                                dest0 = *dest_amount;
                                *dest_amount += value;
                            } else {
                                accounts.insert(dest, value);
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
                } else {
					TransactionStatus::AssetIdNotFound
				}
            }
        }
    }

    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response, Error> {
            match req {
                Request::Balance { id, account } => {
                    if origin == None || origin.unwrap() != &account.0 {
                        return Err(Error::NotAuthorized)
                    }

                    if let Some(position) = self.metadata.iter().position(|metadatum| metadatum.id == id) {
                        let metadatum = self.metadata.get(position).unwrap();
                        let accounts = self.assets.get(&metadatum.id).unwrap();

                        let mut balance: chain::Balance = 0;
                        if let Some(ba) = accounts.get(&account) {
                            balance = *ba;
                        }
                        Ok(Response::Balance { balance })
                    } else {
                        Err(Error::Other(String::from("Asset not found")))
                    }
                },
                Request::TotalSupply { id } => {
                    if let Some(position) = self.metadata.iter().position(|metadatum| metadatum.id == id) {
                        let metadatum = self.metadata.get(position).unwrap();
                        Ok(Response::TotalSupply { total_issuance: metadatum.total_supply })
                    } else {
                        Err(Error::Other(String::from("Asset not found")))
                    }
                },
                Request::Metadata => {
                    Ok(Response::Metadata { metadata: self.metadata.clone() })
                }
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp
        }
    }
}
