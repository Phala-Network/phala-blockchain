#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod contract;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use core::convert::{TryFrom, TryInto};
use core::fmt::Debug;
use sp_core::H256;

// Messages: Phase Wallet

pub mod messaging {
    use alloc::string::String;
    use alloc::vec::Vec;
    use codec::{Decode, Encode};
    use core::fmt::Debug;
    use sp_core::U256;

    #[cfg(feature = "enable_serde")]
    use serde::{Deserialize, Serialize};

    use super::{EcdhPublicKey, MasterPublicKey, WorkerPublicKey};
    use crate::contract;
    pub use phala_mq::types::*;
    pub use phala_mq::{bind_contract32, bind_topic};

    // TODO.kevin: reuse the Payload in secret_channel.rs.
    #[derive(Encode, Decode, Debug)]
    pub enum CommandPayload<T> {
        Plain(T),
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

    #[derive(Encode, Decode, Debug, Clone)]
    pub enum LotteryPalletCommand {
        NewRound {
            round_id: u32,
            total_count: u32,
            winner_count: u32,
        },
        OpenBox {
            round_id: u32,
            token_id: u32,
            btc_address: Vec<u8>,
        },
    }

    #[derive(Encode, Decode, Debug)]
    pub enum LotteryUserCommand {
        SubmitUtxo {
            round_id: u32,
            address: String,
            utxo: (Txid, u32, u64),
        },
        SetAdmin {
            new_admin: String,
        },
    }

    bind_contract32!(LotteryCommand, contract::BTC_LOTTERY);
    #[derive(Encode, Decode, Debug)]
    pub enum LotteryCommand {
        UserCommand(LotteryUserCommand),
        PalletCommand(LotteryPalletCommand),
    }

    impl LotteryCommand {
        pub fn open_box(round_id: u32, token_id: u32, btc_address: Vec<u8>) -> Self {
            Self::PalletCommand(LotteryPalletCommand::OpenBox {
                round_id,
                token_id,
                btc_address,
            })
        }

        pub fn new_round(round_id: u32, total_count: u32, winner_count: u32) -> Self {
            Self::PalletCommand(LotteryPalletCommand::NewRound {
                round_id,
                total_count,
                winner_count,
            })
        }
    }

    pub type Txid = [u8; 32];

    // Messages for Balances

    bind_contract32!(BalancesCommand<AccountId, Balance>, contract::BALANCES);
    #[derive(Debug, Clone, Encode, Decode)]
    pub enum BalancesCommand<AccountId, Balance> {
        Transfer { dest: AccountId, value: Balance },
        TransferToChain { dest: AccountId, value: Balance },
        TransferToTee { who: AccountId, amount: Balance },
    }

    impl<AccountId, Balance> BalancesCommand<AccountId, Balance> {
        pub fn transfer(dest: AccountId, value: Balance) -> Self {
            Self::Transfer { dest, value }
        }

        pub fn transfer_to_chain(dest: AccountId, value: Balance) -> Self {
            Self::TransferToChain { dest, value }
        }
    }

    bind_topic!(BalancesTransfer<AccountId, Balance>, b"^phala/balances/transfer");
    #[derive(Encode, Decode)]
    pub struct BalancesTransfer<AccountId, Balance> {
        pub dest: AccountId,
        pub amount: Balance,
    }

    // Messages for Assets

    bind_contract32!(AssetCommand<AccountId, Balance>, contract::ASSETS);
    #[derive(Encode, Decode, Debug)]
    pub enum AssetCommand<AccountId, Balance> {
        Issue {
            symbol: String,
            total: Balance,
        },
        Destroy {
            id: AssetId,
        },
        Transfer {
            id: AssetId,
            dest: AccountId,
            value: Balance,
            index: u64,
        },
    }

    pub type AssetId = u32;

    // Messages for Web3Analytics

    bind_contract32!(Web3AnalyticsCommand, contract::WEB3_ANALYTICS);
    #[derive(Encode, Decode, Debug)]
    pub enum Web3AnalyticsCommand {
        SetConfiguration { skip_stat: bool },
    }

    // Messages for diem

    bind_contract32!(DiemCommand, contract::DIEM);
    #[derive(Encode, Decode, Debug)]
    pub enum DiemCommand {
        /// Sets the whitelisted accounts, in bcs encoded base64
        AccountInfo {
            account_info_b64: String,
        },
        /// Verifies a transactions
        VerifyTransaction {
            account_address: String,
            transaction_with_proof_b64: String,
        },
        /// Sets the trusted state. The owner can only initialize the bridge with the genesis state
        /// once.
        SetTrustedState {
            trusted_state_b64: String,
            chain_id: u8,
        },
        VerifyEpochProof {
            ledger_info_with_signatures_b64: String,
            epoch_change_proof_b64: String,
        },

        NewAccount {
            seq_number: u64,
        },
        TransferXUS {
            to: String,
            amount: u64,
        },
    }

    // Messages for Kitties

    bind_contract32!(KittiesCommand<AccountId, Hash>, contract::SUBSTRATE_KITTIES);
    #[derive(Encode, Decode, Debug)]
    pub enum KittiesCommand<AccountId, Hash> {
        /// Pack the kitties into the corresponding blind boxes
        Pack {},
        /// Transfer the box to another account, need to judge if the sender is the owner
        Transfer { dest: String, blind_box_id: String },
        /// Open the specific blind box to get the kitty
        Open { blind_box_id: String },
        /// Created a new blind box
        Created(AccountId, Hash),
    }

    bind_topic!(KittyTransfer<AccountId>, b"^phala/kitties/transfer");
    #[derive(Debug, Clone, Encode, Decode, PartialEq)]
    pub struct KittyTransfer<AccountId> {
        pub dest: AccountId,
        pub kitty_id: Vec<u8>,
    }

    // Messages for Geo Location
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub struct CoordinateInfo {
        pub latitude: i32,
        pub longitude: i32,
        pub city_name: String,
    }

    bind_contract32!(GeolocationCommand, contract::GEOLOCATION);
    #[derive(Debug, Clone, Encode, Decode)]
    pub enum GeolocationCommand {
        UpdateGeolocation { geolocation_info: CoordinateInfo },
    }

    impl GeolocationCommand {
        pub fn update_geolocation(geolocation_info: CoordinateInfo) -> Self {
            Self::UpdateGeolocation { geolocation_info }
        }
    }

    /// A fixed point number with 64 integer bits and 64 fractional bits.
    pub type U64F64Bits = u128;

    // Messages: System
    #[derive(Encode, Decode)]
    pub struct WorkerEventWithKey {
        pub pubkey: WorkerPublicKey,
        pub event: WorkerEvent,
    }

    impl Debug for WorkerEventWithKey {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let pubkey = hex::encode(&self.pubkey.0);
            f.debug_struct("WorkerEventWithKey")
                .field("pubkey", &pubkey)
                .field("event", &self.event)
                .finish()
        }
    }

    #[derive(Encode, Decode, Debug)]
    pub struct WorkerInfo {
        pub confidence_level: u8,
    }

    #[derive(Encode, Decode, Debug)]
    pub enum WorkerEvent {
        /// pallet-registry --> worker
        ///  Indicate a worker register succeeded.
        Registered(WorkerInfo),
        /// pallet-registry --> worker
        ///  When a worker register succeed, the chain request the worker to benchmark.
        ///   duration: Number of blocks the benchmark to keep on.
        BenchStart { duration: u32 },
        /// pallet-registry --> worker
        ///  The init bench score caculated by pallet.
        BenchScore(u32),
        /// pallet-mining --> worker
        ///  When a miner start to mine, push this message to the worker to start the benchmark task.
        ///   session_id: Generated by pallet. Each mining session should have a unique session_id.
        MiningStart {
            session_id: u32,
            init_v: U64F64Bits,
            init_p: u32,
        },
        /// pallet-mining --> worker
        ///  When a miner entered CoolingDown state, push this message to the worker, so that it can stop the
        ///  benchmark task.
        MiningStop,
        /// pallet-mining --> worker
        ///  When a miner entered Unresponsive state, push this message to the worker to suppress the subsequent
        ///  heartbeat responses.
        MiningEnterUnresponsive,
        /// pallet-mining --> worker
        ///  When a miner recovered to MiningIdle state from Unresponsive, push this message to the worker to
        ///  resume the subsequent heartbeat responses.
        MiningExitUnresponsive,
    }

    bind_topic!(SystemEvent, b"phala/system/event");
    #[derive(Encode, Decode, Debug)]
    pub enum SystemEvent {
        WorkerEvent(WorkerEventWithKey),
        HeartbeatChallenge(HeartbeatChallenge),
    }

    impl SystemEvent {
        pub fn new_worker_event(pubkey: WorkerPublicKey, event: WorkerEvent) -> SystemEvent {
            SystemEvent::WorkerEvent(WorkerEventWithKey { pubkey, event })
        }
    }

    #[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
    pub struct HeartbeatChallenge {
        pub seed: U256,
        pub online_target: U256,
    }

    bind_topic!(MiningReportEvent, b"phala/mining/report");
    #[derive(Encode, Decode, Clone, Debug)]
    pub enum MiningReportEvent {
        Heartbeat {
            /// The mining session id.
            session_id: u32,
            /// The challenge block number.
            challenge_block: u32,
            /// The challenge block timestamp.
            challenge_time: u64,
            /// Benchmark iterations since mining_start_time.
            iterations: u64,
        },
    }

    bind_topic!(MiningInfoUpdateEvent<BlockNumber>, b"^phala/mining/update");
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub struct MiningInfoUpdateEvent<BlockNumber> {
        /// The block emiting this message.
        pub block_number: BlockNumber,
        /// The timestamp of the block emiting this message.
        pub timestamp_ms: u64,
        /// Workers that do not responce the heartbeat challenge in time. Each delay only report once.
        pub offline: Vec<WorkerPublicKey>,
        /// Workers that received a heartbeat in offline state.
        pub recovered_to_online: Vec<WorkerPublicKey>,
        /// V update and payout info
        pub settle: Vec<SettleInfo>,
        // NOTE: Take care of the is_empty method when adding fields
    }

    impl<BlockNumber> MiningInfoUpdateEvent<BlockNumber> {
        pub fn new(block_number: BlockNumber, timestamp_ms: u64) -> Self {
            Self {
                block_number,
                timestamp_ms,
                offline: Default::default(),
                recovered_to_online: Default::default(),
                settle: Default::default(),
            }
        }

        pub fn is_empty(&self) -> bool {
            self.offline.is_empty() && self.settle.is_empty() && self.recovered_to_online.is_empty()
        }
    }

    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub struct SettleInfo {
        pub pubkey: WorkerPublicKey,
        pub v: U64F64Bits,
        pub payout: U64F64Bits,
        pub treasury: U64F64Bits,
    }

    // Messages: Gatekeeper launch
    bind_topic!(GatekeeperLaunch, b"phala/gatekeeper/launch");
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub enum GatekeeperLaunch {
        FirstGatekeeper(NewGatekeeperEvent),
        MasterPubkeyOnChain(MasterPubkeyEvent),
    }

    impl GatekeeperLaunch {
        pub fn first_gatekeeper(
            pubkey: WorkerPublicKey,
            ecdh_pubkey: EcdhPublicKey,
        ) -> GatekeeperLaunch {
            GatekeeperLaunch::FirstGatekeeper(NewGatekeeperEvent {
                pubkey,
                ecdh_pubkey,
            })
        }

        pub fn master_pubkey_on_chain(master_pubkey: MasterPublicKey) -> GatekeeperLaunch {
            GatekeeperLaunch::MasterPubkeyOnChain(MasterPubkeyEvent { master_pubkey })
        }
    }

    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub struct NewGatekeeperEvent {
        /// The public key of registered gatekeeper
        pub pubkey: WorkerPublicKey,
        /// The ecdh public key of registered gatekeeper
        pub ecdh_pubkey: EcdhPublicKey,
    }

    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub struct MasterPubkeyEvent {
        pub master_pubkey: MasterPublicKey,
    }

    // Messages: Gatekeeper change
    bind_topic!(GatekeeperChange, b"phala/gatekeeper/change");
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub enum GatekeeperChange {
        GatekeeperRegistered(NewGatekeeperEvent),
    }

    impl GatekeeperChange {
        pub fn gatekeeper_registered(
            pubkey: WorkerPublicKey,
            ecdh_pubkey: EcdhPublicKey,
        ) -> GatekeeperChange {
            GatekeeperChange::GatekeeperRegistered(NewGatekeeperEvent {
                pubkey,
                ecdh_pubkey,
            })
        }
    }

    // Messages: Distribution of master key and contract keys
    bind_topic!(KeyDistribution, b"phala/gatekeeper/key");
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub enum KeyDistribution {
        MasterKeyDistribution(DispatchMasterKeyEvent),
    }

    impl KeyDistribution {
        pub fn master_key_distribution(
            dest: WorkerPublicKey,
            ecdh_pubkey: EcdhPublicKey,
            encrypted_master_key: Vec<u8>,
            iv: AeadIV,
        ) -> KeyDistribution {
            KeyDistribution::MasterKeyDistribution(DispatchMasterKeyEvent {
                dest,
                ecdh_pubkey,
                encrypted_master_key,
                iv,
            })
        }
    }

    type AeadIV = [u8; 12];
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub struct DispatchMasterKeyEvent {
        /// The target to dispatch master key
        pub dest: WorkerPublicKey,
        /// The ecdh public key of master key source
        pub ecdh_pubkey: EcdhPublicKey,
        /// Master key encrypted with aead key
        pub encrypted_master_key: Vec<u8>,
        /// Aead IV
        pub iv: AeadIV,
    }

    // Messages: Gatekeeper
    bind_topic!(GatekeeperEvent, b"phala/gatekeeper/event");
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub enum GatekeeperEvent {
        NewRandomNumber(RandomNumberEvent),
        TokenomicParametersChanged(TokenomicParameters),
        RepairV,
    }

    impl GatekeeperEvent {
        pub fn new_random_number(
            block_number: u32,
            random_number: RandomNumber,
            last_random_number: RandomNumber,
        ) -> GatekeeperEvent {
            GatekeeperEvent::NewRandomNumber(RandomNumberEvent {
                block_number,
                random_number,
                last_random_number,
            })
        }
    }

    pub type RandomNumber = [u8; 32];
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub struct RandomNumberEvent {
        pub block_number: u32,
        pub random_number: RandomNumber,
        pub last_random_number: RandomNumber,
    }

    #[cfg_attr(feature = "enable_serde", derive(Serialize, Deserialize))]
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub struct TokenomicParameters {
        // V calculation
        pub pha_rate: U64F64Bits,
        pub rho: U64F64Bits,
        pub budget_per_block: U64F64Bits,
        pub v_max: U64F64Bits,
        pub cost_k: U64F64Bits,
        pub cost_b: U64F64Bits,
        pub slash_rate: U64F64Bits,
        // Payout
        pub treasury_ratio: U64F64Bits,
        pub heartbeat_window: u32,
        // Ve calculation
        pub rig_k: U64F64Bits,
        pub rig_b: U64F64Bits,
        pub re: U64F64Bits,
        pub k: U64F64Bits,
        // Slash calculation
        pub kappa: U64F64Bits,
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

type MachineId = Vec<u8>;
pub type Sr25519Signature = sp_core::sr25519::Signature;
pub type WorkerPublicKey = sp_core::sr25519::Public;
pub type ContractPublicKey = sp_core::sr25519::Public;
pub type MasterPublicKey = sp_core::sr25519::Public;
#[derive(Encode, Decode, Clone, Debug, Eq, PartialEq)]
/// Sr25519 public key
pub struct EcdhPublicKey(pub [u8; 32]);

impl Default for EcdhPublicKey {
    fn default() -> Self {
        EcdhPublicKey([0_u8; 32])
    }
}

impl TryFrom<&[u8]> for EcdhPublicKey {
    type Error = ();
    fn try_from(raw: &[u8]) -> Result<Self, ()> {
        let raw: [u8; 32] = raw.try_into().map_err(|_| ())?;
        Ok(EcdhPublicKey(raw))
    }
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
pub struct WorkerRegistrationInfo<AccountId> {
    pub version: u32,
    pub machine_id: MachineId,
    pub pubkey: WorkerPublicKey,
    pub ecdh_pubkey: EcdhPublicKey,
    pub genesis_block_hash: H256,
    pub features: Vec<u32>,
    pub operator: Option<AccountId>,
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
