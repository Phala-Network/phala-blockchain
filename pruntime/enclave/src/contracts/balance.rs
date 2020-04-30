use std::collections::{BTreeMap};
use serde::{de::{self,Visitor}, Serialize, Deserialize, Serializer, Deserializer};
use crate::std::string::String;
use core::{fmt,str};
use core::cmp::Ord;

use crate::contracts;
use crate::types::TxRef;

extern crate runtime as chain;

const ALICE: &'static str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

#[derive(Serialize, Deserialize, Debug)]
pub struct Balance {
    accounts: BTreeMap<AccountIdWrapper, chain::Balance>,
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
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    FreeBalance {
        account: AccountIdWrapper
    },
    TotalIssuance,
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
    Error(Error)
}

const SUPPLY: u128 = 1_024_000_000_000_000;

impl Balance {
    pub fn new() -> Self{
        let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
        accounts.insert(AccountIdWrapper::from_hex(ALICE), SUPPLY);
        Balance { accounts }
    }
}

impl contracts::Contract<Command, Request, Response> for Balance {
    fn id(&self) -> contracts::ContractId { contracts::BALANCE }

    fn handle_command(&mut self, origin: &chain::AccountId, _txref: &TxRef, cmd: Command) {
        match cmd {
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
                    }
                }
            }
        }
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
}

#[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct AccountIdWrapper( chain::AccountId );

impl<'a> AccountIdWrapper {
    fn from(b: &'a [u8]) -> Self {
        let mut a = AccountIdWrapper::default();
        use core::convert::TryFrom;
        a.0 = sp_core::crypto::AccountId32::try_from(b).unwrap();
        a
    }
    fn from_hex(s: &str) -> Self {
        let bytes = crate::hex::decode_hex(s);  // TODO: error handling
        AccountIdWrapper::from(&bytes)
    }
    fn to_string(&self) -> String {
        crate::hex::encode_hex_compact(self.0.as_ref())
    }
}

impl Serialize for AccountIdWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        let data_hex = self.to_string();
        serializer.serialize_str(&data_hex)
    }
}

impl<'de> Deserialize<'de> for AccountIdWrapper{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_str(AcidVisitor)
    }
}

struct AcidVisitor;

impl<'de> Visitor<'de> for AcidVisitor {
    type Value = AccountIdWrapper;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("AccountID is the hex of [u8;32]")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error,
    {
        if v.len() == 64 {
            let bytes = crate::hex::decode_hex(v);  // TODO: error handling
            Ok(AccountIdWrapper::from(&bytes))
        } else {
            Err(E::custom(format!("AccountId hex length wrong: {}", v)))
        }
    }
}
