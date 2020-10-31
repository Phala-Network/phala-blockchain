use std::collections::{BTreeMap};
use serde::{Serialize, Deserialize};
use crate::std::string::String;
use crate::std::vec::Vec;
use core::str;

use crate::contracts;
use crate::types::TxRef;
use crate::contracts::{AccountIdWrapper, SequenceType};
use super::TransactionStatus;

use parity_scale_codec::{Encode, Decode};
use secp256k1::{SecretKey, Message};
use sp_core::hashing::blake2_256;

extern crate runtime as chain;

pub type AssetId = u32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetMetadata {
    owner: AccountIdWrapper,
    #[serde(with = "super::serde_balance")]
    total_supply: u128,
    symbol: String,
    decimal: u8,
    id: AssetId
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetMetadataBalance {
    metadata: AssetMetadata,
    #[serde(with = "super::serde_balance")]
    balance: chain::Balance,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub struct TransferToken {
    token_id: AssetId,
    dest: AccountIdWrapper,
    amount: chain::Balance,
    sequence: SequenceType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub struct TransferTokenData {
    data: TransferToken,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Assets {
    next_id: AssetId,
    assets: BTreeMap<AssetId, BTreeMap<AccountIdWrapper, chain::Balance>>,
    metadata: BTreeMap<AssetId, AssetMetadata>,
    history: BTreeMap<AccountIdWrapper, Vec<AssetsTx>>,
    sequence: SequenceType,
    queue: Vec<TransferTokenData>,
    #[serde(skip)]
    secret: Option<SecretKey>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetsTx {
    txref: TxRef,
    asset_id: AssetId,
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

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Issue {
        symbol: String,
        decimal: u8,
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
    TransferToChain {
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
    PendingChainTransfer {
        sequence: SequenceType
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
    PendingChainTransfer {
        transfer_queue_b64: String,
    },
    Error(Error)
}

const ALICE: &'static str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const SUPPLY: u128 = 1_024_000_000_000_000;
const SYMBOL: &'static str = "TTT";
const DECIMAL: u8 = 12;

impl Assets {
    pub fn new(secret: Option<SecretKey>) -> Self{
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
            decimal: DECIMAL,
            id: 0
        };

        metadata.insert(0, metadatum);
        assets.insert(0, accounts);

        Assets {
            next_id: 1,
            assets,
            metadata,
            history: Default::default(), sequence: 0,
            queue: Vec::new(),
            secret,
        }
    }
}

impl contracts::Contract<Command, Request, Response> for Assets {
    fn id(&self) -> contracts::ContractId { contracts::ASSETS }

    fn handle_command(&mut self, origin: &chain::AccountId, txref: &TxRef, cmd: Command) -> TransactionStatus {
        match cmd {
            Command::Issue {symbol, decimal, total} => {
                let o = AccountIdWrapper(origin.clone());
                println!("Issue: [{}] -> [{}]: {}", o.to_string(), symbol, total);

                if let None = self.metadata.iter().find(|(_, metadatum)| metadatum.symbol == symbol) {
                    let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
                    accounts.insert(o.clone(), total);

                    let id = self.next_id;
                    let metadatum = AssetMetadata {
                        owner: o.clone(),
                        total_supply: total,
                        symbol,
                        decimal,
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

                    println!("Transfer: [{}] -> [{}]: {}, {}", o.to_string(), dest.to_string(), id, value);
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

                            println!("   src: {:>20} -> {:>20}", src0, src0 - value);
                            println!("  dest: {:>20} -> {:>20}", dest0, dest0 + value);

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
                                println!(" pushed history (src)");
                            }
                            if is_tracked(&dest) {
                                let slot = self.history.entry(dest).or_default();
                                slot.push(tx.clone());
                                println!(" pushed history (dest)");
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
            },
            Command::TransferToChain {id, dest, value} => {
                let o = AccountIdWrapper(origin.clone());

                if let Some(metadatum) = self.metadata.get_mut(&id) {
                    let accounts = self.assets.get_mut(&metadatum.id).unwrap();

                    println!("Transfer token to chain: [{}] -> [{}]: {}, {}", o.to_string(), dest.to_string(), id, value);
                    if let Some(src_amount) = accounts.get_mut(&o) {
                        if *src_amount >= value {
                            if self.secret.is_none() {
                                println!("BadSecret error");
                                return TransactionStatus::BadSecret;
                            }

                            let src0 = *src_amount;
                            *src_amount -= value;

                            metadatum.total_supply -= value;

                            println!("   src: {:>20} -> {:>20}", src0, src0 - value);

                            let sequence = self.sequence + 1;

                            let data = TransferToken {
                                token_id: id,
                                dest,
                                amount: value,
                                sequence,
                            };

                            let msg_hash = blake2_256(&Encode::encode(&data));
                            let mut buffer = [0u8; 32];
                            buffer.copy_from_slice(&msg_hash);
                            let message = Message::parse(&buffer);
                            let signature = secp256k1::sign(&message, &self.secret.as_ref().unwrap());
                            println!("signature={:?}", signature);

                            let transfer_data = TransferTokenData {
                                data,
                                signature: signature.0.serialize().to_vec(),
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

                    if let Some(metadatum) = self.metadata.get(&id) {
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
                    if let Some(metadatum) = self.metadata.get(&id) {
                        Ok(Response::TotalSupply { total_issuance: metadatum.total_supply })
                    } else {
                        Err(Error::Other(String::from("Asset not found")))
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
                    let raw_origin = origin.ok_or(Error::NotAuthorized)?;
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
                },
                Request::PendingChainTransfer {sequence} => {
                    println!("PendingChainTransferToken");
                    let transfer_queue: Vec<&TransferTokenData> = self.queue.iter().filter(|x| x.data.sequence > sequence).collect::<_>();

                    Ok(Response::PendingChainTransfer { transfer_queue_b64: base64::encode(&transfer_queue.encode()) } )
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
            if let phala::RawEvent::TransferTokenToTee(who, token_id, amount) = pe {
                println!("TransferTokenToTee from: {:?}, token id: {:}, amount: {:}", who, token_id, amount);
                let dest = AccountIdWrapper(who);
                println!("   dest: {}", dest.to_string());
                if let Some(metadatum) = self.metadata.get_mut(&token_id) {
                    let accounts = self.assets.get_mut(&metadatum.id).unwrap();
                    if let Some(dest_amount) = accounts.get_mut(&dest) {
                        let dest_amount0 = *dest_amount;
                        *dest_amount += amount;
                        println!("   value: {:>20} -> {:>20}", dest_amount0, *dest_amount);
                    } else {
                        accounts.insert(dest, amount);
                        println!("   value: {:>20} -> {:>20}", 0, amount);
                    }
                    metadatum.total_supply += amount;
                } else {
                    println!("unknown token id: {}", token_id);
                }
            } else if let phala::RawEvent::TransferTokenToChain(who, token_id, amount, sequence) = pe {
                println!("TransferTokenToChain who: {:?}, token id: {:}, amount: {:}", who, token_id, amount);
                let transfer_data = TransferTokenData { data: TransferToken { token_id, dest: AccountIdWrapper(who), amount, sequence }, signature: Vec::new() };
                println!("transfer data:{:?}", transfer_data);
                self.queue.retain(|x| x.data.sequence > transfer_data.data.sequence);
                println!("queue len: {:}", self.queue.len());
            }
        }
    }
}

fn is_tracked(_id: &AccountIdWrapper) -> bool {
    false
}
