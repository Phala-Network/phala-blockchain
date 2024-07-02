//! The pallet managing the wapod workers.

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::{mq, registry};
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
	use sp_runtime::{traits::Zero, SaturatedConversion};
	use wapod_eco_types::{
		bench_app::{MetricsToken, SignedMessage, SigningMessage},
		crypto::CryptoProvider,
		primitives::{BoundedString, BoundedVec},
		ticket::{Prices, SignedWorkerDescription, TicketDescription, WorkerDescription},
	};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type TicketId = u32;
	pub type ListId = u32;
	pub type Address = [u8; 32];

	#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub struct Fraction {
		pub numerator: u32,
		pub denominator: u32,
	}

	impl Fraction {
		fn saturating_mul_u64(&self, rhs: u64) -> u64 {
			let rhs = rhs as u128;
			let numerator = self.numerator as u128;
			let denominator = self.denominator as u128;
			let result = (numerator * rhs) / u128::max(denominator, 1);
			result.saturated_into()
		}
	}

	#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub struct BenchAppInfo {
		version: u32,
		ticket: TicketId,
		score_ratio: Fraction,
	}

	#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub struct WorkerSession {
		pub session_id: [u8; 32],
		pub last_nonce: [u8; 32],
		pub last_metrics_sn: u64,
	}

	struct SpCrypto;
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

	#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub enum WorkerSet {
		Any,
		WorkerList(ListId),
	}

	type AcountId32 = [u8; 32];

	#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub struct TicketInfo {
		pub system: bool,
		pub owner: AcountId32,
		pub workers: WorkerSet,
		pub app_address: [u8; 32],
		pub description: TicketDescription,
	}

	#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub struct WorkerListInfo {
		pub owner: AcountId32,
		pub prices: Prices,
		pub description: BoundedString<1024>,
	}

	#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub struct WorkingState {
		session_id: u32,
		unresponsive: bool,
		last_iterations: u64,
		last_update_time: i64,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type NextTicketId<T: Config> = StorageValue<_, TicketId, ValueQuery>;

	/// Active tickets.
	/// Each ticket holds a payment infomation to a target app address. Multiple ticket can pay for a same app at the sametime.
	#[pallet::storage]
	pub type Tickets<T: Config> = StorageMap<_, Twox64Concat, TicketId, TicketInfo>;

	#[pallet::storage]
	pub type NextWorkerListId<T: Config> = StorageValue<_, ListId, ValueQuery>;

	/// Worker lists.
	///
	/// A worker list is a collection of workers that with the same price.
	#[pallet::storage]
	pub type WorkerLists<T> = StorageMap<_, Twox64Concat, ListId, WorkerListInfo>;

	/// Concrete workers associated to worker list.
	#[pallet::storage]
	pub type WorkerListWorkers<T> =
		StorageDoubleMap<_, Twox64Concat, ListId, Twox64Concat, WorkerPublicKey, ()>;

	/// Information about workers.
	#[pallet::storage]
	pub type WorkerDescriptions<T> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, WorkerDescription>;

	/// V3 information about workers.
	#[pallet::storage]
	pub type WorkerSessions<T> = StorageMap<_, Twox64Concat, WorkerPublicKey, WorkerSession>;

	/// Working state of wapod workers. V2 compatible.
	#[pallet::storage]
	pub type WorkingWorkers<T> = StorageMap<_, Twox64Concat, WorkerPublicKey, WorkingState>;

	/// Allowed app addresses that used to benchmark workers.
	#[pallet::storage]
	pub type BenchmarkApps<T> = StorageMap<_, Twox64Concat, Address, BenchAppInfo>;

	/// Current recommended app address used to benchmark workers.
	#[pallet::storage]
	pub type RecommendedBenchmarkApp<T> = StorageValue<_, Address>;

	#[pallet::error]
	pub enum Error<T> {
		UnsupportedManifestVersion,
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
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		TicketCreated {
			id: TicketId,
		},
		TicketClosed {
			id: TicketId,
		},
		WorkerListCreated {
			id: ListId,
		},
		WorkersAddedToList {
			list_id: ListId,
			workers: Vec<WorkerPublicKey>,
		},
		WorkerRemovedFromList {
			list_id: ListId,
			worker: WorkerPublicKey,
		},
		WorkerDescriptionSet {
			worker: WorkerPublicKey,
		},
		BenchmarkAppAdded {
			address: Address,
		},
		RecommendedBenchmarkAppChanged {
			address: Address,
		},
	}

	const TICKET_ADDRESS_PREFIX: &[u8] = b"wapod/ticket/";

	fn ticket_account_address(ticket_id: u32) -> [u8; 32] {
		let mut address = [0u8; 32];
		let prefix = TICKET_ADDRESS_PREFIX;
		address[..prefix.len()].copy_from_slice(prefix);
		{
			let ticket_bytes = ticket_id.to_be_bytes();
			let offset = prefix.len();
			address[offset..offset + ticket_bytes.len()].copy_from_slice(&ticket_bytes);
		}
		address
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: mq::Config,
		T: registry::Config,
		T::AccountId: Into<AcountId32> + From<AcountId32>,
	{
		/// Create a new ticket
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn create_ticket(
			origin: OriginFor<T>,
			deposit: BalanceOf<T>,
			description: TicketDescription,
			worker_list: u32,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			ensure!(
				description.manifest.version == 1,
				Error::<T>::UnsupportedManifestVersion
			);
			ensure!(!description.manifest.resizable, Error::<T>::NotAllowed);
			ensure!(
				WorkerLists::<T>::contains_key(worker_list),
				Error::<T>::WorkerListNotFound
			);
			let id = Self::add_ticket(TicketInfo {
				system: false,
				owner: owner.clone().into(),
				workers: WorkerSet::WorkerList(worker_list),
				app_address: description.manifest.address(sp_core::hashing::blake2_256),
				description,
			});
			let ticket_account = ticket_account_address(id).into();
			<T as Config>::Currency::transfer(&owner, &ticket_account, deposit, KeepAlive)?;
			Ok(())
		}

		/// Create a new ticket
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn create_system_ticket(
			origin: OriginFor<T>,
			description: TicketDescription,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			ensure!(
				description.manifest.version == 1,
				Error::<T>::UnsupportedManifestVersion
			);
			Self::add_ticket(TicketInfo {
				system: true,
				owner: [0u8; 32].into(),
				workers: WorkerSet::Any,
				app_address: description.manifest.address(sp_core::hashing::blake2_256),
				description,
			});
			Ok(())
		}

		/// Close a ticket
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn close_ticket(origin: OriginFor<T>, ticket_id: TicketId) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let info = Tickets::<T>::get(ticket_id).ok_or(Error::<T>::TicketNotFound)?;
			ensure!(owner == info.owner.into(), Error::<T>::NotAllowed);

			// Refund the deposit
			let ticket_account = ticket_account_address(ticket_id).into();
			let deposit = <T as Config>::Currency::free_balance(&ticket_account);
			if !deposit.is_zero() {
				<T as Config>::Currency::transfer(&ticket_account, &owner, deposit, AllowDeath)?;
			}
			Tickets::<T>::remove(ticket_id);
			Self::deposit_event(Event::TicketClosed { id: ticket_id });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn submit_bench_message(
			origin: OriginFor<T>,
			message: SignedMessage,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			ensure!(
				message.verify::<SpCrypto>(),
				Error::<T>::SignatureVerificationFailed
			);
			let worker_pubkey = WorkerPublicKey(message.worker_pubkey);
			ensure!(
				registry::Pallet::<T>::worker_exsists(&worker_pubkey),
				Error::<T>::InvalidWorkerPubkey
			);
			let bench_app_info =
				BenchmarkApps::<T>::get(&message.app_address).ok_or(Error::<T>::InvalidBenchApp)?;
			match message.message {
				SigningMessage::BenchScore {
					gas_per_second,
					gas_consumed,
					timestamp_secs,
					matrics_token,
				} => {
					use frame_support::traits::UnixTime;

					let now = T::UnixTime::now().as_secs() as i64;
					let diff = (now - timestamp_secs as i64).abs();
					ensure!(diff < 600, Error::<T>::OutdatedMessage);

					Self::update_metrics_token(&worker_pubkey, &matrics_token)?;

					// Update the worker init score
					let gas_6secs = gas_per_second.saturating_mul(6);
					let score = bench_app_info.score_ratio.saturating_mul_u64(gas_6secs);
					let p_init = registry::Pallet::<T>::update_worker_score(&worker_pubkey, score);
					// If the worker is scheduled working by the chain, simulate a heartbeat message.
					if let Some(mut working_state) = WorkingWorkers::<T>::get(&worker_pubkey) {
						let delta_time = now - working_state.last_update_time;
						if delta_time <= 0 {
							return Ok(());
						}
						let iterations =
							bench_app_info.score_ratio.saturating_mul_u64(gas_consumed);
						let delta_iterations = iterations - working_state.last_iterations;
						let p_instant = delta_iterations / delta_time as u64 * 6;
						let p_max = p_init * 120 / 100;
						let p_instant = p_instant.min(p_max) as u32;
						let worker = MessageOrigin::Worker(worker_pubkey.into());

						// Minic the worker heartbeat message
						let worker_report = WorkingReportEvent::HeartbeatV3 {
							iterations,
							session_id: working_state.session_id,
							p_instant,
						};
						mq::Pallet::<T>::push_bound_message(worker, worker_report);

						working_state.last_iterations = iterations;
						working_state.last_update_time = now;
						WorkingWorkers::<T>::insert(&worker_pubkey, working_state);
					}
				}
			};
			Ok(())
		}

		/// Create a new worker list
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn create_worker_list(
			origin: OriginFor<T>,
			prices: Prices,
			description: BoundedString<1024>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::add_worker_list(WorkerListInfo {
				owner: owner.into(),
				prices,
				description,
			});
			Ok(())
		}

		/// Add a worker to a worker list
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn add_workers_to_list(
			origin: OriginFor<T>,
			list_id: ListId,
			workers: BoundedVec<WorkerPublicKey, 32>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let list_info = WorkerLists::<T>::get(list_id).ok_or(Error::<T>::WorkerListNotFound)?;
			ensure!(owner == list_info.owner.into(), Error::<T>::NotAllowed);
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

		/// Remove a worker from a worker list
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn remove_worker_from_list(
			origin: OriginFor<T>,
			list_id: ListId,
			worker: WorkerPublicKey,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let list_info = WorkerLists::<T>::get(list_id).ok_or(Error::<T>::WorkerListNotFound)?;
			ensure!(owner == list_info.owner.into(), Error::<T>::NotAllowed);
			WorkerListWorkers::<T>::remove(list_id, worker);
			Self::deposit_event(Event::WorkerRemovedFromList { list_id, worker });
			Ok(())
		}

		/// Set worker description
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn set_worker_description(
			origin: OriginFor<T>,
			signed_description: SignedWorkerDescription,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let worker_pubkey = WorkerPublicKey(signed_description.worker_pubkey);
			// Worker price can only be set once
			ensure!(
				!WorkerDescriptions::<T>::contains_key(&worker_pubkey),
				Error::<T>::NotAllowed
			);
			ensure!(
				registry::Pallet::<T>::worker_exsists(&worker_pubkey),
				Error::<T>::InvalidWorkerPubkey
			);
			ensure!(
				signed_description.verify::<SpCrypto>(),
				Error::<T>::SignatureVerificationFailed
			);
			WorkerDescriptions::<T>::insert(&worker_pubkey, signed_description.worker_description);
			Self::deposit_event(Event::WorkerDescriptionSet {
				worker: worker_pubkey,
			});
			Ok(())
		}

		/// Add a benchmark app (governance only)
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn add_benchmark_app(
			origin: OriginFor<T>,
			ticket: TicketId,
			version: u32,
			score_ratio: Fraction,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			let address = Tickets::<T>::get(ticket)
				.ok_or(Error::<T>::TicketNotFound)?
				.app_address;
			BenchmarkApps::<T>::insert(
				address,
				BenchAppInfo {
					version,
					ticket,
					score_ratio,
				},
			);
			Self::deposit_event(Event::BenchmarkAppAdded { address });
			Ok(())
		}

		/// Set recommended benchmark app (governance only)
		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn set_recommended_benchmark_app(
			origin: OriginFor<T>,
			address: Address,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			RecommendedBenchmarkApp::<T>::set(Some(address));
			Self::deposit_event(Event::RecommendedBenchmarkAppChanged { address });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		T: mq::Config,
		T::AccountId: Into<AcountId32> + From<AcountId32>,
	{
		fn add_ticket(info: TicketInfo) -> TicketId {
			let id = {
				let id = NextTicketId::<T>::get();
				NextTicketId::<T>::put(id.wrapping_add(1));
				id
			};
			Tickets::<T>::insert(id, info);
			Self::deposit_event(Event::TicketCreated { id });
			id
		}

		fn add_worker_list(info: WorkerListInfo) -> ListId {
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
			matrics_token: &MetricsToken,
		) -> DispatchResult {
			let Some(mut worker_session) = WorkerSessions::<T>::get(worker_pubkey) else {
				return Err(Error::<T>::WorkerNotFound.into());
			};
			ensure!(
				worker_session.last_metrics_sn < matrics_token.metrics_sn,
				Error::<T>::SessionMismatch
			);
			ensure!(
				worker_session.session_id == matrics_token.worker_session,
				Error::<T>::OutdatedMessage
			);
			worker_session.last_metrics_sn = matrics_token.metrics_sn;
			worker_session.last_nonce = matrics_token.nonce;
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
					WorkingWorkers::<T>::insert(
						&worker_pubkey,
						WorkingState {
							session_id,
							unresponsive: false,
							last_iterations: 0,
							last_update_time: 0,
						},
					);
				}
				WorkerEvent::Stopped => {
					WorkingWorkers::<T>::remove(&worker_pubkey);
				}
				WorkerEvent::EnterUnresponsive => {
					WorkingWorkers::<T>::mutate(&worker_pubkey, |state| {
						if let Some(state) = state {
							state.unresponsive = true;
						}
					});
				}
				WorkerEvent::ExitUnresponsive => {
					WorkingWorkers::<T>::mutate(&worker_pubkey, |state| {
						if let Some(state) = state {
							state.unresponsive = false;
						}
					});
				}
			}
			Ok(())
		}
	}
}
