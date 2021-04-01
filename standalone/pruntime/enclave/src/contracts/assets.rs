use anyhow::Result;
use rust_log::info;
use std::collections::{BTreeMap};
use serde::{Serialize, Deserialize};
use crate::std::string::String;
use crate::std::vec::Vec;
use core::{fmt, str};

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
pub struct AssetMetadataBalance {
    metadata: AssetMetadata,
    #[serde(with = "super::serde_balance")]
    balance: chain::Balance,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Assets {
    next_id: u32,
    assets: BTreeMap<u32, BTreeMap<AccountIdWrapper, chain::Balance>>,
    metadata: BTreeMap<u32, AssetMetadata>,
    history: BTreeMap<AccountIdWrapper, Vec<AssetsTx>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetsTx {
    txref: TxRef,
    asset_id: u32,
    from: AccountIdWrapper,
    to: AccountIdWrapper,
    #[serde(with = "super::serde_balance")]
    amount: chain::Balance,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NotAuthorized,
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
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
    Metadata,
    History {
        account: AccountIdWrapper
    },
    ListAssets {
        available_only :bool
    },
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
    History {
        history: Vec<AssetsTx>
    },
    ListAssets {
        assets: Vec<AssetMetadataBalance>
    },
    Error(#[serde(with = "super::serde_anyhow")] anyhow::Error)
}

const ALICE: &'static str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const SUPPLY: u128 = 1_024_000_000_000_000;
const SYMBOL: &'static str = "TTT";

impl Assets {
    pub fn new() -> Self{
        let mut assets = BTreeMap::<u32, BTreeMap::<AccountIdWrapper, chain::Balance>>::new();
        let mut metadata = BTreeMap::<u32, AssetMetadata>::new();

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

        metadata.insert(0, metadatum);
        assets.insert(0, accounts);

        Assets { next_id: 1, assets, metadata, history: Default::default() }
    }
}

impl contracts::Contract<Command, Request, Response> for Assets {
    fn id(&self) -> contracts::ContractId { contracts::ASSETS }

    fn handle_command(&mut self, origin: &chain::AccountId, txref: &TxRef, cmd: Command) -> TransactionStatus {
        match cmd {
            Command::Issue {symbol, total} => {
                let o = AccountIdWrapper(origin.clone());
                info!("Issue: [{}] -> [{}]: {}", o.to_string(), symbol, total);

                if let None = self.metadata.iter().find(|(_, metadatum)| metadatum.symbol == symbol) {
                    let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
                    accounts.insert(o.clone(), total);

                    let id = self.next_id;
                    let metadatum = AssetMetadata {
                        owner: o.clone(),
                        total_supply: total,
                        symbol,
                        id
                    };

                    self.metadata.insert(id, metadatum);
                    self.assets.insert(id.clone(), accounts);
                    self.next_id += 1;

                    TransactionStatus::Ok
                } else {
                    TransactionStatus::SymbolExist
                }
            },
            Command::Destroy {id} => {
                let o = AccountIdWrapper(origin.clone());

                if let Some(metadatum) = self.metadata.get(&id) {
                    if metadatum.owner.to_string() == o.to_string() {
                        self.metadata.remove(&id);
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

                if let Some(metadatum) = self.metadata.get(&id) {
                    let accounts = self.assets.get_mut(&metadatum.id).unwrap();

                    info!("Transfer: [{}] -> [{}]: {}", o.to_string(), dest.to_string(), value);
                    if let Some(src_amount) = accounts.get_mut(&o) {
                        if *src_amount >= value {
                            let src0 = *src_amount;
                            let mut dest0 = 0;

                            *src_amount -= value;
                            if let Some(dest_amount) = accounts.get_mut(&dest) {
                                dest0 = *dest_amount;
                                *dest_amount += value;
                            } else {
                                accounts.insert(dest.clone(), value);
                            }

                            info!("   src: {:>20} -> {:>20}", src0, src0 - value);
                            info!("  dest: {:>20} -> {:>20}", dest0, dest0 + value);

                            let tx = AssetsTx {
                                txref: txref.clone(),
                                asset_id: id,
                                from: o.clone(),
                                to: dest.clone(),
                                amount: value
                            };
                            if is_tracked(&o) {
                                let slot = self.history.entry(o).or_default();
                                slot.push(tx.clone());
                                info!(" pushed history (src)");
                            }
                            if is_tracked(&dest) {
                                let slot = self.history.entry(dest).or_default();
                                slot.push(tx.clone());
                                info!(" pushed history (dest)");
                            }

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
        let inner = || -> Result<Response> {
            match req {
                Request::Balance { id, account } => {
                    if origin == None || origin.unwrap() != &account.0 {
                        return Err(anyhow::Error::msg(Error::NotAuthorized));
                    }

                    if let Some(metadatum) = self.metadata.get(&id) {
                        let accounts = self.assets.get(&metadatum.id).unwrap();
                        let mut balance: chain::Balance = 0;
                        if let Some(ba) = accounts.get(&account) {
                            balance = *ba;
                        }
                        Ok(Response::Balance { balance })
                    } else {
                        Err(anyhow::Error::msg(Error::Other(String::from("Asset not found"))))
                    }
                },
                Request::TotalSupply { id } => {
                    if let Some(metadatum) = self.metadata.get(&id) {
                        Ok(Response::TotalSupply { total_issuance: metadatum.total_supply })
                    } else {
                        Err(anyhow::Error::msg(Error::Other(String::from("Asset not found"))))
                    }
                },
                Request::Metadata => {
                    Ok(Response::Metadata { metadata: self.metadata.values().cloned().collect() })
                },
                Request::History { account } => {
                    let tx_list = self.history.get(&account).cloned().unwrap_or(Default::default());
                    Ok(Response::History { history: tx_list })
                },
                Request::ListAssets { available_only } => {
                    let raw_origin = origin.ok_or_else(|| anyhow::Error::msg(Error::NotAuthorized))?;
                    let o = AccountIdWrapper(raw_origin.clone());
                    // TODO: simplify the two way logic here?
                    let assets = if available_only {
                        self.assets
                            .iter()
                            .filter_map(|(id, balances)| {
                                balances.get(&o).filter(|b| **b > 0).map(|b| (id, *b))
                            })
                            .map(|(id, balance)| {
                                let metadata = self.metadata.get(&id).unwrap().clone();
                                AssetMetadataBalance {
                                    metadata,
                                    balance
                                }
                            })
                            .collect()
                    } else {
                        self.assets
                            .iter()
                            .map(|(id, balances)| {
                                let metadata = self.metadata.get(&id).unwrap().clone();
                                let balance = *balances.get(&o).unwrap_or(&0);
                                AssetMetadataBalance {
                                    metadata,
                                    balance
                                }
                            })
                            .collect()
                    };
                    Ok(Response::ListAssets {assets})
                }
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp
        }
    }
}

fn is_tracked(_id: &AccountIdWrapper) -> bool {
    false
}
