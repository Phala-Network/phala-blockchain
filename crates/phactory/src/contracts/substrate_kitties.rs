use super::account_id_from_hex;
use super::{TransactionError, TransactionResult};
use crate::contracts;
use crate::contracts::AccountId;
use lazy_static;
use sp_core::hashing::blake2_128;
use sp_core::H256 as Hash;
use sp_core::U256;
extern crate runtime as chain;
use parity_scale_codec::{Decode, Encode};

use std::collections::BTreeMap;
use rand::Rng;

use super::NativeContext;
use phala_types::messaging::{KittiesCommand, KittyTransfer, MessageOrigin};

type Command = KittiesCommand<chain::AccountId, chain::Hash>;
type Transfer = KittyTransfer<chain::AccountId>;

const ALICE: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
// 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
// const BOB: &str = "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";
// 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
// const CHARLIE: &str = "90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";
// 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y

lazy_static! {
    // 10000...000, used to tell if this is a NFT
    static ref TYPE_NF_BIT: U256 = U256::from(1) << 255;
    // 1111...11000...00, used to store the type in the upper 128 bits
    static ref TYPE_MASK: U256 = U256::from(!u128::MIN) << 128;
    // 00...00011....11, used to get NFT index in the lower 128 bits
    static ref NF_INDEX_MASK: U256 = U256::from(!u128::MIN);
}

/// SubstrateKitties contract states.
#[derive(Default, Clone)]
pub struct SubstrateKitties {
    schrodingers: BTreeMap<String, Vec<u8>>,
    /// Use Vec<u8> to represent kitty id
    kitties: BTreeMap<Vec<u8>, Kitty>,
    blind_boxes: BTreeMap<String, BlindBox>,
    /// Record the NFT's owner, if we introduce FT later, this field can be moved
    owner: BTreeMap<String, AccountId>,
    /// Record the balance of an account's tokens, the first index is a tuple consists of tokenId and accountId
    /// the second index is the balance, for NFT it's always 1, for FT, it can be any number
    balances: BTreeMap<(String, AccountId), chain::Balance>,
    /// Record the boxes list which the owners own
    owned_boxes: BTreeMap<AccountId, Vec<String>>,
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

#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct BlindBox {
    // Use String to store the U256 type ID, preventing the Serialize implementation
    blind_box_id: String,
}

#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct Kitty {
    id: Vec<u8>,
}

/// The errors that the contract could throw for some queries
#[derive(Encode, Decode, Debug)]
pub enum Error {
    NotAuthorized,
}

/// Query requests. The end users can only query the contract states by sending requests.
/// Queries are not supposed to write to the contract states.
#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {
    /// Users can require to see the blind boxes list
    ObserveBox,
    /// Users can require to see their owned boxes list
    ObserveOwnedBox,
    /// Users can require to see the kitties which are not in the boxes
    ObserveLeftKitties,
    /// Users can require to know the owner of the specific box(NFT only)
    OwnerOf { blind_box_id: String },
}

/// Query responses.
#[derive(Encode, Decode, Debug)]
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
        owner: AccountId,
    },
    /// Something wrong happened
    Error(Error),
}

impl SubstrateKitties {
    /// Initializes the contract
    pub fn new() -> Self {
        let schrodingers = BTreeMap::<String, Vec<u8>>::new();
        let kitties = BTreeMap::<Vec<u8>, Kitty>::new();
        let blind_boxes = BTreeMap::<String, BlindBox>::new();
        let owner = BTreeMap::<String, AccountId>::new();
        let balances = BTreeMap::<(String, AccountId), chain::Balance>::new();
        let owned_boxes = BTreeMap::<AccountId, Vec<String>>::new();
        SubstrateKitties {
            schrodingers,
            kitties,
            blind_boxes,
            owner,
            balances,
            owned_boxes,
            opend_boxes: Vec::new(),
            left_kitties: Vec::new(),
        }
    }
}

impl contracts::NativeContract for SubstrateKitties {
    type Cmd = Command;
    type QReq = Request;
    type QResp = Response;

    // Returns the contract id
    fn id(&self) -> contracts::ContractId {
        contracts::id256(contracts::SUBSTRATE_KITTIES)
    }

    // Handles the commands from transactions on the blockchain. This method doesn't respond.
    fn handle_command(
        &mut self,
        context: &NativeContext,
        origin: MessageOrigin,
        cmd: Self::Cmd,
    ) -> TransactionResult {
        match cmd {
            // Handle the `Pack` command
            Command::Pack {} => {
                // Create corresponding amount of kitties and blind boxes if there are
                // indeed some kitties that need to be packed
                if !self.left_kitties.is_empty() {
                    let mut nonce = 1;
                    let _sender = origin.account()?;
                    // This indicates the token's kind, let's just suppose that 1 represents box,
                    // maybe if we introduce more tokens later, this can be formulated strictly
                    let kind = 1;
                    // For now, the token id is 0....01000...000, this will be used to construct the
                    // box ID below, combined with random index id and NFT flags
                    let token_id: U256 = U256::from(kind) << 128;
                    // Let's just suppose that ALICE owns all the boxes as default, and ALICE can
                    // transfer to anyone that is on the chain
                    let default_owner =
                        account_id_from_hex(ALICE).expect("Admin must has a good address; qed.");
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
                                Some(_) => *self
                                    .balances
                                    .get(&(blind_box_id.clone(), default_owner.clone()))
                                    .unwrap(),
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
                // Returns Ok(()) to indicate a successful transaction
                Ok(())
            }
            Command::Transfer { dest, blind_box_id } => {
                // TODO: check owner & dest not overflow & sender not underflow
                let sender = origin.account()?;
                let original_owner = self.owner.get(&blind_box_id).unwrap().clone();
                let reciever = match account_id_from_hex(&dest) {
                    Ok(a) => a,
                    Err(_) => return Err(TransactionError::BadInput),
                };
                if sender == original_owner {
                    println!(
                        "Transfer: [{}] -> [{}]: {}",
                        hex::encode(&sender),
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
                // Returns Ok(()) to indicate a successful transaction
                Ok(())
            }
            Command::Open { blind_box_id } => {
                let sender = origin.account()?;
                let original_owner = self.owner.get(&blind_box_id).unwrap().clone();
                // Open the box if it's legal and not opened yet
                if sender == original_owner
                    && self.schrodingers.contains_key(&blind_box_id)
                    && !self.opend_boxes.contains(&blind_box_id)
                {
                    // Get the kitty based on blind_box_id
                    let kitty_id = self.schrodingers.get(&blind_box_id).unwrap();

                    let kitty_id = Hash::from_slice(kitty_id);

                    // Queue the message to sync the owner transfer info to pallet
                    let data = Transfer {
                        dest: sender,
                        kitty_id: kitty_id.clone().encode(),
                    };

                    self.opend_boxes.push(blind_box_id.clone());
                    context.mq().send(&data);
                }
                Ok(())
            }
            Command::Created(dest, kitty_id) => {
                if !origin.is_pallet() {
                    error!("Received event from unexpected origin: {:?}", origin);
                    return Err(TransactionError::BadOrigin);
                }
                println!("Created Kitty {:?} by default owner: Kitty!!!", kitty_id);
                println!("   dest: {}", hex::encode(dest));
                let new_kitty_id = kitty_id.to_fixed_bytes();
                let new_kitty = Kitty {
                    id: new_kitty_id.to_vec(),
                };
                self.kitties.insert(new_kitty_id.to_vec(), new_kitty);
                self.left_kitties.push(new_kitty_id.to_vec());
                Ok(())
            }
        }
    }

    // Handles a direct query and responds to the query. It shouldn't modify the contract states.
    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response, Error> {
            match req {
                Request::ObserveBox => Ok(Response::ObserveBox {
                    blind_box: self.blind_boxes.clone(),
                }),
                Request::ObserveOwnedBox => {
                    let sender = origin.unwrap().clone();
                    let owned_boxes = self.owned_boxes.get(&sender);
                    match owned_boxes {
                        Some(_) => Ok(Response::ObserveOwnedBox {
                            owned_box: owned_boxes.unwrap().clone(),
                        }),
                        None => Ok(Response::ObserveOwnedBox {
                            owned_box: Vec::new(),
                        }),
                    }
                }
                Request::ObserveLeftKitties => Ok(Response::ObserveLeftKitties {
                    kitties: self.left_kitties.clone(),
                }),
                Request::OwnerOf { blind_box_id } => Ok(Response::OwnerOf {
                    owner: self.owner.get(&blind_box_id).unwrap().clone(),
                }),
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp,
        }
    }
}
