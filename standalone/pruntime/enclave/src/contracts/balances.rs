use crate::std::collections::BTreeMap;
use crate::std::string::String;

use anyhow::Result;
use core::{fmt, str};
use log::info;
use phala_mq::MessageOrigin;
use serde::{Deserialize, Serialize};

use phala_pallets::pallet_mq::MessageOriginInfo as _;
use phala_pallets::pallet_phala as phala;

use crate::contracts;
use crate::contracts::{AccountIdWrapper, NativeContext};
use crate::TransactionStatus;
extern crate runtime as chain;

use phala_types::messaging::{BalanceCommand, BalanceEvent, BalanceTransfer, PushCommand};

type Command = BalanceCommand<chain::AccountId, chain::Balance>;
type Event = BalanceEvent<chain::AccountId, chain::Balance>;

pub struct Balances {
    total_issuance: chain::Balance,
    accounts: BTreeMap<AccountIdWrapper, chain::Balance>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    FreeBalance { account: AccountIdWrapper },
    TotalIssuance,
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
    Error(#[serde(with = "super::serde_anyhow")] anyhow::Error),
}

impl Balances {
    pub fn new() -> Self {
        Balances {
            total_issuance: 0,
            accounts: BTreeMap::new(),
        }
    }
}

impl contracts::NativeContract for Balances {
    type Cmd = Command;
    type Event = Event;
    type QReq = Request;
    type QResp = Response;

    fn id(&self) -> contracts::ContractId {
        contracts::BALANCES
    }

    fn handle_command(
        &mut self,
        context: &NativeContext,
        origin: MessageOrigin,
        cmd: PushCommand<Command>,
    ) -> TransactionStatus {
        let origin = match origin {
            MessageOrigin::AccountId(acc) => acc,
            _ => return TransactionStatus::BadOrigin,
        };

        let status = match cmd.command {
            Command::Transfer { dest, value } => {
                let o = AccountIdWrapper::from(origin);
                let dest = AccountIdWrapper(dest);
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
                let o = AccountIdWrapper::from(origin);
                let dest = AccountIdWrapper(dest);
                info!(
                    "Transfer to chain: [{}] -> [{}]: {}",
                    o.to_string(),
                    dest.to_string(),
                    value
                );
                if let Some(src_amount) = self.accounts.get_mut(&o) {
                    if *src_amount >= value {
                        let src0 = *src_amount;
                        *src_amount -= value;
                        self.total_issuance -= value;
                        info!("   src: {:>20} -> {:>20}", src0, src0 - value);

                        let data = BalanceTransfer {
                            dest,
                            amount: value,
                        };
                        context.mq().send(&data);
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

    fn handle_event(
        &mut self,
        _context: &NativeContext,
        origin: MessageOrigin,
        event: Self::Event,
    ) {
        if origin != phala::Module::<chain::Runtime>::message_origin() {
            error!("Received event from unexpected origin: {:?}", origin);
            return;
        }
        match event {
            Event::TransferToTee(who, amount) => {
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
            }
        }
    }
}
