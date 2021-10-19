use std::collections::BTreeMap;
use std::string::ToString;
use std::collections::btree_map::Entry::{Occupied, Vacant};

use anyhow::Result;
use core::{fmt, str};
use log::{error, info};

use crate::contracts;
use crate::contracts::AccountId;
use crate::TransactionResult;

//diem type
use core::convert::TryFrom;
use diem_crypto::hash::CryptoHash;
use diem_types::account_address::{AccountAddress, HashAccountAddress};
use diem_types::account_state_blob::AccountStateBlob;
use diem_types::epoch_change::EpochChangeProof;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use diem_types::proof::{AccountStateProof, TransactionAccumulatorProof, TransactionInfoWithProof};
use diem_types::transaction::TransactionInfo;
use diem_types::transaction::{SignedTransaction, Transaction, TransactionPayload};
use diem_types::trusted_state::{TrustedState, TrustedStateChange};
use move_core_types::transaction_argument::TransactionArgument;

use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    //hash::HashValue,
    test_utils::KeyPair,
    Uniform,
};
use diem_types::account_config::{xus_tag, XUS_NAME};
use diem_types::transaction::authenticator::AuthenticationKey;
use diem_types::{
    chain_id::{ChainId, NamedChain},
    transaction::helpers::create_user_txn,
};
type SparseMerkleProof = diem_types::proof::SparseMerkleProof<AccountStateBlob>;
use parity_scale_codec::{Decode, Encode};
use phala_types::messaging::{DiemCommand as Command, MessageOrigin, PushCommand};
use rand::{rngs::OsRng, Rng, SeedableRng};

use super::NativeContext;

const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 1_000_000;
const TX_EXPIRATION: i64 = 180;
const CHAIN_ID_UNINITIALIZED: u8 = 0;
const ALICE_PRIVATE_KEY: &[u8] =
    &hex_literal::hex!("818ad9a64e3d1bbc388f8bf1e43c78d125237b875a1b70a18f412f7d18efbeea");
const ALICE_ADDRESS: &str = "D4F0C053205BA934BB2AC0C4E8479E77";

const ALICE_PHALA: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

#[derive(Clone, Encode, Decode, Debug, PartialEq)]
pub struct Amount {
    pub amount: u64,
    pub currency: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct AccountInfo {
    pub address: AccountAddress,
    pub authentication_key: Option<Vec<u8>>,
    pub sequence_number: u64,
    pub sent_events_key: String,
    pub received_events_key: String,
    pub balances: Vec<Amount>,
}

#[derive(Debug, Encode, Decode, Clone)]
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

#[derive(Encode, Decode, Debug)]
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

#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {
    /// Gets all the verified transactions, in hex hash string
    VerifiedTransactions,
    ///Gets signed transaction, from start
    GetSignedTransactions {
        start: u64,
    },
    CurrentState,
    AccountData,
}

#[derive(Encode, Decode, Debug)]
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
    Error(String),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TransactionData {
    sequence: u64,
    address: Vec<u8>,
    signed_tx: Vec<u8>,
    new_account: bool,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PendingTransaction {
    sequence: u64,
    amount: u64,
    lock_time: u64,
    raw_tx: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
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

#[derive(Debug, Encode, Decode)]
pub struct AccountData {
    is_vasp: bool,
    address: AccountAddress,
    phala_address: AccountId,
    sequence: u64,
    free: u64,
    locked: u64,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct State {
    queue_seq: u64,
    account_address: Vec<String>,
}

pub struct Diem {
    chain_id: u8,
    account_info: Vec<AccountInfo>,
    transactions: BTreeMap<String, Vec<Transaction>>, //address => Transaction
    verified: BTreeMap<String, bool>,                 //Hash => Bool
    seq_number: BTreeMap<String, u64>,                //Address => seq
    // TODO: TrustedState is not serializable; instead we should serialize the embedded Waypoint,
    // which can be accessible when constructing the TrustedState from a LedgerInfo.
    #[serde(skip)]
    init_trusted_state: Option<TrustedState>,
    #[serde(skip)]
    trusted_state: Option<TrustedState>,

    accounts: BTreeMap<AccountId, Account>, //Phala => Diem
    address: BTreeMap<String, AccountId>,   // Diem => Phala
    account_address: Vec<String>,                  //Diem string
    pending_transactions: BTreeMap<String, Vec<PendingTransaction>>,

    //for query signed transactions
    tx_queue: Vec<TransactionData>,
    queue_seq: u64,

    //for tx timeout
    timestamp_usecs: u64,
}

impl Diem {
    pub fn new() -> Self {
        let alice_priv_key =
            Ed25519PrivateKey::from_bytes_unchecked(ALICE_PRIVATE_KEY).expect("Bad private key");
        let alice_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
            KeyPair::from(alice_priv_key);
        let alice_account_address =
            AuthenticationKey::ed25519(&alice_key_pair.public_key).derived_address();

        let alice_account = Account {
            address: alice_account_address,
            key: alice_key_pair,
            sequence: 0,
            event_id: 0,
            free: 0,
            locked: 0,
            is_child: false,
        };

        let alice_addr = AccountId::from_hex(ALICE_PHALA).expect("Bad init master account");
        let mut accounts = BTreeMap::<AccountId, Account>::new();
        accounts.insert(alice_addr.clone(), alice_account);

        let mut address = BTreeMap::<String, AccountId>::new();
        address.insert(ALICE_ADDRESS.to_string(), alice_addr);

        let mut account_address: Vec<String> = Vec::new();
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
        let transaction: Transaction =
            match bcs::from_bytes(&transaction_with_proof.transaction_bytes) {
                Ok(tx) => tx,
                Err(_) => {
                    error!("Decode transaction error");

                    return Err(anyhow::Error::msg(Error::Other(String::from(
                        "Decode transaction error",
                    ))));
                }
            };

        if self.verified.get(&transaction.hash().to_hex()).is_some() {
            return Ok(transaction);
        }

        let signed_tx: &SignedTransaction = transaction
            .as_signed_user_txn()
            .expect("This must be a user tx; qed.");
        let raw_transaction = signed_tx.clone().into_raw_transaction();
        if raw_transaction.sender() != address {
            // Incoming tx doesn't need to be sequential
            let mut found = false;
            if let TransactionPayload::Script(script) = raw_transaction.clone().into_payload() {
                for arg in script.args() {
                    if let TransactionArgument::Address(recv_address) = arg {
                        if recv_address == &address {
                            found = true;
                            break;
                        }
                    }
                }
            }

            if !found {
                error!("Bad receiver address");

                return Err(anyhow::Error::msg(Error::Other(String::from(
                    "Bad receiver address",
                ))));
            }
        } else {
            // Outgoing tx must be synced sequencely
            if let Some(seq) = self.seq_number.get(&account_address) {
                if seq + 1 != raw_transaction.sequence_number() {
                    error!("Bad sequence number");

                    return Err(anyhow::Error::msg(Error::Other(String::from(
                        "Bad sequence number",
                    ))));
                }
            }
            self.seq_number
                .insert(account_address.clone(), raw_transaction.sequence_number());
        }

        Ok(transaction)
    }

    pub fn maybe_update_balance(
        &mut self,
        transaction: &Transaction,
        account_address: String,
        address: AccountAddress,
    ) -> Result<()> {
        let o = self
            .address
            .get(&account_address)
            .ok_or(anyhow::Error::msg(Error::Other(
                "Bad account address".to_string(),
            )))?;
        let mut account = self
            .accounts
            .get_mut(&o)
            .ok_or(anyhow::Error::msg(Error::Other("Bad account".to_string())))?;

        let signed_tx: &SignedTransaction = transaction
            .as_signed_user_txn()
            .expect("Not a signed transaction");
        let raw_transaction = signed_tx.clone().into_raw_transaction();
        let sequence_number = raw_transaction.sequence_number();
        // TODO: check the whitelisted script here
        if let TransactionPayload::Script(script) = raw_transaction.clone().into_payload() {
            if transaction_builder::get_transaction_name(script.code()).as_str()
                != "peer_to_peer_with_metadata_transaction"
            {
                info!("Not a peer to peer transaction");
                return Ok(());
            }
            use TransactionArgument::*;
            let args = script.args();
            if let [Address(_addr), U64(amount), U8Vector(_), U8Vector(_)] = &args[..] {
                if raw_transaction.sender() != address {
                    account.free += amount;

                    let sender_address = raw_transaction.sender().to_string();
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
                    if account.is_child && raw_transaction.sequence_number() != account.sequence {
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

    fn update_pending_transactions(&mut self, account_address: String, sequence_number: u64) {
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
        epoch_change_proof: &EpochChangeProof,
    ) -> Result<()> {
        // Verify the new state
        let trusted_state = self.trusted_state.as_ref().ok_or_else(|| {
            anyhow::Error::msg(Error::Other("TrustedState uninitialized".to_string()))
        })?;
        let trusted_state_change = trusted_state
            .verify_and_ratchet(ledger_info_with_signatures, &epoch_change_proof)
            .or_else(|_| {
                Err(anyhow::Error::msg(Error::Other(
                    "Verify trust state error".to_string(),
                )))
            })?;
        // Update trusted_state on demand
        match trusted_state_change {
            TrustedStateChange::Epoch {
                new_state,
                latest_epoch_change_li,
            } => {
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
        let ledger_info_with_signatures: LedgerInfoWithSignatures =
            transaction_with_proof.ledger_info_with_signatures.clone();

        let ledger_info_to_transaction_info_proof: TransactionAccumulatorProof =
            transaction_with_proof
                .ledger_info_to_transaction_info_proof
                .clone();
        let transaction_info: TransactionInfo = transaction_with_proof.transaction_info.clone();
        let transaction_info_to_account_proof: SparseMerkleProof = transaction_with_proof
            .transaction_info_to_account_proof
            .clone();

        let transaction_info_with_proof =
            TransactionInfoWithProof::new(ledger_info_to_transaction_info_proof, transaction_info);

        let account_transaction_state_proof = AccountStateProof::new(
            transaction_info_with_proof,
            transaction_info_to_account_proof,
        );

        if let Ok(_) = account_transaction_state_proof.verify(
            ledger_info_with_signatures.ledger_info(),
            transaction_with_proof.version,
            address.hash(),
            Some(&transaction_with_proof.account_state_blob),
        ) {
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
    sender_address: String,
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

impl contracts::NativeContract for Diem {
    type Cmd = Command;
    type Event = ();
    type QReq = Request;
    type QResp = Response;

    fn id(&self) -> contracts::ContractId {
        contracts::id256(contracts::DIEM)
    }

    fn handle_command(
        &mut self,
        _context: &NativeContext,
        origin: MessageOrigin,
        cmd: PushCommand<Self::Cmd>,
    ) -> TransactionResult {
        let origin = match origin {
            MessageOrigin::AccountId(acc) => acc,
            _ => return Err(TransactionError::BadOrigin),
        };

        match cmd {
            Command::AccountInfo { account_info_b64 } => {
                info!("command account_info_b64:{:?}", account_info_b64);
                if let Ok(account_data) = base64::decode(&account_info_b64) {
                    let account_info: AccountInfo = match bcs::from_bytes(&account_data) {
                        Ok(result) => result,
                        Err(_) => return Err(TransactionError::BadAccountInfo),
                    };
                    info!("account_info:{:?}", account_info);
                    let exist = self
                        .account_info
                        .iter()
                        .any(|x| x.address == account_info.address);
                    if !exist {
                        self.account_info.push(account_info);
                    }
                    info!("add account_ok");
                    Ok(())
                } else {
                    Err(TransactionError::BadAccountInfo)
                }
            }
            Command::SetTrustedState {
                trusted_state_b64,
                chain_id,
            } => {
                info!(
                    "trusted_state_b64: {:?}, chain_id: {:}",
                    trusted_state_b64, chain_id
                );
                if chain_id != NamedChain::TESTNET.id() && chain_id != NamedChain::TESTING.id() {
                    return Err(TransactionError::BadChainId);
                }

                if self.chain_id == CHAIN_ID_UNINITIALIZED {
                    self.chain_id = chain_id;
                } else if chain_id != self.chain_id {
                    info!("Unexpected chain id, chain_id was not changed.")
                }

                // Only initialize TrustedState once
                if self.init_trusted_state.is_some() {
                    return Ok(());
                }
                match parse_trusted_state(&trusted_state_b64) {
                    Ok(trusted_state) => {
                        self.init_trusted_state = Some(trusted_state.clone());
                        self.trusted_state = Some(trusted_state);
                        info!("init trusted state OK");
                        Ok(())
                    }
                    Err(code) => code,
                }
            }
            Command::VerifyEpochProof {
                ledger_info_with_signatures_b64,
                epoch_change_proof_b64,
            } => {
                info!(
                    "ledger_info_with_signatures_b64: {:?}",
                    ledger_info_with_signatures_b64
                );
                info!("epoch_change_proof_b64: {:?}", epoch_change_proof_b64);
                let ledger_info_with_signatures_data =
                    base64::decode(ledger_info_with_signatures_b64)
                        .or(Err(Err(TransactionError::BadTrustedStateData)))
                        .expect("Bad trusted state data");
                let ledger_info_with_signatures: LedgerInfoWithSignatures =
                    bcs::from_bytes(&ledger_info_with_signatures_data)
                        .expect("Unable to parse ledger info");
                let epoch_change_proof_data = base64::decode(epoch_change_proof_b64)
                    .or(Err(Err(TransactionError::BadEpochChangedProofData)))
                    .expect("Bad epoch changed proof data");
                let epoch_change_proof: EpochChangeProof =
                    bcs::from_bytes(&epoch_change_proof_data)
                        .expect("Unable to parse epoch changed proof data");

                info!(
                    "ledger_info_with_signatures: {:?}",
                    ledger_info_with_signatures
                );
                if let Ok(_) =
                    self.verify_state_proof(&ledger_info_with_signatures, &epoch_change_proof)
                {
                    self.timestamp_usecs = ledger_info_with_signatures
                        .ledger_info()
                        .commit_info()
                        .timestamp_usecs();

                    Ok(())
                } else {
                    Err(TransactionError::FailedToVerify)
                }
            }
            Command::VerifyTransaction {
                account_address,
                transaction_with_proof_b64,
            } => {
                info!(
                    "transaction_with_proof_b64: {:?}",
                    transaction_with_proof_b64
                );

                if let Ok(address) =
                    AccountAddress::from_hex_literal(&("0x".to_string() + &account_address))
                {
                    if !self.account_info.iter().any(|x| x.address == address) {
                        error!("not a contract's account address");

                        return Err(TransactionError::InvalidAccount);
                    }

                    let proof_data =
                        base64::decode(&transaction_with_proof_b64).unwrap_or(Vec::new());
                    let transaction_with_proof: TransactionWithProof =
                        match bcs::from_bytes(&proof_data) {
                            Ok(result) => result,
                            Err(_) => return Err(TransactionError::BadTransactionWithProof),
                        };
                    info!("transaction_with_proof:{:?}", transaction_with_proof);

                    let transaction = match self.get_transaction(
                        transaction_with_proof.clone(),
                        account_address.clone(),
                        address,
                    ) {
                        Ok(result) => result,
                        Err(_) => return Err(TransactionError::FailedToGetTransaction),
                    };

                    let mut transactions: Vec<Transaction> = self
                        .transactions
                        .get(&account_address)
                        .unwrap_or(&Vec::new())
                        .to_vec();
                    if !transactions.iter().any(|x| x.hash() == transaction.hash()) {
                        transactions.push(transaction.clone());
                        self.transactions
                            .insert(account_address.clone(), transactions);
                    }

                    let tx_hash = transaction.hash().to_hex();
                    if self.verified.get(&tx_hash).is_some()
                        && self.verified.get(&tx_hash).unwrap() == &true
                    {
                        info!("transaction has been verified:{:}", self.verified.len());
                        return Ok(());
                    }

                    if tx_hash
                        != transaction_with_proof
                            .transaction_info
                            .transaction_hash()
                            .to_hex()
                    {
                        error!("transaction hash doesn't match");
                        return Err(TransactionError::FailedToVerify);
                    }

                    if let Ok(_) = self.verify_trusted_state(transaction_with_proof.clone()) {
                        let verified =
                            self.verify_transaction_state_proof(transaction_with_proof, address);
                        self.verified.insert(tx_hash, verified);
                        if verified {
                            info!("transaction was verified:{:}", self.verified.len());

                            if let Ok(_) =
                                self.maybe_update_balance(&transaction, account_address, address)
                            {
                                Ok(())
                            } else {
                                Err(TransactionError::FailedToCalculateBalance)
                            }
                        } else {
                            Err(TransactionError::FailedToVerify)
                        }
                    } else {
                        Err(TransactionError::FailedToVerify)
                    }
                } else {
                    Err(TransactionError::InvalidAccount)
                }
            }
            Command::NewAccount { seq_number } => {
                let o = origin.account()?;
                info!("NewAccount {:}, seq_number:{:}", o.to_string(), seq_number);

                let alice =
                    AccountId::from_hex(ALICE_PHALA).expect("Bad init master account");
                if o == alice {
                    error!("Alice can't execute NewAccount command");
                    return Err(TransactionError::InvalidAccount);
                }

                let alice_account = self
                    .accounts
                    .get(&alice)
                    .expect("Alice account was required");

                let alice_key_pair = &alice_account.key;
                // TODO: make deterministic privkey generation
                let mut seed_rng = OsRng;
                let mut rng = rand::rngs::StdRng::from_seed(seed_rng.gen());
                let keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
                    Ed25519PrivateKey::generate(&mut rng).into();
                let auth_key = AuthenticationKey::ed25519(&keypair.public_key).to_vec();
                let receiver_address =
                    AuthenticationKey::ed25519(&keypair.public_key).derived_address();
                info!("new child address:{:?}", receiver_address);
                let receiver_auth_key_prefix = auth_key_prefix(auth_key);

                let script = transaction_builder::encode_create_child_vasp_account_script(
                    xus_tag(),
                    receiver_address,
                    receiver_auth_key_prefix,
                    false,
                    0,
                );

                let txn = create_user_txn(
                    alice_key_pair,
                    TransactionPayload::Script(script),
                    alice_account.address,
                    seq_number,
                    MAX_GAS_AMOUNT,
                    GAS_UNIT_PRICE,
                    XUS_NAME.to_owned(),
                    (self.timestamp_usecs / 1000000) as i64 + TX_EXPIRATION,
                    ChainId::new(self.chain_id),
                )
                .expect("User signed transaction");
                info!("tx:{:?}", txn);

                let transaction_data = TransactionData {
                    sequence: self.queue_seq,
                    address: receiver_address.to_vec(),
                    signed_tx: bcs::to_bytes(&txn).expect("Serialization should work"),
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

                Ok(())
            }
            Command::TransferXUS { to, amount } => {
                let o = origin.account()?;
                info!(
                    "TransferXUS from: {:}, to: {:}, amount: {:}",
                    o.to_string(),
                    to,
                    amount
                );

                if let Some(sender_account) = self.accounts.get_mut(&o) {
                    if !sender_account.is_child {
                        error!("Not Allowed");
                        return Err(TransactionError::TransferringNotAllowed);
                    }
                    if sender_account.free < amount {
                        error!("InsufficientBalance");
                        return Err(TransactionError::InsufficientBalance);
                    }

                    let timestamp_usecs = self.timestamp_usecs;
                    let sender_address = sender_account.address.to_string();
                    if let Some(pending_transactions) =
                        self.pending_transactions.get(&sender_address)
                    {
                        // process timeout
                        if let Some(item) = pending_transactions.into_iter().find(|x| {
                            x.lock_time + TX_EXPIRATION as u64 * 1000000 < timestamp_usecs
                        }) {
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
                            self.pending_transactions
                                .insert(sender_address.clone(), pts);
                        }
                    }

                    if let Ok(receiver) =
                        AccountAddress::from_hex_literal(&("0x".to_string() + &to.to_string()))
                    {
                        if sender_account.address == receiver {
                            error!("Can't fund yourself");
                            return Err(TransactionError::InvalidAccount);
                        }

                        let sequence = account_sequence(
                            self.pending_transactions.clone(),
                            sender_account.sequence,
                            sender_address.clone(),
                        );

                        let script = transaction_builder::encode_peer_to_peer_with_metadata_script(
                            xus_tag(),
                            receiver,
                            amount,
                            vec![],
                            vec![],
                        );

                        let txn = create_user_txn(
                            &sender_account.key,
                            TransactionPayload::Script(script),
                            sender_account.address,
                            sequence,
                            MAX_GAS_AMOUNT,
                            GAS_UNIT_PRICE,
                            XUS_NAME.to_owned(),
                            (self.timestamp_usecs / 1000000) as i64 + TX_EXPIRATION,
                            ChainId::new(self.chain_id),
                        )
                        .expect("User signed transaction");
                        info!("tx:{:?}", txn);

                        let transaction_data = TransactionData {
                            sequence: self.queue_seq,
                            address: receiver.to_vec(),
                            signed_tx: bcs::to_bytes(&txn).expect("Serialization should work"),
                            new_account: false,
                        };
                        self.tx_queue.push(transaction_data);
                        self.queue_seq = self.queue_seq + 1;

                        let pending_tx = PendingTransaction {
                            sequence,
                            amount,
                            lock_time: self.timestamp_usecs,
                            raw_tx: bcs::to_bytes(&txn).expect("Serialization should work"),
                        };

                        match self.pending_transactions.entry(sender_address) {
                            Vacant(entry) => entry.insert(vec![]),
                            Occupied(entry) => entry.into_mut(),
                        }
                        .push(pending_tx);

                        sender_account.locked += amount;
                        sender_account.free -= amount;

                        info!("pending_transactions:{:?}", self.pending_transactions);
                        info!("accounts:{:?}", self.accounts);

                        Ok(())
                    } else {
                        Err(TransactionError::InvalidAccount)
                    }
                } else {
                    Err(TransactionError::InvalidAccount)
                }
            }
        }
    }

    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response> {
            match req {
                Request::VerifiedTransactions => {
                    let hash: Vec<_> = self.verified.keys().cloned().collect();
                    Ok(Response::VerifiedTransactions { hash })
                }
                Request::GetSignedTransactions { start } => {
                    info!("GetSignedTransactions: {:}", start);
                    let queue: Vec<&TransactionData> = self
                        .tx_queue
                        .iter()
                        .filter(|x| x.sequence >= start)
                        .collect();
                    let queue_b64 = base64::encode(&queue.encode());
                    Ok(Response::GetSignedTransactions { queue_b64 })
                }
                Request::CurrentState => {
                    let state = State {
                        queue_seq: self.queue_seq,
                        account_address: self.account_address.clone(),
                    };

                    Ok(Response::CurrentState { state })
                }
                Request::AccountData => {
                    let mut account_data = Vec::new();
                    for account_address in self.account_address.iter() {
                        if let Some(phala_address) = self.address.get(account_address) {
                            if let Some(account) = self.accounts.get(&phala_address) {
                                let ad = AccountData {
                                    is_vasp: !account.is_child,
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
                    Ok(Response::AccountData { data: account_data })
                }
            }
        };
        match inner() {
            Err(error) => Response::Error(error.to_string()),
            Ok(resp) => resp,
        }
    }
}

/// Parses a TrustedState from a bcs encoded LedgerInfoWithSignature in base64
fn parse_trusted_state(trusted_state_b64: &String) -> Result<TrustedState, TransactionResult> {
    let trusted_state_data =
        base64::decode(trusted_state_b64).or(Err(Err(TransactionError::BadTrustedStateData)))?;
    let zero_ledger_info_with_sigs: LedgerInfoWithSignatures =
        bcs::from_bytes(&trusted_state_data).or(Err(Err(TransactionError::BadLedgerInfo)))?;
    TrustedState::try_from(zero_ledger_info_with_sigs.ledger_info())
        .or(Err(Err(TransactionError::BadLedgerInfo)))
}

impl core::fmt::Debug for Diem {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            r#"Dime {{
            accounts: {:?},
            }}"#,
            self.transactions
        )
    }
}
