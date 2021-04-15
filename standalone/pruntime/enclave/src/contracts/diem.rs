use crate::std::collections::BTreeMap;
use crate::std::collections::btree_map::Entry;
use crate::std::string::String;
use crate::std::vec::Vec;

use anyhow::Result;
use core::{fmt, str};
use log::{error, info, warn};
use serde::{Serialize, Deserialize};

use crate::contracts;
use crate::contracts::AccountIdWrapper;
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
    chain_id::{ChainId, NamedChain},
    transaction::{
        helpers::{create_user_txn},
    }
};
use crate::hex;
const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 1_000_000;
const TX_EXPIRATION: i64 = 180;
const CHAIN_ID_UNINITIALIZED: u8 = 0;
const ALICE_PRIVATE_KEY: &str = "818ad9a64e3d1bbc388f8bf1e43c78d125237b875a1b70a18f412f7d18efbeea";
const ALICE_ADDRESS: &str = "D4F0C053205BA934BB2AC0C4E8479E77";

const ALICE_PHALA: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

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
    AccountInfo { account_info_b64: String },
    /// Verifies a transactions
    VerifyTransaction { account_address: String, transaction_with_proof_b64: String },
    /// Sets the trusted state. The owner can only initialize the bridge with the genesis state
    /// once.
    SetTrustedState { trusted_state_b64: String, chain_id: u8 },
    VerifyEpochProof { ledger_info_with_signatures_b64: String, epoch_change_proof_b64: String },

    NewAccount { seq_number: u64 },
    TransferXUS { to: String, amount: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    /// Gets all the verified transactions, in hex hash string
    VerifiedTransactions,
    ///Gets signed transaction, from start
    GetSignedTransactions { start: u64},
    CurrentState,
    AccountData,
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
    CurrentState {
        state: State,
    },
    AccountData {
        data: Vec<AccountData>,
    },
    /// Some other errors
    Error(#[serde(with = "super::serde_anyhow")] anyhow::Error)
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct TransactionData {
    sequence: u64,
    address: Vec<u8>,
    signed_tx: Vec<u8>,
    new_account: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct PendingTransaction {
    sequence: u64,
    amount: u64,
    lock_time: u64,
    raw_tx: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    address: AccountAddress,
    //#[serde(skip)]
    key: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    sequence: u64,
    event_id: u64,
    free: u64,
    locked: u64,
    is_child: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountData {
    address: AccountAddress,
    phala_address: AccountIdWrapper,
    sequence: u64,
    free: u64,
    locked: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    queue_seq: u64,
    account_address: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Diem {
    chain_id: u8,
    account_info: Vec<AccountInfo>,
    transactions: BTreeMap<String, Vec<Transaction>>, //address => Transaction
    verified: BTreeMap<String, bool>, //Hash => Bool
    seq_number: BTreeMap<String, u64>, //Address => seq
    // TODO: TrustedState is not serializable; instead we should serialize the embedded Waypoint,
    // which can be accessible when constructing the TrustedState from a LedgerInfo.
    #[serde(skip)]
    init_trusted_state: Option<TrustedState>,
    #[serde(skip)]
    trusted_state: Option<TrustedState>,

    accounts: BTreeMap<AccountIdWrapper, Account>, //Phala => Diem
    address: BTreeMap<String, AccountIdWrapper>, // Diem => Phala
    account_address: Vec<String>, //Diem string
    pending_transactions: BTreeMap<String, Vec<PendingTransaction>>,

    //for query signed transactions
    tx_queue: Vec<TransactionData>,
    queue_seq: u64,

    //for tx timeout
    timestamp_usecs: u64,
}

impl Diem {
    pub fn new() -> Self {
        let alice_priv_key = Ed25519PrivateKey::from_bytes_unchecked(
            &hex::decode_hex(ALICE_PRIVATE_KEY)).unwrap();
        let alice_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> = KeyPair::from(alice_priv_key);
        let alice_account_address = AuthenticationKey::ed25519(&alice_key_pair.public_key).derived_address();

        let alice_account = Account {
            address: alice_account_address,
            key: alice_key_pair,
            sequence: 0,
            event_id: 0,
            free: 0,
            locked: 0,
            is_child: false,
        };

        let mut accounts = BTreeMap::<AccountIdWrapper, Account>::new();
        accounts.insert(AccountIdWrapper::from_hex(ALICE_PHALA), alice_account);

        let mut address = BTreeMap::<String, AccountIdWrapper>::new();
        address.insert(ALICE_ADDRESS.to_string(), AccountIdWrapper::from_hex(ALICE_PHALA));

        let mut account_address: Vec<String>  = Vec::new();
        account_address.push(ALICE_ADDRESS.to_string());

        Diem {
            chain_id: CHAIN_ID_UNINITIALIZED,
            account_info: Vec::new(),
            transactions: BTreeMap::<String, Vec<Transaction>>::new(),
            verified: BTreeMap::<String, bool>::new(),
            seq_number: BTreeMap::<String, u64>::new(),
            init_trusted_state: None,
            trusted_state: None,
            accounts,
            address,
            account_address,
            pending_transactions: BTreeMap::<String, Vec<PendingTransaction>>::new(),
            queue_seq: 1,
            tx_queue: Vec::new(),
            timestamp_usecs: 0,
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

        let signed_tx: SignedTransaction = transaction.as_signed_user_txn()
            .expect("This must be a user tx; qed.").clone();
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

    pub fn maybe_update_balance(
        &mut self,
        transaction: &Transaction,
        account_address: String,
        address: AccountAddress,
    ) -> Result<()> {
        let o = self.address.get(&account_address).ok_or(anyhow::Error::msg(Error::Other("Bad account address".to_string())))?;
        let mut account = self.accounts.get_mut(&o).ok_or(anyhow::Error::msg(Error::Other("Bad account".to_string())))?;

        let signed_tx: SignedTransaction = transaction.as_signed_user_txn().unwrap().clone();
        let sequence_number = signed_tx.raw_txn.sequence_number;
        // TODO: check the whitelisted script here
        if let TransactionPayload::Script(script) = signed_tx.raw_txn.payload {
            let args = script.args();
            if let [TransactionArgument::Address(_addr), TransactionArgument::U64(amount),
                TransactionArgument::U8Vector(_), TransactionArgument::U8Vector(_)] = &args[..] {
                if signed_tx.raw_txn.sender != address {
                    account.free += amount;

                    let sender_address = signed_tx.raw_txn.sender.to_string();
                    if let Some(so) = self.address.get(&sender_address) {
                        if let Some(sender_account) = self.accounts.get_mut(&so) {
                            if sender_account.locked >= *amount {
                                sender_account.locked -= *amount;
                            }
                            sender_account.sequence += 1;

                            self.update_pending_transactions(account_address, sequence_number);
                        }
                    }
                } else {
                    if account.is_child && signed_tx.raw_txn.sequence_number != account.sequence {
                        return Err(anyhow::Error::msg(Error::Other("Bad sequence".to_string())))?;
                    }

                    if account.locked >= *amount {
                        account.locked -= *amount;
                    }
                    account.sequence += 1;

                    self.update_pending_transactions(account_address, sequence_number);
                }
            }
        }

        info!("pending_transactions:{:?}", self.pending_transactions);
        info!("accounts:{:?}", self.accounts);

        Ok(())
    }

    fn update_pending_transactions(
        &mut self,
        account_address: String,
        sequence_number: u64,
    ) {
        if let Some(pending_transactions) = self.pending_transactions.get(&account_address) {
            let pts: Vec<PendingTransaction> = pending_transactions
                .iter()
                .filter(|x| x.sequence != sequence_number)
                .map(|x| PendingTransaction {
                    sequence: x.sequence,
                    amount: x.amount,
                    lock_time: x.lock_time,
                    raw_tx: x.raw_tx.clone(),
                })
                .collect();
            self.pending_transactions.insert(account_address, pts);
        }
    }

    pub fn verify_state_proof(
        &mut self,
        ledger_info_with_signatures: &LedgerInfoWithSignatures,
        epoch_change_proof: &EpochChangeProof
    ) -> Result<()> {
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

    pub fn verify_trusted_state(
        &mut self,
        transaction_with_proof: TransactionWithProof,
    ) -> Result<()> {
        let epoch_change_proof = &transaction_with_proof.epoch_change_proof;
        let ledger_info_with_signatures = &transaction_with_proof.ledger_info_with_signatures;
        self.verify_state_proof(ledger_info_with_signatures, epoch_change_proof)
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
            error!("Failed to verify transaction");
            false
        }
    }


}

fn account_sequence(
    self_pending_transactions: BTreeMap<String, Vec<PendingTransaction>>,
    sequence: u64,
    sender_address: String
) -> u64 {
    if let Some(pending_transactions) = self_pending_transactions.get(&sender_address) {
        if pending_transactions.len() == 0 {
            sequence
        } else {
            pending_transactions[pending_transactions.len() - 1].sequence
        }
    } else {
        sequence
    }
}

fn auth_key_prefix(auth_key: Vec<u8>) -> Vec<u8> {
    auth_key[0..16].to_vec()
}

impl contracts::Contract<Command, Request, Response> for Diem {
    fn id(&self) -> contracts::ContractId { contracts::DIEM }

    fn handle_command(&mut self, origin: &chain::AccountId, _txref: &TxRef, cmd: Command) -> TransactionStatus {
        match cmd {
            Command::AccountInfo { account_info_b64 } => {
                info!("command account_info_b64:{:?}", account_info_b64);
                if let Ok(account_data) = base64::decode(&account_info_b64) {
                    let account_info: AccountInfo = match bcs::from_bytes(&account_data) {
                        Ok(result) => result,
                        Err(_) => return TransactionStatus::BadAccountInfo,
                    };
                    info!("account_info:{:?}", account_info);
                    let exist = self.account_info.iter().any(|x| x.address == account_info.address);
                    if !exist {
                        self.account_info.push(account_info);
                    }
                    info!("add account_ok");
                    TransactionStatus::Ok
                } else {
                    TransactionStatus::BadAccountInfo
                }
            }
            Command::SetTrustedState { trusted_state_b64, chain_id } => {
                info!("trusted_state_b64: {:?}, chain_id: {:}", trusted_state_b64, chain_id);
                if chain_id != NamedChain::TESTNET.id() && chain_id != NamedChain::TESTING.id() {
                    return TransactionStatus::BadChainId;
                }

                if self.chain_id == CHAIN_ID_UNINITIALIZED {
                    self.chain_id = chain_id;
                } else if chain_id != self.chain_id {
                    info!("Unexpected chain id, chain_id was not changed.")
                }

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
            Command::VerifyEpochProof { ledger_info_with_signatures_b64, epoch_change_proof_b64 } => {
                info!("ledger_info_with_signatures_b64: {:?}", ledger_info_with_signatures_b64);
                info!("epoch_change_proof_b64: {:?}", epoch_change_proof_b64);
                let ledger_info_with_signatures_data = base64::decode(ledger_info_with_signatures_b64)
                    .or(Err(TransactionStatus::BadTrustedStateData)).unwrap();
                let ledger_info_with_signatures: LedgerInfoWithSignatures =
                    bcs::from_bytes(&ledger_info_with_signatures_data).unwrap();
                let epoch_change_proof_data = base64::decode(epoch_change_proof_b64)
                    .or(Err(TransactionStatus::BadTrustedStateData)).unwrap();
                let epoch_change_proof: EpochChangeProof =
                    bcs::from_bytes(&epoch_change_proof_data).unwrap();

                info!("ledger_info_with_signatures: {:?}", ledger_info_with_signatures);
                if let Ok(_) = self.verify_state_proof(&ledger_info_with_signatures, &epoch_change_proof) {
                    self.timestamp_usecs = ledger_info_with_signatures.ledger_info().commit_info().timestamp_usecs();

                    TransactionStatus::Ok
                } else {
                    TransactionStatus::FailedToVerify
                }
            }
            Command::VerifyTransaction { account_address, transaction_with_proof_b64 } => {
                info!("transaction_with_proof_b64: {:?}", transaction_with_proof_b64);

                if let Ok(address) = AccountAddress::from_hex_literal(&("0x".to_string() + &account_address)) {
                    if !self.account_info.iter().any(|x| x.address == address) {
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

                            if let Ok(_) = self.maybe_update_balance(&transaction, account_address, address) {
                                TransactionStatus::Ok
                            } else {
                                TransactionStatus::FailedToCalculateBalance
                            }
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
                let o = AccountIdWrapper(origin.clone());
                info!("NewAccount {:}, seq_number:{:}", o.to_string(), seq_number);

                let alice = AccountIdWrapper::from_hex(ALICE_PHALA);
                if o == alice {
                    error!("Alice can't execute NewAccount command");
                    return TransactionStatus::InvalidAccount;
                }

                let alice_account = self.accounts.get(&alice).unwrap();

                let alice_key_pair = &alice_account.key;
                // TODO: make deterministic privkey generation
                let mut seed_rng = OsRng;
                let mut rng = rand::rngs::StdRng::from_seed(seed_rng.gen());
                let keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
                    Ed25519PrivateKey::generate(&mut rng).into();
                let auth_key = AuthenticationKey::ed25519(&keypair.public_key).to_vec();
                let receiver_address = AuthenticationKey::ed25519(&keypair.public_key).derived_address();
                info!("new child address:{:?}", receiver_address);
                let receiver_auth_key_prefix = auth_key_prefix(auth_key);

                let script = transaction_builder::encode_create_child_vasp_account_script(
                    xus_tag(), receiver_address, receiver_auth_key_prefix,
                    false, 0,
                );

                let txn = create_user_txn(
                    alice_key_pair, TransactionPayload::Script(script),
                    alice_account.address, seq_number,
                    MAX_GAS_AMOUNT, GAS_UNIT_PRICE, XUS_NAME.to_owned(),
                    (self.timestamp_usecs / 1000000) as i64 + TX_EXPIRATION, ChainId::new(self.chain_id),
                ).unwrap();
                info!("tx:{:?}", txn);

                let transaction_data = TransactionData {
                    sequence: self.queue_seq,
                    address: receiver_address.to_vec(),
                    signed_tx: bcs::to_bytes(&txn).unwrap(),
                    new_account: true,
                };
                self.tx_queue.push(transaction_data);
                self.queue_seq = self.queue_seq + 1;

                let account = Account {
                    address: receiver_address,
                    key: keypair,
                    event_id: 0,
                    sequence: 0,
                    free: 0,
                    locked: 0,
                    is_child: true,
                };

                self.accounts.insert(o.clone(), account);
                self.address.insert(receiver_address.to_string(), o);
                self.account_address.push(receiver_address.to_string());

                TransactionStatus::Ok
            }
            Command::TransferXUS { to, amount } => {
                let o = AccountIdWrapper(origin.clone());
                info!("TransferXUS from: {:}, to: {:}, amount: {:}", o.to_string(), to, amount);

                if let Some(sender_account) = self.accounts.get_mut(&o) {
                    if !sender_account.is_child {
                        error!("Not Allowed");
                        return TransactionStatus::TransferringNotAllowed;
                    }
                    if sender_account.free < amount {
                        error!("InsufficientBalance");
                        return TransactionStatus::InsufficientBalance;
                    }

                    let timestamp_usecs = self.timestamp_usecs;
                    let sender_address = sender_account.address.to_string();
                    if let Some(pending_transactions) = self.pending_transactions.get(&sender_address) {
                        // process timeout
                        if let Some(item) = pending_transactions
                            .into_iter()
                            .find(|x| x.lock_time + TX_EXPIRATION as u64 * 1000000 < timestamp_usecs) {
                            info!("tx timeout");
                            for pt in pending_transactions {
                                if pt.sequence >= item.sequence {
                                    sender_account.free += pt.amount;
                                    sender_account.locked -= pt.amount;
                                }
                            }
                            let pts: Vec<PendingTransaction> = pending_transactions
                                .iter()
                                .filter(|x| x.sequence < item.sequence)
                                .map(|x| PendingTransaction {
                                    sequence: x.sequence,
                                    amount: x.amount,
                                    lock_time: x.lock_time,
                                    raw_tx: x.raw_tx.clone(),
                                })
                                .collect();
                            self.pending_transactions.insert(sender_address.clone(), pts);
                        }
                    }

                    if let Ok(receiver) = AccountAddress::from_hex_literal(&("0x".to_string() + &to.to_string())) {
                        if sender_account.address == receiver {
                            error!("Can't fund yourself");
                            return TransactionStatus::InvalidAccount;
                        }

                        let sequence = account_sequence(
                            self.pending_transactions.clone(),
                            sender_account.sequence,
                            sender_address.clone());

                        let script = transaction_builder::encode_peer_to_peer_with_metadata_script(
                            xus_tag(),
                            receiver,
                            amount,
                            vec![],
                            vec![],
                        );

                        let txn = create_user_txn(
                            &sender_account.key, TransactionPayload::Script(script),
                            sender_account.address, sequence,
                            MAX_GAS_AMOUNT, GAS_UNIT_PRICE, XUS_NAME.to_owned(),
                            (self.timestamp_usecs / 1000000) as i64 + TX_EXPIRATION, ChainId::new(self.chain_id),
                        ).unwrap();
                        info!("tx:{:?}", txn);

                        let transaction_data = TransactionData {
                            sequence: self.queue_seq,
                            address: receiver.to_vec(),
                            signed_tx: bcs::to_bytes(&txn).unwrap(),
                            new_account: false,
                        };
                        self.tx_queue.push(transaction_data);
                        self.queue_seq = self.queue_seq + 1;

                        let pending_tx = PendingTransaction {
                            sequence,
                            amount,
                            lock_time: self.timestamp_usecs,
                            raw_tx: bcs::to_bytes(&txn).unwrap(),
                        };

                        if let Some(pending_transactions) = self.pending_transactions.get_mut(&sender_address) {
                            pending_transactions.push(pending_tx);
                        } else {
                            let mut pending_transactions: Vec<PendingTransaction> = Vec::new();
                            pending_transactions.push(pending_tx);
                            self.pending_transactions.insert(sender_address, pending_transactions);
                        }

                        sender_account.locked += amount;
                        sender_account.free -= amount;

                        info!("pending_transactions:{:?}", self.pending_transactions);
                        info!("accounts:{:?}", self.accounts);

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
        let mut inner = || -> Result<Response> {
            match req {
                Request::VerifiedTransactions => {
                    let hash: Vec<_> = self.verified.keys().cloned().collect();
                    Ok(Response::VerifiedTransactions {
                        hash
                    })
                },
                Request::GetSignedTransactions { start} => {
                    info!("GetSignedTransactions: {:}", start);
                    let queue: Vec<&TransactionData> = self
                        .tx_queue
                        .iter()
                        .filter(|x| x.sequence >= start)
                        .collect();
                    let queue_b64 = base64::encode(&queue.encode());
                    Ok(Response::GetSignedTransactions {
                        queue_b64,
                    })
                },
                Request::CurrentState => {
                    let state = State {
                        queue_seq: self.queue_seq,
                        account_address: self.account_address.clone(),
                    };

                    Ok(Response::CurrentState { state })
                },
                Request::AccountData => {
                    let mut account_data = Vec::new();
                    for account_address in self.account_address.clone() {
                        if let Some(phala_address) = self.address.get(&account_address) {
                            if let Some(account) = self.accounts.get(&phala_address) {
                                if account.is_child {
                                    let ad = AccountData {
                                        address: account.address,
                                        phala_address: phala_address.clone(),
                                        sequence: account.sequence,
                                        free: account.free,
                                        locked: account.locked,
                                    };

                                    account_data.push(ad);
                                }
                            }
                        }
                    }
                    Ok(Response::AccountData { data: account_data })
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
