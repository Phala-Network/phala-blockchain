use crate::std::collections::BTreeMap;
use crate::std::string::String;
use crate::std::vec::Vec;

use anyhow::Result;
use core::{fmt, str};
use log::{error, info, warn};
use serde::{Serialize, Deserialize};

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

use parity_scale_codec::{Decode, Encode};
use crate::std::borrow::ToOwned;
use rand::{rngs::OsRng, Rng, SeedableRng};
use diem_crypto::{
    Uniform, hash::HashValue,
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair
};
use diem_types::transaction::authenticator::AuthenticationKey;
use diem_types::account_config::{xus_tag, XUS_NAME};
use diem_types::{
    chain_id::ChainId,
    transaction::{
        helpers::{create_user_txn},
    }
};
use crate::hex;
const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 1_000_000;
const TX_EXPIRATION: i64 = 600;

const ALICE_PRIVATE_KEY: &str = "818ad9a64e3d1bbc388f8bf1e43c78d125237b875a1b70a18f412f7d18efbeea";
const _ALICE_ADDRESS: &str = "D4F0C053205BA934BB2AC0C4E8479E77";

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
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    /// Sets the whitelisted accounts, in bcs encoded base64
    AccountData { account_data_b64: String },
    /// Verifies a transactions
    VerifyTransaction { account_address: String, transaction_with_proof_b64: String },
    /// Sets the trusted state. The owner can only initialize the bridge with the genesis state
    /// once.
    SetTrustedState { trusted_state_b64: String },

    NewAccount { seq_number: u64 },
    TransferXUS { from: String, to: String, amount: u64, seq_number: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    /// Gets all the verified transactions, in hex hash string
    VerifiedTransactions,
    ///Gets signed transaction, from start
    GetSignedTransactions { start: u64},
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// The response with all the the transaction hash verified successfully by the light client
    VerifiedTransactions {
        hash: Vec<String>,
    },
    GetSignedTransactions {
        queue_b64: String,
    },
    /// Some other errors
    Error(#[serde(with = "super::serde_anyhow")] anyhow::Error)
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct TransactionData {
    sequence: u64,
    address: Vec<u8>,
    signed_tx: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct Diem {
    accounts: Vec<AccountInfo>,
    transactions: BTreeMap<String, Vec<Transaction>>, //address => Transaction
    verified: BTreeMap<String, bool>, //Hash => Bool
    seq_number: BTreeMap<String, u64>, //Address => seq
    // TODO: TrustedState is not serializable; instead we should serialize the embedded Waypoint,
    // which can be accessible when constructing the TrustedState from a LedgerInfo.
    #[serde(skip)]
    init_trusted_state: Option<TrustedState>,
    #[serde(skip)]
    trusted_state: Option<TrustedState>,

    accounts_address: Vec<AccountAddress>,
    account_keypairs: BTreeMap<String, KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
    tx_sequence: u64,
    tx_queue: Vec<TransactionData>,
}

impl Diem {
    pub fn new() -> Self {
        let alice_priv_key = Ed25519PrivateKey::from_bytes_unchecked(
            &hex::decode_hex(ALICE_PRIVATE_KEY)).unwrap();
        let alice_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> = KeyPair::from(alice_priv_key);
        let alice_account_address = AuthenticationKey::ed25519(&alice_key_pair.public_key).derived_address();

        let mut accounts_address: Vec<AccountAddress> = Vec::new();
        accounts_address.push(alice_account_address);
        let mut account_keypairs = BTreeMap::<String, KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>::new();
        account_keypairs.insert(alice_account_address.to_string(), alice_key_pair);

        Diem {
            accounts: Vec::new(),
            transactions: BTreeMap::<String, Vec<Transaction>>::new(),
            verified: BTreeMap::<String, bool>::new(),
            seq_number: BTreeMap::<String, u64>::new(),
            init_trusted_state: None,
            trusted_state: None,

            accounts_address,
            account_keypairs,
            tx_sequence: 1,
            tx_queue: Vec::new(),
        }
    }

    pub fn get_transaction(
        &mut self,
        transaction_with_proof: TransactionWithProof,
        account_address: String,
        address: AccountAddress,
    ) -> Result<Transaction> {
        let transaction: Transaction = match bcs::from_bytes(&transaction_with_proof.transaction_bytes) {
            Ok(tx) => tx,
            Err(_) => {
                error!("Decode transaction error");

                return Err(anyhow::Error::msg(Error::Other(String::from("Decode transaction error"))));
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
                error!("Bad receiver address");

                return Err(anyhow::Error::msg(Error::Other(String::from("Bad receiver address"))));
            }
        } else {
            // Outgoing tx must be synced sequencely
            if let Some(seq) = self.seq_number.get(&account_address) {
                if seq + 1 != signed_tx.raw_txn.sequence_number {
                    error!("Bad sequence number");

                    return Err(anyhow::Error::msg(Error::Other(String::from("Bad sequence number"))));
                }
            }
            self.seq_number.insert(account_address.clone(), signed_tx.raw_txn.sequence_number);
        }

        Ok(transaction)
    }

    pub fn verify_trusted_state(
        &mut self,
        transaction_with_proof: TransactionWithProof,
    ) -> Result<()> {
        let epoch_change_proof = &transaction_with_proof.epoch_change_proof;
        let ledger_info_with_signatures = &transaction_with_proof.ledger_info_with_signatures;
        // Verify the new state
        let trusted_state = self.trusted_state.as_ref()
            .ok_or_else(|| anyhow::Error::msg(Error::Other("TrustedState uninitialized".to_string())))?;
        let trusted_state_change = trusted_state.verify_and_ratchet(
            ledger_info_with_signatures, &epoch_change_proof
        ).or_else(|_| Err(anyhow::Error::msg(Error::Other("Verify trust state error".to_string()))))?;
        // Update trusted_state on demand
        match trusted_state_change {
            TrustedStateChange::Epoch { new_state, latest_epoch_change_li } => {
                info!(
                    "verify_trusted_state: Verified epoch changed to {}",
                    latest_epoch_change_li
                        .ledger_info()
                        .next_epoch_state()
                        .expect("no validator set in epoch change ledger info"),
                );
                self.trusted_state = Some(new_state);
            }
            TrustedStateChange::Version { new_state } => {
                if trusted_state.latest_version() < new_state.latest_version() {
                    info!(
                        "verify_trusted_state: Verified version change to: {}",
                        new_state.latest_version()
                    );
                }
                self.trusted_state = Some(new_state);
            }
            TrustedStateChange::NoChange => {
                info!("verify_trusted_state: NoChange");
            }
        }
        Ok(())
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
            true
        } else {
            warn!("Failed to verify transaction");
            false
        }
    }

    pub fn auth_key_prefix(&self, auth_key: Vec<u8>) -> Vec<u8> {
        auth_key[0..16].to_vec()
    }

    pub fn get_account_address(&self, address: String) -> Option<AccountAddress> {
        if let Ok(sender) = AccountAddress::from_hex_literal(&address) {
            if self.accounts_address.contains(&sender) {
                Some(sender)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl contracts::Contract<Command, Request, Response> for Diem {
    fn id(&self) -> contracts::ContractId { contracts::DIEM }

    fn handle_command(&mut self, _origin: &chain::AccountId, _txref: &TxRef, cmd: Command) -> TransactionStatus {
        match cmd {
            Command::AccountData { account_data_b64 } => {
                info!("command account_data_b64:{:?}", account_data_b64);
                if let Ok(account_data) = base64::decode(&account_data_b64) {
                    let account_info: AccountInfo = match bcs::from_bytes(&account_data) {
                        Ok(result) => result,
                        Err(_) => return TransactionStatus::BadAccountInfo,
                    };
                    info!("account_info:{:?}", account_info);
                    let exist = self.accounts.iter().any(|x| x.address == account_info.address);
                    if !exist {
                        self.accounts.push(account_info);
                    }
                    info!("add account_ok");
                    TransactionStatus::Ok
                } else {
                    TransactionStatus::BadAccountData
                }
            }
            Command::SetTrustedState { trusted_state_b64 } => {
                info!("trusted_state_b64: {:?}", trusted_state_b64);
                // Only initialize TrustedState once
                if self.init_trusted_state.is_some() {
                    return TransactionStatus::Ok
                }
                match parse_trusted_state(&trusted_state_b64) {
                    Ok(trusted_state) => {
                        self.init_trusted_state = Some(trusted_state.clone());
                        self.trusted_state = Some(trusted_state);
                        info!("init trusted state OK");
                        TransactionStatus::Ok
                    }
                    Err(code) => code,
                }
            }
            Command::VerifyTransaction { account_address, transaction_with_proof_b64 } => {
                info!("transaction_with_proof_b64: {:?}", transaction_with_proof_b64);

                if let Ok(address) = AccountAddress::from_hex_literal(&account_address) {
                    if !self.accounts.iter().any(|x| x.address == address) {
                        error!("not a contract's account address");

                        return TransactionStatus::InvalidAccount;
                    }

                    let proof_data = base64::decode(&transaction_with_proof_b64).unwrap_or(Vec::new());
                    let transaction_with_proof: TransactionWithProof = match bcs::from_bytes(&proof_data) {
                        Ok(result) => result,
                        Err(_) => return TransactionStatus::BadTransactionWithProof,
                    };
                    info!("transaction_with_proof:{:?}", transaction_with_proof);

                    let transaction = match self.get_transaction(transaction_with_proof.clone(), account_address.clone(), address) {
                        Ok(result) => result,
                        Err(_) => return TransactionStatus::FailedToGetTransaction,
                    };

                    let mut transactions: Vec<Transaction> = self.transactions.get(&account_address).unwrap_or(&Vec::new()).to_vec();
                    if !transactions.iter().any(|x| x.hash() == transaction.hash()) {
                        transactions.push(transaction.clone());
                        self.transactions.insert(account_address.clone(), transactions);
                    }

                    let tx_hash = transaction.hash().to_hex();
                    if self.verified.get(&tx_hash).is_some() && self.verified.get(&tx_hash).unwrap() == &true {
                        info!("transaction has been verified:{:}", self.verified.len());
                        return TransactionStatus::Ok;
                    }

                    if tx_hash != transaction_with_proof.transaction_info.transaction_hash.to_hex() {
                        error!("transaction hash doesn't match");
                        return TransactionStatus::FailedToVerify;
                    }

                    if let Ok(_) = self.verify_trusted_state(transaction_with_proof.clone()) {
                        let verified = self.verify_transaction_state_proof(transaction_with_proof, address);
                        self.verified.insert(tx_hash, verified);
                        if verified {
                            info!("transaction was verified:{:}", self.verified.len());
                            TransactionStatus::Ok
                        } else {
                            TransactionStatus::FailedToVerify
                        }
                    } else {
                        TransactionStatus::FailedToVerify
                    }
                } else {
                    TransactionStatus::InvalidAccount
                }
            }
            Command::NewAccount { seq_number } => {
                println!("NewAccount seq_number:{:}",seq_number);
                let alice_account_address = self.accounts_address[0];
                let alice_key_pair = self.account_keypairs.get(&alice_account_address.to_string()).unwrap().clone();

                let mut seed_rng = OsRng;
                let mut rng = rand::rngs::StdRng::from_seed(seed_rng.gen());
                let keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
                    Ed25519PrivateKey::generate(&mut rng).into();
                let auth_key = AuthenticationKey::ed25519(&keypair.public_key).to_vec();
                let receiver_address = AuthenticationKey::ed25519(&keypair.public_key).derived_address();
                println!("new child address:{:?}", receiver_address);
                let receiver_auth_key_prefix = self.auth_key_prefix(auth_key);

                let mut script = transaction_builder::encode_create_child_vasp_account_script(
                    xus_tag(), receiver_address, receiver_auth_key_prefix,
                    false, 0,
                );

                let txn = create_user_txn(
                    alice_key_pair, TransactionPayload::Script(script),
                    alice_account_address, seq_number,
                    MAX_GAS_AMOUNT, GAS_UNIT_PRICE, XUS_NAME.to_owned(),
                    TX_EXPIRATION, ChainId::new(2),
                ).unwrap();
                println!("tx:{:?}", txn);

                self.accounts_address.push(receiver_address);
                self.account_keypairs.insert(receiver_address.to_string(), keypair);

                let transaction_data = TransactionData {
                    sequence: self.tx_sequence,
                    address: receiver_address.to_string().as_bytes().to_vec(),
                    signed_tx: bcs::to_bytes(&txn).unwrap(),
                };
                self.tx_queue.push(transaction_data);
                self.tx_sequence = self.tx_sequence + 1;

                TransactionStatus::Ok
            }
            Command::TransferXUS { from, to, amount, seq_number } => {
                println!("TransferXUS from: {:}, to: {:}, amount: {:}, seq_number:{:}", from, to, amount, seq_number);
                if let Some(sender) = self.get_account_address(from) {
                    if let Ok(receiver) = AccountAddress::from_hex_literal(&to) {
                        let sender_key_pair = self.account_keypairs.get(&sender.to_string()).unwrap();

                        let script = transaction_builder::encode_peer_to_peer_with_metadata_script(
                            xus_tag(),
                            receiver,
                            amount,
                            vec![],
                            vec![],
                        );

                        let txn = create_user_txn(
                            sender_key_pair, TransactionPayload::Script(script),
                            sender, seq_number,
                            MAX_GAS_AMOUNT, GAS_UNIT_PRICE, XUS_NAME.to_owned(),
                            TX_EXPIRATION, ChainId::new(2),
                        ).unwrap();
                        println!("tx:{:?}", txn);

                        let transaction_data = TransactionData {
                            sequence: self.tx_sequence,
                            address: receiver.to_string().as_bytes().to_vec(),
                            signed_tx: bcs::to_bytes(&txn).unwrap(),
                        };
                        self.tx_queue.push(transaction_data);
                        self.tx_sequence = self.tx_sequence + 1;

                        TransactionStatus::Ok
                    } else {
                        TransactionStatus::InvalidAccount
                    }
                } else {
                    TransactionStatus::InvalidAccount
                }
            }
        }
    }

    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response> {
            match req {
                Request::VerifiedTransactions => {
                    let hash: Vec<_> = self.verified.keys().cloned().collect();
                    Ok(Response::VerifiedTransactions {
                        hash
                    })
                },
                Request::GetSignedTransactions { start} => {
                    println!("GetSignedTransactions: {:}", start);
                    let queue: Vec<&TransactionData> = self
                        .tx_queue
                        .iter()
                        .filter(|x| x.sequence >= start)
                        .collect::<_>();
                    let queue_b64 = base64::encode(&queue.encode());
                    Ok(Response::GetSignedTransactions {
                        queue_b64,
                    })
                }
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

/// Parses a TrustedState from a bcs encoded LedgerInfoWithSignature in base64
fn parse_trusted_state(trusted_state_b64: &String) -> Result<TrustedState, TransactionStatus> {
    let trusted_state_data = base64::decode(trusted_state_b64)
        .or(Err(TransactionStatus::BadTrustedStateData))?;
    let zero_ledger_info_with_sigs: LedgerInfoWithSignatures =
        bcs::from_bytes(&trusted_state_data)
        .or(Err(TransactionStatus::BadLedgerInfo))?;
    TrustedState::try_from(zero_ledger_info_with_sigs.ledger_info())
        .or(Err(TransactionStatus::BadLedgerInfo))
}

impl core::fmt::Debug for Diem {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, r#"Dime {{
            accounts: {:?},
            }}"#, self.transactions)
    }
}
