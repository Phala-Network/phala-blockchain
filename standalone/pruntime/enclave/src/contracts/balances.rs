use crate::std::collections::BTreeMap;
use crate::std::string::String;
use crate::std::vec::Vec;

use anyhow::Result;
use core::{fmt, str};
use log::{debug, info};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_core::crypto::Pair;
use sp_core::ecdsa;

use crate::contracts;
use crate::contracts::AccountIdWrapper;
use crate::types::TxRef;
use crate::TransactionStatus;
extern crate runtime as chain;

type SequenceType = u64;

#[derive(Serialize, Deserialize)]
pub struct Balances {
    total_issuance: chain::Balance,
    accounts: BTreeMap<AccountIdWrapper, chain::Balance>,
    sequence: SequenceType,
    queue: Vec<TransferData>,
    #[serde(skip)]
    id: Option<ecdsa::Pair>,
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
    FreeBalance { account: AccountIdWrapper },
    TotalIssuance,
    PendingChainTransfer { sequence: SequenceType },
}
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct Transfer {
    dest: AccountIdWrapper,
    amount: chain::Balance,
    sequence: SequenceType,
}
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct TransferData {
    data: Transfer,
    signature: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    FreeBalance {
        #[serde(with = "super::serde_balance")]
        balance: chain::Balance,
    },
    TotalIssuance {
        #[serde(with = "super::serde_balance")]
        total_issuance: chain::Balance,
    },
    PendingChainTransfer {
        transfer_queue_b64: String,
    },
    Error(#[serde(with = "super::serde_anyhow")] anyhow::Error),
}

impl Balances {
    pub fn new(id: Option<ecdsa::Pair>) -> Self {
        Balances {
            total_issuance: 0,
            accounts: BTreeMap::new(),
            sequence: 0,
            queue: Vec::new(),
            id,
        }
    }
}

impl contracts::Contract<Command, Request, Response> for Balances {
    fn id(&self) -> contracts::ContractId {
        contracts::BALANCES
    }

    fn handle_command(
        &mut self,
        origin: &chain::AccountId,
        _txref: &TxRef,
        cmd: Command,
    ) -> TransactionStatus {
        let status = match cmd {
            Command::Transfer { dest, value } => {
                let o = AccountIdWrapper(origin.clone());
                info!(
                    "Transfer: [{}] -> [{}]: {}",
                    o.to_string(),
                    dest.to_string(),
                    value
                );
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

                        info!("   src: {:>20} -> {:>20}", src0, src0 - value);
                        info!("  dest: {:>20} -> {:>20}", dest0, dest0 + value);

                        TransactionStatus::Ok
                    } else {
                        TransactionStatus::InsufficientBalance
                    }
                } else {
                    TransactionStatus::NoBalance
                }
            }
            Command::TransferToChain { dest, value } => {
                let o = AccountIdWrapper(origin.clone());
                info!(
                    "Transfer to chain: [{}] -> [{}]: {}",
                    o.to_string(),
                    dest.to_string(),
                    value
                );
                if let Some(src_amount) = self.accounts.get_mut(&o) {
                    if *src_amount >= value {
                        if self.id.is_none() {
                            return TransactionStatus::BadSecret;
                        }

                        let src0 = *src_amount;
                        *src_amount -= value;
                        self.total_issuance -= value;
                        info!("   src: {:>20} -> {:>20}", src0, src0 - value);
                        let sequence = self.sequence + 1;

                        let data = Transfer {
                            dest,
                            amount: value,
                            sequence,
                        };

                        let id = self.id.as_ref().unwrap();
                        let sig = id.sign(&Encode::encode(&data));
                        let transfer_data = TransferData {
                            data,
                            signature: sig.0.to_vec(),
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
        let inner = || -> Result<Response> {
            match req {
                Request::FreeBalance { account } => {
                    if origin == None || origin.unwrap() != &account.0 {
                        return Err(anyhow::Error::msg(Error::NotAuthorized));
                    }
                    let mut balance: chain::Balance = 0;
                    if let Some(ba) = self.accounts.get(&account) {
                        balance = *ba;
                    }
                    Ok(Response::FreeBalance { balance })
                }
                Request::PendingChainTransfer { sequence } => {
                    info!("PendingChainTransfer");
                    let transfer_queue: Vec<&TransferData> = self
                        .queue
                        .iter()
                        .filter(|x| x.data.sequence > sequence)
                        .collect::<_>();

                    Ok(Response::PendingChainTransfer {
                        transfer_queue_b64: base64::encode(&transfer_queue.encode()),
                    })
                }
                Request::TotalIssuance => Ok(Response::TotalIssuance {
                    total_issuance: self.total_issuance,
                }),
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp,
        }
    }

    fn handle_event(&mut self, ce: chain::Event) {
        if let chain::Event::Phala(pe) = ce {
            if let phala::RawEvent::TransferToTee(who, amount) = pe {
                info!("TransferToTee from :{:?}, {:}", who, amount);
                let dest = AccountIdWrapper(who);
                info!("   dest: {}", dest.to_string());
                if let Some(dest_amount) = self.accounts.get_mut(&dest) {
                    let dest_amount0 = *dest_amount;
                    *dest_amount += amount;
                    info!("   value: {:>20} -> {:>20}", dest_amount0, *dest_amount);
                } else {
                    self.accounts.insert(dest, amount);
                    info!("   value: {:>20} -> {:>20}", 0, amount);
                }
                self.total_issuance += amount;
            } else if let phala::RawEvent::TransferToChain(who, amount, sequence) = pe {
                info!("TransferToChain who: {:?}, amount: {:}", who, amount);
                let transfer_data = TransferData {
                    data: Transfer {
                        dest: AccountIdWrapper(who),
                        amount,
                        sequence,
                    },
                    signature: Vec::new(),
                };
                debug!("transfer data:{:?}", transfer_data);
                self.queue
                    .retain(|x| x.data.sequence > transfer_data.data.sequence);
                info!("queue len: {:}", self.queue.len());
            }
        }
    }
}

impl core::fmt::Debug for Balances {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            r#"Balances {{
    total_issuance: {:?},
    accounts: {:?},
    sequence: {:?},
    queue: {:?},
}}"#,
            self.total_issuance, self.accounts, self.sequence, self.queue
        )
    }
}
