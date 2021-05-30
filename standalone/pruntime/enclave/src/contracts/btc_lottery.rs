use crate::contracts;
use crate::contracts::AccountIdWrapper;
use bitcoin;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::transaction::{OutPoint, SigHashType, TxIn, TxOut};
use bitcoin::consensus::encode::serialize;
use bitcoin::network::constants::Network;
use bitcoin::secp256k1::{All, Message, Secp256k1, Signature};
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::{Address, PrivateKey, PublicKey, Script, Transaction, Txid};
use core::str::FromStr;
use lazy_static;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_core::crypto::Pair;
use sp_core::ecdsa;
use std::collections::btree_map::Entry::{Occupied, Vacant};
extern crate runtime as chain;
use crate::std::collections::BTreeMap;
use crate::std::string::{String, ToString};
use crate::std::vec::Vec;
use crate::types::TxRef;
use crate::TransactionStatus;
use log::error;
use rand::{rngs::StdRng, seq::IteratorRandom, SeedableRng};
use sp_core::hashing::blake2_256;
use sp_core::U256;
type SequenceType = u64;
const ALICE: &'static str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const RBF: u32 = 0xffffffff - 2;
lazy_static! {
    // 10000...000, used to tell if this is a NFT
    static ref TYPE_NF_BIT: U256 = U256::from(1) << 255;
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct BtcLottery {
    round_id: u32,
    token_set: BTreeMap<u32, Vec<String>>,
    lottery_set: BTreeMap<u32, BTreeMap<String, PrivateKey>>,
    sequence: SequenceType,
    queue: Vec<SendLotteryData>,
    #[serde(skip)]
    secret: Option<ecdsa::Pair>,
    /// round_id => (txid, vout, amount)?
    utxo: BTreeMap<u32, BTreeMap<Address, (Txid, u32, u64)>>,
    admin: AccountIdWrapper,
}

impl core::fmt::Debug for BtcLottery {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Hi")
    }
}

// These two structs below are used for transferring messages to chain.
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct SendLottery {
    chain_id: u8,
    payload: LotteryPayload,
    sequence: SequenceType,
}
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct SendLotteryData {
    data: SendLottery,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub enum LotteryPayload {
    SignedTx {
        round_id: u32,
        token_id: Vec<u8>,
        tx: Vec<u8>,
    },
    BtcAddresses {
        address_set: Vec<Vec<u8>>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    SubmitUtxo {
        round_id: u32,
        address: String,
        utxo: (Txid, u32, u64),
    },
    SetAdmin {
        new_admin: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    InvalidRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    GetAllRounds,
    GetRoundInfo { round_id: u32 },
    GetRoundAddress { round_id: u32 },
    QueryUtxo { round_id: u32 },
    // GetSignedTx { round_id: u32 },
    PendingLotteryEgress { sequence: SequenceType },
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    GetAllRounds {
        round_id: u32,
    },
    GetRoundInfo {
        token_number: u32,
        winner_count: u32,
    },
    GetRoundAddress {
        prize_addr: Vec<String>,
    },
    QueryUtxo {
        utxo: BTreeMap<Address, (Txid, u32, u64)>,
    },
    // GetSignedTx { round_id: u32 },
    PendingLotteryEgress {
        lottery_queue_b64: String,
    },
    Error(Error),
}

impl BtcLottery {
    /// Initializes the contract
    pub fn new(secret: Option<ecdsa::Pair>) -> Self {
        let token_set = BTreeMap::<u32, Vec<String>>::new();
        let lottery_set = BTreeMap::<u32, BTreeMap<String, PrivateKey>>::new();
        let utxo = BTreeMap::<u32, BTreeMap<Address, (Txid, u32, u64)>>::new();
        let admin = AccountIdWrapper::from_hex(ALICE);
        BtcLottery {
            round_id: 0,
            token_set,
            lottery_set,
            sequence: 0,
            queue: Vec::new(),
            secret,
            utxo,
            admin,
        }
    }

    pub fn sign(
        secp: &Secp256k1<All>,
        digest: &[u8],
        key: &PrivateKey,
    ) -> Result<Signature, bitcoin::secp256k1::Error> {
        Ok(secp.sign(&Message::from_slice(digest)?, &key.key))
    }

    pub fn new_round(&mut self, round_id: u32, total_count: u32, winner_count: u32) {
        if !self.token_set.contains_key(&round_id) && !self.lottery_set.contains_key(&round_id) {
            let sequence = self.sequence + 1;
            let secret = match self.secret.as_ref() {
                Some(s) => s,
                None => {
                    error!("LotteryNewRound: empty secret key");
                    return;
                }
            };
            let token_round_id: U256 = U256::from(round_id) << 128;
            let mut round_token = Vec::new();
            for token_no in 1..=total_count {
                let nft_id = (token_round_id + token_no) | *TYPE_NF_BIT;
                let token_id = format!("{:#x}", nft_id);
                round_token.push(token_id);
            }
            let mut lottery_token = BTreeMap::<String, PrivateKey>::new();
            let raw_seed = blake2_256(&Encode::encode(&(secret.to_raw_vec(), round_id)));
            let mut r: StdRng = SeedableRng::from_seed(raw_seed.clone());
            let sample = round_token
                .iter()
                .choose_multiple(&mut r, winner_count as usize);
            let mut address_set = Vec::new();
            let mut salt = round_id * 10000;
            for winner_id in sample {
                let s = Secp256k1::new();
                let raw_data = (raw_seed.clone(), salt);
                let seed = blake2_256(&Encode::encode(&raw_data));
                let sk = match ExtendedPrivKey::new_master(Network::Bitcoin, &seed) {
                    Ok(e) => e.private_key,
                    Err(err) => {
                        error!(
                            "LotteryNewRound: cannot create a new secret key from the seed: {:?}",
                            &seed
                        );
                        return;
                    }
                };
                let secp = Secp256k1::new();
                let public_key = PublicKey::from_private_key(&secp, &sk);
                let prize_addr = Address::p2pkh(&public_key, Network::Bitcoin);
                address_set.push(prize_addr.to_string().as_bytes().to_vec());
                lottery_token.insert(String::from(winner_id), sk);
                salt += 1;
            }
            self.lottery_set.insert(round_id, lottery_token);
            self.token_set.insert(round_id, round_token);
            self.round_id = round_id;

            let payload = LotteryPayload::BtcAddresses { address_set };
            let data = SendLottery {
                chain_id: 1,
                payload,
                sequence,
            };
            let signature = secret.sign(&Encode::encode(&data));

            println!("signature={:?}", signature);
            let transfer_data = SendLotteryData {
                data,
                signature: signature.0.to_vec(),
            };
            self.queue.push(transfer_data);
            self.sequence = sequence;
        } else {
            error!("Round {} has already started", round_id);
        }
    }

    pub fn open_lottery(&mut self, round_id: u32, token_id: u32, btc_address: Vec<u8>) {
        if self.lottery_set.contains_key(&round_id) && self.utxo.contains_key(&round_id) {
            let token_id = format!("{:#x}", token_id);
            // from Vec<u8> to String
            let btc_address = match String::from_utf8(btc_address.clone()) {
                Ok(e) => e,
                Err(err) => {
                    error!(
                        "LotteryOpenBox: cannot convert btc_address to String: {:?}",
                        &btc_address
                    );
                    return;
                }
            };
            let target = match Address::from_str(&btc_address) {
                Ok(e) => e,
                Err(error) => {
                    error!(
                        "LotteryOpenBox: cannot convert btc_address to Address: {:?}",
                        &btc_address
                    );
                    return;
                }
            };
            let sequence = self.sequence + 1;
            let data = if !self
                .lottery_set
                .get(&round_id)
                .expect("round_id is known in the lottery_set; qed")
                .contains_key(&token_id)
            {
                let payload = LotteryPayload::SignedTx {
                    round_id,
                    token_id: token_id.as_bytes().to_vec(),
                    tx: Vec::new(),
                };
                SendLottery {
                    chain_id: 1,
                    payload,
                    sequence,
                }
            } else {
                let secp = Secp256k1::new();
                let private_key: PrivateKey = *self
                    .lottery_set
                    .get(&round_id)
                    .expect("round_id is known in the lottery_set; qed")
                    .get(&token_id)
                    .expect("token_id is known in the lottery_set; qed");
                let public_key = PublicKey::from_private_key(&secp, &private_key);
                let prize_addr = Address::p2pkh(&public_key, Network::Bitcoin);
                let round_utxo = self
                    .utxo
                    .get(&round_id)
                    .expect("round_id is known in the utxo; qed");
                let (txid, vout, amount) = round_utxo
                    .get(&prize_addr)
                    .expect("address is known in the utxo; qed");
                let mut tx = Transaction {
                    input: vec![TxIn {
                        previous_output: OutPoint {
                            txid: *txid,
                            vout: *vout,
                        },
                        sequence: RBF,
                        witness: Vec::new(),
                        script_sig: Script::new(),
                    }],
                    // TODO: deal with fee
                    output: vec![TxOut {
                        script_pubkey: target.script_pubkey(),
                        value: *amount,
                    }],
                    lock_time: 0,
                    version: 2,
                };
                let sighash =
                    tx.signature_hash(0, &prize_addr.script_pubkey(), SigHashType::All.as_u32());
                let secp_sign: Secp256k1<All> = Secp256k1::<All>::new();
                let tx_sign = match Self::sign(&secp_sign, &sighash[..], &private_key) {
                    Ok(e) => e.serialize_der(),
                    Err(err) => {
                        error!(
                            "LotteryOpenBox: the signing of the tx meets some problems:{}",
                            err
                        );
                        return;
                    }
                };
                let mut with_hashtype = tx_sign.to_vec();
                with_hashtype.push(SigHashType::All.as_u32() as u8);
                tx.input[0].script_sig = Builder::new()
                    .push_slice(with_hashtype.as_slice())
                    .push_slice(public_key.to_bytes().as_slice())
                    .into_script();
                tx.input[0].witness.clear();
                let tx_bytes = serialize(&tx);
                let payload = LotteryPayload::SignedTx {
                    round_id,
                    token_id: token_id.as_bytes().to_vec(),
                    tx: tx_bytes,
                };
                SendLottery {
                    chain_id: 1,
                    payload,
                    sequence,
                }
            };
            let secret = match self.secret.as_ref() {
                Some(s) => s,
                None => {
                    error!("LotteryNewRound: empty secret key");
                    return;
                }
            };
            let signature = secret.sign(&Encode::encode(&data));

            println!("signature={:?}", signature);
            let transfer_data = SendLotteryData {
                data,
                signature: signature.0.to_vec(),
            };
            self.queue.push(transfer_data);
            self.sequence = sequence;
        } else {
            error!("Round {} has already started", round_id);
        }
    }
}

impl contracts::Contract<Command, Request, Response> for BtcLottery {
    // Returns the contract id
    fn id(&self) -> contracts::ContractId {
        contracts::BTC_LOTTERY
    }

    fn handle_command(
        &mut self,
        origin: &chain::AccountId,
        _txref: &TxRef,
        cmd: Command,
    ) -> TransactionStatus {
        match cmd {
            Command::SubmitUtxo {
                round_id,
                address,
                utxo,
            } => {
                let sender = AccountIdWrapper(origin.clone());
                let btc_address = match Address::from_str(&address) {
                    Ok(e) => e,
                    Err(error) => return TransactionStatus::BadCommand,
                };
                if self.admin == sender {
                    let round_utxo = match self.utxo.entry(round_id) {
                        Occupied(entry) => return TransactionStatus::BadCommand,
                        Vacant(entry) => entry.insert(Default::default()),
                    };
                    round_utxo.insert(btc_address, utxo);
                }
                TransactionStatus::Ok
            }
            Command::SetAdmin { new_admin } => {
                // TODO: listen to some specific privileged account instead of ALICE
                let sender = AccountIdWrapper(origin.clone());
                let new_admin = AccountIdWrapper::from_hex(&new_admin);
                if self.admin == sender {
                    self.admin = new_admin;
                }
                TransactionStatus::Ok
            }
        }
    }

    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        match req {
            Request::GetAllRounds => Response::GetAllRounds {
                round_id: self.round_id,
            },
            Request::GetRoundInfo { round_id } => {
                if self.token_set.contains_key(&round_id)
                    && self.lottery_set.contains_key(&round_id)
                {
                    let token_number = self
                        .token_set
                        .get(&round_id)
                        .expect("round_id is known in the token_set; qed")
                        .len();
                    let winner_count = self
                        .lottery_set
                        .get(&round_id)
                        .expect("round_id is known in the lottery_set; qed")
                        .len();
                    Response::GetRoundInfo {
                        token_number: token_number as u32,
                        winner_count: winner_count as u32,
                    }
                } else {
                    Response::Error(Error::InvalidRequest)
                }
            }
            Request::GetRoundAddress { round_id } => {
                if self.lottery_set.contains_key(&round_id) {
                    let temp = self
                        .lottery_set
                        .get(&round_id)
                        .expect("round_id is known in the lottery_set; qed");
                    let mut address_set = Vec::new();
                    for (_, private_key) in temp.iter() {
                        let secp = Secp256k1::new();
                        let public_key = PublicKey::from_private_key(&secp, &private_key);
                        let prize_addr = Address::p2pkh(&public_key, Network::Bitcoin);
                        address_set.push(prize_addr.to_string());
                    }
                    Response::GetRoundAddress {
                        prize_addr: address_set,
                    }
                } else {
                    Response::Error(Error::InvalidRequest)
                }
            }
            Request::QueryUtxo { round_id } => {
                if self.utxo.contains_key(&round_id) {
                    Response::QueryUtxo {
                        utxo: self
                            .utxo
                            .get(&round_id)
                            .expect("round_id is known in the utxo set; qed")
                            .clone(),
                    }
                } else {
                    Response::Error(Error::InvalidRequest)
                }
            }
            Request::PendingLotteryEgress { sequence } => {
                println!("PendingLotteryEgress");
                let transfer_queue: Vec<&SendLotteryData> = self
                    .queue
                    .iter()
                    .filter(|x| x.data.sequence > sequence)
                    .collect::<_>();

                Response::PendingLotteryEgress {
                    lottery_queue_b64: base64::encode(&transfer_queue.encode()),
                }
            }
        }
    }

    fn handle_event(&mut self, ce: chain::Event) {
        if let chain::Event::pallet_bridge_transfer(pe) = ce {
            if let chain::pallet_bridge_transfer::Event::LotteryNewRound(
                round_id,
                total_count,
                winner_count,
            ) = pe
            {
                Self::new_round(self, round_id, total_count, winner_count);
            } else if let chain::pallet_bridge_transfer::Event::LotteryOpenBox(
                round_id,
                token_id,
                btc_address,
            ) = pe
            {
                Self::open_lottery(self, round_id, token_id, btc_address);
            } else if let chain::pallet_bridge_transfer::Event::LotteryPayloadSend(
                chain_id,
                payload,
                sequence,
            ) = pe
            {
                // message dequeue
                self.queue.retain(|x| x.data.sequence > sequence);
                println!("queue len: {:}", self.queue.len());
            }
        }
    }
}
