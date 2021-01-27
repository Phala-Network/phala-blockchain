use std::collections::{BTreeMap};
use serde::{Serialize, Deserialize};
use crate::std::string::String;
use crate::std::string::ToString;
use crate::std::vec::Vec;
use core::str;
use crate::hex;
use crate::contracts;
use crate::types::TxRef;
use crate::contracts::{AccountIdWrapper, SequenceType};
use super::TransactionStatus;

use parity_scale_codec::{Encode, Decode};
use sp_core::ecdsa;
use sp_core::crypto::Pair;

extern crate runtime as chain;

pub type AssetId = Vec<u8>;

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

pub type ParaId = u32;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub enum ChainId {
    RelayChain,
    ParaChain(ParaId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub struct XCurrencyId {
    pub chain_id: ChainId,
    pub currency_id: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub enum NetworkId {
    Any,
    Named(Vec<u8>),
    Polkadot,
    Kusama,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub struct TransferXToken {
    x_currency_id: XCurrencyId,
    para_id: ParaId,
    dest_network: NetworkId,
    dest: AccountIdWrapper,
    amount: chain::Balance,
    sequence: SequenceType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub struct TransferXTokenData {
    data: TransferXToken,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub enum TxQueue {
    TransferTokenData(TransferTokenData),
    TransferXTokenData(TransferXTokenData),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Assets {
    next_id: u32,
    assets: BTreeMap<AssetId, BTreeMap<AccountIdWrapper, chain::Balance>>,
    metadata: BTreeMap<AssetId, AssetMetadata>,
    history: BTreeMap<AccountIdWrapper, Vec<AssetsTx>>,
    sequence: SequenceType,
    queue: Vec<TxQueue>,
    #[serde(skip)]
    pair: Option<ecdsa::Pair>,
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
        id: String,
    },
    Transfer {
        id: String,
        dest: AccountIdWrapper,
        #[serde(with = "super::serde_balance")]
        value: chain::Balance,
    },
    TransferTokenToChain {
        id: String,
        dest: AccountIdWrapper,
        #[serde(with = "super::serde_balance")]
        value: chain::Balance,
    },
    TransferXTokenToChain {
        x_currency_id: XCurrencyId,
        para_id: ParaId,
        dest_network: NetworkId,
        dest: AccountIdWrapper,
        #[serde(with = "super::serde_balance")]
        value: chain::Balance,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    Balance {
        id: String,
        account: AccountIdWrapper
    },
    TotalSupply {
        id: String
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
const SYMBOL: &'static str = "TEST";
const DECIMAL: u8 = 12;

impl Assets {
    pub fn new(pair: Option<ecdsa::Pair>) -> Self{
        let mut assets = BTreeMap::<AssetId, BTreeMap::<AccountIdWrapper, chain::Balance>>::new();
        let mut metadata = BTreeMap::<AssetId, AssetMetadata>::new();

        let owner = AccountIdWrapper::from_hex(ALICE);
        let symbol = String::from(SYMBOL);
        let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
        accounts.insert(owner.clone(), SUPPLY);

        let id = 0u32.to_le_bytes().to_vec();
        let metadatum = AssetMetadata {
            owner: owner.clone(),
            total_supply: SUPPLY,
            symbol,
            decimal: DECIMAL,
            id: id.clone(),
        };

        metadata.insert(id.clone(), metadatum);
        assets.insert(id, accounts);

        Assets {
            next_id: 1,
            assets,
            metadata,
            history: Default::default(), sequence: 0,
            queue: Vec::new(),
            pair,
        }
    }
}

impl contracts::Contract<Command, Request, Response> for Assets {
    fn id(&self) -> contracts::ContractId { contracts::ASSETS }

    fn handle_command(&mut self, origin: &chain::AccountId, txref: &TxRef, cmd: Command) -> TransactionStatus {
        match cmd {
            Command::Issue {symbol, decimal, total} => {
                if decimal == 0 {
                    return TransactionStatus::BadDecimal;
                }
                let o = AccountIdWrapper(origin.clone());
                println!("Issue: [{}] -> [{}]: {}", o.to_string(), symbol, total);

                if let None = self.metadata.iter().find(|(_, metadatum)| metadatum.symbol == symbol) {
                    let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
                    accounts.insert(o.clone(), total);

                    let id = self.next_id.to_le_bytes().to_vec();
                    let metadatum = AssetMetadata {
                        owner: o.clone(),
                        total_supply: total,
                        symbol,
                        decimal,
                        id: id.clone(),
                    };

                    self.metadata.insert(id.clone(), metadatum);
                    self.assets.insert(id.clone(), accounts);
                    self.next_id += 1;

                    TransactionStatus::Ok
                } else {
                    TransactionStatus::SymbolExist
                }
            },
            Command::Destroy {id} => {
                let o = AccountIdWrapper(origin.clone());
                let token_id = hex::decode_hex(&id);
                if let Some(metadatum) = self.metadata.get(&token_id) {
                    if metadatum.owner.to_string() == o.to_string() {
                        if metadatum.decimal > 0 {
                            self.metadata.remove(&token_id);
                            self.assets.remove(&token_id);

                            TransactionStatus::Ok
                        } else {
                            TransactionStatus::DestroyNotAllowed
                        }
                    } else {
                        TransactionStatus::NotAssetOwner
                    }
                } else {
                    TransactionStatus::AssetIdNotFound
                }
            },
            Command::Transfer {id, dest, value} => {
                let o = AccountIdWrapper(origin.clone());
                println!("Transfer id:{:?}", id);
                let token_id = hex::decode_hex(&id);
                println!("Transfer token id:{:?}", token_id.clone());
                if let Some(metadatum) = self.metadata.get(&token_id) {
                    let accounts = self.assets.get_mut(&metadatum.id).unwrap();

                    println!("Transfer: [{}] -> [{}]: {:?}, {}", o.to_string(), dest.to_string(), token_id, value);
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
                                asset_id: token_id,
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
            Command::TransferTokenToChain {id, dest, value} => {
                let o = AccountIdWrapper(origin.clone());
                let token_id = hex::decode_hex(&id);
                if let Some(metadatum) = self.metadata.get_mut(&token_id) {
                    let accounts = self.assets.get_mut(&metadatum.id).unwrap();

                    println!("Transfer token to chain: [{}] -> [{}]: {:?}, {}", o.to_string(), dest.to_string(), token_id, value);
                    if let Some(src_amount) = accounts.get_mut(&o) {
                        if *src_amount >= value {
                            if self.pair.is_none() {
                                println!("Secret error");
                                return TransactionStatus::BadSecret;
                            }

                            let src0 = *src_amount;
                            *src_amount -= value;

                            metadatum.total_supply -= value;

                            println!("   src: {:>20} -> {:>20}", src0, src0 - value);

                            let sequence = self.sequence + 1;

                            let data = TransferToken {
                                token_id,
                                dest,
                                amount: value,
                                sequence,
                            };

                            let pair = self.pair.as_ref().unwrap();
                            let sig = pair.sign(&Encode::encode(&data));
                            let transfer_data = TransferTokenData {
                                data,
                                signature: sig.0.to_vec(),
                            };
                            self.queue.push(TxQueue::TransferTokenData(transfer_data));
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
            },
            Command::TransferXTokenToChain {x_currency_id, para_id, dest_network, dest, value} => {
                let o = AccountIdWrapper(origin.clone());
                println!("Transfer xtoken to chain: [{}] -> [{}]: {:?}, {}", o.to_string(), dest.to_string(), x_currency_id, value);
                //TODO:
                let token_id = [ChainId::encode(&x_currency_id.chain_id), x_currency_id.currency_id.clone()].concat();

                if let Some(metadatum) = self.metadata.get_mut(&token_id) {
                    let accounts = self.assets.get_mut(&metadatum.id).unwrap();
                    println!("Transfer xtoken to chain: [{}] -> [{}]: {:?}, {}", o.to_string(), dest.to_string(), token_id, value);
                    if let Some(src_amount) = accounts.get_mut(&o) {
                        if *src_amount >= value {
                            if self.pair.is_none() {
                                println!("Secret error");
                                return TransactionStatus::BadSecret;
                            }

                            let src0 = *src_amount;
                            *src_amount -= value;

                            metadatum.total_supply -= value;

                            println!("   src: {:>20} -> {:>20}", src0, src0 - value);

                            let sequence = self.sequence + 1;

                            let data = TransferXToken {
                                x_currency_id,
                                para_id,
                                dest_network,
                                dest,
                                amount: value,
                                sequence,
                            };

                            let pair = self.pair.as_ref().unwrap();
                            let sig = pair.sign(&Encode::encode(&data));
                            let transfer_data = TransferXTokenData {
                                data,
                                signature: sig.0.to_vec(),
                            };
                            self.queue.push(TxQueue::TransferXTokenData(transfer_data));
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

                    let token_id = hex::decode_hex(&id);
                    if let Some(metadatum) = self.metadata.get(&token_id) {
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
                    let token_id = hex::decode_hex(&id);
                    if let Some(metadatum) = self.metadata.get(&token_id) {
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
                                let metadata = self.metadata.get(id).unwrap().clone();
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
                                let metadata = self.metadata.get(id).unwrap().clone();
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
                    let transfer_queue: Vec<&TxQueue> = self.queue.iter().filter(|x| {
                        match x {
                            TxQueue::TransferTokenData(tx) => {
                                if tx.data.sequence > sequence {
                                    return true;
                                }

                                false
                            },
                            TxQueue::TransferXTokenData(tx) => {
                                if tx.data.sequence > sequence {
                                    return true;
                                }

                                false
                            }
                        }
                    }).collect::<_>();

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
                println!("TransferTokenToTee from: {:?}, token id: {:?}, amount: {:}", who, token_id, amount);
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
                    println!("unknown token id: {:?}", token_id);
                }
            } else {
                let sequence = match pe {
                    phala::RawEvent::TransferTokenToChain(who, token_id, amount, sequence) => {
                        println!("TransferTokenToChain who: {:?}, token id: {:?}, amount: {:}", who, token_id, amount);
                        sequence
                    },
                    phala::RawEvent::TransferXTokenToChain(who, xtoken_id, amount, sequence) => {
                        println!("TransferXTokenToChain who: {:?}, xtoken id: {:?}, amount: {:}", who, xtoken_id, amount);
                        sequence
                    },
                    _ => return
                };

                self.queue.retain(|x| {
                    let sequence_in_queue = match x {
                        TxQueue::TransferTokenData(tx) => tx.data.sequence,
                        TxQueue:: TransferXTokenData(tx) => tx.data.sequence,
                    };
                    sequence_in_queue > sequence
                });
                println!("queue len: {:}", self.queue.len());
            }
        } else if let chain::Event::xcm_transactor(xa) = ce {
            if let xcm_transactor::RawEvent::DepositAsset(xtoken_id, who, amount, _) = xa {
                println!("DepositAsset from: {:?}, xtoken id: {:?}, amount: {:}", who, xtoken_id, amount);
                let dest = AccountIdWrapper(who);
                println!("   dest: {}", dest.to_string());
                if let Some(metadatum) = self.metadata.get_mut(&xtoken_id) {
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
                    //create new token metadata
                    let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
                    accounts.insert(dest, amount);

                    let metadatum = AssetMetadata {
                        owner: Default::default(),
                        total_supply: amount,
                        symbol: "".to_string(),
                        decimal: 0,
                        id: xtoken_id.clone(),
                    };

                    self.metadata.insert(xtoken_id.clone(), metadatum);
                    self.assets.insert(xtoken_id.clone(), accounts);
                }
            }
        }
    }
}

fn is_tracked(_id: &AccountIdWrapper) -> bool {
    false
}

impl core::fmt::Debug for Assets {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, r#"Assets {{
            next_id: {:?},
            history: {:?},
            sequence: {:?},
            queue: {:?},
        }}"#, self.next_id, self.history, self.sequence, self.queue)
    }
}
