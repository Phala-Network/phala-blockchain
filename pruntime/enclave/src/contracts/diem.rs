use crate::std::collections::BTreeMap;
use crate::std::string::String;
use crate::std::vec::Vec;

use core::str;
use parity_scale_codec::{Encode, Decode};
use serde::{Serialize, Deserialize};
use sp_core::ecdsa;
use sp_core::crypto::Pair;

use crate::contracts;
use crate::TransactionStatus;
use crate::types::TxRef;

//diem type
use crate::std::string::ToString;
use core::convert::TryFrom;
use diem_types::epoch_change::EpochChangeProof;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use diem_types::trusted_state::{TrustedState, TrustedStateChange};
use diem_types::proof::{
    AccountStateProof, TransactionInfoWithProof,
    TransactionAccumulatorProof, SparseMerkleProof,
};
use diem_types::account_address::AccountAddress;
use diem_types::account_state_blob::AccountStateBlob;
use diem_types::transaction::TransactionInfo;
use diem_crypto::hash::CryptoHash;
use diem_types::transaction::{Transaction, SignedTransaction, TransactionPayload, Script};
use move_core_types::transaction_argument::TransactionArgument;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Amount {
    pub amount: u64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: AccountAddress,
    pub authentication_key: Option<Vec<u8>>,
    pub sequence_number: u64,
    pub sent_events_key: String,
    pub received_events_key: String,
    pub balances: Vec<Amount>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionWithProof {
    transaction_bytes: Vec<u8>,

    epoch_change_proof: EpochChangeProof,
    ledger_info_with_signatures: LedgerInfoWithSignatures,

    ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,
    transaction_info: TransactionInfo,
    transaction_info_to_account_proof: SparseMerkleProof,
    account_state_blob: AccountStateBlob,

    version: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NotAuthorized,
    Other(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    AccountData { account_data_b64: String },
    VerifyTransaction { account_address: String, transaction_with_proof_b64: String }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    AccountData { size: u32 },
    VerifyTransaction { total: u32, verified: bool },
    Error(Error)
}

#[derive(Serialize, Deserialize)]
pub struct Diem {
    accounts: Vec<AccountInfo>,
    transactions: BTreeMap<String, Vec<Transaction>>, //address => Transaction
    proofs: BTreeMap<String, TransactionWithProof>, //Hash => Proof
}

impl Diem {
    pub fn new() -> Self {
        Diem {
            accounts: Vec::new(),
            transactions: BTreeMap::<String, Vec<Transaction>>::new(),
            proofs: BTreeMap::<String, TransactionWithProof>::new(),
        }
    }
}

impl contracts::Contract<Command, Request, Response> for Diem {
    fn id(&self) -> contracts::ContractId { contracts::DIEM }

    fn handle_command(&mut self, _origin: &chain::AccountId, _txref: &TxRef, _cmd: Command) -> TransactionStatus {
        TransactionStatus::Ok
    }

    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response, Error> {
            match req {
                Request::AccountData { account_data_b64} => {
                    println!("account_data_b64: {:?}", account_data_b64);
                    let account_data = base64::decode(&account_data_b64).unwrap();
                    let account_info: AccountInfo = bcs::from_bytes(&account_data).unwrap();
                    println!("account_info:{:?}", account_info);
                    let exist = self.accounts.iter().any(|x| x.address == account_info.address);
                    if !exist {
                        self.accounts.push(account_info);
                    }

                    Ok(Response::AccountData { size: self.accounts.len() as u32 })
                }

                Request::VerifyTransaction {account_address, transaction_with_proof_b64} => {
                    println!("transaction_with_proof_b64: {:?}", transaction_with_proof_b64);

                    let address = AccountAddress::from_hex_literal(&account_address).unwrap();
                    let found = self.accounts.iter().any(|x| x.address == address);
                    if !found {
                        println!("Not a contract's account address");
                        return Ok(Response::VerifyTransaction { total: 0, verified: false });
                    }

                    let mut transactions: Vec<Transaction> = self.transactions.get(&account_address).unwrap_or(&Vec::new()).to_vec();
                    let mut total: u32 = transactions.len() as u32;

                    let proof_data = base64::decode(&transaction_with_proof_b64).unwrap();
                    let transaction_with_proof: TransactionWithProof = bcs::from_bytes(&proof_data).unwrap();
                    println!("transaction_with_proof:{:?}", transaction_with_proof);

                    let transaction: Result<Transaction, _> = bcs::from_bytes(&transaction_with_proof.transaction_bytes);
                    if let Ok(tx) = transaction.clone() {
                        let exist = transactions.iter().any(|x| x.hash() == tx.hash());
                        if !exist {
                            transactions.push(tx.clone());
                            self.transactions.insert(account_address.clone(), transactions);
                        }
                        total += 1;
                    } else {
                        println!("decode transaction error");
                        return Ok(Response::VerifyTransaction { total, verified: false });
                    }

                    let signed_tx: SignedTransaction = transaction.clone().unwrap().as_signed_user_txn().unwrap().clone();
                    if address != signed_tx.raw_txn.sender {
                        let mut found = false;
                        if let TransactionPayload::Script(script) = signed_tx.raw_txn.payload {
                            for arg in script.args {
                                if let TransactionArgument::Address(recv_address) = arg {
                                    if recv_address == address {
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if !found {
                            println!("bad sender/receiver address");
                            return Ok(Response::VerifyTransaction { total, verified: false });
                        }
                    }

                    let tx_hash = transaction.unwrap().hash().to_hex();
                    if self.proofs.get(&tx_hash).is_some() {
                        println!("transaction has been verified");
                        return Ok(Response::VerifyTransaction { total, verified: true });
                    }

                    if tx_hash != transaction_with_proof.transaction_info.transaction_hash.to_hex() {
                        println!("transaction hash unmatched");
                        return Ok(Response::VerifyTransaction { total, verified: false });
                    }

                    let epoch_change_proof: EpochChangeProof = transaction_with_proof.epoch_change_proof.clone();
                    let ledger_info_with_signatures: LedgerInfoWithSignatures = transaction_with_proof.ledger_info_with_signatures.clone();

                    let zero_ledger_info_with_sigs = epoch_change_proof.ledger_info_with_sigs[0].clone();
                    let mut trusted_state = TrustedState::try_from(zero_ledger_info_with_sigs.ledger_info()).unwrap();

                    match trusted_state.verify_and_ratchet(&ledger_info_with_signatures, &epoch_change_proof) {
                        Ok(trusted_state_change) => {
                            match trusted_state_change {
                                TrustedStateChange::Epoch {
                                    new_state,
                                    latest_epoch_change_li,
                                } => {
                                    println!(
                                        "Verified epoch changed to {}",
                                        latest_epoch_change_li
                                            .ledger_info()
                                            .next_epoch_state()
                                            .expect("no validator set in epoch change ledger info"),
                                    );
                                    trusted_state = new_state;
                                }
                                TrustedStateChange::Version { new_state } => {
                                    if trusted_state.latest_version() < new_state.latest_version() {
                                        println!("Verified version change to: {}", new_state.latest_version());
                                    }
                                    trusted_state = new_state;
                                }
                                TrustedStateChange::NoChange => {
                                    println!("NoChange");
                                }
                            }
                        },
                        Err(_) => {
                            println!("verify trust state error");
                            return Ok(Response::VerifyTransaction { total, verified: false });
                        }
                    }

                    let ledger_info_to_transaction_info_proof: TransactionAccumulatorProof =
                        transaction_with_proof.ledger_info_to_transaction_info_proof.clone();
                    let transaction_info: TransactionInfo = transaction_with_proof.transaction_info.clone();
                    let transaction_info_to_account_proof: SparseMerkleProof =
                        transaction_with_proof.transaction_info_to_account_proof.clone();

                    let transaction_info_with_proof = TransactionInfoWithProof::new(
                        ledger_info_to_transaction_info_proof.clone(),
                        transaction_info.clone()
                    );

                    let account_transaction_state_proof = AccountStateProof::new(
                        transaction_info_with_proof.clone(),
                        transaction_info_to_account_proof.clone(),
                    );

                    let mut verified = false;
                    match account_transaction_state_proof.verify(ledger_info_with_signatures.ledger_info(),
                        transaction_with_proof.version, address.hash(), Some(&transaction_with_proof.account_state_blob)) {
                        Ok(_) => {
                            println!("Transaction was verified");

                            self.proofs.insert(tx_hash, transaction_with_proof);

                            verified = true;
                        },
                        Err(_) => {
                            println!("Transaction verification was failed");
                        }
                    }

                    Ok(Response::VerifyTransaction { total, verified })
                },
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp
        }
    }

    fn handle_event(&mut self, _ce: chain::Event) {

    }
}

impl core::fmt::Debug for Diem {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, r#"Dime {{
            accounts: {:?},
            }}"#, self.transactions)
    }
}
