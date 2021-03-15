use crate::std::collections::BTreeMap;
use crate::std::string::String;
use crate::std::vec::Vec;

use core::str;
use parity_scale_codec::{Encode, Decode};
use serde::{Serialize, Deserialize};
use sp_core::ecdsa;
use sp_core::crypto::Pair;

use crate::contracts;
use crate::contracts::AccountIdWrapper;
use crate::TransactionStatus;
use crate::types::TxRef;
use crate::msg::msg_trait::MsgTrait;
use crate::msg::msg_tunnel::{STATIC_MSG_TUNNEL_MUT,MsgType};
use phala_types::{TransferData,Transfer};
extern crate runtime as chain;

const ALICE: &'static str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

type SequenceType = u64;
type BalanceTransferData = TransferData<AccountIdWrapper,chain::Balance>;

#[derive(Serialize, Deserialize)]
pub struct Balances {
    total_issuance: chain::Balance,
    accounts: BTreeMap<AccountIdWrapper, chain::Balance>,
    #[serde(skip)]
    id: Option<ecdsa::Pair>,
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

impl Balances {
    pub fn new(id: Option<ecdsa::Pair>) -> Self {
        let mut accounts = BTreeMap::<AccountIdWrapper, chain::Balance>::new();
        accounts.insert(AccountIdWrapper::from_hex(ALICE), SUPPLY);
        Balances {
            total_issuance: 0,
            accounts,
            id,
        }
    }
}

impl MsgTrait<BalanceTransferData,u64,u64,Vec<BalanceTransferData>,u64> for Balances {
    
    fn push(&mut self,transfer_data: BalanceTransferData){
        let  mut msg_tunnel_mut = STATIC_MSG_TUNNEL_MUT.lock().unwrap();
        let msg_type = MsgType::BALANCE;
        let encode = transfer_data.encode();
        msg_tunnel_mut.push(msg_type,encode);
    }
    fn received(&mut self,seq:u64){
        let  mut msg_tunnel_mut = STATIC_MSG_TUNNEL_MUT.lock().unwrap();
        let msg_type = MsgType::BALANCE;
        msg_tunnel_mut.received(msg_type,seq);
    }
    fn get_msgs(&mut self,seq:u64)->Vec<BalanceTransferData>{
        let  mut msg_tunnel_mut = STATIC_MSG_TUNNEL_MUT.lock().unwrap();
        let msg_type = MsgType::BALANCE;
        let msg_detail_list = msg_tunnel_mut.get_msg_detail_list(msg_type,seq).unwrap();
        let mut v:Vec<BalanceTransferData> = Vec::new();
        for i in msg_detail_list {
            let body_message = &i.body;
            let c = BalanceTransferData::decode(&mut &body_message[..]);
            v.push(c.unwrap());
        }
        v
    }
    fn get_sequence(&mut self)->u64{
        let  mut msg_tunnel_mut = STATIC_MSG_TUNNEL_MUT.lock().unwrap();
        let msg_type = MsgType::BALANCE;
        msg_tunnel_mut.get_sequence(msg_type)
    }
    
}

impl contracts::Contract<Command, Request, Response> for Balances {
    fn id(&self) -> contracts::ContractId { contracts::BALANCES }

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
                        if self.id.is_none() {
                            return TransactionStatus::BadSecret;
                        }

                        let src0 = *src_amount;
                        *src_amount -= value;
                        self.total_issuance -= value;
                        println!("   src: {:>20} -> {:>20}", src0, src0 - value);

                        let sequence = self.get_sequence();
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
                        self.push(transfer_data);



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
                    let transfer_queue: Vec<BalanceTransferData> = self.get_msgs(sequence);
                    Ok(Response::PendingChainTransfer { transfer_queue_b64: base64::encode(&transfer_queue.encode()) } )
                },
                Request::TotalIssuance => {
                    Ok(Response::TotalIssuance { total_issuance: self.total_issuance })
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
                let dest = AccountIdWrapper(who);
                println!("   dest: {}", dest.to_string());
                if let Some(dest_amount) = self.accounts.get_mut(&dest) {
                    let dest_amount0 = *dest_amount;
                    *dest_amount += amount;
                    println!("   value: {:>20} -> {:>20}", dest_amount0, *dest_amount);
                } else {
                    self.accounts.insert(dest, amount);
                    println!("   value: {:>20} -> {:>20}", 0, amount);
                }
                self.total_issuance += amount;
            } else if let phala::RawEvent::TransferToChain(who, amount, sequence) = pe {
                println!("TransferToChain who: {:?}, amount: {:}", who, amount);
                self.received(sequence);
            }
        }
    }
}

impl core::fmt::Debug for Balances {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, r#"Balances {{
    total_issuance: {:?},
    accounts: {:?},
}}"#, self.total_issuance, self.accounts)
    }
}
