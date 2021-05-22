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
use crate::std::string::String;
use crate::std::vec::Vec;
use crate::types::TxRef;
use crate::TransactionStatus;
use log::info;
use rand::{rngs::SmallRng, seq::index::IndexVec, seq::IteratorRandom, Rng, SeedableRng};
use sp_core::hashing::blake2_128;
use sp_core::U256;
type SequenceType = u64;
const ALICE: &'static str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const RBF: u32 = 0xffffffff - 2;
const LOTTERY: &'static str = "BTC LOTTERY";
lazy_static! {
    // 10000...000, used to tell if this is a NFT
    static ref TYPE_NF_BIT: U256 = U256::from(1) << 255;
    // 1111...11000...00, used to store the type in the upper 128 bits
    static ref TYPE_MASK: U256 = U256::from(!u128::MIN) << 128;
    // 00...00011....11, used to get NFT index in the lower 128 bits
    static ref NF_INDEX_MASK: U256 = U256::from(!u128::MIN);
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct BtcLottery {
    round_id: u32,
    token_set: BTreeMap<u32, Vec<String>>,
    lottery_set: BTreeMap<u32, BTreeMap<String, PrivateKey>>,
    sequence: SequenceType,
    queue: Vec<BtcTransferData>,
    #[serde(skip)]
    secret: Option<ecdsa::Pair>,
    /// round_id => (txid, vout, amount)?
    utxo: BTreeMap<u32, BTreeMap<Address, (Txid, u32, u64)>>,
    admin: Vec<AccountIdWrapper>,
}

impl core::fmt::Debug for BtcLottery {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Hi")
    }
}

// These two structs below are used for transferring messages to chain.
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct BtcTransfer {
    round_id: u32,
    chain_id: u8,
    token_id: Vec<u8>,
    tx: Vec<u8>,
    sequence: SequenceType,
}
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct BtcTransferData {
    data: BtcTransfer,
    signature: Vec<u8>,
}

// Phala端调用
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    // 提交某个round下的UTXO（不验证，只用于公示）
    SubmitUtxo {
        round_id: u32,
        address: String,
        utxo: (Txid, u32, u64),
    },
    // 设置管理员账号（只有管理员自己才能更改）
    SetAdmin {
        new_admin: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    // 查询返回当前round id
    GetAllRounds,
    // 查询某个round的信息：NFT总数、winner总数
    GetRoundInfo { round_id: u32 },
    // 查询某个round的Utxo
    QueryUtxo { round_id: u32 },
    // 获取某个round的已签名交易
    // GetSignedTx { round_id: u32 },
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
    QueryUtxo {
        utxo: BTreeMap<Address, (Txid, u32, u64)>,
    },
    // GetSignedTx { round_id: u32 },
}

impl BtcLottery {
    /// Initializes the contract
    pub fn new(secret: Option<ecdsa::Pair>) -> Self {
        let token_set = BTreeMap::<u32, Vec<String>>::new();
        let lottery_set = BTreeMap::<u32, BTreeMap<String, PrivateKey>>::new();
        let utxo = BTreeMap::<u32, BTreeMap<Address, (Txid, u32, u64)>>::new();
        BtcLottery {
            round_id: 1,
            token_set,
            lottery_set,
            sequence: 0,
            queue: Vec::new(),
            secret,
            utxo,
            admin: Vec::new(),
        }
    }

    pub fn sign(
        secp: &Secp256k1<All>,
        digest: &[u8],
        key: &PrivateKey,
    ) -> Result<Signature, bitcoin::secp256k1::Error> {
        Ok(secp.sign(&Message::from_slice(digest)?, &key.key))
    }
}

impl contracts::Contract<Command, Request, Response> for BtcLottery {
    // Returns the contract id
    fn id(&self) -> contracts::ContractId {
        contracts::BTC_LOTTERY
    }

    fn handle_command(
        &mut self,
        _origin: &chain::AccountId,
        _txref: &TxRef,
        cmd: Command,
    ) -> TransactionStatus {
        match cmd {
            Command::SubmitUtxo {
                round_id,
                address,
                utxo,
            } => {
                let sender = AccountIdWrapper(_origin.clone());
                let btc_address = match Address::from_str(&address) {
                    Ok(e) => e,
                    Err(error) => return TransactionStatus::BadCommand,
                };
                if self.admin.contains(&sender) {
                    let round_utxo = match self.utxo.entry(round_id) {
                        Occupied(entry) => entry.into_mut(),
                        Vacant(entry) => entry.insert(Default::default()),
                    };
                    round_utxo.insert(btc_address, utxo);
                }
                TransactionStatus::Ok
            }
            Command::SetAdmin { new_admin } => {
                // TODO: listen to some specific privileged account instead of ALICE
                let origin_admin = AccountIdWrapper::from_hex(ALICE);
                let sender = AccountIdWrapper(_origin.clone());
                let new_admin = AccountIdWrapper::from_hex(&new_admin);
                if sender == origin_admin {
                    self.admin.push(new_admin);
                }
                TransactionStatus::Ok
            }
        }
    }

    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        match req {
            Request::GetAllRounds => {
                return Response::GetAllRounds {
                    round_id: self.round_id,
                }
            }
            Request::GetRoundInfo { round_id } => {
                if (self.token_set.contains_key(&round_id)
                    && self.lottery_set.contains_key(&round_id))
                {
                    let token_number = self
                        .token_set
                        .get(&round_id)
                        .expect("round_id is invalid in token_set")
                        .len();
                    let winner_count = self
                        .lottery_set
                        .get(&round_id)
                        .expect("round_id is invalid in lottery_set")
                        .len();
                    return Response::GetRoundInfo {
                        token_number: token_number as u32,
                        winner_count: winner_count as u32,
                    };
                } else {
                    return Response::GetRoundInfo {
                        token_number: 0,
                        winner_count: 0,
                    };
                }
            }
            Request::QueryUtxo { round_id } => {
                if (self.utxo.contains_key(&round_id)) {
                    return Response::QueryUtxo {
                        utxo: self
                            .utxo
                            .get(&round_id)
                            .expect("round_id is invalid in utxo")
                            .clone(),
                    };
                } else {
                    return Response::QueryUtxo {
                        utxo: BTreeMap::<Address, (Txid, u32, u64)>::new(),
                    };
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
                let raw_seed = self.secret.as_ref().unwrap().to_raw_vec();
                let token_id: U256 = U256::from(round_id) << 128;
                let mut round_token = match self.token_set.get(&round_id) {
                    Some(_) => self
                        .token_set
                        .get(&round_id)
                        .expect("invalid round_id in token_set")
                        .clone(),
                    None => Vec::new(),
                };
                for index_id in 0..total_count {
                    let nft_id = (token_id + index_id) | *TYPE_NF_BIT;
                    let token_id = format!("{:#x}", nft_id);
                    round_token.push(token_id);
                }
                let lottery_token = match self.lottery_set.entry(round_id) {
                    Occupied(entry) => entry.into_mut(),
                    Vacant(entry) => entry.insert(Default::default()),
                };
                let mut r = rand::rngs::SmallRng::from_entropy();
                let sample = round_token
                    .iter()
                    .choose_multiple(&mut r, winner_count as usize);
                let mut salt = round_id * 10000;
                for winner_id in sample {
                    let s = Secp256k1::new();
                    let raw_data = (raw_seed.clone(), salt);
                    let seed = blake2_128(&Encode::encode(&raw_data));
                    let sk = match ExtendedPrivKey::new_master(Network::Bitcoin, &seed) {
                        Ok(e) => e.private_key,
                        Err(err) => {
                            info!("error in handle_event");
                            return;
                        }
                    };
                    lottery_token.insert(String::from(winner_id), sk);
                    salt += 1;
                }
                self.token_set.insert(round_id, round_token);
                self.round_id = round_id;
            } else if let chain::pallet_bridge_transfer::Event::LotteryOpenBox(
                round_id,
                token_id,
                btc_address,
            ) = pe
            {
                if (self.lottery_set.contains_key(&round_id) && self.utxo.contains_key(&round_id)) {
                    let token_id = format!("{:#x}", token_id);
                    // from Vec<u8> to String
                    let btc_address = match String::from_utf8(btc_address.clone()) {
                        Ok(e) => e,
                        Err(err) => {
                            info!("error in handle_event");
                            return;
                        }
                    };
                    let sender = AccountIdWrapper::from_hex(ALICE);
                    let target = match Address::from_str(&btc_address) {
                        Ok(e) => e,
                        Err(error) => {
                            info!("error in handle_event");
                            return;
                        }
                    };
                    let sequence = self.sequence + 1;
                    let data = if (!self
                        .lottery_set
                        .get(&round_id)
                        .expect("round_id is invalid in lottery_set")
                        .contains_key(&token_id))
                    {
                        BtcTransfer {
                            round_id,
                            chain_id: 1,
                            token_id: token_id.as_bytes().to_vec(),
                            tx: Vec::new(),
                            sequence,
                        }
                    } else {
                        let secp = Secp256k1::new();
                        let private_key: PrivateKey = *self
                            .lottery_set
                            .get(&round_id)
                            .expect("round_id is invalid in lottery_set")
                            .get(&token_id)
                            .expect("token_id is invalid in lottery_set");
                        let public_key = PublicKey::from_private_key(&secp, &private_key);
                        let prize_addr = Address::p2pkh(&public_key, Network::Bitcoin);
                        let round_utxo = self
                            .utxo
                            .get(&round_id)
                            .expect("round_id is invalid in utxo");
                        let (txid, vout, amount) = round_utxo
                            .get(&prize_addr)
                            .expect("address is invalid in utxo");
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
                        let sighash = tx.signature_hash(
                            0,
                            &prize_addr.script_pubkey(),
                            SigHashType::All.as_u32(),
                        );
                        let secp_sign: Secp256k1<All> = Secp256k1::<All>::new();
                        let tx_sign = match Self::sign(&secp_sign, &sighash[..], &private_key) {
                            Ok(e) => e.serialize_der(),
                            Err(err) => {
                                info!("error in handle_event");
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
                        BtcTransfer {
                            round_id,
                            chain_id: 1,
                            token_id: token_id.as_bytes().to_vec(),
                            tx: tx_bytes,
                            sequence,
                        }
                    };
                    let secret = self.secret.as_ref().unwrap();
                    let signature = secret.sign(&Encode::encode(&data));

                    println!("signature={:?}", signature);
                    let transfer_data = BtcTransferData {
                        data,
                        signature: signature.0.to_vec(),
                    };
                    self.queue.push(transfer_data);
                    self.sequence = sequence;
                }
            } else if let chain::pallet_bridge_transfer::Event::BTCSignedTxSend(
                round_id,
                chain_id,
                token_id,
                tx,
                sequence,
            ) = pe
            {
                let transfer_data = BtcTransferData {
                    data: BtcTransfer {
                        round_id,
                        chain_id,
                        token_id: token_id.to_vec(),
                        tx,
                        sequence,
                    },
                    signature: Vec::new(),
                };
                println!("transfer data:{:?}", transfer_data);
                // message dequeue
                self.queue
                    .retain(|x| x.data.sequence > transfer_data.data.sequence);
                println!("queue len: {:}", self.queue.len());
            }
        }
    }
}
