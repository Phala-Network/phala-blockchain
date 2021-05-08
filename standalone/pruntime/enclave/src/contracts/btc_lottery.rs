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
extern crate runtime as chain;
use crate::std::collections::BTreeMap;
use crate::std::string::String;
use crate::std::vec::Vec;
use crate::types::TxRef;
use crate::TransactionStatus;
use rand::Rng;
use sgx_rand::sample;
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
    // Txid, vout, amount
    utxo: BTreeMap<u32, BTreeMap<Address, (Txid, u32, u64)>>,
    admin: Vec<AccountIdWrapper>,
}

impl core::fmt::Debug for BtcLottery {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Hi")
    }
}

/// These two structs below are used for transferring messages to chain.
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct BtcTransfer {
    dest: AccountIdWrapper,
    tx: Vec<u8>,
    sequence: SequenceType,
}
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct BtcTransferData {
    data: BtcTransfer,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NotAuthorized,
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
    Error(Error),
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
    ) -> Result<Signature, Error> {
        Ok(secp.sign(&Message::from_slice(digest).unwrap(), &key.key))
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
                // 判断origin，只有Admin可以调用
                // 把utxo存储合约中，供以后查询
                let sender = AccountIdWrapper(_origin.clone());
                let btc_address = Address::from_str(&address).unwrap();
                if self.admin.contains(&sender) {
                    let mut round_utxo = match self.utxo.get(&round_id) {
                        Some(_) => self.utxo.get(&round_id).unwrap().clone(),
                        None => BTreeMap::<Address, (Txid, u32, u64)>::new(),
                    };
                    round_utxo.insert(btc_address, utxo);
                    self.utxo.insert(round_id, round_utxo);
                }
                TransactionStatus::Ok
            }
            Command::SetAdmin { new_admin } => {
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
        let inner = || -> Result<Response, Error> {
            match req {
                Request::GetAllRounds => {
                    return Ok(Response::GetAllRounds {
                        round_id: self.round_id.clone(),
                    })
                }
                Request::GetRoundInfo { round_id } => {
                    let token_number = self.token_set.get(&round_id).unwrap().len();
                    let winner_count = self.lottery_set.get(&round_id).unwrap().len();
                    return Ok(Response::GetRoundInfo {
                        token_number: token_number as u32,
                        winner_count: winner_count as u32,
                    });
                }
                Request::QueryUtxo { round_id } => {
                    return Ok(Response::QueryUtxo {
                        utxo: self.utxo.get(&round_id).unwrap().clone(),
                    })
                }
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp,
        }
    }

    fn handle_event(&mut self, ce: chain::Event) {
        if let chain::Event::pallet_kitties(pe) = ce {
            if let chain::pallet_kitties::RawEvent::NewLottery(total_count, winner_count) = pe {
                let raw_seed = LOTTERY.as_bytes();
                let mut nonce = 1;
                // This indicates the token's kind, let's just suppose that 1 represents box,
                // maybe if we introduce more tokens later, this can be formulated strictly
                let kind = 1;
                // For now, the token id is 0....01000...000, this will be used to construct the
                // box ID below, combined with random index id and NFT flags
                let token_id: U256 = U256::from(kind) << 128;
                for token in 0..total_count {
                    let mut rng = rand::thread_rng();
                    let seed: [u8; 16] = rng.gen();
                    let raw_data = (seed, nonce);
                    nonce += 1;
                    let hash_data = blake2_128(&Encode::encode(&raw_data));
                    let index_id = u128::from_be_bytes(hash_data);
                    let nft_id = (token_id + index_id) | *TYPE_NF_BIT;
                    let token_id = format!("{:#x}", nft_id);
                    let mut round_token = match self.token_set.get(&self.round_id) {
                        Some(_) => self.token_set.get(&self.round_id).unwrap().clone(),
                        None => Vec::new(),
                    };
                    round_token.push(token_id);
                    self.token_set.insert(self.round_id, round_token);
                }
                let mut r = sgx_rand::thread_rng();
                let sample = sample(
                    &mut r,
                    self.token_set.get(&self.round_id).unwrap().iter(),
                    winner_count as usize,
                );
                let mut lottery_token = match self.lottery_set.get(&self.round_id) {
                    Some(_) => self.lottery_set.get(&self.round_id).unwrap().clone(),
                    None => BTreeMap::<String, PrivateKey>::new(),
                };
                for winner_id in sample {
                    let mut salt = self.round_id * 10000;
                    let s = Secp256k1::new();
                    let raw_data = (raw_seed, salt);
                    let seed = blake2_128(&Encode::encode(&raw_data));
                    let pk = ExtendedPrivKey::new_master(Network::Bitcoin, &seed)
                        .unwrap()
                        .private_key;
                    lottery_token.insert(String::from(winner_id), pk);
                    salt += 1;
                }
                self.lottery_set.insert(self.round_id, lottery_token);
                self.round_id += 1;
            } else if let chain::pallet_kitties::RawEvent::Open(round_id, token_id, btc_address) =
                pe
            {
                let token_id = format!("{:#x}", token_id);
                let btc_address = format!("{:#x}", btc_address);
                let sender = AccountIdWrapper::from_hex(ALICE);
                let target = Address::from_str(&btc_address).unwrap();
                let sequence = self.sequence + 1;
                let data;
                if (!self
                    .lottery_set
                    .get(&round_id)
                    .unwrap()
                    .contains_key(&token_id))
                {
                    data = BtcTransfer {
                        dest: sender.clone(),
                        tx: Vec::new(),
                        sequence,
                    };
                } else {
                    let secp = Secp256k1::new();
                    let private_key: PrivateKey = *self
                        .lottery_set
                        .get(&round_id)
                        .unwrap()
                        .get(&token_id)
                        .unwrap();
                    let public_key = PublicKey::from_private_key(&secp, &private_key);
                    let private_addr = Address::p2pkh(&public_key, Network::Bitcoin);
                    let round_utxo = self.utxo.get(&round_id).unwrap();
                    let (txid, vout, amount) = round_utxo.get(&private_addr).unwrap();
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
                        output: vec![TxOut {
                            script_pubkey: target.script_pubkey(),
                            value: *amount,
                        }],
                        lock_time: 0,
                        version: 2,
                    };
                    let sighash = tx.signature_hash(
                        0,
                        &private_addr.script_pubkey(),
                        SigHashType::All.as_u32(),
                    );
                    let secp_sign: Secp256k1<All> = Secp256k1::<All>::new();
                    // let tx_sign = secp_sign
                    //     .sign(&Message::from_slice(&sighash[..]), &private_key.key)
                    //     .serialize_der();
                    let tx_sign = Self::sign(&secp_sign, &sighash[..], &private_key)
                        .unwrap()
                        .serialize_der();
                    let mut with_hashtype = tx_sign.to_vec();
                    with_hashtype.push(SigHashType::All.as_u32() as u8);
                    tx.input[0].script_sig = Builder::new()
                        .push_slice(with_hashtype.as_slice())
                        .push_slice(public_key.to_bytes().as_slice())
                        .into_script();
                    tx.input[0].witness.clear();
                    let tx_bytes = serialize(&tx);
                    data = BtcTransfer {
                        dest: sender.clone(),
                        tx: tx_bytes,
                        sequence,
                    };
                }
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
        }
    }
}
