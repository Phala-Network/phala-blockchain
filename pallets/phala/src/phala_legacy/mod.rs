extern crate alloc;
use sp_core::U256;
use sp_std::convert::TryFrom;
use sp_std::prelude::*;
use sp_std::{cmp, vec};

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch, ensure};
use frame_system::{ensure_root, ensure_signed, Pallet as System};

use alloc::{borrow::ToOwned, vec::Vec};
use codec::Decode;
use frame_support::{
	dispatch::DispatchResult,
	traits::{
		Currency, ExistenceRequirement::AllowDeath, Get, Imbalance, OnUnbalanced, Randomness,
		UnixTime,
	},
	PalletId,
};
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	Permill, SaturatedConversion,
};

#[macro_use]
mod benchmarking;

// modules
mod hashing;
pub mod weights;

// types
extern crate phala_types as types;
use types::{
	messaging::Message, BlockRewardInfo, MinerStatsDelta, PRuntimeInfo, PayoutPrefs, PayoutReason,
	RoundInfo, RoundStats, Score, SignedDataType, SignedWorkerMessage, StashInfo, StashWorkerStats,
	TransferData, WorkerInfo, WorkerMessagePayload, WorkerStateEnum,
};

// constants
mod constants;
pub use constants::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use weights::WeightInfo;

type BalanceOf<T> =
	<<T as Config>::TEECurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Config>::TEECurrency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

// Events

pub trait OnRoundEnd {
	fn on_round_end(_round: u32) {}
}
impl OnRoundEnd for () {}

pub trait OnMessageReceived {
	fn on_message_received(_message: &Message) -> DispatchResult {
		Ok(())
	}
}
impl OnMessageReceived for () {}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	type TEECurrency: Currency<Self::AccountId>;
	type UnixTime: UnixTime;
	type Treasury: OnUnbalanced<NegativeImbalanceOf<Self>>;
	type WeightInfo: WeightInfo;
	type OnRoundEnd: OnRoundEnd;
	type OnLotteryMessage: OnMessageReceived;

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
	type OfflineOffenseSlash: Get<BalanceOf<Self>>;
	type OfflineReportReward: Get<BalanceOf<Self>>;
}

decl_storage! {
	trait Store for Module<T: Config> as Phala {
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

		// Stats (poc3-only)
		WorkerComputeReward: map hasher(twox_64_concat) T::AccountId => u32;
		PayoutComputeReward: map hasher(twox_64_concat) T::AccountId => u32;

		RoundWorkerStats get(fn round_worker_stats): map hasher(twox_64_concat) T::AccountId => StashWorkerStats<BalanceOf<T>>;

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
		/// The last block where a worker has on-chain activity, updated by `sync_worker_message`
		LastWorkerActivity: map hasher(twox_64_concat) T::AccountId => T::BlockNumber;

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
		/// Miners must submit the heartbeat in `(now - reward_window, now]`
		RewardWindow get(fn reward_window): T::BlockNumber;
		/// Miners could be slashed in `(now - slash_window, now - reward_window]`
		SlashWindow get(fn slash_window): T::BlockNumber;
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
					}),
					confidence_level: 128u8,
					runtime_version: 0
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

			// TODO: reconsider the window length
			RewardWindow::<T>::put(T::BlockNumber::from(8u32));  // 5 blocks (3 for finalizing)
			SlashWindow::<T>::put(T::BlockNumber::from(40u32));  // 5x larger window
			TargetOnlineRewardCount::put(20u32);
			TargetComputeRewardCount::put(10u32);
			TargetVirtualTaskCount::put(5u32);
		});
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Config>::AccountId,
		Balance = BalanceOf<T>,
	{
		// Chain events
		CommandPushed(AccountId, u32, Vec<u8>, u64),
		TransferToTee(AccountId, Balance),
		TransferToChain(AccountId, Balance, u64),
		/// [stash, identity_key, machine_id]
		WorkerRegistered(AccountId, Vec<u8>, Vec<u8>),
		/// [stash, machine_id]
		WorkerUnregistered(AccountId, Vec<u8>),
		Heartbeat(AccountId, u32),
		Offline(AccountId),
		/// Some worker got slashed. [stash, payout_addr, lost_amount, reporter, win_amount]
		Slash(AccountId, AccountId, Balance, AccountId, Balance),
		_GotCredits(AccountId, u32, u32), // [DEPRECATED] [account, updated, delta]
		WorkerStateUpdated(AccountId),
		WhitelistAdded(Vec<u8>),
		WhitelistRemoved(Vec<u8>),
		RewardSeed(BlockRewardInfo),
		/// [stash, identity_key, seq]
		WorkerMessageReceived(AccountId, Vec<u8>, u64),
		/// [round, stash]
		MinerStarted(u32, AccountId),
		/// [round, stash]
		MinerStopped(u32, AccountId),
		/// [round]
		NewMiningRound(u32),
		_Payout(AccountId, Balance, Balance), // [DEPRECATED] dest, reward, treasury
		/// [stash, dest]
		PayoutMissed(AccountId, AccountId),
		/// A worker is reset due to renew registration or slash, causing the reset of the worker
		/// ingress sequence. [stash, machine_id]
		WorkerReset(AccountId, Vec<u8>),
		/// [dest, reward, treasury, reason]
		PayoutReward(AccountId, Balance, Balance, PayoutReason),
		/// A lottery contract message was received. [sequence]
		LotteryMessageReceived(u64),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		InvalidIASSigningCert,
		InvalidIASReportSignature,
		InvalidQuoteStatus,
		OutdatedIASReport,
		BadIASReport,
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
		// Messaging
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
		/// The off-line report is beyond the slash window
		TooAncientReport,
		/// The reported worker is still alive
		ReportedWorkerStillAlive,
		/// The reported worker is not mining
		ReportedWorkerNotMining,
		/// The report has an invalid proof
		InvalidProof,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

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
		#[weight = T::WeightInfo::push_command()]
		pub fn push_command(origin, contract_id: u32, payload: Vec<u8>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let num = Self::command_number().unwrap_or(0);
			CommandNumber::put(num + 1);
			Self::deposit_event(RawEvent::CommandPushed(who, contract_id, payload, num));
			Ok(())
		}

		// Registry
		/// Crerate a new stash or update an existing one.
		#[weight = T::WeightInfo::set_stash()]
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
		#[weight = T::WeightInfo::set_payout_prefs()]
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
		#[weight = T::WeightInfo::register_worker()]
		pub fn register_worker(origin, encoded_runtime_info: Vec<u8>, report: Vec<u8>, signature: Vec<u8>, raw_signing_cert: Vec<u8>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Stash::<T>::contains_key(&who), Error::<T>::NotController);
			let stash = Stash::<T>::get(&who);
			// Validate report
			let sig_cert = webpki::EndEntityCert::try_from(&raw_signing_cert[..]);
			ensure!(sig_cert.is_ok(), Error::<T>::InvalidIASSigningCert);
			let sig_cert = sig_cert.unwrap();
			let verify_result = sig_cert.verify_signature(
				&webpki::RSA_PKCS1_2048_8192_SHA256,
				&report,
				&signature
			);
			ensure!(verify_result.is_ok(), Error::<T>::InvalidIASSigningCert);

			let now = T::UnixTime::now().as_secs().saturated_into::<u64>();

			// Validate certificate
			let chain: Vec<&[u8]> = Vec::new();
			let time_now = webpki::Time::from_seconds_since_unix_epoch(now);
			let tls_server_cert_valid = sig_cert.verify_is_valid_tls_server_cert(
				SUPPORTED_SIG_ALGS,
				&IAS_SERVER_ROOTS,
				&chain,
				time_now
			);
			ensure!(tls_server_cert_valid.is_ok(), Error::<T>::InvalidIASSigningCert);

			// Validate related fields
			let parsed_report: serde_json::Value = serde_json::from_slice(&report).unwrap();

			// Validate time
			let raw_report_timestamp = parsed_report["timestamp"].as_str().unwrap_or("UNKNOWN").to_owned() + "Z";
			let report_timestamp = chrono::DateTime::parse_from_rfc3339(&raw_report_timestamp).map_err(|_| Error::<T>::BadIASReport)?.timestamp();
			ensure!((now as i64 - report_timestamp) < 7200, Error::<T>::OutdatedIASReport);

			// Filter valid `isvEnclaveQuoteStatus`
			let quote_status = &parsed_report["isvEnclaveQuoteStatus"].as_str().unwrap_or("UNKNOWN");
			let mut confidence_level: u8 = 128;
			if IAS_QUOTE_STATUS_LEVEL_1.contains(quote_status) {
				confidence_level = 1;
			} else if IAS_QUOTE_STATUS_LEVEL_2.contains(quote_status) {
				confidence_level = 2;
			} else if IAS_QUOTE_STATUS_LEVEL_3.contains(quote_status) {
				confidence_level = 3;
			} else if IAS_QUOTE_STATUS_LEVEL_5.contains(quote_status) {
				confidence_level = 5;
			}
			ensure!(
				confidence_level != 128,
				Error::<T>::InvalidQuoteStatus
			);

			if confidence_level < 5 {
				// Filter AdvisoryIDs. `advisoryIDs` is optional
				if let Some(advisory_ids) = parsed_report["advisoryIDs"].as_array() {
					for advisory_id in advisory_ids {
						let advisory_id = advisory_id.as_str().ok_or(Error::<T>::BadIASReport)?;

						if !IAS_QUOTE_ADVISORY_ID_WHITELIST.contains(&advisory_id) {
							confidence_level = 4;
						}
					}
				}
			}

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
			let runtime_version = runtime_info.version;
			let machine_id = runtime_info.machine_id.to_vec();
			let pubkey = runtime_info.pubkey.as_ref().to_vec();

			Self::register_worker_internal(&stash, &machine_id, &pubkey, &runtime_info.features, confidence_level, runtime_version)
				.map_err(Into::into)
		}

		#[weight = 0]
		pub fn reset_worker(origin) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Stash::<T>::contains_key(&who), Error::<T>::NotController);
			let stash = Stash::<T>::get(&who);
			let worker_info = WorkerState::<T>::get(&stash);
			let machine_id = worker_info.machine_id;

			Self::deposit_event(RawEvent::WorkerReset(stash.clone(), machine_id.clone()));

			WorkerIngress::<T>::insert(stash, 0);
			Ok(())
		}

		#[weight = T::WeightInfo::force_register_worker()]
		fn force_register_worker(origin, stash: T::AccountId, machine_id: Vec<u8>, pubkey: Vec<u8>) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			ensure!(StashState::<T>::contains_key(&stash), Error::<T>::StashNotFound);
			Self::register_worker_internal(&stash, &machine_id, &pubkey, &vec![1, 4], 0, 0)?;
			Ok(())
		}

		#[weight = T::WeightInfo::force_set_contract_key()]
		fn force_set_contract_key(origin, id: u32, pubkey: Vec<u8>) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			ContractKey::insert(id, pubkey);
			Ok(())
		}

		// Mining

		#[weight = T::WeightInfo::start_mining_intention()]
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

		#[weight = T::WeightInfo::stop_mining_intention()]
		fn stop_mining_intention(origin) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Stash::<T>::contains_key(&who), Error::<T>::ControllerNotFound);
			let stash = Stash::<T>::get(who);

			Self::stop_mining_internal(&stash)?;
			Ok(())
		}

		// Token

		#[weight = T::WeightInfo::transfer_to_tee()]
		fn transfer_to_tee(origin, #[compact] amount: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			T::TEECurrency::transfer(&who, &Self::account_id(), amount, AllowDeath)
				.map_err(|_| Error::<T>::CannotDeposit)?;
			Self::deposit_event(RawEvent::TransferToTee(who, amount));
			Ok(())
		}

		#[weight = T::WeightInfo::transfer_to_chain()]
		fn transfer_to_chain(origin, data: Vec<u8>) -> dispatch::DispatchResult {
			// This is a specialized Contract-to-Chain message passing where the confidential
			// contract is always Balances (id = 2)
			// Anyone can call this method. As long as the message meets all the requirements
			// (signature, sequence id, etc), it's considered as a valid message.
			const CONTRACT_ID: u32 = 2;
			ensure_signed(origin)?;
			let transfer_data: TransferData<<T as frame_system::Config>::AccountId, BalanceOf<T>>
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

		// Messaging

		#[weight = T::WeightInfo::sync_worker_message()]
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
					Self::add_heartbeat(&stash, block_num.into());
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


		// Violence
		#[weight = 0]
		fn report_offline(
			origin, stash: T::AccountId, block_num: T::BlockNumber
		) -> dispatch::DispatchResult {
			let reporter = ensure_signed(origin)?;
			let now = System::<T>::block_number();
			let slash_window = SlashWindow::<T>::get();
			ensure!(block_num + slash_window > now, Error::<T>::TooAncientReport);

			// TODO: should slash force replacement of TEE worker as well!
			// TODO: how to handle the report to the previous round?
			let round_start = Round::<T>::get().start_block;
			ensure!(block_num >= round_start, Error::<T>::TooAncientReport);
			// Worker is online (Mining / PendingStopping)
			ensure!(WorkerState::<T>::contains_key(&stash), Error::<T>::StashNotFound);
			let worker_info = WorkerState::<T>::get(&stash);
			let is_mining = match worker_info.state {
				WorkerStateEnum::Mining(_) | WorkerStateEnum::MiningStopping => true,
				_ => false,
			};
			ensure!(is_mining, Error::<T>::ReportedWorkerNotMining);
			// Worker is not alive
			ensure!(
				LastWorkerActivity::<T>::get(&stash) < block_num,
				Error::<T>::ReportedWorkerStillAlive
			);
			// Check worker's pubkey xor privkey < target (!)
			let reward_info = BlockRewardSeeds::<T>::get(block_num);
			ensure!(
				check_pubkey_hit_target(worker_info.pubkey.as_slice(), &reward_info),
				Error::<T>::InvalidProof
			);

			Self::slash_offline(&stash, &reporter, &worker_info.machine_id)?;
			Ok(())
		}

		// Debug only

		#[weight = T::WeightInfo::force_next_round()]
		fn force_next_round(origin) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			ForceNextRound::put(true);
			Ok(())
		}

		#[weight = T::WeightInfo::force_add_fire()]
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

		#[weight = T::WeightInfo::force_set_virtual_tasks()]
		fn force_set_virtual_tasks(origin, target: u32) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			TargetVirtualTaskCount::put(target);
			Ok(())
		}

		#[weight = T::WeightInfo::force_reset_fire()]
		fn force_reset_fire(origin) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			Fire2::<T>::remove_all();
			AccumulatedFire2::<T>::kill();
			Ok(())
		}

		#[weight = 0]
		fn force_set_window(
			origin, reward_window: Option<T::BlockNumber>, slash_window: Option<T::BlockNumber>
		) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			let old_reward = RewardWindow::<T>::get();
			let old_slash = SlashWindow::<T>::try_get()
				.unwrap_or(DEFAULT_BLOCK_REWARD_TO_KEEP.into());
			let reward = reward_window.unwrap_or(old_reward);
			let slash = slash_window.unwrap_or(old_slash);
			ensure!(slash >= reward, Error::<T>::InvalidInput);
			// Clean up (now - old, now - new] when the new slash window is shorter
			if slash < old_slash {
				let now = System::<T>::block_number();
				if now > slash {
					let last_empty_idx = if now >= old_slash {
						(now - old_slash).into()
					} else {
						Zero::zero()
					};
					let mut i = now - slash;
					while i > last_empty_idx {
						BlockRewardSeeds::<T>::remove(i);
						i -= One::one();
					}
				}
			}
			RewardWindow::<T>::put(reward);
			SlashWindow::<T>::put(slash);
			Ok(())
		}

		// Whitelist

		#[weight = T::WeightInfo::add_mrenclave()]
		fn add_mrenclave(origin, mr_enclave: Vec<u8>, mr_signer: Vec<u8>, isv_prod_id: Vec<u8>, isv_svn: Vec<u8>) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			ensure!(mr_enclave.len() == 32 && mr_signer.len() == 32 && isv_prod_id.len() == 2 && isv_svn.len() == 2, Error::<T>::InvalidInputBadLength);
			Self::add_mrenclave_to_whitelist(&mr_enclave, &mr_signer, &isv_prod_id, &isv_svn)?;
			Ok(())
		}

		#[weight = T::WeightInfo::remove_mrenclave_by_raw_data()]
		fn remove_mrenclave_by_raw_data(origin, mr_enclave: Vec<u8>, mr_signer: Vec<u8>, isv_prod_id: Vec<u8>, isv_svn: Vec<u8>) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			ensure!(mr_enclave.len() == 32 && mr_signer.len() == 32 && isv_prod_id.len() == 2 && isv_svn.len() == 2, Error::<T>::InvalidInputBadLength);
			Self::remove_mrenclave_from_whitelist_by_raw_data(&mr_enclave, &mr_signer, &isv_prod_id, &isv_svn)?;
			Ok(())
		}

		#[weight = T::WeightInfo::remove_mrenclave_by_index()]
		fn remove_mrenclave_by_index(origin, index: u32) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			Self::remove_mrenclave_from_whitelist_by_index(index as usize)?;
			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
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

	fn stop_mining_internal(stash: &T::AccountId) -> dispatch::DispatchResult {
		let mut worker_info = WorkerState::<T>::get(&stash);
		match worker_info.state {
			WorkerStateEnum::Mining(_) => {
				worker_info.state = WorkerStateEnum::MiningStopping;
				Self::deposit_event(RawEvent::WorkerStateUpdated(stash.clone()));
			}
			WorkerStateEnum::MiningPending => {
				worker_info.state = WorkerStateEnum::Free;
				Self::deposit_event(RawEvent::WorkerStateUpdated(stash.clone()));
			}
			WorkerStateEnum::Free | WorkerStateEnum::MiningStopping => return Ok(()),
			_ => return Err(Error::<T>::InvalidState.into()),
		}
		WorkerState::<T>::insert(&stash, worker_info);
		Self::mark_dirty(stash.clone());
		Ok(())
	}

	/// Unlinks a worker from a stash account. Only call when they are linked.
	fn unlink_worker(
		stash: &T::AccountId,
		machine_id: &Vec<u8>,
		stats_delta: &mut MinerStatsDelta,
	) {
		Self::kick_worker(stash, stats_delta);
		WorkerState::<T>::remove(stash);
		MachineOwner::<T>::remove(machine_id);
		Self::deposit_event(RawEvent::WorkerUnregistered(
			stash.clone(),
			machine_id.clone(),
		));
	}

	/// Kicks a worker if it's online. Only do this to force offline a worker.
	fn kick_worker(stash: &T::AccountId, stats_delta: &mut MinerStatsDelta) -> bool {
		WorkerIngress::<T>::remove(stash);
		let mut info = WorkerState::<T>::get(stash);
		match info.state {
			WorkerStateEnum::<T::BlockNumber>::Mining(_)
			| WorkerStateEnum::<T::BlockNumber>::MiningStopping => {
				// Shutdown the worker and update MinerStatesDelta
				stats_delta.num_worker -= 1;
				if let Some(score) = &info.score {
					stats_delta.num_power -= score.overall_score as i32;
				}
				// Set the state to Free
				info.last_updated = T::UnixTime::now().as_millis().saturated_into::<u64>();
				info.state = WorkerStateEnum::Free;
				WorkerState::<T>::insert(&stash, info);
				// MinerStopped event
				let round = Round::<T>::get().round;
				Self::deposit_event(RawEvent::MinerStopped(round, stash.clone()));
				// TODO: slash?
				return true;
			}
			_ => (),
		};
		false
	}

	fn register_worker_internal(
		stash: &T::AccountId,
		machine_id: &Vec<u8>,
		pubkey: &Vec<u8>,
		worker_features: &Vec<u32>,
		confidence_level: u8,
		runtime_version: u32,
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
			Self::deposit_event(RawEvent::WorkerReset(stash.clone(), machine_id.clone()));
			WorkerInfo {
				machine_id: machine_id.clone(), // should not change, but we set it anyway
				pubkey: pubkey.clone(),         // could change if the worker forgot the identity
				last_updated,
				score,            // could change if we do profiling
				confidence_level, // could change on redo RA
				runtime_version,  // could change on redo RA
				..info            // keep .state
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
				confidence_level,
				runtime_version,
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

	fn add_heartbeat(account: &T::AccountId, block_num: T::BlockNumber) {
		// TODO: remove?
		let heartbeats = Heartbeats::<T>::get(account);
		Heartbeats::<T>::insert(account, heartbeats + 1);
		// Record the worker activity to avoid slash of honest worker
		LastWorkerActivity::<T>::insert(account, block_num);
	}

	fn clear_heartbeats() {
		// TODO: remove?
		Heartbeats::<T>::remove_all();
	}

	/// Slashes a worker and put it offline by force
	///
	/// The `stash` account will be slashed by 100 FIRE, and the `reporter` account will earn half
	/// as a reward. This method ensures no worker will be slashed twice.
	fn slash_offline(
		stash: &T::AccountId,
		reporter: &T::AccountId,
		machine_id: &Vec<u8>,
	) -> dispatch::DispatchResult {
		// We have to kick the worker by force to avoid double slash
		PendingExitingDelta::mutate(|stats_delta| Self::kick_worker(stash, stats_delta));
		Self::deposit_event(RawEvent::WorkerReset(stash.clone(), machine_id.clone()));

		// Assume ensure!(StashState::<T>::contains_key(&stash));
		let payout = StashState::<T>::get(&stash).payout_prefs.target;
		let lost_amount = T::OfflineOffenseSlash::get();
		let win_amount = T::OfflineReportReward::get();
		// TODO: what if the worker suddently change its payout address?
		// Not necessary a problem on PoC-3 testnet, because it's unwise to switch the payout
		// address in anyway. On mainnet, we should slash the stake instead.
		let to_sub = Self::try_sub_fire(&payout, lost_amount);
		Self::add_fire(reporter, win_amount);

		let prev = RoundWorkerStats::<T>::get(&stash);
		let worker_state = StashWorkerStats {
			slash: prev.slash + to_sub,
			compute_received: prev.compute_received,
			online_received: prev.online_received,
		};
		RoundWorkerStats::<T>::insert(&stash, worker_state);

		Self::deposit_event(RawEvent::Slash(
			stash.clone(),
			payout.clone(),
			lost_amount,
			reporter.clone(),
			win_amount,
		));
		Ok(())
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
		RoundWorkerStats::<T>::remove_all();
		Self::deposit_event(RawEvent::NewMiningRound(new_round));
	}

	fn handle_block_reward(now: T::BlockNumber, round: &RoundInfo<T::BlockNumber>) {
		let slash_window = SlashWindow::<T>::get();
		// Remove the expired reward from the storage
		if now > slash_window {
			BlockRewardSeeds::<T>::remove(now - slash_window);
		}
		// Generate the seed and targets
		let seed_hash = T::Randomness::random(RANDOMNESS_SUBJECT).0;
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
					let coin_reward =
						Self::payout(online, payout_target, PayoutReason::OnlineReward);
					let prev = RoundWorkerStats::<T>::get(&stash);
					let worker_state = StashWorkerStats {
						slash: prev.slash,
						compute_received: prev.compute_received,
						online_received: prev.online_received + coin_reward,
					};
					RoundWorkerStats::<T>::insert(&stash, worker_state);
				}
				// Adjusted compute worker reward
				if claim_compute {
					let compute = Self::pretax_compute_reward(
						round_reward,
						round_stats.frac_target_compute_reward,
						round_stats.compute_workers,
					);
					let coin_reward =
						Self::payout(compute, payout_target, PayoutReason::ComputeReward);
					let prev = RoundWorkerStats::<T>::get(&stash);
					let worker_state = StashWorkerStats {
						slash: prev.slash,
						compute_received: prev.compute_received + coin_reward,
						online_received: prev.online_received,
					};
					RoundWorkerStats::<T>::insert(&stash, worker_state);

					// TODO: remove after PoC-3
					WorkerComputeReward::<T>::mutate(stash, |x| *x += 1);
					PayoutComputeReward::<T>::mutate(payout_target, |x| *x += 1);
				}
			}
			// TODO: do we need to check xor threshold?
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
	fn payout(value: BalanceOf<T>, target: &T::AccountId, reason: PayoutReason) -> BalanceOf<T> {
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
		coin_reward.peek()
	}

	fn add_fire(dest: &T::AccountId, amount: BalanceOf<T>) {
		Fire2::<T>::mutate(dest, |x| *x += amount);
		AccumulatedFire2::<T>::mutate(|x| *x += amount);
	}

	fn try_sub_fire(dest: &T::AccountId, amount: BalanceOf<T>) -> BalanceOf<T> {
		let to_sub = cmp::min(amount, Fire2::<T>::get(dest));
		Fire2::<T>::mutate(dest, |x| *x -= to_sub);
		AccumulatedFire2::<T>::mutate(|x| *x -= to_sub);
		to_sub
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

fn check_pubkey_hit_target(raw_pubkey: &[u8], reward_info: &BlockRewardInfo) -> bool {
	let pkh = sp_io::hashing::blake2_256(raw_pubkey);
	let id: U256 = pkh.into();
	let x = id ^ reward_info.seed;
	x <= reward_info.online_target
}
