use super::{account_id_from_hex, TransactionError, TransactionResult};
use crate::chain;
use crate::contracts::{self, AccountId};

use std::{
    collections::{
        btree_map::Entry::{Occupied, Vacant},
        BTreeMap,
    },
    str::FromStr,
    string::ToString,
};

use anyhow::Result;
use lazy_static;
use log::error;
use parity_scale_codec::{Decode, Encode};
use phala_mq::{MessageOrigin, Sr25519MessageChannel as MessageChannel};
use rand::{rngs::StdRng, seq::IteratorRandom, SeedableRng};
use sp_core::{crypto::Pair, hashing::blake2_256, sr25519, U256};
use sp_runtime_interface::pass_by::PassByInner as _;

use bitcoin;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::transaction::{OutPoint, SigHashType, TxIn, TxOut};
use bitcoin::consensus::encode::serialize;
use bitcoin::network::constants::Network;
use bitcoin::secp256k1::{All, Secp256k1, Signature};
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::{Address, PrivateKey, PublicKey, Script, Transaction, Txid as BtcTxid};
use bitcoin_hashes::Hash as _;

use phala_types::messaging::{
    Lottery, LotteryCommand as Command, LotteryPalletCommand, LotteryUserCommand, Txid,
};

use super::NativeContext;

type SequenceType = u64;
const ALICE: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const RBF: u32 = 0xffffffff - 2;
lazy_static! {
    // 10000...000, used to tell if this is a NFT
    static ref TYPE_NF_BIT: U256 = U256::from(1) << 255;
}

pub struct BtcLottery {
    round_id: u32,
    token_set: BTreeMap<u32, Vec<String>>,
    lottery_set: BTreeMap<u32, BTreeMap<String, PrivateKey>>,
    tx_set: Vec<Vec<u8>>,
    sequence: SequenceType,        // Starting from zero
    secret: Option<sr25519::Pair>, // TODO: replace it with a seed.
    /// round_id => (txid, vout, amount)?
    utxo: BTreeMap<u32, BTreeMap<Address, (Txid, u32, u64)>>,
    admin: AccountId,
}

impl core::fmt::Debug for BtcLottery {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Hi")
    }
}

#[derive(Encode, Decode, Debug)]
pub enum Error {
    InvalidRequest,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {
    GetAllRounds,
    GetRoundInfo { round_id: u32 },
    GetRoundAddress { round_id: u32 },
    QueryUtxo { round_id: u32 },
    GetSignedTx { round_id: u32 },
}

type AddressString = String;

#[derive(Encode, Decode, Debug)]
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
        utxo: Vec<(AddressString, (Txid, u32, u64))>,
    },
    GetSignedTx {
        tx_set: Vec<Vec<u8>>,
    },
    PendingLotteryEgress {
        length: u64,
        lottery_queue_b64: String,
    },
    Error(Error),
}

impl BtcLottery {
    /// Initializes the contract
    pub fn new(secret: Option<sr25519::Pair>) -> Self {
        let token_set = BTreeMap::<u32, Vec<String>>::new();
        let lottery_set = BTreeMap::<u32, BTreeMap<String, PrivateKey>>::new();
        let utxo = BTreeMap::<u32, BTreeMap<Address, (Txid, u32, u64)>>::new();
        let admin = account_id_from_hex(ALICE).expect("Bad initial admin hex");
        BtcLottery {
            round_id: 0,
            token_set,
            lottery_set,
            tx_set: Vec::new(),
            sequence: 0,
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
        Ok(secp.sign(&bitcoin::secp256k1::Message::from_slice(digest)?, &key.key))
    }

    fn check_secret_key(&self) -> bool {
        if self.secret.is_none() {
            error!("Empty secret key");
            false
        } else {
            true
        }
    }

    pub fn new_round(
        &mut self,
        mq: &MessageChannel,
        round_id: u32,
        total_count: u32,
        winner_count: u32,
    ) {
        info!("new_round({}, {}, {})", round_id, total_count, winner_count);
        if !self.check_secret_key() {
            return;
        }
        if !self.token_set.contains_key(&round_id) && !self.lottery_set.contains_key(&round_id) {
            let _sequence = self.sequence;
            let secret = self.secret.as_ref().expect("Key is checked; qed.");
            let token_round_id: U256 = U256::from(round_id) << 128;
            let mut round_token = Vec::new();
            for token_no in 1..=total_count {
                let nft_id = (token_round_id + token_no) | *TYPE_NF_BIT;
                let token_id = format!("{:#x}", nft_id);
                round_token.push(token_id);
            }
            info!("new_round: n round_token: {}", round_token.len());
            let mut lottery_token = BTreeMap::<String, PrivateKey>::new();
            let raw_seed = blake2_256(&Encode::encode(&(secret.to_raw_vec(), round_id)));
            let mut r: StdRng = SeedableRng::from_seed(raw_seed);
            let sample = round_token
                .iter()
                .choose_multiple(&mut r, winner_count as usize);

            info!("new_round: n sampled: {}", sample.len());
            let mut address_set = Vec::new();
            let mut salt = round_id * 10000;
            for winner_id in sample {
                let raw_data = (raw_seed, salt);
                let seed = blake2_256(&Encode::encode(&raw_data));
                let sk = match ExtendedPrivKey::new_master(Network::Bitcoin, &seed) {
                    Ok(e) => e.private_key,
                    Err(_err) => {
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

            mq.send(&Lottery::BtcAddresses { address_set });
        } else {
            error!("Round {} has already started", round_id);
        }
    }

    pub fn open_lottery(
        &mut self,
        mq: &MessageChannel,
        round_id: u32,
        token_no: u32,
        btc_address: Vec<u8>,
    ) {
        if !self.check_secret_key() {
            return;
        }
        if self.lottery_set.contains_key(&round_id) && self.utxo.contains_key(&round_id) {
            let token_round_id: U256 = U256::from(round_id) << 128;
            let nft_id = (token_round_id + token_no) | *TYPE_NF_BIT;
            let token_id = format!("{:#x}", nft_id);
            // from Vec<u8> to String
            let btc_address = match String::from_utf8(btc_address.clone()) {
                Ok(e) => e,
                Err(_err) => {
                    error!(
                        "LotteryOpenBox: cannot convert btc_address to String: {:?}",
                        &btc_address
                    );
                    return;
                }
            };
            let target = match Address::from_str(&btc_address) {
                Ok(e) => e,
                Err(_error) => {
                    error!(
                        "LotteryOpenBox: cannot convert btc_address to Address: {:?}",
                        &btc_address
                    );
                    return;
                }
            };
            let data = if !self
                .lottery_set
                .get(&round_id)
                .expect("round_id is known in the lottery_set; qed")
                .contains_key(&token_id)
            {
                Lottery::SignedTx {
                    round_id,
                    token_id: token_id.as_bytes().to_vec(),
                    tx: Vec::new(),
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
                            txid: BtcTxid::from_inner(*txid),
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
                self.tx_set.push(tx_bytes.clone());
                Lottery::SignedTx {
                    round_id,
                    token_id: token_id.as_bytes().to_vec(),
                    tx: tx_bytes,
                }
            };
            mq.send(&data);
        } else {
            error!("Round {} has already started", round_id);
        }
    }
}

impl contracts::NativeContract for BtcLottery {
    type Cmd = Command;
    type QReq = Request;
    type QResp = Response;

    // Returns the contract id
    fn id(&self) -> contracts::ContractId {
        contracts::id256(contracts::BTC_LOTTERY)
    }

    fn handle_command(
        &mut self,
        context: &NativeContext,
        origin: MessageOrigin,
        cmd: Self::Cmd,
    ) -> TransactionResult {
        match cmd {
            Command::PalletCommand(cmd) => self.handle_pallet_command(context, origin, cmd),
            Command::UserCommand(cmd) => self.handle_user_command(context, origin, cmd),
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
                        let public_key = PublicKey::from_private_key(&secp, private_key);
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
                    let utxo = self
                        .utxo
                        .get(&round_id)
                        .expect("round_id is known in the utxo set; qed")
                        .iter()
                        .map(|(addr, utxo)| (addr.to_string(), *utxo))
                        .collect();
                    Response::QueryUtxo { utxo }
                } else {
                    Response::Error(Error::InvalidRequest)
                }
            }
            Request::GetSignedTx { round_id: _ } => Response::GetSignedTx {
                tx_set: self.tx_set.clone(),
            },
        }
    }
}

impl BtcLottery {
    fn handle_user_command(
        &mut self,
        _context: &NativeContext,
        origin: MessageOrigin,
        cmd: LotteryUserCommand,
    ) -> TransactionResult {
        let origin: chain::AccountId = match origin {
            MessageOrigin::AccountId(id) => (*id.inner()).into(),
            _ => return Err(TransactionError::BadOrigin),
        };

        match cmd {
            LotteryUserCommand::SubmitUtxo {
                round_id,
                address,
                utxo,
            } => {
                let sender = origin;
                let btc_address = match Address::from_str(&address) {
                    Ok(e) => e,
                    Err(_) => return Err(TransactionError::BadCommand),
                };
                if self.admin == sender {
                    let round_utxo = match self.utxo.entry(round_id) {
                        Occupied(_entry) => return Err(TransactionError::BadCommand),
                        Vacant(entry) => entry.insert(Default::default()),
                    };
                    round_utxo.insert(btc_address, utxo);
                }
                Ok(())
            }
            LotteryUserCommand::SetAdmin { new_admin } => {
                // TODO: listen to some specific privileged account instead of ALICE
                let sender = origin;
                if let Ok(new_admin) = account_id_from_hex(&new_admin) {
                    if self.admin == sender {
                        self.admin = new_admin;
                    }
                    Ok(())
                } else {
                    Err(TransactionError::InvalidAccount)
                }
            }
        }
    }

    fn handle_pallet_command(
        &mut self,
        context: &NativeContext,
        origin: MessageOrigin,
        ce: LotteryPalletCommand,
    ) -> TransactionResult {
        if !origin.is_pallet() {
            error!("Received trasfer event from invalid origin: {:?}", origin);
            return Err(TransactionError::BadOrigin);
        }
        info!("Received trasfer event from {:?}", origin);
        match ce {
            LotteryPalletCommand::NewRound {
                round_id,
                total_count,
                winner_count,
            } => Self::new_round(self, context.mq(), round_id, total_count, winner_count),
            LotteryPalletCommand::OpenBox {
                round_id,
                token_id,
                btc_address,
            } => Self::open_lottery(self, context.mq(), round_id, token_id, btc_address),
        }
        Ok(())
    }
}
