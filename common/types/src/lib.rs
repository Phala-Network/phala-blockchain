#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use core::fmt::Debug;
use sp_core::U256;

#[cfg(feature = "enable_serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "pruntime")]
pub mod pruntime;

// Messages: Phase Wallet

#[derive(Encode, Decode)]
pub struct Transfer<AccountId, Balance> {
    pub dest: AccountId,
    pub amount: Balance,
    pub sequence: u64,
}

#[derive(Encode, Decode)]
pub struct TransferData<AccountId, Balance> {
    pub data: Transfer<AccountId, Balance>,
    pub signature: Vec<u8>,
}

pub mod messaging {
    use alloc::vec::Vec;
    use codec::{Decode, Encode};
    use core::fmt::Debug;
    use sp_core::H256;

    pub use phala_mq::Origin as MessageOrigin;

    /// The topic in the message queue, indicating a group of destination message receivers
    #[derive(Encode, Decode, Clone, Debug, Eq, PartialEq)]
    pub enum Topic {
        /// The topic targets a cetrain receiver identified by `MessageOrigin`
        Targeted(MessageOrigin),
        /// A general topic that can be subscribed by anyone
        Named(Vec<u8>),
    }

    impl Topic {
        pub fn is_offchain(&self) -> bool {
            if let Topic::Targeted(origin) = self {
                origin.is_offchain()
            } else {
                false
            }
        }
    }

    // Messages: Lottery

    #[derive(Encode, Decode, Clone, Debug)]
    pub enum Lottery {
        SignedTx {
            round_id: u32,
            token_id: Vec<u8>,
            tx: Vec<u8>,
        },
        BtcAddresses {
            address_set: Vec<Vec<u8>>,
        },
    }

    /// A generic message
    #[derive(Encode, Decode, Clone, Debug, Eq, PartialEq)]
    pub struct Message {
        pub sender: MessageOrigin,
        pub destination: Topic,
        pub payload: Vec<u8>,
    }

    impl Message {
        pub fn parse<T: Decode>(&self) -> Result<T, ()> {
            Decode::decode(&mut &self.payload[..]).or(Err(()))
        }
    }

    /// Signed generic message
    #[derive(Encode, Decode, Clone, Debug, Eq, PartialEq)]
    pub struct SignedMessage {
        pub message: Message,
        pub sequence: u64,
        pub signature: Vec<u8>,
    }
}

// Messages: System

#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "enable_serde", derive(Serialize, Deserialize))]
pub enum WorkerMessagePayload {
    Heartbeat {
        block_num: u32,
        claim_online: bool,
        claim_compute: bool,
    },
}

#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "enable_serde", derive(Serialize, Deserialize))]
pub struct WorkerMessage {
    pub payload: WorkerMessagePayload,
    pub sequence: u64,
}

#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "enable_serde", derive(Serialize, Deserialize))]
pub struct SignedWorkerMessage {
    pub data: WorkerMessage,
    pub signature: Vec<u8>,
}

// Message support trait

pub trait SignedDataType<T> {
    fn raw_data(&self) -> Vec<u8>;
    fn signature(&self) -> T;
}

impl<AccountId: Encode, Balance: Encode> SignedDataType<Vec<u8>>
    for TransferData<AccountId, Balance>
{
    fn raw_data(&self) -> Vec<u8> {
        Encode::encode(&self.data)
    }
    fn signature(&self) -> Vec<u8> {
        self.signature.clone()
    }
}

impl SignedDataType<Vec<u8>> for SignedWorkerMessage {
    fn raw_data(&self) -> Vec<u8> {
        Encode::encode(&self.data)
    }
    fn signature(&self) -> Vec<u8> {
        self.signature.clone()
    }
}

impl SignedDataType<Vec<u8>> for messaging::SignedMessage {
    fn raw_data(&self) -> Vec<u8> {
        Encode::encode(&(&self.message, &self.sequence))
    }
    fn signature(&self) -> Vec<u8> {
        self.signature.clone()
    }
}

// Types used in storage

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone)]
pub enum WorkerStateEnum<BlockNumber> {
    Empty,
    Free,
    Gatekeeper,
    MiningPending,
    Mining(BlockNumber),
    MiningStopping,
}

impl<BlockNumber> Default for WorkerStateEnum<BlockNumber> {
    fn default() -> Self {
        WorkerStateEnum::Empty
    }
}

#[derive(Encode, Decode, Debug, Default, Clone)]
pub struct WorkerInfo<BlockNumber> {
    // identity
    pub machine_id: Vec<u8>,
    pub pubkey: Vec<u8>,
    pub last_updated: u64,
    // mining
    pub state: WorkerStateEnum<BlockNumber>,
    // performance
    pub score: Option<Score>,
    // confidence-level
    pub confidence_level: u8,
    // version
    pub runtime_version: u32,
}

#[derive(Encode, Decode, Default)]
pub struct StashInfo<AccountId: Default> {
    pub controller: AccountId,
    pub payout_prefs: PayoutPrefs<AccountId>,
}

#[derive(Encode, Decode, Default)]
pub struct PayoutPrefs<AccountId: Default> {
    pub commission: u32,
    pub target: AccountId,
}

#[derive(Encode, Decode, Debug, Default, Clone)]
pub struct Score {
    pub overall_score: u32,
    pub features: Vec<u32>,
}

type MachineId = [u8; 16];
type WorkerPublicKey = [u8; 33];
#[derive(Encode, Decode, Debug)]
pub struct PRuntimeInfo {
    pub version: u32,
    pub machine_id: MachineId,
    pub pubkey: WorkerPublicKey,
    pub features: Vec<u32>,
}

#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
pub struct BlockRewardInfo {
    pub seed: U256,
    pub online_target: U256,
    pub compute_target: U256,
}

#[derive(Encode, Decode, Debug, Default)]
pub struct RoundInfo<BlockNumber> {
    pub round: u32,
    pub start_block: BlockNumber,
}

#[derive(Encode, Decode, Debug, Default)]
pub struct StashWorkerStats<Balance> {
    pub slash: Balance,
    pub compute_received: Balance,
    pub online_received: Balance,
}

#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
pub struct RoundStats {
    pub round: u32,
    pub online_workers: u32,
    pub compute_workers: u32,
    /// The targeted online reward counts in fraction (base: 100_000)
    pub frac_target_online_reward: u32,
    pub total_power: u32,
    /// The targeted compute reward counts in fraction (base: 100_000)
    pub frac_target_compute_reward: u32,
}

#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
pub struct MinerStatsDelta {
    pub num_worker: i32,
    pub num_power: i32,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
pub enum PayoutReason {
    OnlineReward,
    ComputeReward,
}

impl Default for PayoutReason {
    fn default() -> Self {
        PayoutReason::OnlineReward
    }
}
