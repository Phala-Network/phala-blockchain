//! The pallet managing the wapod workers.
//!
//! This pallet is responsible for managing worker descriptions, sessions, and tickets.
//! It also handles worker lists, benchmarking, and settlement of tickets.

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	// MARK: - Imports
	use crate::{mq, registry, PhalaConfig};
	use alloc::collections::BTreeMap;
	use alloc::vec::Vec;
	use alloc::boxed::Box;
	use frame_support::{
		dispatch::DispatchResult,
		ensure,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::*, StorageVersion},
	};
	use frame_system::pallet_prelude::*;
	use phala_types::{
		messaging::{DecodedMessage, MessageOrigin, SystemEvent, WorkerEvent, WorkingReportEvent},
		WorkerPublicKey,
	};
	use scale_info::TypeInfo;
	use sp_runtime::{traits::Zero, SaturatedConversion, Saturating};
	use wapod_types::{
		bench_app::{BenchScore, SignedMessage, SigningMessage},
		crypto::{verify::Verifiable, CryptoProvider},
		metrics::{ClaimMap, MetricsToken, SignedAppsMetrics, VersionedAppsMetrics},
		primitives::{BoundedString, BoundedVec},
		session::{SessionUpdate, SignedSessionUpdate},
		ticket::{Balance, Prices, SignedWorkerDescription, WorkerDescription},
	};

	// MARK: - Types
	type BalanceOf<T> =
		<<T as PhalaConfig>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type TicketId = u64;
	pub type ListId = u64;
	/// Address type used for apps and workers.
	pub type Address = [u8; 32];

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, Copy, PartialEq, Eq, MaxEncodedLen)]
	pub struct Fraction {
		pub numerator: u32,
		pub denominator: u32,
	}

	impl Fraction {
		/// Performs a saturating multiplication of the fraction with a u64 value.
		fn saturating_mul_u64(&self, rhs: u64) -> u64 {
			let rhs = rhs as u128;
			let numerator = self.numerator as u128;
			let denominator = self.denominator as u128;
			let result = (numerator * rhs) / u128::max(denominator, 1);
			result.saturated_into()
		}
	}

	/// Information about a benchmark app.
	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	pub struct BenchAppInfo {
		/// The ticket id used to deploy the benchmark app.
		ticket: TicketId,
		/// The ratio of mapping gas to score.
		score_ratio: Fraction,
	}

	/// Represents a worker's session information. When a worker restarts, it creates a new session.
	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	pub struct WorkerSession<AccountId> {
		/// The session id of the worker.
		pub session_id: [u8; 32],
		/// The nonce that last used in metrics report.
		pub last_nonce: [u8; 32],
		/// The last metrics sequence number.
		pub last_metrics_sn: u64,
		/// The account id that receives the ticket reward.
		pub reward_receiver: AccountId,
	}

	/// Cryptographic provider for signature verification.
	pub struct SpCrypto;
	impl CryptoProvider for SpCrypto {
		fn sr25519_verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
			let Ok(public_key) = public_key.try_into() else {
				return false;
			};
			let Ok(signature) = signature.try_into() else {
				return false;
			};
			sp_io::crypto::sr25519_verify(&signature, message, &public_key)
		}
		fn keccak_256(data: &[u8]) -> [u8; 32] {
			sp_io::hashing::keccak_256(data)
		}
		fn blake2b_256(data: &[u8]) -> [u8; 32] {
			sp_io::hashing::blake2_256(data)
		}
	}

	/// Defines the set of workers that a ticket can be deployed to.
	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	pub enum WorkerSet {
		/// The ticket can be deployed to any worker.
		Any,
		/// The ticket can be deployed to workers in the specified list.
		WorkerList(ListId),
	}

	/// Information about a ticket.
	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	pub struct TicketInfo<AccountId> {
		/// If true, the ticket is a system (benchmark) ticket.
		pub system: bool,
		/// The account id of the ticket owner.
		/// None if the ticket is a system ticket.
		pub owner: Option<AccountId>,
		/// The set of workers that the ticket can be deployed to.
		pub workers: WorkerSet,
		/// The address of the target app.
		pub address: Address,
		/// The resource prices of the ticket.
		pub prices: Prices,
		/// The IPFS CID of the target app's manifest.
		pub manifest_cid: BoundedString<128>,
	}

	/// Information about a worker list.
	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	pub struct WorkerListInfo<AccountId> {
		/// The account id of the list owner.
		pub owner: AccountId,
		/// The resource prices of the workers in the list.
		pub prices: Prices,
		/// The description of the list.
		pub description: BoundedString<1024>,
	}

	/// Settlement information for a ticket and worker pair.
	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq, MaxEncodedLen, Default)]
	pub struct SettlementInfo<Balance> {
		/// The current session id of the App.
		pub current_session_id: [u8; 32],
		/// The total amount paid in the current App session.
		/// If the ticket is created after the worker's session, this value will be initialized
		/// to the cost calculated from the first metrics report but without actual payment.
		pub current_session_paid: Balance,
		/// The total amount paid to the worker.
		pub total_paid: Balance,
	}

	/// Represents the computation state of a worker. V2 compatible.
	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	pub struct ComputationState {
		/// The id of the current computation session.
		session_id: u32,
		/// Whether the worker is unresponsive.
		unresponsive: bool,
		/// The number of iterations in the last heartbeat.
		last_iterations: u64,
		/// The time of the last heartbeat.
		last_update_time: i64,
	}

	// MARK: - Pallet config
	#[pallet::config]
	pub trait Config: frame_system::Config + PhalaConfig {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Crypto: CryptoProvider;
	}

	// MARK: - Pallet storage

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// The next ticket ID to use.
	#[pallet::storage]
	pub type NextTicketId<T: Config> = StorageValue<_, TicketId, ValueQuery>;

	/// Active tickets.
	/// Each ticket holds payment information for a target app address.
	/// Multiple tickets can pay for the same app at the same time.
	#[pallet::storage]
	pub type Tickets<T: Config> = StorageMap<_, Twox64Concat, TicketId, TicketInfo<T::AccountId>>;

	/// Settlement information for each (ticket, worker) pair.
	#[pallet::storage]
	pub type TicketSettlementInfo<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		TicketId,
		Twox64Concat,
		WorkerPublicKey,
		SettlementInfo<BalanceOf<T>>,
		ValueQuery,
	>;

	/// The next worker list ID to use.
	#[pallet::storage]
	pub type NextWorkerListId<T> = StorageValue<_, ListId, ValueQuery>;

	/// Worker lists.
	///
	/// A worker list is a collection of workers that with the same price.
	#[pallet::storage]
	pub type WorkerLists<T: Config> =
		StorageMap<_, Twox64Concat, ListId, WorkerListInfo<T::AccountId>>;

	/// Workers associated to worker list.
	#[pallet::storage]
	pub type WorkerListWorkers<T> =
		StorageDoubleMap<_, Twox64Concat, ListId, Twox64Concat, WorkerPublicKey, ()>;

	/// Information about workers.
	#[pallet::storage]
	pub type WorkerDescriptions<T> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, WorkerDescription>;

	/// V3 information about workers.
	#[pallet::storage]
	pub type WorkerSessions<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, WorkerSession<T::AccountId>>;

	/// Working state of wapod workers. V2 compatible.
	#[pallet::storage]
	pub type ComputationWorkers<T> = StorageMap<_, Twox64Concat, WorkerPublicKey, ComputationState>;

	/// Allowed app addresses that used to benchmark workers.
	#[pallet::storage]
	pub type BenchmarkApps<T> = StorageMap<_, Twox64Concat, Address, BenchAppInfo>;

	/// Current recommended app address used to benchmark workers.
	#[pallet::storage]
	pub type RecommendedBenchmarkApp<T> = StorageValue<_, Address>;

	// MARK: - Pallet error

	/// Errors that can occur in this pallet.
	#[pallet::error]
	pub enum Error<T> {
		NotAllowed,
		WorkerNotFound,
		WorkerListNotFound,
		TicketNotFound,
		SignatureVerificationFailed,
		InvalidWorkerPubkey,
		InvalidBenchApp,
		OutdatedMessage,
		InvalidMessageSender,
		PriceMismatch,
		SessionMismatch,
		InvalidRewardReceiver,
		InvalidParameter,
	}

	// MARK: - Pallet events

	/// Events emitted by this pallet.
	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new ticket was created.
		TicketCreated { id: TicketId },
		/// A ticket was closed.
		TicketClosed { id: TicketId },
		/// A new worker list was created.
		WorkerListCreated { id: ListId },
		/// Some workers were added to a list.
		WorkersAddedToList {
			list_id: ListId,
			workers: Vec<WorkerPublicKey>,
		},
		/// A worker was removed from a list.
		WorkerRemovedFromList {
			list_id: ListId,
			worker: WorkerPublicKey,
		},
		/// A worker's description was set.
		WorkerDescriptionSet { worker: WorkerPublicKey },
		/// A benchmark app was added.
		BenchmarkAppAdded { address: Address },
		/// The recommended benchmark app was changed.
		RecommendedBenchmarkAppChanged { address: Address },
		/// A ticket was settled.
		Settled {
			ticket_id: TicketId,
			worker: WorkerPublicKey,
			paid: BalanceOf<T>,
			session_cost: BalanceOf<T>,
			paid_to: T::AccountId,
		},
		/// A settlement failed.
		SettleFailed {
			ticket_id: TicketId,
			worker: WorkerPublicKey,
			paid: BalanceOf<T>,
			session_cost: BalanceOf<T>,
		},
		/// A worker's session was updated.
		WorkerSessionUpdated {
			worker: WorkerPublicKey,
			session: [u8; 32],
		},
		/// A simulated heartbeat was emitted (V3).
		HeartbeatV3 {
			worker: WorkerPublicKey,
			/// The computation session id.
			session_id: u32,
			/// The v2 iterations converted from gas consumed.
			iterations: u64,
			/// The p_instant is estimated by the pallet, might not be the same as the GK state.
			p_instant: u32,
		},
	}

	fn ticket_account_address<T>(ticket_id: TicketId) -> T
	where
		T: Encode + Decode,
	{
		wapod_types::ticket::ticket_account_address(ticket_id, crate::hashing::blake2_256)
	}

	// MARK: - Pallet calls
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: mq::Config,
		T: registry::Config,
		BalanceOf<T>: From<Balance>,
	{
		// MARK: - Worker management

		/// Sets the description and price for a worker.
		///
		/// This function allows setting the description for a worker, which can only be done once.
		/// The description must be signed by the worker via its RPC.
		///
		/// # Parameters
		/// - `signed_description`: A signed description of the worker.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn worker_description_set(
			origin: OriginFor<T>,
			signed_description: SignedWorkerDescription,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let worker_pubkey = WorkerPublicKey(signed_description.worker_pubkey);
			// Worker price can only be set once
			ensure!(
				!WorkerDescriptions::<T>::contains_key(worker_pubkey),
				Error::<T>::NotAllowed
			);
			ensure!(
				registry::Pallet::<T>::worker_exsists(&worker_pubkey),
				Error::<T>::InvalidWorkerPubkey
			);
			ensure!(
				signed_description.verify::<T::Crypto>(),
				Error::<T>::SignatureVerificationFailed
			);
			WorkerDescriptions::<T>::insert(worker_pubkey, signed_description.worker_description);
			Self::deposit_event(Event::WorkerDescriptionSet {
				worker: worker_pubkey,
			});
			Ok(())
		}

		/// Updates the session for a worker.
		///
		/// This function should be called when a worker restarts or resets to update its session with a new session ID.
		/// The new session must be initiated with the last nonce on-chain to ensure the session is not lanched before the last metrics report.
		///
		/// # Parameters
		/// - `update`: A signed session update containing the new session information.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn worker_session_update(
			origin: OriginFor<T>,
			update: SignedSessionUpdate,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			ensure!(
				update.verify::<T::Crypto>(),
				Error::<T>::SignatureVerificationFailed
			);
			let worker_pubkey = WorkerPublicKey(update.public_key);
			ensure!(
				registry::Pallet::<T>::worker_exsists(&worker_pubkey),
				Error::<T>::InvalidWorkerPubkey
			);
			let update = update.update;
			if let Some(session) = WorkerSessions::<T>::get(worker_pubkey) {
				let computed_session =
					SessionUpdate::session_from_seed::<T::Crypto>(update.seed, &session.last_nonce);
				ensure!(
					computed_session == update.session,
					Error::<T>::OutdatedMessage
				);
			};
			let Ok(reward_receiver) = Decode::decode(&mut &update.reward_receiver[..]) else {
				return Err(Error::<T>::InvalidRewardReceiver.into());
			};
			WorkerSessions::<T>::insert(
				worker_pubkey,
				WorkerSession {
					session_id: update.session,
					last_nonce: update.seed,
					last_metrics_sn: 0,
					reward_receiver,
				},
			);
			Self::deposit_event(Event::WorkerSessionUpdated {
				worker: worker_pubkey,
				session: update.session,
			});
			return Ok(());
		}

		/// Creates a new worker list.
		///
		/// This function creates a new worker list with specified description, prices, and initial workers.
		///
		/// # Parameters
		/// - `description`: A string describing the worker list.
		/// - `prices`: The resource prices for each worker in the list.
		/// - `init_workers`: An optional list of initial workers to add to the list.
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn worker_list_create(
			origin: OriginFor<T>,
			description: BoundedString<1024>,
			prices: Prices,
			init_workers: BoundedVec<WorkerPublicKey, 32>,
		) -> DispatchResult {
			let owner = ensure_signed(origin.clone())?;
			let id = Self::add_worker_list(WorkerListInfo {
				owner,
				prices,
				description,
			});
			if !init_workers.is_empty() {
				Self::worker_list_add_workers(origin, id, init_workers)?;
			}
			Ok(())
		}

		/// Adds workers to an existing worker list.
		///
		/// # Parameters
		/// - `list_id`: The ID of the worker list to add workers to.
		/// - `workers`: A vector of worker public keys to add to the list.
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn worker_list_add_workers(
			origin: OriginFor<T>,
			list_id: ListId,
			workers: BoundedVec<WorkerPublicKey, 32>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let list_info = WorkerLists::<T>::get(list_id).ok_or(Error::<T>::WorkerListNotFound)?;
			ensure!(owner == list_info.owner, Error::<T>::NotAllowed);
			for worker in workers.iter() {
				let Some(worker_info) = WorkerDescriptions::<T>::get(worker) else {
					return Err(Error::<T>::WorkerNotFound.into());
				};
				ensure!(
					worker_info.prices == Default::default()
						|| worker_info.prices == list_info.prices,
					Error::<T>::PriceMismatch
				);
				WorkerListWorkers::<T>::insert(list_id, worker, ());
			}
			Self::deposit_event(Event::WorkersAddedToList {
				list_id,
				workers: workers.into(),
			});
			Ok(())
		}

		/// Removes a worker from a worker list.
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn worker_list_remove_worker(
			origin: OriginFor<T>,
			list_id: ListId,
			worker: WorkerPublicKey,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let list_info = WorkerLists::<T>::get(list_id).ok_or(Error::<T>::WorkerListNotFound)?;
			ensure!(owner == list_info.owner, Error::<T>::NotAllowed);
			WorkerListWorkers::<T>::remove(list_id, worker);
			Self::deposit_event(Event::WorkerRemovedFromList { list_id, worker });
			Ok(())
		}

		// MARK: - Tickets

		/// Creates a new ticket for an application.
		///
		/// The address must match the address of the target application. Otherwise, this call will not fail but the ticket will not be settled,
		/// and the worker will likely reject the ticket.
		///
		/// # Parameters
		/// - `deposit`: The amount to be deposited into the ticket account.
		/// - `address`: The address of the target application.
		/// - `manifest_cid`: The IPFS CID of the application manifest.
		/// - `worker_list`: The ID of the worker list allowed to settle this ticket.
		/// - `prices`:
		/// 	The resource prices for the ticket. This will be merged with the prices of the worker list while settling the ticket.
		/// 	Leave None for the field to use the prices of the worker list.
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn ticket_create(
			origin: OriginFor<T>,
			deposit: BalanceOf<T>,
			address: Address,
			manifest_cid: BoundedString<128>,
			worker_list: ListId,
			prices: Box<Prices>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let Some(list_info) = WorkerLists::<T>::get(worker_list) else {
				return Err(Error::<T>::WorkerListNotFound.into());
			};
			let id = Self::add_ticket(TicketInfo {
				system: false,
				owner: Some(owner.clone()),
				workers: WorkerSet::WorkerList(worker_list),
				address,
				manifest_cid,
				prices: prices.merge(&list_info.prices),
			});
			let ticket_account = ticket_account_address(id);
			<T as PhalaConfig>::Currency::transfer(&owner, &ticket_account, deposit, KeepAlive)?;
			Ok(())
		}

		/// Closes an ticket.
		///
		/// # Parameters
		/// - `ticket_id`: The ID of the ticket to be closed.
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn ticket_close(origin: OriginFor<T>, ticket_id: TicketId) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let info = Tickets::<T>::get(ticket_id).ok_or(Error::<T>::TicketNotFound)?;
			ensure!(Some(&owner) == info.owner.as_ref(), Error::<T>::NotAllowed);

			// Refund the deposit
			let ticket_account = ticket_account_address(ticket_id);
			let deposit = <T as PhalaConfig>::Currency::free_balance(&ticket_account);
			if !deposit.is_zero() {
				<T as PhalaConfig>::Currency::transfer(
					&ticket_account,
					&owner,
					deposit,
					AllowDeath,
				)?;
			}
			Tickets::<T>::remove(ticket_id);
			// TODO: remove the remaining entries
			_ = TicketSettlementInfo::<T>::clear_prefix(ticket_id, 64, None);
			Self::deposit_event(Event::TicketClosed { id: ticket_id });
			Ok(())
		}

		/// Submits application metrics for settlement.
		///
		/// This function allows workers to submit signed metrics for applications, which are used to settle tickets.
		///
		/// # Parameters
		/// - `message`: A signed message containing the application metrics.
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn ticket_settle(
			origin: OriginFor<T>,
			message: SignedAppsMetrics,
			claim_map: ClaimMap,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let worker_pubkey = {
				// signature verification
				ensure!(
					message.verify::<T::Crypto>(),
					Error::<T>::SignatureVerificationFailed
				);
				let worker_pubkey = WorkerPublicKey(message.worker_pubkey);
				ensure!(
					registry::Pallet::<T>::worker_exsists(&worker_pubkey),
					Error::<T>::InvalidWorkerPubkey
				);
				worker_pubkey
			};

			let VersionedAppsMetrics::V0(all_metrics) = message.metrics;

			let worker_info = {
				// ensure the session matches
				let MetricsToken { session, sn, nonce } = all_metrics.token;
				let Some(mut session_info) = WorkerSessions::<T>::get(worker_pubkey) else {
					return Err(Error::<T>::WorkerNotFound.into());
				};
				ensure!(
					session_info.session_id == session,
					Error::<T>::SessionMismatch
				);
				ensure!(
					session_info.last_metrics_sn < sn,
					Error::<T>::OutdatedMessage
				);
				session_info.last_nonce = nonce;
				session_info.last_metrics_sn = sn;
				WorkerSessions::<T>::insert(worker_pubkey, &session_info);
				session_info
			};

			{
				// update metrics for each ticket

				// Up to 64 entries
				let claim_map: BTreeMap<_, _> = claim_map.into_iter().collect();

				// Up to 64 entries
				for metrics in all_metrics.apps {
					let Some(ticket_ids) = claim_map.get(&metrics.address) else {
						continue;
					};
					// Up to 5 entries
					for ticket_id in ticket_ids.iter() {
						let ticket =
							Tickets::<T>::get(ticket_id).ok_or(Error::<T>::TicketNotFound)?;

						if ticket.prices.is_empty() {
							continue;
						}

						ensure!(ticket.address == metrics.address, Error::<T>::NotAllowed);
						match ticket.workers {
							WorkerSet::Any => (),
							WorkerSet::WorkerList(list_id) => {
								ensure!(
									WorkerListWorkers::<T>::contains_key(list_id, worker_pubkey),
									Error::<T>::NotAllowed
								);
							}
						}

						let mut settlement =
							TicketSettlementInfo::<T>::get(ticket_id, worker_pubkey);

						// Pay out and update the settlement infomation
						let cost = ticket.prices.cost_of(&metrics).into();
						if settlement.current_session_id != metrics.session {
							settlement.current_session_id = metrics.session;
							settlement.current_session_paid = cost;
						}
						let pay_out = cost.saturating_sub(settlement.current_session_paid);
						if !pay_out.is_zero() {
							let ticket_account = ticket_account_address(*ticket_id);
							let transfer_result = <T as PhalaConfig>::Currency::transfer(
								&ticket_account,
								&worker_info.reward_receiver,
								pay_out,
								KeepAlive,
							);
							match transfer_result {
								Ok(_) => {
									settlement.current_session_paid = cost;
									settlement.total_paid =
										settlement.total_paid.saturating_add(pay_out);
									Self::deposit_event(Event::Settled {
										ticket_id: *ticket_id,
										worker: worker_pubkey,
										paid: pay_out,
										session_cost: cost,
										paid_to: worker_info.reward_receiver.clone(),
									});
								}
								Err(_) => {
									Self::deposit_event(Event::SettleFailed {
										ticket_id: *ticket_id,
										worker: worker_pubkey,
										paid: pay_out,
										session_cost: cost,
									});
								}
							}
						}
						TicketSettlementInfo::<T>::insert(ticket_id, worker_pubkey, settlement);
					}
				}
			}
			Ok(())
		}

		// MARK: - Benchmark

		/// Submits a benchmark score for a worker.
		///
		/// This function allows submitting a benchmark score for a worker, which can be used to update the worker's initial score or simulate a V2 heartbeat.
		///
		/// # Parameters
		/// - `as_init_score`: If true, the score will update the worker's initial score; otherwise, it will simulate a V2 heartbeat.
		/// - `message`: A signed message containing the benchmark score.
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn benchmark_score_submit(
			origin: OriginFor<T>,
			as_init_score: bool,
			message: SignedMessage,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			ensure!(
				message.verify::<T::Crypto>(),
				Error::<T>::SignatureVerificationFailed
			);
			let worker_pubkey = WorkerPublicKey(message.worker_pubkey);
			ensure!(
				registry::Pallet::<T>::worker_exsists(&worker_pubkey),
				Error::<T>::InvalidWorkerPubkey
			);
			let bench_app_info =
				BenchmarkApps::<T>::get(message.app_address).ok_or(Error::<T>::InvalidBenchApp)?;
			match message.message {
				SigningMessage::BenchScore(BenchScore {
					gas_per_second,
					gas_consumed,
					timestamp_secs,
					metrics_token,
				}) => {
					use frame_support::traits::UnixTime;

					let now = T::UnixTime::now().as_secs() as i64;
					let diff = (now - timestamp_secs as i64).abs();
					ensure!(diff < 600, Error::<T>::OutdatedMessage);

					Self::update_metrics_token(&worker_pubkey, &metrics_token)?;

					// Update the worker init score
					if as_init_score {
						let iterations_per_sec = bench_app_info
							.score_ratio
							.saturating_mul_u64(gas_per_second);
						let score = iterations_per_sec.saturating_mul(6);
						registry::Pallet::<T>::update_worker_init_score(&worker_pubkey, score);
					} else if let Some(mut computation_state) =
						ComputationWorkers::<T>::get(worker_pubkey)
					{
						// If the worker is scheduled computing by the chain, simulate a heartbeat message.
						let p_init =
							registry::Pallet::<T>::worker_init_score(&worker_pubkey).unwrap_or(0);
						let iterations =
							bench_app_info.score_ratio.saturating_mul_u64(gas_consumed);
						let delta_time = now - computation_state.last_update_time;
						if delta_time <= 0 {
							computation_state.last_iterations = iterations;
							ComputationWorkers::<T>::insert(worker_pubkey, computation_state);
							return Ok(());
						}
						let delta_iterations = iterations - computation_state.last_iterations;
						let p_instant = delta_iterations / delta_time as u64 * 6;
						let p_max = p_init * 120 / 100;
						let p_instant = p_instant.min(p_max) as u32;
						let worker = MessageOrigin::Worker(worker_pubkey);

						// Minic the worker heartbeat message
						let worker_report = WorkingReportEvent::HeartbeatV3 {
							iterations,
							session_id: computation_state.session_id,
							p_instant,
						};
						mq::Pallet::<T>::push_bound_message(worker, worker_report);
						Self::deposit_event(Event::HeartbeatV3 {
							worker: worker_pubkey,
							session_id: computation_state.session_id,
							iterations,
							p_instant,
						});

						computation_state.last_iterations = iterations;
						computation_state.last_update_time = now;
						ComputationWorkers::<T>::insert(worker_pubkey, computation_state);
					}
				}
			};
			Ok(())
		}

		/// Adds a new benchmark application (governance only).
		///
		/// This function allows the governance to add a new benchmark application to the whitelist.
		///
		/// # Parameters
		/// - `address`: The address of the benchmark application.
		/// - `manifest_cid`: The IPFS CID of the application manifest.
		/// - `score_ratio`: The ratio for mapping gas to score.
		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn benchmark_app_add(
			origin: OriginFor<T>,
			address: Address,
			manifest_cid: BoundedString<128>,
			score_ratio: Fraction,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			ensure!(score_ratio.denominator > 0, Error::<T>::InvalidParameter);

			let ticket = Self::add_ticket(TicketInfo {
				system: true,
				owner: None,
				workers: WorkerSet::Any,
				address,
				manifest_cid,
				prices: Default::default(),
			});
			BenchmarkApps::<T>::insert(
				address,
				BenchAppInfo {
					ticket,
					score_ratio,
				},
			);
			Self::deposit_event(Event::BenchmarkAppAdded { address });
			Ok(())
		}

		/// Sets the recommended benchmark application (governance only).
		///
		/// # Parameters
		/// - `address`: The address of the recommended benchmark application.
		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn benchmark_app_set_recommended(
			origin: OriginFor<T>,
			address: Address,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			RecommendedBenchmarkApp::<T>::set(Some(address));
			Self::deposit_event(Event::RecommendedBenchmarkAppChanged { address });
			Ok(())
		}

		/// Combine benchmark_app_add and benchmark_app_set_recommended
		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn benchmark_app_set(
			origin: OriginFor<T>,
			address: Address,
			manifest_cid: BoundedString<128>,
			score_ratio: Fraction,
		) -> DispatchResult {
			Self::benchmark_app_add(origin.clone(), address, manifest_cid, score_ratio)?;
			Self::benchmark_app_set_recommended(origin, address)
		}

		/// Remove a benchmark application from the whitelist (governance only).
		#[pallet::call_index(12)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn benchmark_app_remove(origin: OriginFor<T>, address: Address) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			BenchmarkApps::<T>::remove(address);
			Ok(())
		}
	}

	// MARK: Support functions

	impl<T: Config> Pallet<T>
	where
		T: mq::Config,
	{
		fn add_ticket(info: TicketInfo<T::AccountId>) -> TicketId {
			let id = {
				let id = NextTicketId::<T>::get();
				NextTicketId::<T>::put(id.wrapping_add(1));
				id
			};
			Tickets::<T>::insert(id, info);
			Self::deposit_event(Event::TicketCreated { id });
			id
		}

		fn add_worker_list(info: WorkerListInfo<T::AccountId>) -> ListId {
			let id = {
				let id = NextWorkerListId::<T>::get();
				NextWorkerListId::<T>::put(id.wrapping_add(1));
				id
			};
			WorkerLists::<T>::insert(id, info);
			Self::deposit_event(Event::WorkerListCreated { id });
			id
		}

		fn update_metrics_token(
			worker_pubkey: &WorkerPublicKey,
			metrics_token: &MetricsToken,
		) -> DispatchResult {
			let Some(mut worker_session) = WorkerSessions::<T>::get(worker_pubkey) else {
				return Err(Error::<T>::WorkerNotFound.into());
			};
			ensure!(
				worker_session.last_metrics_sn < metrics_token.sn,
				Error::<T>::OutdatedMessage
			);
			ensure!(
				worker_session.session_id == metrics_token.session,
				Error::<T>::SessionMismatch
			);
			worker_session.last_metrics_sn = metrics_token.sn;
			worker_session.last_nonce = metrics_token.nonce;
			WorkerSessions::<T>::insert(worker_pubkey, worker_session);
			Ok(())
		}

		pub fn on_worker_event_received(message: DecodedMessage<SystemEvent>) -> DispatchResult {
			ensure!(message.sender.is_pallet(), Error::<T>::InvalidMessageSender);
			let SystemEvent::WorkerEvent(event) = message.payload else {
				return Ok(());
			};
			let worker_pubkey = event.pubkey;
			match event.event {
				WorkerEvent::Registered(_) => (),
				WorkerEvent::BenchStart { duration: _ } => (),
				WorkerEvent::BenchScore(_) => (),
				WorkerEvent::Started { session_id, .. } => {
					ComputationWorkers::<T>::insert(
						worker_pubkey,
						ComputationState {
							session_id,
							unresponsive: false,
							last_iterations: 0,
							last_update_time: 0,
						},
					);
				}
				WorkerEvent::Stopped => {
					ComputationWorkers::<T>::remove(worker_pubkey);
				}
				WorkerEvent::EnterUnresponsive => {
					ComputationWorkers::<T>::mutate(worker_pubkey, |state| {
						if let Some(state) = state {
							state.unresponsive = true;
						}
					});
				}
				WorkerEvent::ExitUnresponsive => {
					ComputationWorkers::<T>::mutate(worker_pubkey, |state| {
						if let Some(state) = state {
							state.unresponsive = false;
						}
					});
				}
			}
			Ok(())
		}
	}

	// MARK: Runtime api
	impl<T: Config> Pallet<T>
	where
		T: mq::Config,
	{
		pub fn balance_of_ticket(ticket_id: TicketId) -> BalanceOf<T> {
			let account = ticket_account_address(ticket_id);
			<T as PhalaConfig>::Currency::free_balance(&account)
		}
	}

	sp_api::decl_runtime_apis! {
		/// The API of the wapod workers pallet.
		pub trait WapodWorkersApi<Balance> where Balance: codec::Codec {
			/// Get balance of given ticket.
			fn balance_of_ticket(ticket_id: TicketId) -> Balance;
		}
	}

	// MARK: - Tests

	#[cfg(test)]
	mod tests {
		use super::*;
		use crate::mock::{
			ecdh_pubkey, elapse_seconds, new_test_ext, set_block_1, take_events, worker_pubkey,
			MockCrypto, PhalaRegistry, PhalaWapodWorkers, RuntimeEvent as TestEvent,
			RuntimeOrigin as Origin, Test,
		};
		// Pallets
		use frame_support::{assert_err, assert_ok};
		use phala_types::messaging::WorkerEventWithKey;
		use wapod_types::{
			bench_app::{BenchScore, SignedMessage, SigningMessage},
			metrics::{
				AppMetrics, AppsMetrics, MetricsToken, SignedAppsMetrics, VersionedAppsMetrics,
			},
			primitives::BoundedVec,
			session::{SessionUpdate, SignedSessionUpdate},
			ticket::{Balance, Prices, SignedWorkerDescription, WorkerDescription},
		};

		fn setup_workers(n: u8) {
			for i in 1..=n {
				let worker = worker_pubkey(i);
				assert_ok!(PhalaRegistry::force_register_worker(
					Origin::root(),
					worker,
					ecdh_pubkey(1),
					Some(1)
				));
			}
		}

		struct Events(Vec<TestEvent>);

		impl Events {
			fn take() -> Self {
				Self(take_events())
			}
			fn has(&self, event: impl Into<TestEvent>) -> bool {
				let event = event.into();
				self.0.iter().any(|x| x == &event)
			}
			fn find_settlement_for(
				&self,
				ticket: TicketId,
				worker: WorkerPublicKey,
			) -> Option<Balance> {
				for event in self.0.iter() {
					if let TestEvent::PhalaWapodWorkers(Event::<Test>::Settled {
						ticket_id,
						worker: worker_id,
						paid,
						..
					}) = event
					{
						if ticket_id == &ticket && worker_id == &worker {
							return Some(*paid);
						}
					}
				}
				None
			}
			fn find_heartbeat_v3(&self, worker: WorkerPublicKey) -> Option<(u64, u32)> {
				for event in self.0.iter() {
					if let TestEvent::PhalaWapodWorkers(Event::<Test>::HeartbeatV3 {
						worker: worker_id,
						session_id: _,
						iterations,
						p_instant,
					}) = event
					{
						if worker_id == &worker {
							return Some((*iterations, *p_instant));
						}
					}
				}
				None
			}
		}

		fn create_worker_list() -> ListId {
			create_worker_list_with_price(Prices::default())
		}

		fn create_worker_list_with_price(prices: Prices) -> ListId {
			let result = PhalaWapodWorkers::worker_list_create(
				Origin::signed(1),
				"my list".to_string().into(),
				prices,
				vec![].into(),
			);
			assert_ok!(result);

			let mut list_id = None;
			for event in take_events() {
				if let TestEvent::PhalaWapodWorkers(Event::<Test>::WorkerListCreated { id }) = event
				{
					list_id = Some(id);
				}
			}
			list_id.expect("No WorkerListCreated event emitted")
		}

		fn setup_worker_list(n: u8, prices: Prices) -> ListId {
			for id in 1..=n {
				let result = PhalaWapodWorkers::worker_description_set(
					Origin::signed(1),
					SignedWorkerDescription {
						worker_pubkey: worker_pubkey(id).0,
						worker_description: WorkerDescription {
							prices: prices.clone(),
							description: "my worker".to_string().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);
				assert_ok!(result);
			}

			let list_id = create_worker_list_with_price(prices);
			let result = PhalaWapodWorkers::worker_list_add_workers(
				Origin::signed(1),
				list_id,
				(1..=n).map(worker_pubkey).collect::<Vec<_>>().into(),
			);
			assert_ok!(result);
			for id in 1..=n {
				assert!(WorkerListWorkers::<Test>::contains_key(
					list_id,
					worker_pubkey(id)
				))
			}
			list_id
		}

		#[test]
		fn can_not_set_worker_description_without_register() {
			new_test_ext().execute_with(|| {
				set_block_1();

				let result = PhalaWapodWorkers::worker_description_set(
					Origin::signed(1),
					SignedWorkerDescription {
						worker_pubkey: worker_pubkey(0).0,
						worker_description: WorkerDescription {
							prices: Prices::default(),
							description: "my worker".to_string().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);

				assert_err!(result, Error::<Test>::InvalidWorkerPubkey);
			});
		}

		#[test]
		fn can_set_worker_description() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let result = PhalaWapodWorkers::worker_description_set(
					Origin::signed(1),
					SignedWorkerDescription {
						worker_pubkey: worker_pubkey(1).0,
						worker_description: WorkerDescription {
							prices: Prices::default(),
							description: "my worker".to_string().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);

				assert_ok!(result);

				let events = Events::take();
				assert!(events.has(Event::WorkerDescriptionSet {
					worker: worker_pubkey(1)
				}));
			});
		}

		#[test]
		fn can_not_update_session_without_register() {
			new_test_ext().execute_with(|| {
				set_block_1();

				let result = PhalaWapodWorkers::worker_session_update(
					Origin::signed(1),
					SignedSessionUpdate {
						public_key: worker_pubkey(0).0,
						update: SessionUpdate {
							session: [1; 32],
							seed: [1; 32],
							reward_receiver: [1; 32].to_vec().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);

				assert_err!(result, Error::<Test>::InvalidWorkerPubkey);
			});
		}

		#[test]
		fn can_update_session_first_time() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let result = PhalaWapodWorkers::worker_session_update(
					Origin::signed(1),
					SignedSessionUpdate {
						public_key: worker_pubkey(1).0,
						update: SessionUpdate {
							session: [1; 32],
							seed: [1; 32],
							reward_receiver: [1; 32].to_vec().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);

				assert_ok!(result);

				let events = Events::take();
				assert!(events.has(Event::WorkerSessionUpdated {
					worker: worker_pubkey(1),
					session: [1; 32],
				}));
			});
		}

		#[test]
		fn can_not_update_session_second_time_with_incorrect_nonce() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let result = PhalaWapodWorkers::worker_session_update(
					Origin::signed(1),
					SignedSessionUpdate {
						public_key: worker_pubkey(1).0,
						update: SessionUpdate {
							session: [1; 32],
							seed: [1; 32],
							reward_receiver: [1; 32].to_vec().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);

				assert_ok!(result);

				let result = PhalaWapodWorkers::worker_session_update(
					Origin::signed(1),
					SignedSessionUpdate {
						public_key: worker_pubkey(1).0,
						update: SessionUpdate {
							session: [1; 32],
							seed: [2; 32],
							reward_receiver: [1; 32].to_vec().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);

				assert_err!(result, Error::<Test>::OutdatedMessage);

				let last_nonce = WorkerSessions::<Test>::get(worker_pubkey(1))
					.unwrap()
					.last_nonce;
				let seed = [2; 32];

				// Can update session with correct nonce.
				let result = PhalaWapodWorkers::worker_session_update(
					Origin::signed(1),
					SignedSessionUpdate {
						public_key: worker_pubkey(1).0,
						update: SessionUpdate {
							session: SessionUpdate::session_from_seed::<MockCrypto>(
								seed,
								&last_nonce,
							),
							seed,
							reward_receiver: [1; 32].to_vec().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);
				assert_ok!(result);
			});
		}

		#[test]
		fn can_not_add_worker_without_price_to_list() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let list_id = create_worker_list();

				let result = PhalaWapodWorkers::worker_list_add_workers(
					Origin::signed(1),
					list_id,
					vec![worker_pubkey(1)].into(),
				);

				assert_err!(result, Error::<Test>::WorkerNotFound);
			});
		}

		#[test]
		fn can_not_add_worker_to_list_when_price_mismatch() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let result = PhalaWapodWorkers::worker_description_set(
					Origin::signed(1),
					SignedWorkerDescription {
						worker_pubkey: worker_pubkey(1).0,
						worker_description: WorkerDescription {
							prices: Prices {
								tip_price: Some(1),
								..Default::default()
							},
							description: "my worker".to_string().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);
				assert_ok!(result);

				let list_id = create_worker_list();

				let result = PhalaWapodWorkers::worker_list_add_workers(
					Origin::signed(1),
					list_id,
					vec![worker_pubkey(1)].into(),
				);

				assert_err!(result, Error::<Test>::PriceMismatch);
			});
		}

		#[test]
		fn can_add_worker_to_list() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let result = PhalaWapodWorkers::worker_description_set(
					Origin::signed(1),
					SignedWorkerDescription {
						worker_pubkey: worker_pubkey(1).0,
						worker_description: WorkerDescription {
							prices: Prices::default(),
							description: "my worker".to_string().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);
				assert_ok!(result);

				let list_id = create_worker_list();

				let result = PhalaWapodWorkers::worker_list_add_workers(
					Origin::signed(1),
					list_id,
					vec![worker_pubkey(1)].into(),
				);

				assert_ok!(result);

				let events = Events::take();
				assert!(events.has(Event::WorkersAddedToList {
					list_id,
					workers: vec![worker_pubkey(1)],
				}));
			});
		}

		#[test]
		fn can_remove_worker_from_list() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let result = PhalaWapodWorkers::worker_description_set(
					Origin::signed(1),
					SignedWorkerDescription {
						worker_pubkey: worker_pubkey(1).0,
						worker_description: WorkerDescription {
							prices: Prices::default(),
							description: "my worker".to_string().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				);
				assert_ok!(result);

				let list_id = create_worker_list();

				let result = PhalaWapodWorkers::worker_list_add_workers(
					Origin::signed(1),
					list_id,
					vec![worker_pubkey(1)].into(),
				);
				assert_ok!(result);

				let result = PhalaWapodWorkers::worker_list_remove_worker(
					Origin::signed(1),
					list_id,
					worker_pubkey(1),
				);
				assert_ok!(result);

				let events = Events::take();
				assert!(events.has(Event::WorkerRemovedFromList {
					list_id,
					worker: worker_pubkey(1)
				}));
			});
		}

		#[test]
		fn can_create_ticket() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let list_id = create_worker_list();

				let result = PhalaWapodWorkers::ticket_create(
					Origin::signed(1),
					0,
					[1; 32],
					"manifest".to_string().into(),
					list_id,
					Default::default(),
				);
				assert_ok!(result);

				let events = Events::take();
				assert!(events.has(Event::TicketCreated { id: 0 }));
			});
		}

		#[test]
		fn can_close_ticket() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let list_id = create_worker_list();

				let result = PhalaWapodWorkers::ticket_create(
					Origin::signed(1),
					0,
					[1; 32],
					"manifest".to_string().into(),
					list_id,
					Default::default(),
				);
				assert_ok!(result);

				let result = PhalaWapodWorkers::ticket_close(Origin::signed(1), 0);
				assert_ok!(result);

				let events = Events::take();
				assert!(events.has(Event::TicketClosed { id: 0 }));
			});
		}

		#[test]
		fn can_settle_ticket() {
			new_test_ext().execute_with(|| {
				set_block_1();

				setup_workers(1);

				let gas_price = 1;
				let tip_price = 2;
				let gas_consumed = 20;
				let tip = 10;
				let prices = Prices {
					gas_price: Some(gas_price),
					tip_price: Some(tip_price),
					..Default::default()
				};
				let reward = tip as u128 * tip_price + gas_consumed as u128 * gas_price;
				let reward_receiver = 42_u64;
				let app_address = [2; 32];
				let list_id = setup_worker_list(1, prices);
				let claim_map = BoundedVec::from(vec![(app_address, vec![list_id].into())]);

				assert_ok!(PhalaWapodWorkers::worker_session_update(
					Origin::signed(1),
					SignedSessionUpdate {
						public_key: worker_pubkey(1).0,
						update: SessionUpdate {
							session: [1; 32],
							seed: [1; 32],
							reward_receiver: reward_receiver.encode().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				));

				assert_ok!(PhalaWapodWorkers::ticket_create(
					Origin::signed(1),
					100,
					app_address,
					"manifest".to_string().into(),
					list_id,
					Default::default(),
				));

				assert!(WorkerListWorkers::<Test>::contains_key(
					list_id,
					worker_pubkey(1)
				));

				let result = PhalaWapodWorkers::ticket_settle(
					Origin::signed(1),
					SignedAppsMetrics {
						worker_pubkey: worker_pubkey(1).0,
						metrics: VersionedAppsMetrics::V0(AppsMetrics {
							token: MetricsToken {
								session: [1; 32],
								sn: 1,
								nonce: [1; 32],
							},
							apps: vec![AppMetrics {
								address: app_address,
								session: [1; 32],
								tip,
								gas_consumed,
								..Default::default()
							}]
							.into(),
						}),
						signature: b"valid".to_vec().into(),
					},
					claim_map.clone(),
				);
				assert_ok!(result);

				// No payment for the first commit
				let events = Events::take();
				let paid = events.find_settlement_for(0, worker_pubkey(1));
				assert_eq!(paid, None);

				let result = PhalaWapodWorkers::ticket_settle(
					Origin::signed(1),
					SignedAppsMetrics {
						worker_pubkey: worker_pubkey(1).0,
						metrics: VersionedAppsMetrics::V0(AppsMetrics {
							token: MetricsToken {
								session: [1; 32],
								sn: 2,
								nonce: [1; 32],
							},
							apps: vec![AppMetrics {
								address: app_address,
								session: [1; 32],
								tip: tip * 2,
								gas_consumed: gas_consumed * 2,
								..Default::default()
							}]
							.into(),
						}),
						signature: b"valid".to_vec().into(),
					},
					claim_map,
				);
				assert_ok!(result);
				// Should pay for the second commit
				let events = Events::take();
				let paid = events.find_settlement_for(0, worker_pubkey(1));
				assert_eq!(paid, Some(reward));
			});
		}

		#[test]
		fn can_submit_benchmark_score() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);

				let gas_per_second = 500;
				let timestamp_secs = 10;
				let gas_consumed = gas_per_second * timestamp_secs;
				let app_address = [2; 32];
				let receiver = 42u64;

				let metrics_token = MetricsToken {
					session: [1; 32],
					sn: 1,
					nonce: [1; 32],
				};

				// set up the benchmark app
				assert_ok!(PhalaWapodWorkers::benchmark_app_set(
					Origin::root(),
					app_address,
					"manifest".to_string().into(),
					Fraction {
						numerator: 1,
						denominator: 10,
					},
				));

				// set up worker session
				assert_ok!(PhalaWapodWorkers::worker_session_update(
					Origin::signed(1),
					SignedSessionUpdate {
						public_key: worker_pubkey(1).0,
						update: SessionUpdate {
							session: [1; 32],
							seed: [1; 32],
							reward_receiver: receiver.encode().into(),
						},
						signature: b"valid".to_vec().into(),
					},
				));

				let message = SignedMessage {
					worker_pubkey: worker_pubkey(1).0,
					app_address,
					signature: b"valid".to_vec().into(),
					message: SigningMessage::BenchScore(BenchScore {
						gas_per_second,
						gas_consumed,
						timestamp_secs,
						metrics_token,
					}),
				};

				let result = PhalaWapodWorkers::benchmark_score_submit(
					Origin::signed(1),
					true,
					message.clone(),
				);
				assert_ok!(result);
				let init_score = PhalaRegistry::worker_init_score(&worker_pubkey(1));
				assert_eq!(init_score, Some(300));

				take_events();

				// simulate a V2 heartbeat
				PhalaWapodWorkers::on_worker_event_received(DecodedMessage {
					sender: MessageOrigin::Pallet(vec![]),
					destination: vec![].into(),
					payload: SystemEvent::WorkerEvent(WorkerEventWithKey {
						pubkey: worker_pubkey(1),
						event: WorkerEvent::Started {
							init_p: 1000,
							init_v: 2000,
							session_id: 1,
						},
					}),
				})
				.expect("Failed to send mq message");

				let message = SignedMessage {
					worker_pubkey: worker_pubkey(1).0,
					app_address,
					signature: b"valid".to_vec().into(),
					message: SigningMessage::BenchScore(BenchScore {
						gas_per_second,
						gas_consumed: gas_consumed * 2,
						timestamp_secs: timestamp_secs * 2,
						metrics_token: MetricsToken {
							session: [1; 32],
							sn: 2,
							nonce: [1; 32],
						},
					}),
				};

				assert_ok!(PhalaWapodWorkers::benchmark_score_submit(
					Origin::signed(1),
					false,
					message
				));
				// No heartbeat the first time
				let events = Events::take();
				assert!(events.find_heartbeat_v3(worker_pubkey(1)).is_none());

				elapse_seconds(timestamp_secs);

				let message = SignedMessage {
					worker_pubkey: worker_pubkey(1).0,
					app_address,
					signature: b"valid".to_vec().into(),
					message: SigningMessage::BenchScore(BenchScore {
						gas_per_second,
						gas_consumed: gas_consumed * 3,
						timestamp_secs: timestamp_secs * 3,
						metrics_token: MetricsToken {
							session: [1; 32],
							sn: 3,
							nonce: [1; 32],
						},
					}),
				};
				assert_ok!(PhalaWapodWorkers::benchmark_score_submit(
					Origin::signed(1),
					false,
					message
				));
				let events = Events::take();
				let (iterations, p_instant) = events.find_heartbeat_v3(worker_pubkey(1)).unwrap();
				assert_eq!((iterations, p_instant), (1500, 300));
			});
		}
	}
}
