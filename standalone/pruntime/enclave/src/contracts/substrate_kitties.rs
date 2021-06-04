use serde::{Deserialize, Serialize};

use crate::contracts;
use crate::contracts::AccountIdWrapper;
use crate::types::TxRef;
use crate::TransactionStatus;
use lazy_static;
use sp_core::crypto::Pair;
use sp_core::ecdsa;
use sp_core::hashing::blake2_128;
use sp_core::H256 as Hash;
use sp_core::U256;
extern crate runtime as chain;
use parity_scale_codec::{Decode, Encode};

use crate::std::collections::BTreeMap;
use crate::std::format;
use crate::std::string::String;
use crate::std::vec::Vec;
use rand::Rng;

const ALICE: &'static str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
// 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
const BOB: &'static str = "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";
// 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
// const CHARLIE: &'static str = "90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";
// 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y
type SequenceType = u64;

lazy_static! {
    // 10000...000, used to tell if this is a NFT
    static ref TYPE_NF_BIT: U256 = U256::from(1) << 255;
    // 1111...11000...00, used to store the type in the upper 128 bits
    static ref TYPE_MASK: U256 = U256::from(!u128::MIN) << 128;
    // 00...00011....11, used to get NFT index in the lower 128 bits
    static ref NF_INDEX_MASK: U256 = U256::from(!u128::MIN);
}

/// SubstrateKitties contract states.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SubstrateKitties {
    schrodingers: BTreeMap<String, Vec<u8>>,
    /// Use Vec<u8> to represent kitty id
    kitties: BTreeMap<Vec<u8>, Kitty>,
    blind_boxes: BTreeMap<String, BlindBox>,
    /// Record the NFT's owner, if we introduce FT later, this field can be moved
    owner: BTreeMap<String, AccountIdWrapper>,
    /// Record the balance of an account's tokens, the first index is a tuple consists of tokenId and accountId
    /// the second index is the balance, for NFT it's always 1, for FT, it can be any number
    balances: BTreeMap<(String, AccountIdWrapper), chain::Balance>,
    /// Record the boxes list which the owners own
    owned_boxes: BTreeMap<AccountIdWrapper, Vec<String>>,
    sequence: SequenceType,
    queue: Vec<KittyTransferData>,
    #[serde(skip)]
    secret: Option<ecdsa::Pair>,
    /// Record the boxes the users opened
    opend_boxes: Vec<String>,
    /// This variable records if there are kitties that not in the boxes
    left_kitties: Vec<Vec<u8>>,
}

impl core::fmt::Debug for SubstrateKitties {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Hi")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BlindBox {
    // Use String to store the U256 type ID, preventing the Serialize implementation
    blind_box_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Kitty {
    id: Vec<u8>,
}

/// These two structs below are used for transferring messages to chain.
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct KittyTransfer {
    dest: AccountIdWrapper,
    kitty_id: Vec<u8>,
    sequence: SequenceType,
}
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct KittyTransferData {
    data: KittyTransfer,
    signature: Vec<u8>,
}
/// The commands that the contract accepts from the blockchain. Also called transactions.
/// Commands are supposed to update the states of the contract.
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    /// Pack the kitties into the corresponding blind boxes
    Pack {},
    /// Transfer the box to another account, need to judge if the sender is the owner
    Transfer { dest: String, blind_box_id: String },
    /// Open the specific blind box to get the kitty
    Open { blind_box_id: String },
}

/// The errors that the contract could throw for some queries
#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NotAuthorized,
}

/// Query requests. The end users can only query the contract states by sending requests.
/// Queries are not supposed to write to the contract states.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    /// Users can require to see the blind boxes list
    ObserveBox,
    /// Users can require to see their owned boxes list
    ObserveOwnedBox,
    /// Users can require to see the kitties which are not in the boxes
    ObserveLeftKitties,
    /// Users can require to know the owner of the specific box(NFT only)
    OwnerOf {
        blind_box_id: String,
    },

    PendingKittyTransfer {
        sequence: SequenceType,
    },
}

/// Query responses.
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    ObserveBox {
        blind_box: BTreeMap<String, BlindBox>,
    },
    ObserveOwnedBox {
        owned_box: Vec<String>,
    },
    ObserveLeftKitties {
        kitties: Vec<Vec<u8>>,
    },
    OwnerOf {
        owner: AccountIdWrapper,
    },
    PendingKittyTransfer {
        transfer_queue_b64: String,
    },
    /// Something wrong happened
    Error(Error),
}

impl SubstrateKitties {
    /// Initializes the contract
    pub fn new(secret: Option<ecdsa::Pair>) -> Self {
        let schrodingers = BTreeMap::<String, Vec<u8>>::new();
        let kitties = BTreeMap::<Vec<u8>, Kitty>::new();
        let blind_boxes = BTreeMap::<String, BlindBox>::new();
        let owner = BTreeMap::<String, AccountIdWrapper>::new();
        let balances = BTreeMap::<(String, AccountIdWrapper), chain::Balance>::new();
        let owned_boxes = BTreeMap::<AccountIdWrapper, Vec<String>>::new();
        SubstrateKitties {
            schrodingers,
            kitties,
            blind_boxes,
            owner,
            balances,
            owned_boxes,
            sequence: 0,
            queue: Vec::new(),
            secret,
            opend_boxes: Vec::new(),
            left_kitties: Vec::new(),
        }
    }
}

impl contracts::Contract<Command, Request, Response> for SubstrateKitties {
    // Returns the contract id
    fn id(&self) -> contracts::ContractId {
        contracts::SUBSTRATE_KITTIES
    }

    // Handles the commands from transactions on the blockchain. This method doesn't respond.
    fn handle_command(
        &mut self,
        _origin: &chain::AccountId,
        _txref: &TxRef,
        cmd: Command,
    ) -> TransactionStatus {
        match cmd {
            // Handle the `Pack` command
            Command::Pack {} => {
                // Create corresponding amount of kitties and blind boxes if there are
                // indeed some kitties that need to be packed
                if !self.left_kitties.is_empty() {
                    let mut nonce = 1;
                    let _sender = AccountIdWrapper(_origin.clone());
                    // This indicates the token's kind, let's just suppose that 1 represents box,
                    // maybe if we introduce more tokens later, this can be formulated strictly
                    let kind = 1;
                    // For now, the token id is 0....01000...000, this will be used to construct the
                    // box ID below, combined with random index id and NFT flags
                    let token_id: U256 = U256::from(kind) << 128;
                    // Let's just suppose that ALICE owns all the boxes as default, and ALICE can
                    // transfer to anyone that is on the chain
                    let default_owner = AccountIdWrapper::from_hex(ALICE)
                        .expect("Admin must has a good address; qed.");
                    for (kitty_id, _kitty) in self.kitties.iter() {
                        let mut rng = rand::thread_rng();
                        let seed: [u8; 16] = rng.gen();
                        let raw_data = (seed, nonce, &kitty_id);
                        nonce += 1;
                        let hash_data = blake2_128(&Encode::encode(&raw_data));
                        let index_id = u128::from_be_bytes(hash_data);
                        let nft_id = (token_id + index_id) | *TYPE_NF_BIT;
                        let blind_box_id = format!("{:#x}", nft_id);
                        let new_blind_box = BlindBox {
                            blind_box_id: blind_box_id.clone(),
                        };
                        let mut box_list = match self.owned_boxes.get(&default_owner) {
                            Some(_) => self.owned_boxes.get(&default_owner).unwrap().clone(),
                            None => Vec::new(),
                        };
                        box_list.push(blind_box_id.clone());
                        if nft_id & *TYPE_NF_BIT == *TYPE_NF_BIT {
                            // is NFT
                            self.owned_boxes.insert(default_owner.clone(), box_list);
                            self.owner
                                .insert(blind_box_id.clone(), default_owner.clone());
                            self.balances
                                .insert((blind_box_id.clone(), default_owner.clone()), 1);
                        } else if nft_id & *TYPE_NF_BIT == U256::from(0) {
                            // is FT
                            let mut ft_balance = match self
                                .balances
                                .get(&(blind_box_id.clone(), default_owner.clone()))
                            {
                                Some(_) => self
                                    .balances
                                    .get(&(blind_box_id.clone(), default_owner.clone()))
                                    .unwrap()
                                    .clone(),
                                None => 0,
                            };
                            ft_balance += 1;
                            self.balances
                                .insert((blind_box_id.clone(), default_owner.clone()), ft_balance);
                        }
                        self.schrodingers
                            .insert(blind_box_id.clone(), (*kitty_id).clone());
                        self.blind_boxes.insert(blind_box_id.clone(), new_blind_box);
                    }
                    // After this, new kitties are all packed into boxes
                    self.left_kitties.clear();
                }
                // Returns TransactionStatus::Ok to indicate a successful transaction
                TransactionStatus::Ok
            }
            Command::Transfer { dest, blind_box_id } => {
                // TODO: check owner & dest not overflow & sender not underflow
                let sender = AccountIdWrapper(_origin.clone());
                let original_owner = self.owner.get(&blind_box_id).unwrap().clone();
                let reciever = match AccountIdWrapper::from_hex(&dest) {
                    Ok(a) => a,
                    Err(_) => return TransactionStatus::BadInput,
                };
                if sender == original_owner {
                    println!(
                        "Transfer: [{}] -> [{}]: {}",
                        sender.to_string(),
                        dest,
                        blind_box_id
                    );
                    self.owner.insert(blind_box_id.clone(), reciever.clone());

                    let mut box_list = self.owned_boxes.get(&sender).unwrap().clone();
                    box_list.retain(|x| x != &blind_box_id);
                    self.owned_boxes.insert(sender, box_list);

                    // Add this opened box to the new owner
                    let mut new_owned_list = match self.owned_boxes.get(&reciever) {
                        Some(_) => self.owned_boxes.get(&reciever).unwrap().clone(),
                        None => Vec::new(),
                    };
                    new_owned_list.push(blind_box_id);
                    self.owned_boxes.insert(reciever, new_owned_list);
                }
                // Returns TransactionStatus::Ok to indicate a successful transaction
                TransactionStatus::Ok
            }
            Command::Open { blind_box_id } => {
                let sender = AccountIdWrapper(_origin.clone());
                let original_owner = self.owner.get(&blind_box_id).unwrap().clone();
                // Open the box if it's legal and not opened yet
                if sender == original_owner
                    && self.schrodingers.contains_key(&blind_box_id)
                    && !self.opend_boxes.contains(&blind_box_id)
                {
                    // Get the kitty based on blind_box_id
                    let kitty_id = self.schrodingers.get(&blind_box_id).unwrap();
                    let sequence = self.sequence + 1;

                    let kitty_id = Hash::from_slice(&kitty_id);

                    // Queue the message to sync the owner transfer info to pallet
                    let data = KittyTransfer {
                        dest: sender.clone(),
                        kitty_id: kitty_id.clone().encode(),
                        sequence,
                    };

                    self.opend_boxes.push(blind_box_id.clone());

                    // let msg_hash = blake2_256(&Encode::encode(&data));
                    // let mut buffer = [0u8; 32];
                    // buffer.copy_from_slice(&msg_hash);
                    // let message = Message::parse(&buffer);
                    // let signature = secp256k1::sign(&message, &self.secret.as_ref().unwrap());

                    let secret = self.secret.as_ref().unwrap();
                    let signature = secret.sign(&Encode::encode(&data));

                    println!("signature={:?}", signature);
                    let transfer_data = KittyTransferData {
                        data,
                        signature: signature.0.to_vec(),
                    };
                    self.queue.push(transfer_data);
                    self.sequence = sequence;
                }
                TransactionStatus::Ok
            }
        }
    }

    // Handles a direct query and responds to the query. It shouldn't modify the contract states.
    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response, Error> {
            match req {
                Request::ObserveBox => {
                    return Ok(Response::ObserveBox {
                        blind_box: self.blind_boxes.clone(),
                    })
                }
                Request::ObserveOwnedBox => {
                    let sender = AccountIdWrapper(_origin.unwrap().clone());
                    let owned_boxes = self.owned_boxes.get(&sender);
                    match owned_boxes {
                        Some(_) => {
                            return Ok(Response::ObserveOwnedBox {
                                owned_box: owned_boxes.unwrap().clone(),
                            })
                        }
                        None => {
                            return Ok(Response::ObserveOwnedBox {
                                owned_box: Vec::new(),
                            })
                        }
                    };
                }
                Request::ObserveLeftKitties => {
                    return Ok(Response::ObserveLeftKitties {
                        kitties: self.left_kitties.clone(),
                    })
                }
                Request::OwnerOf { blind_box_id } => {
                    return Ok(Response::OwnerOf {
                        owner: self.owner.get(&blind_box_id).unwrap().clone(),
                    })
                }
                Request::PendingKittyTransfer { sequence } => {
                    println!("PendingKittyTransfer");
                    let transfer_queue: Vec<&KittyTransferData> = self
                        .queue
                        .iter()
                        .filter(|x| x.data.sequence > sequence)
                        .collect::<_>();

                    Ok(Response::PendingKittyTransfer {
                        transfer_queue_b64: base64::encode(&transfer_queue.encode()),
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
            // create_kitties() is called on the chain
            if let chain::pallet_kitties::RawEvent::Created(account_id, kitty_id) = pe {
                println!("Created Kitty {:?} by default owner: Kitty!!!", kitty_id);
                let dest = AccountIdWrapper(account_id);
                println!("   dest: {}", dest.to_string());
                let new_kitty_id = kitty_id.to_fixed_bytes();
                let new_kitty = Kitty {
                    id: new_kitty_id.to_vec(),
                };
                self.kitties.insert(new_kitty_id.to_vec(), new_kitty);
                self.left_kitties.push(new_kitty_id.to_vec());
            } else if let chain::pallet_kitties::RawEvent::TransferToChain(
                account_id,
                kitty_id,
                sequence,
            ) = pe
            {
                // owner transfer info already recieved
                println!("TransferToChain who: {:?}", account_id);
                let new_kitty_id = format!("{:#x}", kitty_id);
                println!("Kitty: {} is transerferred!!", new_kitty_id);
                let transfer_data = KittyTransferData {
                    data: KittyTransfer {
                        dest: AccountIdWrapper(account_id),
                        kitty_id: kitty_id.encode(),
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
