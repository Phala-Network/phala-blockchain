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
use diem_types::transaction::{Transaction, SignedTransaction, TransactionPayload};
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
    VerifyTransaction { account_address: String, transaction_with_proof_b64: String },
    SetTrustedState { trusted_state_b64: String},
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    AccountData { size: u32 },
    VerifyTransaction { total: u32, verified: bool },
    SetTrustedState { status: bool },
    Error(Error)
}

#[derive(Serialize, Deserialize)]
pub struct Diem {
    accounts: Vec<AccountInfo>,
    transactions: BTreeMap<String, Vec<Transaction>>, //address => Transaction
    verified: BTreeMap<String, bool>, //Hash => Bool
    seq_number: BTreeMap<String, u64>, //Address => seq
    #[serde(skip)]
    init_trusted_state: Option<TrustedState>,
    #[serde(skip)]
    new_trusted_state: Option<TrustedState>,
}

impl Diem {
    pub fn new() -> Self {
        Diem {
            accounts: Vec::new(),
            transactions: BTreeMap::<String, Vec<Transaction>>::new(),
            verified: BTreeMap::<String, bool>::new(),
            seq_number: BTreeMap::<String, u64>::new(),
            init_trusted_state: None,
            new_trusted_state: None,
        }
    }

    pub fn get_transaction(
        &mut self,
        transaction_with_proof: TransactionWithProof,
        account_address: String,
        address: AccountAddress,
    ) -> Result<Transaction, Error> {
        let transaction: Transaction = match bcs::from_bytes(&transaction_with_proof.transaction_bytes) {
            Ok(tx) => tx,
            Err(_) => {
                println!("Decode transaction error");

                return Err(Error::Other(String::from("Decode transaction error")));
            }
        };

        if self.verified.get(&transaction.hash().to_hex()).is_some() {
            return Ok(transaction);
        }

        let signed_tx: SignedTransaction = transaction.as_signed_user_txn().unwrap().clone();
        if signed_tx.raw_txn.sender != address {
            // Incoming tx doesn't need to be sequential
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
                println!("Bad receiver address");

                return Err(Error::Other(String::from("Bad receiver address")));
            }
        } else {
            // Outgoing tx must be synced sequencely
            if let Some(seq) = self.seq_number.get(&account_address) {
                if seq + 1 != signed_tx.raw_txn.sequence_number {
                    println!("Bad sequence number");

                    return Err(Error::Other(String::from("Bad sequence number")));
                }
            }
            self.seq_number.insert(account_address.clone(), signed_tx.raw_txn.sequence_number);
        }

        Ok(transaction)
    }

    pub fn verify_trusted_state(
        &mut self,
        transaction_with_proof: TransactionWithProof,
    ) -> Result<(), Error> {
        let epoch_change_proof: EpochChangeProof = transaction_with_proof.epoch_change_proof.clone();
        let ledger_info_with_signatures: LedgerInfoWithSignatures =
            transaction_with_proof.ledger_info_with_signatures.clone();
        let zero_ledger_info_with_sigs = epoch_change_proof.ledger_info_with_sigs[0].clone();

        if let Ok(trusted_state_change) = self.new_trusted_state.as_ref().unwrap()
            .verify_and_ratchet(&ledger_info_with_signatures, &epoch_change_proof) {
            match trusted_state_change {
                TrustedStateChange::Epoch { new_state, latest_epoch_change_li } => {
                    println!(
                        "Verified epoch changed to {}",
                        latest_epoch_change_li
                            .ledger_info()
                            .next_epoch_state()
                            .expect("no validator set in epoch change ledger info"),
                    );
                    self.new_trusted_state = Some(new_state);
                }
                TrustedStateChange::Version { new_state } => {
                    if self.new_trusted_state.as_ref().unwrap().latest_version() < new_state.latest_version() {
                        println!("Verified version change to: {}", new_state.latest_version());
                    }
                    self.new_trusted_state = Some(new_state);
                }
                TrustedStateChange::NoChange => {
                    println!("NoChange");
                }
            }

            Ok(())
        } else {
            println!("Verify trust state error");

            Err(Error::Other("Verify trust state error".to_string()))
        }
    }

    pub fn verify_transaction_state_proof(
        &self,
        transaction_with_proof: TransactionWithProof,
        address: AccountAddress,
    ) -> bool {
        let ledger_info_with_signatures: LedgerInfoWithSignatures = transaction_with_proof.ledger_info_with_signatures.clone();

        let ledger_info_to_transaction_info_proof: TransactionAccumulatorProof =
            transaction_with_proof.ledger_info_to_transaction_info_proof.clone();
        let transaction_info: TransactionInfo = transaction_with_proof.transaction_info.clone();
        let transaction_info_to_account_proof: SparseMerkleProof =
            transaction_with_proof.transaction_info_to_account_proof.clone();

        let transaction_info_with_proof = TransactionInfoWithProof::new(
            ledger_info_to_transaction_info_proof,
            transaction_info
        );

        let account_transaction_state_proof = AccountStateProof::new(
            transaction_info_with_proof,
            transaction_info_to_account_proof,
        );

        if let Ok(_) = account_transaction_state_proof.verify(
            ledger_info_with_signatures.ledger_info(),
            transaction_with_proof.version, address.hash(),
            Some(&transaction_with_proof.account_state_blob))
        {
            println!("Transaction was verified");
            true
        } else {
            println!("Failed to verify transaction");
            false
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
                Request::SetTrustedState { trusted_state_b64 } => {
                    println!("trusted_state_b64: {:?}", trusted_state_b64);
                    if self.init_trusted_state.is_none() {
                        let trusted_state_data = base64::decode(&trusted_state_b64)
                            .map_err(|_| Error::Other("Bad trusted state base64 data".to_string()))?;
                        let zero_ledger_info_with_sigs: LedgerInfoWithSignatures = bcs::from_bytes(&trusted_state_data)
                            .map_err(|_| Error::Other("Bad zero ledger info with sigs".to_string()))?;
                        let trusted_state = TrustedState::try_from(zero_ledger_info_with_sigs.ledger_info())
                            .map_err(|_| Error::Other("Get trusted state error".to_string()))?;
                        self.init_trusted_state = Some(trusted_state.clone());
                        self.new_trusted_state = Some(trusted_state);

                        println!("init trusted state OK");
                    }
                    Ok(Response::SetTrustedState { status: true })
                }
                Request::AccountData { account_data_b64} => {
                    println!("account_data_b64: {:?}", account_data_b64);
                    let account_data = base64::decode(&account_data_b64)
                        .map_err(|_|Error::Other("Bad account base64 data".to_string()))?;
                    let account_info: AccountInfo = bcs::from_bytes(&account_data)
                        .map_err(|_|Error::Other("Can't decode account info".to_string()))?;
                    println!("account_info:{:?}", account_info);
                    let exist = self.accounts.iter().any(|x| x.address == account_info.address);
                    if !exist {
                        self.accounts.push(account_info);
                    }

                    Ok(Response::AccountData { size: self.accounts.len() as u32 })
                }

                Request::VerifyTransaction {account_address, transaction_with_proof_b64} => {
                    println!("transaction_with_proof_b64: {:?}", transaction_with_proof_b64);

                    let address = AccountAddress::from_hex_literal(&account_address)
                        .map_err(|_|Error::Other("Bad account address".to_string()))?;
                    if !self.accounts.iter().any(|x| x.address == address) {
                        println!("Not a contract's account address");

                        return Err(Error::Other(String::from("Not a contract's account address")));
                    }

                    let proof_data = base64::decode(&transaction_with_proof_b64).unwrap_or(Vec::new());
                    let transaction_with_proof: TransactionWithProof = bcs::from_bytes(&proof_data)
                        .map_err(|_| Error::Other("Bad transaction with proof".to_string()))?;
                    println!("transaction_with_proof:{:?}", transaction_with_proof);

                    let transaction = self.get_transaction(transaction_with_proof.clone(), account_address.clone(), address)?;

                    let mut transactions: Vec<Transaction> = self.transactions.get(&account_address).unwrap_or(&Vec::new()).to_vec();
                    let mut total: u32 = transactions.len() as u32;
                    if !transactions.iter().any(|x| x.hash() == transaction.hash()) {
                        transactions.push(transaction.clone());
                        self.transactions.insert(account_address.clone(), transactions);
                        total += 1;
                    }

                    let tx_hash = transaction.hash().to_hex();
                    if self.verified.get(&tx_hash).is_some() && self.verified.get(&tx_hash).unwrap() == &true {
                        println!("transaction has been verified");
                        return Ok(Response::VerifyTransaction { total, verified: true });
                    }

                    if tx_hash != transaction_with_proof.transaction_info.transaction_hash.to_hex() {
                        println!("transaction hash doesn't match");
                        return Ok(Response::VerifyTransaction { total, verified: false });
                    }

                    self.verify_trusted_state(transaction_with_proof.clone())?;

                    let verified = self.verify_transaction_state_proof(transaction_with_proof, address);
                    self.verified.insert(tx_hash, verified);

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
