use std::string::ToString;
use anyhow::Result;
use core::fmt;
use log::info;
use parity_scale_codec::{Decode, Encode};
use phala_mq::MessageOrigin;
use std::collections::BTreeMap;

use super::{NativeContext, TransactionError, TransactionResult};
use crate::contracts;
use crate::contracts::AccountId;
use phala_types::messaging::{AssetCommand, AssetId};

type Command = AssetCommand<chain::AccountId, chain::Balance>;

extern crate runtime as chain;

#[derive(Encode, Decode, Debug, Clone)]
pub struct AssetMetadata {
    owner: AccountId,
    total_supply: u128,
    symbol: String,
    id: u32,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct AssetMetadataBalance {
    metadata: AssetMetadata,
    balance: chain::Balance,
}

#[derive(Debug, Clone)]
pub struct Assets {
    next_id: u32,
    assets: BTreeMap<u32, BTreeMap<AccountId, chain::Balance>>,
    metadata: BTreeMap<u32, AssetMetadata>,
    history: BTreeMap<AccountId, Vec<AssetsTx>>,
}
#[derive(Encode, Decode, Debug, Clone)]
pub struct AssetsTx {
    index: u64,
    asset_id: u32,
    from: AccountId,
    to: AccountId,
    amount: chain::Balance,
}

#[derive(Encode, Decode, Debug)]
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

#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {
    Balance { id: AssetId, account: AccountId },
    TotalSupply { id: AssetId },
    Metadata,
    History { account: AccountId },
    ListAssets { available_only: bool },
}
#[derive(Encode, Decode, Debug)]
pub enum Response {
    Balance { balance: chain::Balance },
    TotalSupply { total_issuance: chain::Balance },
    Metadata { metadata: Vec<AssetMetadata> },
    History { history: Vec<AssetsTx> },
    ListAssets { assets: Vec<AssetMetadataBalance> },
    Error(String),
}

impl Assets {
    pub fn new() -> Self {
        let assets = BTreeMap::<u32, BTreeMap<AccountId, chain::Balance>>::new();
        let metadata = BTreeMap::<u32, AssetMetadata>::new();
        Assets {
            next_id: 0,
            assets,
            metadata,
            history: Default::default(),
        }
    }
}

impl contracts::NativeContract for Assets {
    type Cmd = Command;
    type QReq = Request;
    type QResp = Response;

    fn id(&self) -> contracts::ContractId {
        contracts::id256(contracts::ASSETS)
    }

    fn handle_command(
        &mut self,
        _context: &NativeContext,
        origin: MessageOrigin,
        cmd: Self::Cmd,
    ) -> TransactionResult {
        match cmd {
            Command::Issue { symbol, total } => {
                let o = origin.account()?;
                info!("Issue: [{}] -> [{}]: {}", hex::encode(&o), symbol, total);

                if !self
                    .metadata
                    .iter()
                    .any(|(_, metadatum)| metadatum.symbol == symbol)
                {
                    let mut accounts = BTreeMap::<AccountId, chain::Balance>::new();
                    accounts.insert(o.clone(), total);

                    let id = self.next_id;
                    let metadatum = AssetMetadata {
                        owner: o,
                        total_supply: total,
                        symbol,
                        id,
                    };

                    self.metadata.insert(id, metadatum);
                    self.assets.insert(id, accounts);
                    self.next_id += 1;

                    Ok(())
                } else {
                    Err(TransactionError::SymbolExist)
                }
            }
            Command::Destroy { id } => {
                let o = origin.account()?;

                if let Some(metadatum) = self.metadata.get(&id) {
                    if metadatum.owner == o {
                        self.metadata.remove(&id);
                        self.assets.remove(&id);

                        Ok(())
                    } else {
                        Err(TransactionError::NotAssetOwner)
                    }
                } else {
                    Err(TransactionError::AssetIdNotFound)
                }
            }
            Command::Transfer {
                id,
                dest,
                value,
                index,
            } => {
                let o = origin.account()?;

                if let Some(metadatum) = self.metadata.get(&id) {
                    let accounts = self.assets.get_mut(&metadatum.id).unwrap();

                    info!(
                        "Transfer: [{}] -> [{}]: {}",
                        hex::encode(&o),
                        hex::encode(&dest),
                        value
                    );
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
                                index,
                                asset_id: id,
                                from: o.clone(),
                                to: dest.clone(),
                                amount: value,
                            };
                            if is_tracked(&o) {
                                let slot = self.history.entry(o).or_default();
                                slot.push(tx.clone());
                                info!(" pushed history (src)");
                            }
                            if is_tracked(&dest) {
                                let slot = self.history.entry(dest).or_default();
                                slot.push(tx);
                                info!(" pushed history (dest)");
                            }

                            Ok(())
                        } else {
                            Err(TransactionError::InsufficientBalance)
                        }
                    } else {
                        Err(TransactionError::NoBalance)
                    }
                } else {
                    Err(TransactionError::AssetIdNotFound)
                }
            }
        }
    }

    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Self::QReq) -> Self::QResp {
        let inner = || -> Result<Response> {
            match req {
                Request::Balance { id, account } => {
                    if origin == None || origin.unwrap() != &account {
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
                        Err(anyhow::Error::msg(Error::Other(String::from(
                            "Asset not found",
                        ))))
                    }
                }
                Request::TotalSupply { id } => {
                    if let Some(metadatum) = self.metadata.get(&id) {
                        Ok(Response::TotalSupply {
                            total_issuance: metadatum.total_supply,
                        })
                    } else {
                        Err(anyhow::Error::msg(Error::Other(String::from(
                            "Asset not found",
                        ))))
                    }
                }
                Request::Metadata => Ok(Response::Metadata {
                    metadata: self.metadata.values().cloned().collect(),
                }),
                Request::History { account } => {
                    let tx_list = self.history.get(&account).cloned().unwrap_or_default();
                    Ok(Response::History { history: tx_list })
                }
                Request::ListAssets { available_only } => {
                    let raw_origin =
                        origin.ok_or_else(|| anyhow::Error::msg(Error::NotAuthorized))?;
                    let o = raw_origin.clone();
                    // TODO: simplify the two way logic here?
                    let assets = if available_only {
                        self.assets
                            .iter()
                            .filter_map(|(id, balances)| {
                                balances.get(&o).filter(|b| **b > 0).map(|b| (id, *b))
                            })
                            .map(|(id, balance)| {
                                let metadata = self.metadata.get(id).unwrap().clone();
                                AssetMetadataBalance { metadata, balance }
                            })
                            .collect()
                    } else {
                        self.assets
                            .iter()
                            .map(|(id, balances)| {
                                let metadata = self.metadata.get(id).unwrap().clone();
                                let balance = *balances.get(&o).unwrap_or(&0);
                                AssetMetadataBalance { metadata, balance }
                            })
                            .collect()
                    };
                    Ok(Response::ListAssets { assets })
                }
            }
        };
        match inner() {
            Err(error) => Response::Error(error.to_string()),
            Ok(resp) => resp,
        }
    }
}

fn is_tracked(_id: &AccountId) -> bool {
    false
}
