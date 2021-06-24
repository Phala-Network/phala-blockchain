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

pub mod messaging {
    use alloc::vec::Vec;
    use alloc::string::String;
    use codec::{Decode, Encode};
    use core::fmt::Debug;

    pub use phala_mq::types::*;
    pub use phala_mq::bind_topic;

    #[derive(Encode, Decode, Debug)]
    pub struct PushCommand<Cmd> {
        pub command: Cmd,
        pub number: u64,
    }

    impl<Cmd: BindTopic> BindTopic for PushCommand<Cmd> {
        const TOPIC: &'static [u8] = <Cmd as BindTopic>::TOPIC;
    }

    // TODO.kevin:
    //    We should create a crate for each contract just like developing apps.
    //    Then the following types should be put in their own crates.
    // Messages: Lottery

    bind_topic!(Lottery, b"^phala/BridgeTransfer");
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

    bind_topic!(LotteryCommand, b"phala/lottery/command");
    #[derive(Encode, Decode, Debug)]
    pub enum LotteryCommand {
        SubmitUtxo {
            round_id: u32,
            address: String,
            utxo: (Txid, u32, u64),
        },
        SetAdmin {
            new_admin: String,
        },
    }

    pub type Txid = [u8; 32];


    // Messages for Balances

    bind_topic!(BalanceEvent<AccountId, Balance>, b"phala/balances/event");
    #[derive(Debug, Clone, Encode, Decode)]
    pub enum BalanceEvent<AccountId, Balance> {
        TransferToTee(AccountId, Balance),
    }

    bind_topic!(BalanceCommand<AccountId, Balance>, b"phala/balances/command");
    #[derive(Debug, Clone, Encode, Decode)]
    pub enum BalanceCommand<AccountId, Balance> {
        Transfer {
            dest: AccountId,
            value: Balance,
        },
        TransferToChain {
            dest: AccountId,
            value: Balance,
        },
    }

    bind_topic!(BalanceTransfer<AccountId, Balance>, b"^phala/balances/transfer");
    #[derive(Encode, Decode)]
    pub struct BalanceTransfer<AccountId, Balance> {
        pub dest: AccountId,
        pub amount: Balance,
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
        self.data_be_signed()
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
pub type WorkerPublicKey = sp_core::ecdsa::Public;
pub type ContractPublicKey = sp_core::ecdsa::Public;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
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
