#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use sp_core::U256;
use sp_std::prelude::*;
use sp_std::cmp;

use frame_support::{ensure, decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{Module as System, ensure_signed, ensure_root};

use alloc::vec::Vec;
use sp_runtime::{traits::AccountIdConversion, ModuleId, SaturatedConversion};
use frame_support::{
	traits::{Currency, ExistenceRequirement::AllowDeath, Randomness, UnixTime},
};
use codec::Decode;

// modules
mod hashing;

// types
extern crate phala_types as types;
use types::{
	WorkerMessagePayload, SignedWorkerMessage,
	TransferData, SignedDataType,
	WorkerInfo, StashInfo, PayoutPrefs, Score, PRuntimeInfo, MiningInfo, BlockRewardInfo
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::TEECurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
const PALLET_ID: ModuleId = ModuleId(*b"Phala!!!");
const RANDOMNESS_SUBJECT: &'static [u8] = b"PhalaPoW";
const BUILTIN_MACHINE_ID: &'static str = "BUILTIN";
const MINING_ROUND_PERIOD: u32 = 20;  // 2 min
const BLOCK_REWARD_TO_KEEP: u32 = 20;
// TODO: move to storage
const NUM_ONLINE_REWRARD_TARGET: u32 = 20;
const NUM_COMPUTE_REWRARD_TARGET: u32 = 10;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Randomness: Randomness<Self::Hash>;
	type TEECurrency: Currency<Self::AccountId>;
	type UnixTime: UnixTime;
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
		/// Map from stash account to worker info (indexed: MachineOwner)
		WorkerState get(fn worker_state): map hasher(blake2_128_concat) T::AccountId => WorkerInfo;
		/// Map from stash account to stash info (indexed: Stash)
		StashState get(fn stash_state): map hasher(blake2_128_concat) T::AccountId => StashInfo<T::AccountId>;
		/// Map from stash account to mining info (indexed: MiningDirty)
		MiningState get(fn mining_state): map hasher(blake2_128_concat) T::AccountId => MiningInfo<T::BlockNumber>;
		// Power and Fire
		/// Power measures the total computation power contributed to the network (PoC3 specific)
		Power get(fn power): map hasher(blake2_128_concat) T::AccountId => u32;
		/// Fire measures the total reward the miner can get from the network (PoC3 specific)
		Fire get(fn fire): map hasher(blake2_128_concat) T::AccountId => u32;
		/// Heartbeat counts (indexied: Total~, Max~, ActiveWorkers)
		Heartbeats get(fn heartbeats): map hasher(blake2_128_concat) T::AccountId => u32;

		// Indices
		/// Map from machine_id to stash
		MachineOwner get(fn machine_owner): map hasher(blake2_128_concat) Vec<u8> => T::AccountId;
		/// Map from controller to stash
		Stash get(fn stash): map hasher(blake2_128_concat) T::AccountId => T::AccountId;
		/// Sum of all online workers in this round
		OnlineWorkers get(fn online_workers): u32;
		/// Total Power points
		TotalPower get(fn total_power): u32;
		/// Total Fire points
		TotalFire get(fn total_fire): BalanceOf<T>;

		// Round management
		/// The current mining round id
		Round get(fn round): u64;
		/// Accounts with pending updates
		PendingUpdate get(fn pending_updates): Vec<T::AccountId>;

		// Probabilistic rewarding
		BlockRewardSeeds: map hasher(twox_64_concat) T::BlockNumber => BlockRewardInfo;

		// Key Management
		/// Map from contract id to contract public key (TODO: migrate to real contract key from
		/// worker identity key)
		ContractKey get(fn contract_key): map hasher(twox_64_concat) u32 => Vec<u8>;

		// Configurations
		/// MREnclave Whitelist
		MREnclaveWhitelist get(fn mr_enclave_whitelist): Vec<Vec<u8>>;
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
				let worker_info = WorkerInfo {
					machine_id,
					pubkey: pubkey.clone(),
					..Default::default()
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
		});
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId, Balance = BalanceOf<T> {
		// Debug events
		LogString(Vec<u8>),
		LogI32(i32),
		// Chain events
		CommandPushed(AccountId, u32, Vec<u8>, u64),
		TransferToTee(AccountId, Balance),
		TransferToChain(AccountId, Balance, u64),
		WorkerRegistered(AccountId, Vec<u8>, Vec<u8>),  // stash, identity_key, machine_id
		WorkerUnregistered(AccountId, Vec<u8>),
		Heartbeat(AccountId, u32),
		Offline(AccountId),
		Slash(AccountId, Balance, u32),
		GotCredits(AccountId, u32, u32),  // account, updated, delta
		MiningStateUpdated(Vec<AccountId>),
		WhitelistAdded(Vec<u8>),
		WhitelistRemoved(Vec<u8>),
		RewardSeed(BlockRewardInfo),
		WorkerMessageReceived(AccountId, Vec<u8>, u64),  // stash, identity_key, seq
		MinerStarted(u64, AccountId),  // round, stash
		MinerStopped(u64, AccountId),  // round, stash
		NewMiningRound(u64),  // round
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
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		fn on_finalize() {
			let now = System::<T>::block_number();
			Self::handle_block_reward(now);
			Self::handle_round_ends(now);
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
						target: who.clone(),
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
			// Add into the registry
			// TODO: Now we just force remove the worker and thus stop the mining. Should we just
			// update the worker info if there's an existing one?
			let perv_worker_info = Self::remove_machine_if_present(&machine_id);
			let last_updated = T::UnixTime::now().as_millis().saturated_into::<u64>();
			let pubkey = runtime_info.pubkey.to_vec();
			let score = Some(Score {
				overall_score: calc_overall_score(&runtime_info.features).map_err(|()| Error::<T>::InvalidInput)?,
				features: runtime_info.features
			});
			let worker_info = match perv_worker_info {
				Some(info) => WorkerInfo {
					pubkey: pubkey.clone(),
					last_updated,
					score,
					..info
				},
				None => WorkerInfo {
					machine_id: machine_id.clone(),
					pubkey: pubkey.clone(),
					last_updated,
					score,
					status: 0,
				},
			};
			WorkerState::<T>::insert(&stash, worker_info);
			MachineOwner::<T>::insert(&machine_id, &stash);
			WorkerIngress::<T>::insert(&stash, 0);
			Self::deposit_event(RawEvent::WorkerRegistered(stash, pubkey, machine_id));
			Ok(())
		}

		#[weight = 0]
		fn force_register_worker(origin, stash: T::AccountId, machine_id: Vec<u8>, pubkey: Vec<u8>) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			ensure!(StashState::<T>::contains_key(&stash), Error::<T>::StashNotFound);
			Self::remove_machine_if_present(&machine_id);
			let worker_info = WorkerInfo {
				machine_id: machine_id.clone(),
				pubkey: pubkey.clone(),
				last_updated: T::UnixTime::now().as_millis().saturated_into::<u64>(),
				status: 0,
				score: Some(Score {
					overall_score: 100,
					features: vec![1, 4]  // 1: one core, 4: the max feature level, score = 100
				}),
			};
			WorkerState::<T>::insert(&stash, worker_info);
			MachineOwner::<T>::insert(&machine_id, &stash);
			WorkerIngress::<T>::insert(&stash, 0);
			Self::deposit_event(RawEvent::WorkerRegistered(stash, pubkey, machine_id));
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
			WorkerState::<T>::mutate(&stash, |worker_info| worker_info.status = 1);
			if !MiningState::<T>::contains_key(&stash) {
				MiningState::<T>::insert(&stash, MiningInfo {
					is_mining: false,
					start_block: None,
				});
				// TODO: clean up when MiningState(stash) is not used
			}
			Self::mark_dirty(stash);
			Ok(())
		}

		#[weight = 0]
		fn stop_mining_intention(origin) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Stash::<T>::contains_key(&who), Error::<T>::ControllerNotFound);
			let stash = Stash::<T>::get(who);
			WorkerState::<T>::mutate(&stash, |worker_info| worker_info.status = 0);
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
		fn handle_worker_message(origin, msg: Vec<u8>) -> dispatch::DispatchResult {
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
			Self::verify_signature(&worker_info.pubkey, &signed)?;
			// Dispatch message
			match signed.data.payload {
				WorkerMessagePayload::Heartbeat { block_num, claim_online, claim_compute } => {
					let id_pubkey = &worker_info.pubkey;
					let score = match worker_info.score {
						Some(score) => score.overall_score,
						None => 0
					};
					Self::add_heartbeat(&who);	// TODO: necessary?
					Self::handle_claim_reward(&stash, claim_online, claim_compute, score);
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

		// TODO: remove this
		#[weight = 0]
		fn dbg_mark_violation(origin, stash: T::AccountId) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			ensure!(MiningState::<T>::contains_key(&stash), Error::<T>::StashNotFound);
			// 0. clean
			let mut mining_info = MiningState::<T>::get(&stash);
			if mining_info.start_block == None {
				return Ok(());
			}
			let now = System::<T>::block_number();
			Self::clean_account(&stash, mining_info.start_block.unwrap(), now);
			// 1. disable miner
			let mut worker_info = WorkerState::<T>::get(&stash);
			worker_info.status = 0;
			WorkerState::<T>::insert(&stash, worker_info);
			// 2. force stop
			mining_info.is_mining = false;
			mining_info.start_block = None;
			MiningState::<T>::insert(&stash, mining_info);
			// 3. TODO: add slash
			Self::deposit_event(RawEvent::Slash(stash.clone(), 0u32.into(), 0u32));
			// 4. Create events
			Self::mark_dirty(stash.clone());
			Self::deposit_event(RawEvent::MiningStateUpdated(vec![stash]));
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
	pub fn verify_signature(serialized_pk: &Vec<u8>, data: &impl SignedDataType<Vec<u8>>) -> dispatch::DispatchResult {
		use sp_std::convert::TryFrom;
		ensure!(serialized_pk.len() == 33, Error::<T>::InvalidPubKey);
		let pubkey = sp_core::ecdsa::Public::try_from(serialized_pk.as_slice())
			.map_err(|_| Error::<T>::InvalidPubKey)?;
		let raw_sig = data.signature();
		ensure!(raw_sig.len() == 65, Error::<T>::InvalidSignatureBadLen);
		let sig = sp_core::ecdsa::Signature::try_from(raw_sig.as_slice())
			.map_err(|_| Error::<T>::InvalidSignature)?;
		let data = data.raw_data();

		ensure!(sp_io::crypto::ecdsa_verify(&sig, &data, &pubkey), Error::<T>::FailedToVerify);
		Ok(())
	}

	/// Try to remove a registered worker from the registry by its `machine_id` identity if
	/// presents, keeping the stash untouched
	fn remove_machine_if_present(machine_id: &Vec<u8>) -> Option<WorkerInfo> {
		if !MachineOwner::<T>::contains_key(machine_id) {
			return None;
		}
		let stash = MachineOwner::<T>::take(machine_id);
		let worker_info = WorkerState::<T>::take(&stash);
		Self::deposit_event(RawEvent::WorkerUnregistered(stash, machine_id.clone()));
		Some(worker_info)
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

	// XXX: Remove manual account clean
	fn clean_account(account: &T::AccountId, start: T::BlockNumber, now: T::BlockNumber) {
		if start >= now {
			return;
		}
		let blocks = now - start;
		let worker_info = WorkerState::<T>::get(account);
		let score = match worker_info.score {
			Some(score) => score.overall_score,
			None => 1  // TODO: change to zero
		};
		let points: u32 = score * blocks.saturated_into::<u32>();
		// Add credits
		let fire = Fire::<T>::get(account);
		Fire::<T>::insert(account, fire + points);
		TotalPower::mutate(|x| *x += points);
		Self::deposit_event(RawEvent::GotCredits(account.clone(), fire + points, points));
	}

	fn add_heartbeat(account: &T::AccountId) {
		let heartbeats = Heartbeats::<T>::get(account);
		Heartbeats::<T>::insert(account, heartbeats + 1);
	}

	fn clear_heartbeats() {
		Heartbeats::<T>::remove_all();
	}

	fn extend_mrenclave(mr_enclave: &[u8], mr_signer: &[u8], isv_prod_id: &[u8], isv_svn: &[u8]) -> Vec<u8> {
		let mut t_mrenclave = Vec::new();
		t_mrenclave.extend_from_slice(mr_enclave);
		t_mrenclave.extend_from_slice(isv_prod_id);
		t_mrenclave.extend_from_slice(isv_svn);
		t_mrenclave.extend_from_slice(mr_signer);
		t_mrenclave
	}

	fn add_mrenclave_to_whitelist(mr_enclave: &[u8], mr_signer: &[u8], isv_prod_id: &[u8], isv_svn: &[u8]) -> dispatch::DispatchResult {
		let mut whitelist = MREnclaveWhitelist::get();
		let white_mrenclave = Self::extend_mrenclave(mr_enclave, mr_signer, isv_prod_id, isv_svn);
		ensure!(!whitelist.contains(&white_mrenclave), Error::<T>::MREnclaveAlreadyExist);
		whitelist.push(white_mrenclave.clone());
		MREnclaveWhitelist::put(whitelist);
		Self::deposit_event(RawEvent::WhitelistAdded(white_mrenclave));
		Ok(())
	}

	fn remove_mrenclave_from_whitelist_by_raw_data(mr_enclave: &[u8], mr_signer: &[u8], isv_prod_id: &[u8], isv_svn: &[u8]) -> dispatch::DispatchResult {
		let mut whitelist = MREnclaveWhitelist::get();
		let t_mrenclave = Self::extend_mrenclave(mr_enclave, mr_signer, isv_prod_id, isv_svn);
		ensure!(whitelist.contains(&t_mrenclave), Error::<T>::MREnclaveNotFound);
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

	fn handle_round_ends(now: T::BlockNumber) {
		let now_u32: u32 = now.saturated_into();
		if now_u32 % MINING_ROUND_PERIOD != MINING_ROUND_PERIOD - 1 {
			return;
		}
		Self::clear_heartbeats();
		let new_round = Round::get() + 1;

		// Process the pending update miner accoutns
		let mut delta = 0i32;
		let dirty_accounts = PendingUpdate::<T>::get();
		for account in dirty_accounts.iter() {
			let mut updated = false;
			let worker_info = WorkerState::<T>::get(&account);
			let mut mining_info = MiningState::<T>::get(&account);
			let intention = worker_info.status == 1;
			if mining_info.is_mining != intention {
				// TODO: check enough stake, etc
				mining_info.is_mining = intention;
				if intention {
					// Start from the next block
					mining_info.start_block = Some(now + 1u32.into());
					delta += 1;
					Self::deposit_event(RawEvent::MinerStarted(new_round, account.clone()));
				} else {
					mining_info.start_block = None;
					delta -= 1;
					Self::deposit_event(RawEvent::MinerStopped(new_round, account.clone()));
				}
				updated = true;
			}
			// TODO: slash
			if updated {
				MiningState::<T>::insert(&account, mining_info);
			}
		}
		OnlineWorkers::mutate(|n| *n = ((*n as i32) + delta) as u32);

		// dispatch tasks
		//	 TODO: randomly dispatch tasks and rewards

		// Start new round
		Self::clear_dirty();
		Round::put(new_round);
		Self::deposit_event(RawEvent::NewMiningRound(new_round));
	}

	fn handle_block_reward(now: T::BlockNumber) {
		// Remove the expired reward from the storage
		if now > BLOCK_REWARD_TO_KEEP.into() {
			BlockRewardSeeds::<T>::remove(now - BLOCK_REWARD_TO_KEEP.into());
		}
		// Generate the seed and targets
		let seed_hash = T::Randomness::random(RANDOMNESS_SUBJECT);
		let seed: U256 = AsRef::<[u8]>::as_ref(&seed_hash).into();
		let online = OnlineWorkers::get();
		let seed_info = BlockRewardInfo {
			seed,
			online_target: {
				if online == 0 {
					U256::zero()
				} else {
					// XXX: Clap the frequency to some upper limit
					u256_target(cmp::min(NUM_ONLINE_REWRARD_TARGET, online), online)
				}
			},
			compute_target: U256::zero(),
		};
		// Save
		BlockRewardSeeds::<T>::insert(now, &seed_info);
		Self::deposit_event(RawEvent::RewardSeed(seed_info));
	}

	fn handle_claim_reward(stash: &T::AccountId, claim_online: bool, _claim_compute: bool, score: u32) {
		// Check is mining
		let mining_info = MiningState::<T>::get(stash);
		if !mining_info.is_mining {
			return;
		}

		// XXX: check latency

		if claim_online {
			// XXX: give out credits
			let fire = 10000;
			Power::<T>::mutate(stash, |x| *x += score);
			Fire::<T>::mutate(stash, |x| *x += fire);
		}

		// TODO: do we need to check xor threshold?
		// TODO: Check is_compute
	}
}

fn calc_overall_score(features: &Vec<u32>) -> Result<u32, ()> {
	if features.len() != 2 {
		return Err(())
	}
	let core = features[0];
	let feature_level = features[1];
	Ok(core * (feature_level * 10 + 60))
}

fn u256_target(m: u32, n: u32) -> U256 {
	// m of n (MAX * (n / m))
	if m > n || n == 0 {
		panic!("Invalid parameter");
	}
	U256::MAX / n * m
}
