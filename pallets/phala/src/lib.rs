#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use sp_core::U256;
use sp_std::prelude::*;
use sp_std::{cmp, vec};

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch, ensure};
use frame_system::{ensure_root, ensure_signed, Module as System};

use alloc::vec::Vec;
use codec::Decode;
use frame_support::{
    traits::{
        Currency, ExistenceRequirement::AllowDeath, Get, Imbalance, OnUnbalanced, Randomness,
        UnixTime,
    },
    weights::Weight,
};
use sp_runtime::{traits::AccountIdConversion, ModuleId, Permill, SaturatedConversion};

// modules
mod hashing;

// types
extern crate phala_types as types;
use types::{
    BlockRewardInfo, MinerStatsDelta, PRuntimeInfo, PayoutPrefs, PayoutReason, RoundInfo,
    RoundStats, Score, SignedDataType, SignedWorkerMessage, StashInfo, TransferData, WorkerInfo,
    WorkerMessagePayload, WorkerStateEnum,
};

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Trait>::TEECurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Trait>::TEECurrency as Currency<
    <T as frame_system::Trait>::AccountId,
>>::NegativeImbalance;

const PALLET_ID: ModuleId = ModuleId(*b"Phala!!!");
const RANDOMNESS_SUBJECT: &'static [u8] = b"PhalaPoW";
const BUILTIN_MACHINE_ID: &'static str = "BUILTIN";
const BLOCK_REWARD_TO_KEEP: u32 = 20;
const ROUND_STATS_TO_KEEP: u32 = 2;
pub const PERCENTAGE_BASE: u32 = 100_000;

pub trait OnRoundEnd {
    fn on_round_end(_round: u32) {}
}
impl OnRoundEnd for () {}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    type TEECurrency: Currency<Self::AccountId>;
    type UnixTime: UnixTime;
    type Treasury: OnUnbalanced<NegativeImbalanceOf<Self>>;
    type OnRoundEnd: OnRoundEnd;

    // Parameters
    type MaxHeartbeatPerWorkerPerHour: Get<u32>; // 2 tx
    type RoundInterval: Get<Self::BlockNumber>; // 1 hour
    type DecayInterval: Get<Self::BlockNumber>; // 180 days
    type DecayFactor: Get<Permill>; // 75%
    type InitialReward: Get<BalanceOf<Self>>; // 129600000 PHA
    type TreasuryRation: Get<u32>; // 20%
    type RewardRation: Get<u32>; // 80%
    type OnlineRewardPercentage: Get<Permill>; // rel: 37.5% post-taxed: 30%
    type ComputeRewardPercentage: Get<Permill>; // rel: 62.5% post-taxed: 50%
}

decl_storage! {
    trait Store for Module<T: Trait> as PhalaModule {
        // Messaging
        /// Number of all commands
        CommandNumber get(fn command_number): Option<u64>;
        /// Contract assignment
        ContractAssign get(fn contract_assign): map hasher(twox_64_concat) u32 => T::AccountId;
        /// Ingress message queue
        IngressSequence get(fn ingress_sequence): map hasher(twox_64_concat) u32 => u64;
        /// Worker Ingress message queue
        WorkerIngress get(fn worker_ingress): map hasher(twox_64_concat) T::AccountId => u64;

        // Worker registry
        /// Map from stash account to worker info
        ///
        /// (Indexed: MachineOwner, PendingUpdate, PendingExitingDelta, OnlineWorkers, TotalPower)
        WorkerState get(fn worker_state):
            map hasher(blake2_128_concat) T::AccountId => WorkerInfo<T::BlockNumber>;
        /// Map from stash account to stash info (indexed: Stash)
        StashState get(fn stash_state):
            map hasher(blake2_128_concat) T::AccountId => StashInfo<T::AccountId>;
        // Power and Fire
        /// Fire measures the total reward the miner can get (PoC3 1604-I specific)
        Fire get(fn fire): map hasher(blake2_128_concat) T::AccountId => BalanceOf<T>;
        /// Fire2 measures the total reward the miner can get (PoC3 1605-II specific)
        Fire2 get(fn fire2): map hasher(twox_64_concat) T::AccountId => BalanceOf<T>;
        /// Heartbeat counts
        Heartbeats get(fn heartbeats): map hasher(blake2_128_concat) T::AccountId => u32;

        // Indices
        /// Map from machine_id to stash
        MachineOwner get(fn machine_owner): map hasher(blake2_128_concat) Vec<u8> => T::AccountId;
        /// Map from controller to stash
        Stash get(fn stash): map hasher(blake2_128_concat) T::AccountId => T::AccountId;
        /// Number of all online workers in this round
        OnlineWorkers get(fn online_workers): u32;
        /// Number of all computation workers that will be elected in this round
        ComputeWorkers get(fn compute_workers): u32;
        /// Total Power points in this round. Updated at handle_round_ends().
        TotalPower get(fn total_power): u32;
        /// Total Fire points (1605-I specific)
        AccumulatedFire get(fn accumulated_fire): BalanceOf<T>;
        /// Total Fire points (1605-II specific)
        AccumulatedFire2 get(fn accumulated_fire2): BalanceOf<T>;

        // Round management
        /// The current mining round id
        Round get(fn round): RoundInfo<T::BlockNumber>;
        /// Indicates if we force the next round when the block finalized
        ForceNextRound: bool;
        /// Stash accounts with pending updates
        PendingUpdate get(fn pending_updates): Vec<T::AccountId>;
        /// The delta of the worker stats applaying at the end of this round due to exiting miners.
        PendingExitingDelta get(fn pending_exiting): MinerStatsDelta;
        /// Historical round stats; only the current and the last round are kept.
        RoundStatsHistory get(fn round_stats_history):
            map hasher(twox_64_concat) u32 => RoundStats;

        // Probabilistic rewarding
        BlockRewardSeeds: map hasher(twox_64_concat) T::BlockNumber => BlockRewardInfo;

        // Key Management
        /// Map from contract id to contract public key (TODO: migrate to real contract key from
        /// worker identity key)
        ContractKey get(fn contract_key): map hasher(twox_64_concat) u32 => Vec<u8>;

        // Configurations
        /// MREnclave Whitelist
        MREnclaveWhitelist get(fn mr_enclave_whitelist): Vec<Vec<u8>>;
        TargetOnlineRewardCount get(fn target_online_reward_count): u32;
        TargetComputeRewardCount get(fn target_compute_reward_count): u32;
        TargetVirtualTaskCount get(fn target_virtual_task_count): u32;
        /// Miners must submit the heartbeat within the reward window
        RewardWindow get(fn reward_window): T::BlockNumber;
    }

    add_extra_genesis {
        config(stakers): Vec<(T::AccountId, T::AccountId, Vec<u8>)>;  // <stash, controller, pubkey>
        config(contract_keys): Vec<Vec<u8>>;
        build(|config: &GenesisConfig<T>| {
            let base_mid = BUILTIN_MACHINE_ID.as_bytes().to_vec();
            for (i, (stash, controller, pubkey)) in config.stakers.iter().enumerate() {
                // Mock worker / stash info
                let mut machine_id = base_mid.clone();
                machine_id.push(b'0' + (i as u8));
                let worker_info = WorkerInfo::<T::BlockNumber> {
                    machine_id,
                    pubkey: pubkey.clone(),
                    last_updated: 0,
                    state: WorkerStateEnum::Free,
                    score: Some(Score {
                        overall_score: 100,
                        features: vec![1, 4]
                    })
                };
                WorkerState::<T>::insert(&stash, worker_info);
                let stash_info = StashInfo {
                    controller: controller.clone(),
                    payout_prefs: PayoutPrefs {
                        commission: 0,
                        target: stash.clone(),
                    }
                };
                StashState::<T>::insert(&stash, stash_info);
                // Update indices (skip MachineOwenr because we won't use it in anyway)
                Stash::<T>::insert(&controller, &stash);
            }
            // Insert the default contract key here
            for (i, key) in config.contract_keys.iter().enumerate() {
                ContractKey::insert(i as u32, key);
            }

            RewardWindow::<T>::put(T::BlockNumber::from(8u32));
            TargetOnlineRewardCount::put(20u32);
            TargetComputeRewardCount::put(10u32);
            TargetVirtualTaskCount::put(5u32);
        });
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        // Debug events
        LogString(Vec<u8>),
        LogI32(i32),
        // Chain events
        CommandPushed(AccountId, u32, Vec<u8>, u64),
        TransferToTee(AccountId, Balance),
        TransferToChain(AccountId, Balance, u64),
        WorkerRegistered(AccountId, Vec<u8>, Vec<u8>), // stash, identity_key, machine_id
        WorkerUnregistered(AccountId, Vec<u8>),        // stash, machine_id
        Heartbeat(AccountId, u32),
        Offline(AccountId),
        Slash(AccountId, Balance, u32),
        GotCredits(AccountId, u32, u32), // account, updated, delta
        WorkerStateUpdated(AccountId),
        WhitelistAdded(Vec<u8>),
        WhitelistRemoved(Vec<u8>),
        RewardSeed(BlockRewardInfo),
        WorkerMessageReceived(AccountId, Vec<u8>, u64), // stash, identity_key, seq
        MinerStarted(u32, AccountId),                   // round, stash
        MinerStopped(u32, AccountId),                   // round, stash
        NewMiningRound(u32),                            // round
        _Payout(AccountId, Balance, Balance),           // [DEPRECATED] dest, reward, treasury
        PayoutMissed(AccountId, AccountId),             // stash, dest
        WorkerRenewed(AccountId, Vec<u8>),              // stash, machine_id
        PayoutReward(AccountId, Balance, Balance, PayoutReason), // dest, reward, treasury, reason
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        InvalidIASSigningCert,
        InvalidIASReportSignature,
        InvalidQuoteStatus,
        InvalidRuntimeInfo,
        InvalidRuntimeInfoHash,
        MinerNotFound,
        BadMachineId,
        InvalidPubKey,
        InvalidSignature,
        InvalidSignatureBadLen,
        FailedToVerify,
        /// Not a controller account.
        NotController,
        /// Not a stash account.
        NotStash,
        /// Controller not found
        ControllerNotFound,
        /// Stash not found
        StashNotFound,
        /// Stash already bonded
        AlreadyBonded,
        /// Controller already paired
        AlreadyPaired,
        /// Commission is not between 0 and 100
        InvalidCommission,
        // Messagging
        /// Cannot decode the message
        InvalidMessage,
        /// Wrong sequence number of a message
        BadMessageSequence,
        // Token
        /// Failed to deposit tokens to pRuntime due to some internal errors in `Currency` module
        CannotDeposit,
        /// Failed to withdraw tokens from pRuntime reservation due to some internal error in
        /// `Currency` module
        CannotWithdraw,
        /// Bad input parameter
        InvalidInput,
        /// Bad input parameter length
        InvalidInputBadLength,
        /// Invalid contract
        InvalidContract,
        /// Internal Error
        InternalError,
        /// Wrong MRENCLAVE
        WrongMREnclave,
        /// Wrong MRENCLAVE whitelist index
        WrongWhitelistIndex,
        /// MRENCLAVE already exist
        MREnclaveAlreadyExist,
        /// MRENCLAVE not found
        MREnclaveNotFound,
        /// Unable to complete this action because it's an invalid state transition
        InvalidState,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            migrations_3_2_0::apply::<Self>()
        }

        fn on_finalize() {
            let now = System::<T>::block_number();
            let round = Round::<T>::get();
            Self::handle_block_reward(now, &round);
            // Should we end the current round?
            let interval = T::RoundInterval::get();
            if ForceNextRound::get() || now % interval == interval - 1u32.into() {
                ForceNextRound::put(false);
                Self::handle_round_ends(now, &round);
            }
        }

        // Messaging
        #[weight = 0]
        pub fn push_command(origin, contract_id: u32, payload: Vec<u8>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            let num = Self::command_number().unwrap_or(0);
            CommandNumber::put(num + 1);
            Self::deposit_event(RawEvent::CommandPushed(who, contract_id, payload, num));
            Ok(())
        }

        // Registry
        /// Crerate a new stash or update an existing one.
        #[weight = 0]
        pub fn set_stash(origin, controller: T::AccountId) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!Stash::<T>::contains_key(&controller), Error::<T>::AlreadyPaired);
            ensure!(!StashState::<T>::contains_key(&controller), Error::<T>::AlreadyBonded);
            let stash_state = if StashState::<T>::contains_key(&who) {
                // Remove previous controller
                let prev = StashState::<T>::get(&who);
                Stash::<T>::remove(&prev.controller);
                StashInfo {
                    controller: controller.clone(),
                    ..prev
                }
            } else {
                StashInfo {
                    controller: controller.clone(),
                    payout_prefs: PayoutPrefs {
                        commission: 0,
                        target: who.clone(),  // Set to the stash by default
                    }
                }
            };
            StashState::<T>::insert(&who, stash_state);
            Stash::<T>::insert(&controller, who);
            Ok(())
        }

        /// Update the payout preferences. Must be called by the controller.
        #[weight = 0]
        pub fn set_payout_prefs(origin, payout_commission: Option<u32>,
                                payout_target: Option<T::AccountId>)
                                -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Stash::<T>::contains_key(who.clone()), Error::<T>::NotController);
            let stash = Stash::<T>::get(who.clone());
            ensure!(StashState::<T>::contains_key(&stash), Error::<T>::StashNotFound);
            let mut stash_info = StashState::<T>::get(&stash);
            if let Some(val) = payout_commission {
                ensure!(val <= 100, Error::<T>::InvalidCommission);
                stash_info.payout_prefs.commission = val;
            }
            if let Some(val) = payout_target {
                stash_info.payout_prefs.target = val;
            }
            StashState::<T>::insert(&stash, stash_info);
            Ok(())
        }

        /// Register a worker node with a valid Remote Attestation report
        #[weight = 0]
        pub fn register_worker(origin, encoded_runtime_info: Vec<u8>, report: Vec<u8>, signature: Vec<u8>, raw_signing_cert: Vec<u8>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Stash::<T>::contains_key(&who), Error::<T>::NotController);
            let stash = Stash::<T>::get(&who);
            // Validate report
            let sig_cert = webpki::EndEntityCert::from(&raw_signing_cert);
            ensure!(sig_cert.is_ok(), Error::<T>::InvalidIASSigningCert);
            let sig_cert = sig_cert.unwrap();
            let verify_result = sig_cert.verify_signature(
                &webpki::RSA_PKCS1_2048_8192_SHA256,
                &report,
                &signature
            );
            ensure!(verify_result.is_ok(), Error::<T>::InvalidIASSigningCert);
            // TODO: Validate certificate
            // let chain: Vec<&[u8]> = Vec::new();
            // let now_func = webpki::Time::from_seconds_since_unix_epoch(1573419050);
            // match sig_cert.verify_is_valid_tls_server_cert(
            // 	SUPPORTED_SIG_ALGS,
            // 	&IAS_SERVER_ROOTS,
            // 	&chain,
            // 	now_func
            // ) {
            // 	Ok(()) => (),
            // 	Err(_) => panic!("verify cert failed")
            // };

            // Validate related fields
            let parsed_report: serde_json_no_std::Value = serde_json_no_std::from_slice(&report).unwrap();
            ensure!(
                &parsed_report["isvEnclaveQuoteStatus"] == "OK" || &parsed_report["isvEnclaveQuoteStatus"] == "CONFIGURATION_NEEDED" || &parsed_report["isvEnclaveQuoteStatus"] == "GROUP_OUT_OF_DATE",
                Error::<T>::InvalidQuoteStatus
            );
            // Extract quote fields
            let raw_quote_body = parsed_report["isvEnclaveQuoteBody"].as_str().unwrap();
            let quote_body = base64::decode(&raw_quote_body).unwrap();
            // Check the following fields
            let mr_enclave = &quote_body[112..144];
            let mr_signer = &quote_body[176..208];
            let isv_prod_id = &quote_body[304..306];
            let isv_svn = &quote_body[306..308];
            let whitelist = MREnclaveWhitelist::get();
            let t_mrenclave = Self::extend_mrenclave(mr_enclave, mr_signer, isv_prod_id, isv_svn);
            ensure!(whitelist.contains(&t_mrenclave), Error::<T>::WrongMREnclave);
            // Validate report data
            let report_data = &quote_body[368..432];
            let runtime_info_hash = hashing::blake2_512(&encoded_runtime_info);
            ensure!(runtime_info_hash.to_vec() == report_data, Error::<T>::InvalidRuntimeInfoHash);
            let runtime_info = PRuntimeInfo::decode(&mut &encoded_runtime_info[..]).map_err(|_| Error::<T>::InvalidRuntimeInfo)?;
            let machine_id = runtime_info.machine_id.to_vec();
            let pubkey = runtime_info.pubkey.to_vec();
            Self::register_worker_internal(&stash, &machine_id, &pubkey, &runtime_info.features)
                .map_err(Into::into)
        }

        #[weight = 0]
        fn force_register_worker(origin, stash: T::AccountId, machine_id: Vec<u8>, pubkey: Vec<u8>) -> dispatch::DispatchResult {
            ensure_root(origin)?;
            ensure!(StashState::<T>::contains_key(&stash), Error::<T>::StashNotFound);
            Self::register_worker_internal(&stash, &machine_id, &pubkey, &vec![1, 4])?;
            Ok(())
        }

        #[weight = 0]
        fn force_set_contract_key(origin, id: u32, pubkey: Vec<u8>) -> dispatch::DispatchResult {
            ensure_root(origin)?;
            ContractKey::insert(id, pubkey);
            Ok(())
        }

        // Mining

        #[weight = 0]
        fn start_mining_intention(origin) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Stash::<T>::contains_key(&who), Error::<T>::ControllerNotFound);
            let stash = Stash::<T>::get(who);
            let mut worker_info = WorkerState::<T>::get(&stash);

            match worker_info.state {
                WorkerStateEnum::Free => {
                    worker_info.state = WorkerStateEnum::MiningPending;
                    Self::deposit_event(RawEvent::WorkerStateUpdated(stash.clone()));
                },
                // WorkerStateEnum::MiningStopping => {
                // 	worker_info.state = WorkerStateEnum::Mining;
                // 	Self::deposit_event(RawEvent::WorkerStateUpdated(stash.clone()));
                // }
                WorkerStateEnum::Mining(_) | WorkerStateEnum::MiningPending => return Ok(()),
                _ => return Err(Error::<T>::InvalidState.into())
            };
            WorkerState::<T>::insert(&stash, worker_info);
            Self::mark_dirty(stash);
            Ok(())
        }

        #[weight = 0]
        fn stop_mining_intention(origin) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Stash::<T>::contains_key(&who), Error::<T>::ControllerNotFound);
            let stash = Stash::<T>::get(who);

            let mut worker_info = WorkerState::<T>::get(&stash);
            match worker_info.state {
                WorkerStateEnum::Mining(_) => {
                    worker_info.state = WorkerStateEnum::MiningStopping;
                    Self::deposit_event(RawEvent::WorkerStateUpdated(stash.clone()));
                },
                WorkerStateEnum::MiningPending => {
                    worker_info.state = WorkerStateEnum::Free;
                    Self::deposit_event(RawEvent::WorkerStateUpdated(stash.clone()));
                },
                WorkerStateEnum::Free | WorkerStateEnum::MiningStopping => return Ok(()),
                _ => return Err(Error::<T>::InvalidState.into())
            }
            WorkerState::<T>::insert(&stash, worker_info);
            Self::mark_dirty(stash);
            Ok(())
        }

        // Token

        #[weight = 0]
        fn transfer_to_tee(origin, #[compact] amount: BalanceOf<T>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            T::TEECurrency::transfer(&who, &Self::account_id(), amount, AllowDeath)
                .map_err(|_| Error::<T>::CannotDeposit)?;
            Self::deposit_event(RawEvent::TransferToTee(who, amount));
            Ok(())
        }

        #[weight = 0]
        fn transfer_to_chain(origin, data: Vec<u8>) -> dispatch::DispatchResult {
            // This is a specialized Contract-to-Chain message passing where the confidential
            // contract is always Balances (id = 2)
            // Anyone can call this method. As long as the message meets all the requirements
            // (signature, sequence id, etc), it's considered as a valid message.
            const CONTRACT_ID: u32 = 2;
            ensure_signed(origin)?;
            let transfer_data: TransferData<<T as frame_system::Trait>::AccountId, BalanceOf<T>>
                = Decode::decode(&mut &data[..]).map_err(|_| Error::<T>::InvalidInput)?;
            // Check sequence
            let sequence = IngressSequence::get(CONTRACT_ID);
            ensure!(transfer_data.data.sequence == sequence + 1, Error::<T>::BadMessageSequence);
            // Contract key
            ensure!(ContractKey::contains_key(CONTRACT_ID), Error::<T>::InvalidContract);
            let pubkey = ContractKey::get(CONTRACT_ID);
            // Validate TEE signature
            Self::verify_signature(&pubkey, &transfer_data)?;
            // Release funds
            T::TEECurrency::transfer(
                &Self::account_id(), &transfer_data.data.dest, transfer_data.data.amount,
                AllowDeath)
                .map_err(|_| Error::<T>::CannotWithdraw)?;
            // Announce the successful execution
            IngressSequence::insert(CONTRACT_ID, sequence + 1);
            Self::deposit_event(RawEvent::TransferToChain(transfer_data.data.dest, transfer_data.data.amount, sequence + 1));
            Ok(())
        }

        #[weight = 0]
        fn sync_worker_message(origin, msg: Vec<u8>) -> dispatch::DispatchResult {
            // TODO: allow anyone to relay the message
            let who = ensure_signed(origin)?;
            let signed: SignedWorkerMessage = Decode::decode(&mut &msg[..]).map_err(|_| Error::<T>::InvalidInput)?;
            // Worker queue sequence
            ensure!(Stash::<T>::contains_key(&who), Error::<T>::ControllerNotFound);
            let stash = Stash::<T>::get(&who);
            let expected_seq = WorkerIngress::<T>::get(&stash);
            ensure!(signed.data.sequence == expected_seq, Error::<T>::BadMessageSequence);
            // Validate signature
            let worker_info = WorkerState::<T>::get(&stash);
            if worker_info.state == WorkerStateEnum::<_>::Empty {
                return Err(Error::<T>::InvalidState.into());
            }
            Self::verify_signature(&worker_info.pubkey, &signed)?;
            // Dispatch message
            match signed.data.payload {
                WorkerMessagePayload::Heartbeat { block_num, claim_online, claim_compute } => {
                    let stash_info = StashState::<T>::get(&stash);
                    let id_pubkey = &worker_info.pubkey;
                    let score = match worker_info.score {
                        Some(score) => score.overall_score,
                        None => 0
                    };
                    Self::add_heartbeat(&who);	// TODO: necessary?
                    Self::handle_claim_reward(
                        &stash, &stash_info.payout_prefs.target, claim_online, claim_compute,
                        score, block_num.into());
                    Self::deposit_event(RawEvent::Heartbeat(stash.clone(), block_num));
                    Self::deposit_event(RawEvent::WorkerMessageReceived(
                        stash.clone(), id_pubkey.clone(), expected_seq));
                }
            }
            // Advance ingress sequence
            WorkerIngress::<T>::insert(&stash, expected_seq + 1);
            Ok(())
        }

        // Borrowing

        // Debug only

        #[weight = 0]
        fn force_next_round(origin) -> dispatch::DispatchResult {
            ensure_root(origin)?;
            ForceNextRound::put(true);
            Ok(())
        }

        #[weight = 0]
        fn force_add_fire(origin, targets: Vec<T::AccountId>, amounts: Vec<BalanceOf<T>>)
        -> dispatch::DispatchResult {
            ensure_root(origin)?;
            ensure!(targets.len() == amounts.len(), Error::<T>::InvalidInput);
            for i in 0..targets.len() {
                let target = &targets[i];
                let amount = amounts[i];
                Self::add_fire(target, amount);
            }
            Ok(())
        }

        #[weight = 0]
        fn force_set_virtual_tasks(origin, target: u32) -> dispatch::DispatchResult {
            ensure_root(origin)?;
            TargetVirtualTaskCount::put(target);
            Ok(())
        }

        // Whitelist

        #[weight = 0]
        fn add_mrenclave(origin, mr_enclave: Vec<u8>, mr_signer: Vec<u8>, isv_prod_id: Vec<u8>, isv_svn: Vec<u8>) -> dispatch::DispatchResult {
            ensure_root(origin)?;
            ensure!(mr_enclave.len() == 32 && mr_signer.len() == 32 && isv_prod_id.len() == 2 && isv_svn.len() == 2, Error::<T>::InvalidInputBadLength);
            Self::add_mrenclave_to_whitelist(&mr_enclave, &mr_signer, &isv_prod_id, &isv_svn)?;
            Ok(())
        }

        #[weight = 0]
        fn remove_mrenclave_by_raw_data(origin, mr_enclave: Vec<u8>, mr_signer: Vec<u8>, isv_prod_id: Vec<u8>, isv_svn: Vec<u8>) -> dispatch::DispatchResult {
            ensure_root(origin)?;
            ensure!(mr_enclave.len() == 32 && mr_signer.len() == 32 && isv_prod_id.len() == 2 && isv_svn.len() == 2, Error::<T>::InvalidInputBadLength);
            Self::remove_mrenclave_from_whitelist_by_raw_data(&mr_enclave, &mr_signer, &isv_prod_id, &isv_svn)?;
            Ok(())
        }

        #[weight = 0]
        fn remove_mrenclave_by_index(origin, index: u32) -> dispatch::DispatchResult {
            ensure_root(origin)?;
            Self::remove_mrenclave_from_whitelist_by_index(index as usize)?;
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

    pub fn is_controller(controller: T::AccountId) -> bool {
        Stash::<T>::contains_key(&controller)
    }
    pub fn verify_signature(
        serialized_pk: &Vec<u8>,
        data: &impl SignedDataType<Vec<u8>>,
    ) -> dispatch::DispatchResult {
        use sp_std::convert::TryFrom;
        ensure!(serialized_pk.len() == 33, Error::<T>::InvalidPubKey);
        let pubkey = sp_core::ecdsa::Public::try_from(serialized_pk.as_slice())
            .map_err(|_| Error::<T>::InvalidPubKey)?;
        let raw_sig = data.signature();
        ensure!(raw_sig.len() == 65, Error::<T>::InvalidSignatureBadLen);
        let sig = sp_core::ecdsa::Signature::try_from(raw_sig.as_slice())
            .map_err(|_| Error::<T>::InvalidSignature)?;
        let data = data.raw_data();

        ensure!(
            sp_io::crypto::ecdsa_verify(&sig, &data, &pubkey),
            Error::<T>::FailedToVerify
        );
        Ok(())
    }

    /// Unlinks a worker from a stash account. Only call when they are linked.
    fn unlink_worker(
        stash: &T::AccountId,
        machine_id: &Vec<u8>,
        stats_delta: &mut MinerStatsDelta,
    ) {
        WorkerIngress::<T>::remove(stash);
        MachineOwner::<T>::remove(machine_id);
        let info = WorkerState::<T>::take(stash);
        match info.state {
            WorkerStateEnum::<T::BlockNumber>::Mining(_)
            | WorkerStateEnum::<T::BlockNumber>::MiningStopping => {
                stats_delta.num_worker -= 1;
                if let Some(score) = &info.score {
                    stats_delta.num_power -= score.overall_score as i32;
                }
            }
            _ => (),
        };
        Self::deposit_event(RawEvent::WorkerUnregistered(
            stash.clone(),
            machine_id.clone(),
        ));
    }

    fn register_worker_internal(
        stash: &T::AccountId,
        machine_id: &Vec<u8>,
        pubkey: &Vec<u8>,
        worker_features: &Vec<u32>,
    ) -> Result<(), Error<T>> {
        let mut delta = PendingExitingDelta::get();
        let info = WorkerState::<T>::get(stash);
        let machine_owner = MachineOwner::<T>::get(machine_id);
        let renew_only = &info.machine_id == machine_id && !machine_id.is_empty();
        // Unlink existing machine and stash
        if !renew_only {
            // Worker linked to another stash
            if machine_owner != Default::default() && &machine_owner != stash {
                Self::unlink_worker(&machine_owner, machine_id, &mut delta);
            }
            // Stash linked to another worker
            if info.state != WorkerStateEnum::<T::BlockNumber>::Empty && !info.machine_id.is_empty()
            {
                Self::unlink_worker(stash, &info.machine_id, &mut delta);
            }
        }
        // Updated WorkerInfo fields
        let last_updated = T::UnixTime::now().as_millis().saturated_into::<u64>();
        let score = Some(Score {
            overall_score: calc_overall_score(worker_features)
                .map_err(|()| Error::<T>::InvalidInput)?,
            features: worker_features.clone(),
        });
        // New WorkerInfo
        let new_info = if renew_only {
            // Just renewed
            Self::deposit_event(RawEvent::WorkerRenewed(stash.clone(), machine_id.clone()));
            WorkerInfo {
                machine_id: machine_id.clone(), // should not change, but we set it anyway
                pubkey: pubkey.clone(),         // could change if the worker forgot the identity
                last_updated,
                score,  // could change if we do profiling
                ..info  // keep .state
            }
        } else {
            // Link a new worker
            Self::deposit_event(RawEvent::WorkerRegistered(
                stash.clone(),
                pubkey.clone(),
                machine_id.clone(),
            ));
            WorkerInfo {
                machine_id: machine_id.clone(),
                pubkey: pubkey.clone(),
                last_updated,
                state: WorkerStateEnum::Free,
                score,
            }
        };
        WorkerState::<T>::insert(stash, new_info);
        MachineOwner::<T>::insert(machine_id, stash);
        PendingExitingDelta::put(delta);
        WorkerIngress::<T>::insert(stash, 0);
        Ok(())
    }

    fn clear_dirty() {
        PendingUpdate::<T>::kill();
    }

    fn mark_dirty(account: T::AccountId) {
        let mut updates = PendingUpdate::<T>::get();
        let existed = updates.iter().find(|x| x == &&account);
        if existed == None {
            updates.push(account);
            PendingUpdate::<T>::put(updates);
        }
    }

    fn add_heartbeat(account: &T::AccountId) {
        let heartbeats = Heartbeats::<T>::get(account);
        Heartbeats::<T>::insert(account, heartbeats + 1);
    }

    fn clear_heartbeats() {
        Heartbeats::<T>::remove_all();
    }

    fn extend_mrenclave(
        mr_enclave: &[u8],
        mr_signer: &[u8],
        isv_prod_id: &[u8],
        isv_svn: &[u8],
    ) -> Vec<u8> {
        let mut t_mrenclave = Vec::new();
        t_mrenclave.extend_from_slice(mr_enclave);
        t_mrenclave.extend_from_slice(isv_prod_id);
        t_mrenclave.extend_from_slice(isv_svn);
        t_mrenclave.extend_from_slice(mr_signer);
        t_mrenclave
    }

    fn add_mrenclave_to_whitelist(
        mr_enclave: &[u8],
        mr_signer: &[u8],
        isv_prod_id: &[u8],
        isv_svn: &[u8],
    ) -> dispatch::DispatchResult {
        let mut whitelist = MREnclaveWhitelist::get();
        let white_mrenclave = Self::extend_mrenclave(mr_enclave, mr_signer, isv_prod_id, isv_svn);
        ensure!(
            !whitelist.contains(&white_mrenclave),
            Error::<T>::MREnclaveAlreadyExist
        );
        whitelist.push(white_mrenclave.clone());
        MREnclaveWhitelist::put(whitelist);
        Self::deposit_event(RawEvent::WhitelistAdded(white_mrenclave));
        Ok(())
    }

    fn remove_mrenclave_from_whitelist_by_raw_data(
        mr_enclave: &[u8],
        mr_signer: &[u8],
        isv_prod_id: &[u8],
        isv_svn: &[u8],
    ) -> dispatch::DispatchResult {
        let mut whitelist = MREnclaveWhitelist::get();
        let t_mrenclave = Self::extend_mrenclave(mr_enclave, mr_signer, isv_prod_id, isv_svn);
        ensure!(
            whitelist.contains(&t_mrenclave),
            Error::<T>::MREnclaveNotFound
        );
        let len = whitelist.len();
        for i in 0..len {
            if whitelist[i] == t_mrenclave {
                whitelist.remove(i);
                break;
            }
        }
        MREnclaveWhitelist::put(whitelist);
        Self::deposit_event(RawEvent::WhitelistRemoved(t_mrenclave));
        Ok(())
    }

    fn remove_mrenclave_from_whitelist_by_index(index: usize) -> dispatch::DispatchResult {
        let mut whitelist = MREnclaveWhitelist::get();
        ensure!(whitelist.len() > index, Error::<T>::WrongWhitelistIndex);
        let t_mrenclave = whitelist[index].clone();
        whitelist.remove(index);
        MREnclaveWhitelist::put(&whitelist);
        Self::deposit_event(RawEvent::WhitelistRemoved(t_mrenclave));
        Ok(())
    }

    /// Updates RoundStatsHistory and only keeps ROUND_STATS_TO_KEEP revisions.
    ///
    /// Shall call this function only when the new round have started.
    fn update_round_stats(round: u32, online_workers: u32, compute_workers: u32, total_power: u32) {
        if round >= ROUND_STATS_TO_KEEP {
            RoundStatsHistory::remove(round - ROUND_STATS_TO_KEEP);
        }
        let online_target = TargetOnlineRewardCount::get();
        let frac_target_online_reward = Self::clipped_target_number(online_target, online_workers);
        let frac_target_compute_reward =
            Self::clipped_target_number(TargetComputeRewardCount::get(), compute_workers);

        RoundStatsHistory::insert(
            round,
            RoundStats {
                round,
                online_workers,
                compute_workers,
                frac_target_online_reward,
                frac_target_compute_reward,
                total_power,
            },
        );
    }

    fn handle_round_ends(now: T::BlockNumber, round: &RoundInfo<T::BlockNumber>) {
        // Dependencies
        T::OnRoundEnd::on_round_end(round.round);

        // Handle PhalaModule specific tasks
        Self::clear_heartbeats();

        // Mining rounds
        let new_round = round.round + 1;
        let new_block = now + 1u32.into();

        // Process the pending update miner accoutns
        let mut delta = 0i32;
        let mut power_delta = 0i32;
        let dirty_accounts = PendingUpdate::<T>::get();
        for account in dirty_accounts.iter() {
            let mut updated = false;
            if !WorkerState::<T>::contains_key(&account) {
                // The worker just disappeared by force quit. In this case, the stats delta is
                // caught by PendingExitingDelta
                continue;
            }
            let mut worker_info = WorkerState::<T>::get(&account);
            match worker_info.state {
                WorkerStateEnum::MiningPending => {
                    // TODO: check enough stake, etc
                    worker_info.state = WorkerStateEnum::Mining(new_block);
                    delta += 1;
                    // Start from the next block
                    if let Some(ref score) = worker_info.score {
                        power_delta += score.overall_score as i32;
                    }
                    Self::deposit_event(RawEvent::MinerStarted(new_round, account.clone()));
                    updated = true;
                }
                WorkerStateEnum::MiningStopping => {
                    worker_info.state = WorkerStateEnum::Free;
                    delta -= 1;
                    if let Some(ref score) = worker_info.score {
                        power_delta -= score.overall_score as i32;
                    }
                    Self::deposit_event(RawEvent::MinerStopped(new_round, account.clone()));
                    updated = true;
                }
                _ => {}
            }
            // TODO: slash
            if updated {
                WorkerState::<T>::insert(&account, worker_info);
                Self::deposit_event(RawEvent::WorkerStateUpdated(account.clone()));
            }
        }
        // Handle PendingExitingDelta
        let exit_delta = PendingExitingDelta::take();
        delta += exit_delta.num_worker;
        power_delta += exit_delta.num_power;
        // New stats
        let new_online = (OnlineWorkers::get() as i32 + delta) as u32;
        OnlineWorkers::put(new_online);
        let new_total_power = ((TotalPower::get() as i32) + power_delta) as u32;
        TotalPower::put(new_total_power);
        // Computation tasks
        let compute_workers = cmp::min(new_online, TargetVirtualTaskCount::get());
        ComputeWorkers::put(compute_workers);

        // Start new round
        Self::clear_dirty();
        Round::<T>::put(RoundInfo {
            round: new_round,
            start_block: new_block,
        });
        Self::update_round_stats(new_round, new_online, compute_workers, new_total_power);
        Self::deposit_event(RawEvent::NewMiningRound(new_round));
    }

    fn handle_block_reward(now: T::BlockNumber, round: &RoundInfo<T::BlockNumber>) {
        // Remove the expired reward from the storage
        if now > BLOCK_REWARD_TO_KEEP.into() {
            BlockRewardSeeds::<T>::remove(now - BLOCK_REWARD_TO_KEEP.into());
        }
        // Generate the seed and targets
        let seed_hash = T::Randomness::random(RANDOMNESS_SUBJECT);
        let seed: U256 = AsRef::<[u8]>::as_ref(&seed_hash).into();
        let round_stats = RoundStatsHistory::get(round.round);
        let seed_info = BlockRewardInfo {
            seed,
            online_target: {
                if round_stats.online_workers == 0 {
                    U256::zero()
                } else {
                    u256_target(
                        round_stats.frac_target_online_reward as u64,
                        (round_stats.online_workers as u64) * (PERCENTAGE_BASE as u64),
                    )
                }
            },
            compute_target: {
                if round_stats.compute_workers == 0 {
                    U256::zero()
                } else {
                    u256_target(
                        round_stats.frac_target_compute_reward as u64,
                        (round_stats.compute_workers as u64) * (PERCENTAGE_BASE as u64),
                    )
                }
            },
        };
        // Save
        BlockRewardSeeds::<T>::insert(now, &seed_info);
        Self::deposit_event(RawEvent::RewardSeed(seed_info));
    }

    fn handle_claim_reward(
        stash: &T::AccountId,
        payout_target: &T::AccountId,
        claim_online: bool,
        claim_compute: bool,
        score: u32,
        claiming_block: T::BlockNumber,
    ) {
        // Check is mining
        let worker_info = WorkerState::<T>::get(stash);
        if let WorkerStateEnum::Mining(_) = worker_info.state {
            // Confirmed too late. Just skip.
            let now = System::<T>::block_number();
            let reward_window = RewardWindow::<T>::get();
            if claiming_block + reward_window < now {
                Self::deposit_event(RawEvent::PayoutMissed(stash.clone(), payout_target.clone()));
                return;
            }
            if claim_online || claim_compute {
                let round_stats = Self::round_stats_at(claiming_block);
                if round_stats.online_workers == 0 {
                    panic!("No online worker but the miner is claiming the rewards; qed");
                }
                let round_reward = Self::round_mining_reward_at(claiming_block);
                // Adjusted online worker reward
                if claim_online {
                    let online = Self::pretax_online_reward(
                        round_reward,
                        score,
                        round_stats.total_power,
                        round_stats.frac_target_online_reward,
                        round_stats.online_workers,
                    );
                    Self::payout(online, payout_target, PayoutReason::OnlineReward);
                }
                // Adjusted compute worker reward
                if claim_compute {
                    let compute = Self::pretax_compute_reward(
                        round_reward,
                        round_stats.frac_target_compute_reward,
                        round_stats.compute_workers,
                    );
                    Self::payout(compute, payout_target, PayoutReason::ComputeReward);
                }
            }

            // TODO: do we need to check xor threshold?
            // TODO: Check is_compute
        }
    }

    /// Calculates the clipped target transaction number for this round
    fn clipped_target_number(num_target: u32, num_workers: u32) -> u32 {
        // Miner tx per block: t <= max_tx_per_hour * N/T
        let round_blocks = T::RoundInterval::get().saturated_into::<u32>();
        let upper_clipped = cmp::min(
            num_target * PERCENTAGE_BASE,
            (T::MaxHeartbeatPerWorkerPerHour::get() as u64
                * (num_workers as u64)
                * (PERCENTAGE_BASE as u64)
                / (round_blocks as u64)) as u32,
        );
        upper_clipped
    }

    /// Calculates the total mining reward for this round
    fn round_mining_reward_at(_blocknum: T::BlockNumber) -> BalanceOf<T> {
        let initial_reward: BalanceOf<T> = T::InitialReward::get()
            / BalanceOf::<T>::from(
                (T::DecayInterval::get() / T::RoundInterval::get()).saturated_into::<u32>(),
            );
        // BalanceOf::<T>::from();
        let round_reward = initial_reward;
        // TODO: consider the halvings
        //
        // let n = (blocknum / T::DecayInterval::get()) as u32;
        // let decay_reward = decay_reward * DecayFactor.pow(n)
        round_reward
    }

    /// Gets the RoundStats information at the given blocknum, not earlier than the last round.
    fn round_stats_at(block: T::BlockNumber) -> RoundStats {
        let current_round = Round::<T>::get();
        let round = if block < current_round.start_block {
            current_round.round - 1
        } else {
            current_round.round
        };
        RoundStatsHistory::get(round)
    }

    /// Calculates the adjusted online reward for a specific miner
    fn pretax_online_reward(
        round_reward: BalanceOf<T>,
        score: u32,
        total_power: u32,
        frac_target_online_reward: u32,
        workers: u32,
    ) -> BalanceOf<T> {
        Self::pretax_reward(
            round_reward,
            score,
            total_power,
            frac_target_online_reward,
            T::OnlineRewardPercentage::get(),
            workers,
        )
    }

    /// Calculates the adjust computation reward (every miner has same reward now)
    fn pretax_compute_reward(
        round_reward: BalanceOf<T>,
        frac_target_compute_reward: u32,
        compute_workers: u32,
    ) -> BalanceOf<T> {
        Self::pretax_reward(
            round_reward,
            // Since all compute worker has the equal reward, we set the propotion reward of a
            // worker to `1 / compute_workers`
            1,
            compute_workers,
            frac_target_compute_reward,
            T::ComputeRewardPercentage::get(),
            compute_workers,
        )
    }

    fn pretax_reward(
        round_reward: BalanceOf<T>,
        weight: u32,
        total_weight: u32,
        frac_target_reward: u32,
        reward_percentage: Permill,
        workers: u32,
    ) -> BalanceOf<T> {
        let round_blocks = T::RoundInterval::get().saturated_into::<u64>();
        // The target reward for this miner
        let target =
            round_reward * BalanceOf::<T>::from(weight) / BalanceOf::<T>::from(total_weight);
        // Adjust based on the mathematical expectation
        let adjusted: BalanceOf<T> = target
            * ((workers as u64) * (PERCENTAGE_BASE as u64)).saturated_into()
            / ((round_blocks as u64) * (frac_target_reward as u64)).saturated_into();
        let reward = reward_percentage * adjusted;
        reward
    }

    /// Actually pays out the reward
    fn payout(value: BalanceOf<T>, target: &T::AccountId, reason: PayoutReason) {
        // Retion the reward and the treasury deposit
        let coins = T::TEECurrency::issue(value);
        let (coin_reward, coin_treasury) =
            coins.ration(T::RewardRation::get(), T::TreasuryRation::get());
        // Payout!
        // TODO: in real => T::TEECurrency::resolve_creating(payout_target, coin_reward);
        Self::deposit_event(RawEvent::PayoutReward(
            target.clone(),
            coin_reward.peek(),
            coin_treasury.peek(),
            reason,
        ));
        Self::add_fire(&target, coin_reward.peek());
        T::Treasury::on_unbalanced(coin_treasury);
    }

    fn add_fire(dest: &T::AccountId, amount: BalanceOf<T>) {
        Fire2::<T>::mutate(dest, |x| *x += amount);
        AccumulatedFire2::<T>::mutate(|x| *x += amount);
    }
}

fn calc_overall_score(features: &Vec<u32>) -> Result<u32, ()> {
    if features.len() != 2 {
        return Err(());
    }
    let core = features[0];
    let feature_level = features[1];
    Ok(core * (feature_level * 10 + 60))
}

fn u256_target(m: u64, n: u64) -> U256 {
    // m of n (MAX * (n / m))
    if m > n || n == 0 {
        panic!("Invalid parameter");
    }
    U256::MAX / n * m
}

// Migration from 3.1.0 to 3.2.0
mod migrations_3_2_0;
impl<T: Trait> migrations_3_2_0::V31ToV32 for Module<T> {
    type Module = Module<T>;
    type AccountId = T::AccountId;
    type BlockNumber = T::BlockNumber;
}
