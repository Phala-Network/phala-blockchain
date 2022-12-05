//! Manages mining lifecycle, reward and slashes
#![allow(clippy::all)]

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::mq::{self, MessageOriginInfo};
	use crate::registry;
	use crate::base_pool;
	use crate::{BalanceOf, NegativeImbalanceOf, PhalaConfig};
	use frame_support::traits::WithdrawReasons;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		storage::{storage_prefix, unhashed, PrefixIterator, migration},
		traits::{
			Currency, ExistenceRequirement::KeepAlive, OnUnbalanced, Randomness, StorageVersion,
			UnixTime, ConstBool,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use phala_types::{
		messaging::{
			DecodedMessage, GatekeeperEvent, HeartbeatChallenge, MessageOrigin, SettleInfo,
			SystemEvent, TokenomicParameters as TokenomicParams, WorkerEvent,
			WorkingInfoUpdateEvent, WorkingReportEvent,
		},
		WorkerPublicKey,
	};
	use scale_info::TypeInfo;
	use sp_core::U256;
	use sp_runtime::{
		traits::{AccountIdConversion, One, Zero},
		SaturatedConversion,
	};
	use sp_std::cmp;

	use crate::balance_convert::FixedPointConvert;
	use codec::{Decode, Encode};
	use fixed::types::U64F64 as FixedPoint;
	use fixed_macro::types::U64F64 as fp;
	use fixed_sqrt::FixedSqrt;

	const DEFAULT_EXPECTED_HEARTBEAT_COUNT: u32 = 20;
	const COMPUTING_PALLETID: PalletId = PalletId(*b"phala/pp");

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub enum WorkerState {
		Ready,
		WorkerIdle,
		_Unused,
		WorkerUnresponsive,
		WorkerCoolingDown,
	}

	impl WorkerState {
		fn can_unbind(&self) -> bool {
			matches!(self, WorkerState::Ready | WorkerState::WorkerCoolingDown)
		}
		fn can_settle(&self) -> bool {
			// TODO(hangyin):
			//
			// We don't allow a settlement in WorkerCoolingDown or Ready. After a worker is stopped,
			// it's released immediately and the slash is pre-settled (to make sure the force
			// withdrawal can be processed correctly).
			//
			// We have to either figure out how to allow settlement in CoolingDown state, or
			// complete disable it as we do now. Note that when CoolingDown settle is not allowed,
			// we still have to make sure the slashed V is periodically updated on the blockchain.
			matches!(
				self,
				WorkerState::WorkerIdle | WorkerState::WorkerUnresponsive
			)
		}
		pub fn is_computing(&self) -> bool {
			matches!(
				self,
				WorkerState::WorkerIdle | WorkerState::WorkerUnresponsive
			)
		}
	}

	/// The benchmark information of a worker
	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct Benchmark {
		/// The initial performance score copied from the registry pallet
		p_init: u32,
		/// The instant performance score
		p_instant: u32,
		/// The latest benchmark iterations
		///
		/// Used to calculate `p_instant`.
		iterations: u64,
		/// The unix timestamp of the computing start time
		working_start_time: u64,
		/// The unix timestamp of block that triggers the last heartbeat challenge
		challenge_time_last: u64,
	}

	impl Benchmark {
		/// Records the latest benchmark status snapshot and updates `p_instant`
		///
		/// Note: `now` and `challenge_time` are in seconds.
		fn update(&mut self, now: u64, iterations: u64, challenge_time: u64) -> Result<(), ()> {
			// `now` must be larger than `challenge_time_last` because it's impossible to report
			// the heartbeat at the same block with the challenge.
			if now <= self.challenge_time_last {
				return Err(());
			}
			// Lower iteration indicates the worker has been restarted. This is acceptable, but we
			// have to reset the on-chain counter as well (causing a temporary zero p-instant).
			if iterations < self.iterations {
				self.iterations = iterations;
			}
			let delta_ts = now - self.challenge_time_last;
			let delta_iter = iterations - self.iterations;
			self.challenge_time_last = challenge_time;
			self.iterations = iterations;
			// Normalize the instant P value:
			// 1. Normalize to iterations in 6 sec
			// 2. Cap it to 120% `initial_score`
			let p_instant = (delta_iter * 6 / delta_ts) as u32;
			self.p_instant = p_instant.min(self.p_init * 12 / 10);
			Ok(())
		}
	}

	/// The state of a worker
	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct SessionInfo {
		/// The current state of the worker bounded with session
		pub state: WorkerState,
		/// The intiial V, in `U64F64` bits
		pub ve: u128,
		/// The last updated V, in `U64F64` bits
		pub v: u128,
		/// The unix timestamp of the last V update time
		v_updated_at: u64,
		/// Benchmark info
		benchmark: Benchmark,
		/// The unix timestamp of the cool down starting time
		///
		/// The value is meaningless if the state is not in
		/// [`WorkerCoolingDown`](WorkerState::WorkerCoolingDown) state.
		cool_down_start: u64,
		/// The statistics of the current computing session
		stats: SessionStats,
	}

	impl SessionInfo {
		/// Calculates the final final returned and slashed stake
		fn calc_final_stake<Balance>(&self, orig_stake: Balance) -> (Balance, Balance)
		where
			Balance: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert,
		{
			// TODO(hangyin): deal with slash later
			// For simplicity the slash is disabled until StakePool v2 is implemented.
			(orig_stake, Zero::zero())
		}
	}

	pub trait OnReward {
		fn on_reward(_settle: &[SettleInfo]) {}
	}

	pub trait OnUnbound {
		/// Called when a worker was unbound from a session.
		///
		/// `force` is set if the unbinding caused an unexpected session shutdown.
		fn on_unbound(_worker: &WorkerPublicKey, _force: bool) {}
	}

	pub trait OnStopped<Balance> {
		/// Called with a session is stopped and can already calculate the final slash and stake.
		///
		/// It guarantees the number will be the same as the return value of `reclaim()`
		fn on_stopped(_worker: &WorkerPublicKey, _orig_stake: Balance, _slashed: Balance) {}
	}

	/// The stats of a computing session
	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Default, RuntimeDebug)]
	pub struct SessionStats {
		/// The total received reward in this computing session, in `U32F32` bits
		total_reward: u128,
	}

	impl SessionStats {
		fn on_reward(&mut self, payout_bits: u128) {
			let payout: u128 = FixedPointConvert::from_bits(payout_bits);
			self.total_reward += payout;
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + PhalaConfig + mq::Config + registry::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type ExpectedBlockTimeSec: Get<u32>;
		type MinInitP: Get<u32>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type OnReward: OnReward;
		type OnUnbound: OnUnbound;
		type OnStopped: OnStopped<BalanceOf<Self>>;
		type OnTreasurySettled: OnUnbalanced<NegativeImbalanceOf<Self>>;
		// Let the StakePool to take over the slash events.

		/// The origin to update tokenomic.
		type UpdateTokenomicOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type ComputationMigrationAccountId: Get<Self::AccountId>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Tokenomic parameters used by Gatekeepers to compute the V promote.
	#[pallet::storage]
	pub type TokenomicParameters<T> = StorageValue<_, TokenomicParams>;

	/// The scheduled new tokenomic params to update at the end of this block.
	#[pallet::storage]
	pub type ScheduledTokenomicUpdate<T> = StorageValue<_, TokenomicParams>;

	/// Total online workers including WorkerIdle and WorkerUnresponsive workers.
	///
	/// Increased when a worker is turned to WorkerIdle; decreased when turned to CoolingDown.
	#[pallet::storage]
	#[pallet::getter(fn online_workers)]
	pub type OnlineWorkers<T> = StorageValue<_, u32, ValueQuery>;

	/// The expected heartbeat count at every block (default: 20)
	#[pallet::storage]
	pub type ExpectedHeartbeatCount<T> = StorageValue<_, u32>;

	/// Won't sent heartbeat challenges to the the worker if enabled.
	#[pallet::storage]
	pub type HeartbeatPaused<T> = StorageValue<_, bool, ValueQuery, ConstBool<true>>;

	/// The miner state.
	///
	/// The session state is created when a worker is bounded with a session, but it will be kept even
	/// if the worker is force unbound. A re-bind of a worker will reset the computing state.
	#[pallet::storage]
	#[pallet::getter(fn sessions)]
	pub type Sessions<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, SessionInfo>;

	/// The bound worker for a session account
	#[pallet::storage]
	pub type SessionBindings<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, WorkerPublicKey>;

	/// The bound worker account for a worker
	#[pallet::storage]
	pub type WorkerBindings<T: Config> = StorageMap<_, Twox64Concat, WorkerPublicKey, T::AccountId>;

	/// The cool down period (in sec)
	#[pallet::storage]
	#[pallet::getter(fn cool_down_period)]
	pub type CoolDownPeriod<T> = StorageValue<_, u64, ValueQuery>;

	/// The next id to assign to a computing session
	#[pallet::storage]
	pub type NextSessionId<T> = StorageValue<_, u32, ValueQuery>;

	/// The block number when the computing starts. Used to calculate halving.
	#[pallet::storage]
	pub type ComputingStartBlock<T: Config> = StorageValue<_, T::BlockNumber>;

	/// The interval of halving (75% decay) in block number.
	#[pallet::storage]
	pub type ComputingHalvingInterval<T: Config> = StorageValue<_, T::BlockNumber>;

	/// The stakes of session accounts.
	///
	/// Only presents for computing and cooling down sessions.
	#[pallet::storage]
	#[pallet::getter(fn stakes)]
	pub type Stakes<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Cool down expiration changed (in sec).
		///
		/// Indicates a change in [`CoolDownPeriod`].
		CoolDownExpirationChanged { period: u64 },
		/// A worker starts computing.
		///
		/// Affected states:
		/// - the worker info at [`Sessions`] is updated with `WorkerIdle` state
		/// - [`NextSessionId`] for the session is incremented
		/// - [`Stakes`] for the session is updated
		/// - [`OnlineWorkers`] is incremented
		WorkerStarted {
			session: T::AccountId,
			init_v: u128,
			init_p: u32,
		},
		/// Worker stops computing.
		///
		/// Affected states:
		/// - the worker info at [`Sessions`] is updated with `WorkerCoolingDown` state
		/// - [`OnlineWorkers`] is decremented
		WorkerStopped { session: T::AccountId },
		/// Worker is reclaimed, with its slash settled.
		WorkerReclaimed {
			session: T::AccountId,
			original_stake: BalanceOf<T>,
			slashed: BalanceOf<T>,
		},
		/// Worker & session are bounded.
		///
		/// Affected states:
		/// - [`SessionBindings`] for the session account is pointed to the worker
		/// - [`WorkerBindings`] for the worker is pointed to the session account
		/// - the worker info at [`Sessions`] is updated with `Ready` state
		SessionBound {
			session: T::AccountId,
			worker: WorkerPublicKey,
		},
		/// Worker & worker are unbound.
		///
		/// Affected states:
		/// - [`SessionBindings`] for the session account is removed
		/// - [`WorkerBindings`] for the worker is removed
		SessionUnbound {
			session: T::AccountId,
			worker: WorkerPublicKey,
		},
		/// Worker enters unresponsive state.
		///
		/// Affected states:
		/// - the worker info at [`Sessions`] is updated from `WorkerIdle` to `WorkerUnresponsive`
		WorkerEnterUnresponsive { session: T::AccountId },
		/// Worker returns to responsive state.
		///
		/// Affected states:
		/// - the worker info at [`Sessions`] is updated from `WorkerUnresponsive` to `WorkerIdle`
		WorkerExitUnresponsive { session: T::AccountId },
		/// Worker settled successfully.
		///
		/// It results in the v in [`Sessions`] being updated. It also indicates the downstream
		/// stake pool has received the computing reward (payout), and the treasury has received the
		/// tax.
		SessionSettled {
			session: T::AccountId,
			v_bits: u128,
			payout_bits: u128,
		},
		/// Some internal error happened when settling a worker's ledger.
		InternalErrorWorkerSettleFailed { worker: WorkerPublicKey },
		/// Block subsidy halved by 25%.
		///
		/// This event will be followed by a [`TokenomicParametersChanged`](#variant.TokenomicParametersChanged)
		/// event indicating the change of the block subsidy budget in the parameter.
		SubsidyBudgetHalved,
		/// Some internal error happened when trying to halve the subsidy
		InternalErrorWrongHalvingConfigured,
		/// Tokenomic parameter changed.
		///
		/// Affected states:
		/// - [`TokenomicParameters`] is updated.
		TokenomicParametersChanged,
		/// A session settlement was dropped because the on-chain version is more up-to-date.
		///
		/// This is a temporary walk-around of the computing staking design. Will be fixed by
		/// StakePool v2.
		SessionSettlementDropped {
			session: T::AccountId,
			v: u128,
			payout: u128,
		},
		/// Benchmark Updated
		BenchmarkUpdated {
			session: T::AccountId,
			p_instant: u32,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The transaction is sent by an unauthorized sender
		BadSender,
		/// Deprecated.
		_InvalidMessage,
		/// The worker is not registered in the registry.
		WorkerNotRegistered,
		/// Deprecated
		_GatekeeperNotRegistered,
		/// Not permitted because the session is already bound with another worker.
		DuplicateBoundSession,
		/// There's no benchmark result on the blockchain.
		BenchmarkMissing,
		/// session not found.
		SessionNotFound,
		/// Not permitted because the session is not bound with a worker.
		SessionNotBound,
		/// Worker is not in `Ready` state to proceed.
		WorkerNotReady,
		/// Worker is not in `Computation` state to stop computing.
		WorkerNotComputing,
		/// Not permitted because the worker is not bound with a worker account.
		WorkerNotBound,
		/// Cannot reclaim the worker because it's still in cooldown period.
		CoolDownNotReady,
		/// Cannot start computing because there's too little stake.
		InsufficientStake,
		/// Cannot start computing because there's too much stake (exceeds Vmax).
		TooMuchStake,
		/// Internal error. The tokenomic parameter is not set.
		InternalErrorBadTokenomicParameters,
		/// Not permitted because the worker is already bound with another session account.
		DuplicateBoundWorker,
		/// Indicating the initial benchmark score is too low to start computing.
		BenchmarkTooLow,
		/// Internal error. A worker should never start with existing stake in the storage.
		InternalErrorCannotStartWithExistingStake,
		/// Migration root not authorized
		NotMigrationRoot,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert,
	{
		/// Sets the cool down expiration time in seconds.
		///
		/// Can only be called by root.
		#[pallet::weight(0)]
		pub fn set_cool_down_expiration(origin: OriginFor<T>, period: u64) -> DispatchResult {
			ensure_root(origin)?;

			CoolDownPeriod::<T>::put(period);
			Self::deposit_event(Event::<T>::CoolDownExpirationChanged { period });
			Ok(())
		}

		/// Unbinds a worker from the given session (or pool sub-account).
		///
		/// It will trigger a force stop of computing if the worker is still in computing state. Anyone
		/// can call it.
		#[pallet::weight(0)]
		pub fn unbind(origin: OriginFor<T>, session: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let pubkey = Self::ensure_session_bound(&session)?;
			let worker =
				registry::Workers::<T>::get(&pubkey).ok_or(Error::<T>::WorkerNotRegistered)?;
			ensure!(worker.operator == Some(who), Error::<T>::BadSender);
			// Always notify the subscriber. Please note that even if the worker is not computing, we
			// still have to notify the subscriber that an unbinding operation has just happened.
			Self::unbind_session(&session, true)
		}

		/// Triggers a force heartbeat request to all workers by sending a MAX pow target
		///
		/// Only for integration test.
		#[pallet::weight(1)]
		pub fn force_heartbeat(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			Self::push_message(SystemEvent::HeartbeatChallenge(HeartbeatChallenge {
				seed: U256::zero(),
				online_target: U256::MAX,
			}));
			Ok(())
		}

		/// Start computing
		///
		/// Only for integration test.
		#[pallet::weight(1)]
		pub fn force_start_computing(
			origin: OriginFor<T>,
			session: T::AccountId,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::start_computing(session, stake)?;
			Ok(())
		}

		/// Stop computing
		///
		/// Only for integration test.
		#[pallet::weight(1)]
		pub fn force_stop_computing(origin: OriginFor<T>, session: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Self::stop_computing(session)?;
			Ok(())
		}

		/// Updates the tokenomic parameters at the end of this block.
		///
		/// Can only be called by the tokenomic admin.
		#[pallet::weight(1)]
		pub fn update_tokenomic(
			origin: OriginFor<T>,
			new_params: TokenomicParams,
		) -> DispatchResult {
			T::UpdateTokenomicOrigin::ensure_origin(origin)?;
			ScheduledTokenomicUpdate::<T>::put(new_params);
			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn migrate_miners(origin: OriginFor<T>, max_iterations: u32) -> DispatchResult {
			let mut who = ensure_signed(origin)?;
			ensure!(
				who == T::ComputationMigrationAccountId::get(),
				Error::<T>::NotMigrationRoot
			);
			let mining_prefix = storage_prefix(b"PhalaMining", b"Miners");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"Sessions");

			Self::move_prefix(
				mining_prefix.as_slice(),
				computation_prefix.as_slice(),
				max_iterations,
			);

			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn migrate_miner_bindings(origin: OriginFor<T>, max_iterations: u32) -> DispatchResult {
			let mut who = ensure_signed(origin)?;
			ensure!(
				who == T::ComputationMigrationAccountId::get(),
				Error::<T>::NotMigrationRoot
			);
			let mining_prefix = storage_prefix(b"PhalaMining", b"MinerBindings");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"SessionBindings");

			Self::move_prefix(
				mining_prefix.as_slice(),
				computation_prefix.as_slice(),
				max_iterations,
			);

			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn migrate_worker_bindings(
			origin: OriginFor<T>,
			max_iterations: u32,
		) -> DispatchResult {
			let mut who = ensure_signed(origin)?;
			ensure!(
				who == T::ComputationMigrationAccountId::get(),
				Error::<T>::NotMigrationRoot
			);
			let mining_prefix = storage_prefix(b"PhalaMining", b"WorkerBindings");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"WorkerBindings");

			Self::move_prefix(
				mining_prefix.as_slice(),
				computation_prefix.as_slice(),
				max_iterations,
			);

			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn migrate_stakes(origin: OriginFor<T>, max_iterations: u32) -> DispatchResult {
			let mut who = ensure_signed(origin)?;
			ensure!(
				who == T::ComputationMigrationAccountId::get(),
				Error::<T>::NotMigrationRoot
			);
			let mining_prefix = storage_prefix(b"PhalaMining", b"Stakes");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"Stakes");

			Self::move_prefix(
				mining_prefix.as_slice(),
				computation_prefix.as_slice(),
				max_iterations,
			);

			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn migrate_storage_values(origin: OriginFor<T>) -> DispatchResult {
			let mut who = ensure_signed(origin)?;
			ensure!(
				who == T::ComputationMigrationAccountId::get(),
				Error::<T>::NotMigrationRoot
			);
			let mining_prefix = storage_prefix(b"PhalaMining", b"TokenomicParameters");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"TokenomicParameters");
			Self::move_value(mining_prefix.as_slice(), computation_prefix.as_slice());

			let mining_prefix = storage_prefix(b"PhalaMining", b"ScheduledTokenomicUpdate");
			let computation_prefix =
				storage_prefix(b"PhalaComputation", b"ScheduledTokenomicUpdate");
			Self::move_value(mining_prefix.as_slice(), computation_prefix.as_slice());

			let mining_prefix = storage_prefix(b"PhalaMining", b"OnlineMiners");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"OnlineWorkers");
			Self::move_value(mining_prefix.as_slice(), computation_prefix.as_slice());

			let mining_prefix = storage_prefix(b"PhalaMining", b"ExpectedHeartbeatCount");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"ExpectedHeartbeatCount");
			Self::move_value(mining_prefix.as_slice(), computation_prefix.as_slice());

			let mining_prefix = storage_prefix(b"PhalaMining", b"CoolDownPeriod");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"CoolDownPeriod");
			Self::move_value(mining_prefix.as_slice(), computation_prefix.as_slice());

			let mining_prefix = storage_prefix(b"PhalaMining", b"NextSessionId");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"NextSessionId");
			Self::move_value(mining_prefix.as_slice(), computation_prefix.as_slice());

			let mining_prefix = storage_prefix(b"PhalaMining", b"MiningStartBlock");
			let computation_prefix = storage_prefix(b"PhalaComputation", b"ComputingStartBlock");
			Self::move_value(mining_prefix.as_slice(), computation_prefix.as_slice());

			let mining_prefix = storage_prefix(b"PhalaMining", b"MiningHalvingInterval");
			let computation_prefix =
				storage_prefix(b"PhalaComputation", b"ComputingHalvingInterval");
			Self::move_value(mining_prefix.as_slice(), computation_prefix.as_slice());
			Ok(())
		}
		
		/// Pause or resume the heartbeat challenges.
		///
		/// This API is introduced to pause the computing rewards for a period while we upgrading
		/// StakePool v2. Worker's rewards would still be accumulated in GK in the pausing period
		/// but never be paid out until the heartbeat is resumed.
		///
		/// Can only be called by root.
		#[pallet::weight(1)]
		pub fn set_heartbeat_paused(origin: OriginFor<T>, paused: bool) -> DispatchResult {
			ensure_root(origin)?;
			HeartbeatPaused::<T>::put(paused);
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert,
	{
		fn on_finalize(n: T::BlockNumber) {
			Self::heartbeat_challenge();
			// Apply tokenomic update if possible
			if let Some(tokenomic) = ScheduledTokenomicUpdate::<T>::take() {
				Self::update_tokenomic_parameters(tokenomic);
			}
			// Apply subsidy
			if let Some(interval) = ComputingHalvingInterval::<T>::get() {
				let block_elapsed = n - ComputingStartBlock::<T>::get().unwrap_or_default();
				// Halve when it reaches the last block in an interval
				if interval > Zero::zero() && block_elapsed % interval == interval - One::one() {
					let r = Self::trigger_subsidy_halving();
					if r.is_err() {
						Self::deposit_event(Event::<T>::InternalErrorWrongHalvingConfigured);
					}
				}
			}
		}
	}

	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert,
	{
		pub fn account_id() -> T::AccountId {
			COMPUTING_PALLETID.into_account_truncating()
		}

		pub fn move_value(from_prefix: &[u8], to_prefix: &[u8]) {
			if from_prefix == to_prefix {
				return;
			}
			let value = unhashed::get_raw(from_prefix).unwrap_or_default();
			unhashed::put_raw(to_prefix, &value);

			return;
		}

		pub fn move_prefix(from_prefix: &[u8], to_prefix: &[u8], max_iterations: u32) {
			if from_prefix == to_prefix {
				return;
			}

			let mut iter = PrefixIterator::<_>::new(
				from_prefix.clone().into(),
				from_prefix.into(),
				|key, value| Ok((key.to_vec(), value.to_vec())),
			);
			iter = iter.drain();
			let mut i = 0;

			for (key, value) in iter {
				let full_key = [to_prefix, &key].concat();
				unhashed::put_raw(&full_key, &value);
				i += 1;
				if i >= max_iterations {
					return;
				}
			}
		}

		fn heartbeat_challenge() {
			if HeartbeatPaused::<T>::get() {
				return;
			}
			// Random seed for the heartbeat challenge
			let seed_hash = T::Randomness::random(crate::constants::RANDOMNESS_SUBJECT).0;
			let seed: U256 = AsRef::<[u8]>::as_ref(&seed_hash).into();
			// PoW target for the random sampling
			let online_workers = OnlineWorkers::<T>::get();
			let num_tx =
				ExpectedHeartbeatCount::<T>::get().unwrap_or(DEFAULT_EXPECTED_HEARTBEAT_COUNT);
			let online_target = pow_target(num_tx, online_workers, T::ExpectedBlockTimeSec::get());
			let seed_info = HeartbeatChallenge {
				seed,
				online_target,
			};
			Self::push_message(SystemEvent::HeartbeatChallenge(seed_info));
		}

		fn trigger_subsidy_halving() -> Result<(), ()> {
			let mut tokenomic = TokenomicParameters::<T>::get().ok_or(())?;
			let budget_per_block = FixedPoint::from_bits(tokenomic.budget_per_block);
			let new_budget = budget_per_block * fp!(0.75);
			tokenomic.budget_per_block = new_budget.to_bits();
			Self::deposit_event(Event::<T>::SubsidyBudgetHalved);
			Self::update_tokenomic_parameters(tokenomic);
			Ok(())
		}

		pub fn on_working_message_received(
			message: DecodedMessage<WorkingReportEvent>,
		) -> DispatchResult {
			if let MessageOrigin::Worker(worker) = message.sender {
				match message.payload {
					WorkingReportEvent::Heartbeat {
						iterations,
						challenge_time,
						..
					}
					| WorkingReportEvent::HeartbeatV2 {
						iterations,
						challenge_time,
						..
					} => {
						// Handle with great care!
						//
						// In some cases, a message can be delayed, but the worker has been already
						// unbound or removed (possible?). So when we receive a message, don't
						// assume the worker is always there and the worker state is complete. So
						// far it sounds safe to just discard this message, but not interrupt the
						// entire message queue.
						//
						// So we call `ensure_worker_bound` here, and return an error if the worker
						// is not bound. However if the worker is indeed bound, the rest of the
						// code assumes the Sessions, Workers, and worker score must exist.
						let session = Self::ensure_worker_bound(&worker)?;
						let mut session_info =
							Self::sessions(&session).expect("Bound worker; qed.");
						let _worker =
							registry::Workers::<T>::get(&worker).expect("Bound worker; qed.");
						let now = Self::now_sec();
						let challenge_time_sec = challenge_time / 1000;
						session_info
							.benchmark
							.update(now, iterations, challenge_time_sec)
							.expect("Benchmark report must be valid; qed.");
						Self::deposit_event(Event::<T>::BenchmarkUpdated {
							session: session.clone(),
							p_instant: session_info.benchmark.p_instant,
						});
						Sessions::<T>::insert(&session, session_info);
					}
				};
			}
			Ok(())
		}

		pub fn on_gk_message_received(
			message: DecodedMessage<WorkingInfoUpdateEvent<T::BlockNumber>>,
		) -> DispatchResult {
			if !matches!(message.sender, MessageOrigin::Gatekeeper) {
				return Err(Error::<T>::BadSender.into());
			}

			let event = message.payload;
			if !event.is_empty() {
				let emit_ts = event.timestamp_ms / 1000;
				let now = Self::now_sec();

				// worker offline, update bound worker state to unresponsive
				for worker in event.offline {
					if let Some(account) = WorkerBindings::<T>::get(&worker) {
						let mut session_info = match Self::sessions(&account) {
							Some(session) => session,
							None => continue, // Skip non-existing workers
						};
						// Skip non-computing workers
						if !session_info.state.is_computing() {
							continue;
						}
						session_info.state = WorkerState::WorkerUnresponsive;
						Sessions::<T>::insert(&account, &session_info);
						Self::deposit_event(Event::<T>::WorkerEnterUnresponsive {
							session: account,
						});
						Self::push_message(SystemEvent::new_worker_event(
							worker,
							WorkerEvent::EnterUnresponsive,
						));
					}
				}

				// worker recovered to online, update bound worker state to idle
				for worker in event.recovered_to_online {
					if let Some(account) = WorkerBindings::<T>::get(&worker) {
						let mut session_info = match Self::sessions(&account) {
							Some(worker) => worker,
							None => continue, // Skip non-existing workers
						};
						// Skip non-computing workers
						if !session_info.state.is_computing() {
							continue;
						}
						session_info.state = WorkerState::WorkerIdle;
						Sessions::<T>::insert(&account, &session_info);
						Self::deposit_event(Event::<T>::WorkerExitUnresponsive {
							session: account,
						});
						Self::push_message(SystemEvent::new_worker_event(
							worker,
							WorkerEvent::ExitUnresponsive,
						));
					}
				}

				for info in &event.settle {
					// Do not crash here
					if Self::try_handle_settle(info, now, emit_ts).is_err() {
						Self::deposit_event(Event::<T>::InternalErrorWorkerSettleFailed {
							worker: info.pubkey,
						})
					}
				}

				T::OnReward::on_reward(&event.settle);
			}

			Ok(())
		}

		/// Tries to handle settlement of a session.
		///
		/// We really don't want to crash the interrupt the message processing. So when there's an
		/// error we return it, and let the caller to handle it gracefully.
		///
		/// `now` and `emit_ts` are both in second.
		fn try_handle_settle(info: &SettleInfo, now: u64, emit_ts: u64) -> DispatchResult {
			if let Some(account) = WorkerBindings::<T>::get(&info.pubkey) {
				let mut session_info =
					Self::sessions(&account).ok_or(Error::<T>::SessionNotFound)?;
				debug_assert!(session_info.state.can_settle(), "Worker cannot settle now");
				if session_info.v_updated_at >= emit_ts {
					// Received a late update of the settlement. For now we just drop it.
					Self::deposit_event(Event::<T>::SessionSettlementDropped {
						session: account,
						v: info.v,
						payout: info.payout,
					});
					return Ok(());
				}
				// Otherwise it's a normal update. Let's proceed.
				session_info.v = info.v; // in bits
				session_info.v_updated_at = now;
				session_info.stats.on_reward(info.payout);
				Sessions::<T>::insert(&account, &session_info);
				// Handle treasury deposit
				let treasury_deposit = FixedPointConvert::from_bits(info.treasury);
				let imbalance = Self::withdraw_imbalance_from_subsidy_pool(treasury_deposit)?;
				T::OnTreasurySettled::on_unbalanced(imbalance);
				Self::deposit_event(Event::<T>::SessionSettled {
					session: account,
					v_bits: info.v,
					payout_bits: info.payout,
				});
			}
			Ok(())
		}

		fn can_reclaim(session_info: &SessionInfo, check_cooldown: bool) -> bool {
			if session_info.state != WorkerState::WorkerCoolingDown {
				return false;
			}
			if !check_cooldown {
				return true;
			}
			let now = Self::now_sec();
			now - session_info.cool_down_start >= Self::cool_down_period()
		}

		/// Turns the worker back to Ready state after cooling down and trigger stake releasing.
		///
		/// Requires:
		/// 1. The workers is in CoolingDown state and the cool down period has passed
		pub fn reclaim(
			session: T::AccountId,
			check_cooldown: bool,
		) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
			let mut session_info =
				Sessions::<T>::get(&session).ok_or(Error::<T>::SessionNotFound)?;
			ensure!(
				Self::can_reclaim(&session_info, check_cooldown),
				Error::<T>::CoolDownNotReady
			);
			session_info.state = WorkerState::Ready;
			session_info.cool_down_start = 0u64;
			Sessions::<T>::insert(&session, &session_info);

			let orig_stake = Stakes::<T>::take(&session).unwrap_or_default();
			let (_returned, slashed) = session_info.calc_final_stake(orig_stake);

			Self::deposit_event(Event::<T>::WorkerReclaimed {
				session,
				original_stake: orig_stake,
				slashed,
			});
			Ok((orig_stake, slashed))
		}

		/// Binds a session to a worker
		///
		/// This will bind the worker account to the worker, and then create a `Workers` entry to
		/// track the computing session in the future. The computing session will exist until the worker
		/// and the worker is unbound.
		///
		/// Requires:
		/// 1. The worker is already registered
		/// 2. The worker has an initial benchmark
		/// 3. Both the worker and the worker are not bound
		/// 4. There's no stake in CD associated with the worker
		pub fn bind(session: T::AccountId, pubkey: WorkerPublicKey) -> DispatchResult {
			let worker =
				registry::Workers::<T>::get(&pubkey).ok_or(Error::<T>::WorkerNotRegistered)?;
			// Check the worker has finished the benchmark
			ensure!(worker.initial_score != None, Error::<T>::BenchmarkMissing);
			// Check worker and worker not bound
			ensure!(
				Self::ensure_session_bound(&session).is_err(),
				Error::<T>::DuplicateBoundSession
			);
			ensure!(
				Self::ensure_worker_bound(&pubkey).is_err(),
				Error::<T>::DuplicateBoundWorker
			);
			// Make sure we are not overriding a running worker (even if the worker is unbound)
			let can_bind = match Sessions::<T>::get(&session) {
				Some(info) => info.state == WorkerState::Ready,
				None => true,
			};
			ensure!(can_bind, Error::<T>::WorkerNotReady);

			let now = Self::now_sec();
			SessionBindings::<T>::insert(&session, &pubkey);
			WorkerBindings::<T>::insert(&pubkey, &session);
			Sessions::<T>::insert(
				&session,
				SessionInfo {
					state: WorkerState::Ready,
					ve: 0,
					v: 0,
					v_updated_at: now,
					benchmark: Benchmark {
						p_init: 0u32,
						p_instant: 0u32,
						iterations: 0u64,
						working_start_time: now,
						challenge_time_last: 0u64,
					},
					cool_down_start: 0u64,
					stats: Default::default(),
				},
			);

			Self::deposit_event(Event::<T>::SessionBound {
				session,
				worker: pubkey,
			});
			Ok(())
		}

		/// Unbinds a session from a worker
		///
		/// - `notify`: whether to notify the subscribe the unbinding event.
		///
		/// Requires:
		/// 1. The session is bounded with a worker
		pub fn unbind_session(session: &T::AccountId, notify: bool) -> DispatchResult {
			let worker = Self::ensure_session_bound(session)?;
			let session_info = Sessions::<T>::get(session)
				.expect("A bounded worker must has the associated SessionInfo; qed.");

			let force = !session_info.state.can_unbind();
			if force {
				// Force unbinding. Stop the worker first.
				// Note that `stop_computing` will notify the subscribers with the slashed value.
				Self::stop_computing(session.clone())?;
				// TODO: consider the final state sync (could cause slash) when stopping computing
			}
			SessionBindings::<T>::remove(session);
			WorkerBindings::<T>::remove(&worker);
			Self::deposit_event(Event::<T>::SessionUnbound {
				session: session.clone(),
				worker,
			});
			if notify {
				T::OnUnbound::on_unbound(&worker, force);
			}

			Ok(())
		}

		/// Starts computing with the given `stake`, assuming the stake is already locked externally
		///
		/// A minimal P is required to avoid some edge case (e.g. worker not getting full benchmark
		/// with a close-to-zero score).
		pub fn start_computing(session: T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
			let worker = SessionBindings::<T>::get(&session).ok_or(Error::<T>::SessionNotFound)?;
			ensure!(
				Sessions::<T>::get(&session).unwrap().state == WorkerState::Ready,
				Error::<T>::WorkerNotReady
			);
			// Double check the Stake shouldn't be overrode
			ensure!(
				Stakes::<T>::get(&session) == None,
				Error::<T>::InternalErrorCannotStartWithExistingStake,
			);

			let session_info =
				registry::Workers::<T>::get(&worker).expect("Bounded worker must exist; qed.");
			let p = session_info
				.initial_score
				.ok_or(Error::<T>::BenchmarkMissing)?;
			// Disallow some weird benchmark score.
			ensure!(p >= T::MinInitP::get(), Error::<T>::BenchmarkTooLow);

			let tokenomic = Self::tokenomic()?;
			let min_stake = tokenomic.minimal_stake(p);
			ensure!(stake >= min_stake, Error::<T>::InsufficientStake);

			let ve = tokenomic.ve(stake, p, session_info.confidence_level);
			let v_max = tokenomic.v_max();
			ensure!(ve <= v_max, Error::<T>::TooMuchStake);

			let now = Self::now_sec();

			Stakes::<T>::insert(&session, stake);
			Sessions::<T>::mutate(&session, |info| {
				if let Some(info) = info {
					info.state = WorkerState::WorkerIdle;
					info.ve = ve.to_bits();
					info.v = ve.to_bits();
					info.v_updated_at = now;
					info.benchmark.p_init = p;
				}
			});
			OnlineWorkers::<T>::mutate(|v| *v += 1);

			let session_id = NextSessionId::<T>::get();
			NextSessionId::<T>::put(session_id + 1);
			Self::push_message(SystemEvent::new_worker_event(
				worker,
				WorkerEvent::Started {
					session_id,
					init_v: ve.to_bits(),
					init_p: p,
				},
			));
			Self::deposit_event(Event::<T>::WorkerStarted {
				session,
				init_v: ve.to_bits(),
				init_p: p,
			});
			Ok(())
		}

		/// Stops computing, entering cool down state
		///
		/// Requires:
		/// 1. The worker is in WorkerIdle, or WorkerUnresponsive state
		pub fn stop_computing(session: T::AccountId) -> DispatchResult {
			let worker = SessionBindings::<T>::get(&session).ok_or(Error::<T>::SessionNotBound)?;
			let mut session_info =
				Sessions::<T>::get(&session).ok_or(Error::<T>::SessionNotFound)?;

			ensure!(
				session_info.state != WorkerState::Ready
					&& session_info.state != WorkerState::WorkerCoolingDown,
				Error::<T>::WorkerNotComputing
			);

			let now = Self::now_sec();
			session_info.state = WorkerState::WorkerCoolingDown;
			session_info.cool_down_start = now;
			Sessions::<T>::insert(&session, &session_info);
			OnlineWorkers::<T>::mutate(|v| *v -= 1); // v cannot be 0

			// Calculate remaining stake (assume there's no more slash after calling `stop_computing`)
			let orig_stake = Stakes::<T>::get(&session).unwrap_or_default();
			let (_returned, slashed) = session_info.calc_final_stake(orig_stake);
			// Notify the subscriber with the slash prediction
			T::OnStopped::on_stopped(&worker, orig_stake, slashed);

			Self::push_message(SystemEvent::new_worker_event(worker, WorkerEvent::Stopped));
			Self::deposit_event(Event::<T>::WorkerStopped { session });
			Ok(())
		}

		/// Returns if the worker is already bounded to a session
		pub fn ensure_worker_bound(pubkey: &WorkerPublicKey) -> Result<T::AccountId, Error<T>> {
			WorkerBindings::<T>::get(&pubkey).ok_or(Error::<T>::WorkerNotBound)
		}

		/// Returns if the session is already bounded to a worker
		pub fn ensure_session_bound(session: &T::AccountId) -> Result<WorkerPublicKey, Error<T>> {
			SessionBindings::<T>::get(&session).ok_or(Error::<T>::SessionNotBound)
		}

		fn update_tokenomic_parameters(params: TokenomicParams) {
			TokenomicParameters::<T>::put(params.clone());
			Self::push_message(GatekeeperEvent::TokenomicParametersChanged(params));
			Self::deposit_event(Event::<T>::TokenomicParametersChanged);
		}

		pub fn withdraw_subsidy_pool(target: &T::AccountId, value: BalanceOf<T>) -> DispatchResult {
			let wallet = Self::account_id();
			<T as PhalaConfig>::Currency::transfer(&wallet, target, value, KeepAlive)
		}

		pub fn withdraw_imbalance_from_subsidy_pool(
			value: BalanceOf<T>,
		) -> Result<NegativeImbalanceOf<T>, DispatchError> {
			let wallet = Self::account_id();
			<T as PhalaConfig>::Currency::withdraw(
				&wallet,
				value,
				WithdrawReasons::TRANSFER,
				frame_support::traits::ExistenceRequirement::AllowDeath,
			)
		}

		fn tokenomic() -> Result<Tokenomic<T>, Error<T>> {
			let params = TokenomicParameters::<T>::get()
				.ok_or(Error::<T>::InternalErrorBadTokenomicParameters)?;
			Ok(Tokenomic::<T>::new(params))
		}

		fn now_sec() -> u64 {
			<T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>()
		}
	}

	struct Tokenomic<T> {
		params: TokenomicParams,
		mark: PhantomData<T>,
	}

	impl<T> Tokenomic<T>
	where
		T: Config,
		BalanceOf<T>: FixedPointConvert,
	{
		fn new(params: TokenomicParams) -> Self {
			Tokenomic {
				params,
				mark: Default::default(),
			}
		}

		/// Gets the minimal stake with the given performance score
		fn minimal_stake(&self, p: u32) -> BalanceOf<T> {
			let p = FixedPoint::from_num(p);
			let k = FixedPoint::from_bits(self.params.k);
			let min_stake = k * p.sqrt();
			FixedPointConvert::from_fixed(&min_stake)
		}

		/// Calculate the initial Ve
		fn ve(&self, s: BalanceOf<T>, p: u32, confidence_level: u8) -> FixedPoint {
			let f1 = FixedPoint::from_num(1);
			let score = Self::confidence_score(confidence_level);
			let re = FixedPoint::from_bits(self.params.re);
			let tweaked_re = (re - f1) * score + f1;
			let s = s.to_fixed();
			let c = self.rig_cost(p);
			tweaked_re * (s + c)
		}

		/// Gets the max v in fixed point
		fn v_max(&self) -> FixedPoint {
			FixedPoint::from_bits(self.params.v_max)
		}

		/// Gets the estimated rig costs in PHA
		fn rig_cost(&self, p: u32) -> FixedPoint {
			let cost_k = FixedPoint::from_bits(self.params.rig_k);
			let cost_b = FixedPoint::from_bits(self.params.rig_b);
			let p = FixedPoint::from_num(p);
			cost_k * p + cost_b
		}

		/// Gets the operating cost per block
		#[cfg(test)]
		fn op_cost(&self, p: u32) -> FixedPoint {
			let cost_k = FixedPoint::from_bits(self.params.cost_k);
			let cost_b = FixedPoint::from_bits(self.params.cost_b);
			let p = FixedPoint::from_num(p);
			cost_k * p + cost_b
		}

		/// Converts confidence level to score
		fn confidence_score(confidence_level: u8) -> FixedPoint {
			use fixed_macro::types::U64F64 as fp;
			const SCORES: [FixedPoint; 5] = [fp!(1), fp!(1), fp!(1), fp!(0.8), fp!(0.7)];
			if (1..=5).contains(&confidence_level) {
				SCORES[confidence_level as usize - 1]
			} else {
				SCORES[0]
			}
		}

		/// Gets kappa in FixedPoint
		fn _kappa(&self) -> FixedPoint {
			FixedPoint::from_bits(self.params.kappa)
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub cool_down_period_sec: u32,
		pub tokenomic_parameters: TokenomicParams,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		/// Default tokenomic parameters for Phala
		fn default() -> Self {
			use fixed_macro::types::U64F64 as fp;
			let block_sec = 12;
			let hour_sec = 3600;
			let day_sec = 24 * hour_sec;
			let year_sec = 365 * day_sec;

			let pha_rate = fp!(1);
			let rho = fp!(1.000000666600231); // hourly: 1.00020,  1.0002 ** (1/300) convert to per-block
			let slash_rate = fp!(0.001) * block_sec / hour_sec; // hourly rate: 0.001, convert to per-block
			let budget_per_block = fp!(720000) * block_sec / day_sec;
			let v_max = fp!(30000);
			let cost_k = fp!(0.0415625) * block_sec / year_sec / pha_rate; // annual 0.0415625, convert to per-block
			let cost_b = fp!(88.59375) * block_sec / year_sec / pha_rate; // annual 88.59375, convert to per-block
			let treasury_ratio = fp!(0.2);
			let heartbeat_window = 10; // 10 blocks
			let rig_k = fp!(0.3) / pha_rate;
			let rig_b = fp!(0) / pha_rate;
			let re = fp!(1.3);
			let k = fp!(100);
			let kappa = fp!(1);

			Self {
				cool_down_period_sec: 604800, // 7 days
				tokenomic_parameters: TokenomicParams {
					pha_rate: pha_rate.to_bits(),
					rho: rho.to_bits(),
					budget_per_block: budget_per_block.to_bits(),
					v_max: v_max.to_bits(),
					cost_k: cost_k.to_bits(),
					cost_b: cost_b.to_bits(),
					slash_rate: slash_rate.to_bits(),
					treasury_ratio: treasury_ratio.to_bits(),
					heartbeat_window,
					rig_k: rig_k.to_bits(),
					rig_b: rig_b.to_bits(),
					re: re.to_bits(),
					k: k.to_bits(),
					kappa: kappa.to_bits(),
				},
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			CoolDownPeriod::<T>::put(self.cool_down_period_sec as u64);
			TokenomicParameters::<T>::put(self.tokenomic_parameters.clone());
			Pallet::<T>::queue_message(GatekeeperEvent::TokenomicParametersChanged(
				self.tokenomic_parameters.clone(),
			));
		}
	}

	fn pow_target(num_tx: u32, num_workers: u32, secs_per_block: u32) -> U256 {
		use fixed::types::U32F32;
		if num_workers == 0 {
			return U256::zero();
		}
		let num_workers = U32F32::from_num(num_workers);
		let num_tx = U32F32::from_num(num_tx);
		// Limit tx per block for a single worker
		//     t <= max_tx_per_hour * N/T (max_tx_per_hour = 2)
		let max_tx = num_workers * U32F32::from_num(2) / U32F32::from_num(3600 / secs_per_block);
		let target_tx = cmp::min(num_tx, max_tx);
		// Convert to U256 target
		//     target = MAX * tx / num_workers
		let frac: u32 = (target_tx / num_workers)
			.checked_shl(24)
			.expect("No overflow; qed.")
			.to_num();
		(U256::MAX >> 24) * frac
	}

	impl<T: Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}

	#[cfg(test)]
	mod test {
		use super::*;
		use crate::mock::{
			elapse_seconds, new_test_ext, set_block_1, setup_workers, take_events, take_messages,
			worker_pubkey, BlockNumber, RuntimeEvent as TestEvent, RuntimeOrigin as Origin, Test,
			DOLLARS,
		};
		// Pallets
		use crate::mock::{PhalaComputation, PhalaRegistry, System};

		use fixed_macro::types::U64F64 as fp;
		use frame_support::{assert_noop, assert_ok};

		#[test]
		fn test_computing_wallet_setup() {
			new_test_ext().execute_with(|| {
				assert!(System::account_exists(&PhalaComputation::account_id()));
			});
		}

		#[test]
		fn test_pow_target() {
			// No target
			assert_eq!(pow_target(20, 0, 12), U256::zero());
			// Capped target (py3: ``)
			assert_eq!(
				pow_target(20, 20, 12),
				U256::from_dec_str(
					"771946525395830978497002573683960742805751636319313395421818009383503547160"
				)
				.unwrap()
			);
			// Not capped target (py3: `int(((1 << 256) - 1) * 20 / 200_000)`)
			assert_eq!(
				pow_target(20, 200_000, 12),
				U256::from_dec_str(
					"11574228623567775471528085581038571683760509746329738253007553123311417715"
				)
				.unwrap()
			);
		}

		#[test]
		fn test_heartbeat_challenge() {
			new_test_ext().execute_with(|| {
				use phala_types::messaging::{SystemEvent, Topic};

				set_block_1();
				OnlineWorkers::<Test>::put(20);
				Pallet::<Test>::heartbeat_challenge();
				// Extract messages
				let msgs = take_messages();
				let message = match msgs.as_slice() {
					[m] => m,
					_ => panic!("Wrong message events"),
				};
				// Check the event target
				assert_eq!(message.destination, Topic::new("phala/system/event"));
				// Check the oubound message parameters
				let target = match message.decode_payload::<SystemEvent>() {
					Some(SystemEvent::HeartbeatChallenge(r)) => r.online_target,
					_ => panic!("Wrong outbound message"),
				};
				assert_eq!(target, U256::from_dec_str(
					"771946525395830978497002573683960742805751636319313395421818009383503547160"
				).unwrap());
			});
		}

		#[test]
		fn test_bind_unbind() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				// Bind & unbind normally
				let _ = take_events();
				assert_ok!(PhalaComputation::bind(1, worker_pubkey(1)));
				assert_ok!(PhalaComputation::unbind_session(&1, false));
				assert_eq!(
					take_events().as_slice(),
					[
						TestEvent::PhalaComputation(Event::SessionBound {
							session: 1,
							worker: worker_pubkey(1)
						}),
						TestEvent::PhalaComputation(Event::SessionUnbound {
							session: 1,
							worker: worker_pubkey(1)
						})
					]
				);
				// Checks edge cases
				assert_noop!(
					PhalaComputation::bind(1, worker_pubkey(100)),
					Error::<Test>::WorkerNotRegistered,
				);
				assert_noop!(
					PhalaComputation::unbind_session(&1, false),
					Error::<Test>::SessionNotBound,
				);
				// No double binding
				assert_ok!(PhalaComputation::bind(2, worker_pubkey(2)));
				assert_noop!(
					PhalaComputation::bind(2, worker_pubkey(1)),
					Error::<Test>::DuplicateBoundSession
				);
				assert_noop!(
					PhalaComputation::bind(1, worker_pubkey(2)),
					Error::<Test>::DuplicateBoundWorker
				);
				// Force unbind should be tested via StakePool
			});
		}

		#[test]
		#[should_panic]
		fn test_stakepool_callback_panic() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				assert_ok!(PhalaComputation::bind(1, worker_pubkey(1)));
				assert_ok!(PhalaComputation::start_computing(1, 1000 * DOLLARS));
				// Force unbind without StakePool registration will cause a panic
				let _ = PhalaComputation::unbind(Origin::signed(1), 1);
			});
		}

		#[test]
		fn test_tokenomic() {
			new_test_ext().execute_with(|| {
				set_block_1();
				let params = TokenomicParameters::<Test>::get().unwrap();
				let tokenomic = Tokenomic::<Test>::new(params);
				fn pow(x: FixedPoint, n: u32) -> FixedPoint {
					let mut i = n;
					let mut x_pow2 = x;
					let mut z = FixedPoint::from_num(1);
					while i > 0 {
						if i & 1 == 1 {
							z *= x_pow2;
						}
						x_pow2 *= x_pow2;
						i >>= 1;
					}
					z
				}
				// Vmax
				assert_eq!(tokenomic.v_max(), fp!(30000));
				// Minimal stake
				assert_eq!(tokenomic.minimal_stake(1000), 3162_277660146355);
				// Ve for different confidence level
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 1),
					fp!(1690.0000000000000000282)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 2),
					fp!(1690.0000000000000000282)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 3),
					fp!(1690.0000000000000000282)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 4),
					fp!(1612.0000000000000000247)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 5),
					fp!(1572.9999999999999999877)
				);
				// Rig cost estimation
				assert_eq!(tokenomic.rig_cost(500), fp!(150.0000000000000000054));
				assert_eq!(tokenomic.rig_cost(2000), fp!(600.0000000000000000217));
				assert_eq!(tokenomic.rig_cost(2800), fp!(840.00000000000000003036));

				const BLOCK_SEC: u32 = 12;
				const HOUR_BLOCKS: u32 = 3600 / BLOCK_SEC;
				// Slash per hour (around 0.1%)
				let slash_rate = FixedPoint::from_bits(tokenomic.params.slash_rate);
				let slash_decay = FixedPoint::from_num(1) - slash_rate;
				assert_eq!(pow(slash_decay, HOUR_BLOCKS), fp!(0.9990004981683704595));
				// rho per hour
				let rho = FixedPoint::from_bits(tokenomic.params.rho);
				assert_eq!(pow(rho, HOUR_BLOCKS), fp!(1.00019999999998037056));
				// Budget per day
				let budget_per_block = FixedPoint::from_bits(tokenomic.params.budget_per_block);
				assert_eq!(budget_per_block * 3600 * 24 / 12, fp!(720000));
				// Cost estimation per year
				assert_eq!(
					tokenomic.op_cost(2000) / 12 * 3600 * 24 * 365,
					fp!(171.71874999975847951583)
				);
				// Subsidy halving to 75%
				let _ = take_messages();
				let _ = take_events();
				assert_ok!(PhalaComputation::trigger_subsidy_halving());
				let params = TokenomicParameters::<Test>::get().unwrap();
				assert_eq!(FixedPoint::from_bits(params.budget_per_block), fp!(75));
				assert_ok!(PhalaComputation::trigger_subsidy_halving());
				let params = TokenomicParameters::<Test>::get().unwrap();
				assert_eq!(FixedPoint::from_bits(params.budget_per_block), fp!(56.25));
				let msgs = take_messages();
				assert_eq!(msgs.len(), 2);
				let ev = take_events();
				assert_eq!(
					ev,
					vec![
						TestEvent::PhalaComputation(Event::<Test>::SubsidyBudgetHalved),
						TestEvent::PhalaComputation(Event::<Test>::TokenomicParametersChanged),
						TestEvent::PhalaComputation(Event::<Test>::SubsidyBudgetHalved),
						TestEvent::PhalaComputation(Event::<Test>::TokenomicParametersChanged),
					]
				);
			});
		}

		#[test]
		fn tokenomic_update_is_postponed() {
			new_test_ext().execute_with(|| {
				set_block_1();
				let tokenomic = TokenomicParameters::<Test>::get().unwrap();
				assert_ok!(PhalaComputation::update_tokenomic(
					Origin::root(),
					tokenomic
				));
				let ev = take_events();
				assert_eq!(ev.len(), 0);
				PhalaComputation::on_finalize(1);
				let ev = take_events();
				assert_eq!(ev.len(), 1);
				assert_eq!(ScheduledTokenomicUpdate::<Test>::get(), None);
			});
		}

		#[test]
		fn test_benchmark_report() {
			use phala_types::messaging::{
				DecodedMessage, MessageOrigin, Topic, WorkingReportEvent,
			};
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				// 100 iters per sec
				PhalaRegistry::internal_set_benchmark(&worker_pubkey(1), Some(600));
				assert_ok!(PhalaComputation::bind(1, worker_pubkey(1)));
				assert_ok!(PhalaComputation::start_computing(1, 3000 * DOLLARS));
				// Though only the computing workers can send heartbeat, but we don't enforce it in
				// the pallet, but just by pRuntime. Therefore we can directly throw a heartbeat
				// response to test benchmark report.

				// 110% boost
				elapse_seconds(100);
				assert_ok!(PhalaComputation::on_working_message_received(
					DecodedMessage::<WorkingReportEvent> {
						sender: MessageOrigin::Worker(worker_pubkey(1)),
						destination: Topic::new(*b"phala/mining/report"),
						payload: WorkingReportEvent::Heartbeat {
							session_id: 0,
							challenge_block: 2,
							challenge_time: 100_000,
							iterations: 11000,
						},
					}
				));
				let worker = PhalaComputation::sessions(1).unwrap();
				assert_eq!(
					worker.benchmark,
					Benchmark {
						p_init: 600,
						p_instant: 660,
						iterations: 11000,
						working_start_time: 0,
						challenge_time_last: 100,
					}
				);

				// 150% boost (capped)
				elapse_seconds(100);
				assert_ok!(PhalaComputation::on_working_message_received(
					DecodedMessage::<WorkingReportEvent> {
						sender: MessageOrigin::Worker(worker_pubkey(1)),
						destination: Topic::new(*b"phala/mining/report"),
						payload: WorkingReportEvent::Heartbeat {
							session_id: 0,
							challenge_block: 3,
							challenge_time: 200_000,
							iterations: 11000 + 15000,
						},
					}
				));
				let worker = PhalaComputation::sessions(1).unwrap();
				assert_eq!(
					worker.benchmark,
					Benchmark {
						p_init: 600,
						p_instant: 720,
						iterations: 26000,
						working_start_time: 0,
						challenge_time_last: 200,
					}
				);
			});
		}

		#[test]
		fn test_benchmark_update() {
			let mut b = Benchmark {
				p_init: 100,
				p_instant: 0,
				iterations: 0,
				working_start_time: 0,
				challenge_time_last: 0,
			};
			// Normal
			assert!(b.update(100, 1000, 90).is_ok());
			assert_eq!(
				b,
				Benchmark {
					p_init: 100,
					p_instant: 60,
					iterations: 1000,
					working_start_time: 0,
					challenge_time_last: 90,
				}
			);
			// Reset counter
			assert!(b.update(200, 999, 190).is_ok());
			assert_eq!(
				b,
				Benchmark {
					p_init: 100,
					p_instant: 0,
					iterations: 999,
					working_start_time: 0,
					challenge_time_last: 190,
				}
			);
		}

		#[test]
		fn drop_late_arrived_update() {
			new_test_ext().execute_with(|| {
				use phala_types::messaging::Topic;

				set_block_1();
				setup_workers(1);
				PhalaRegistry::internal_set_benchmark(&worker_pubkey(1), Some(600));
				assert_ok!(PhalaComputation::bind(1, worker_pubkey(1)));
				elapse_seconds(100);
				assert_ok!(PhalaComputation::start_computing(1, 3000 * DOLLARS));
				take_events();
				assert_ok!(PhalaComputation::on_gk_message_received(DecodedMessage::<
					WorkingInfoUpdateEvent<BlockNumber>,
				> {
					sender: MessageOrigin::Gatekeeper,
					destination: Topic::new(*b"^phala/mining/update"),
					payload: WorkingInfoUpdateEvent::<BlockNumber> {
						block_number: 1,
						timestamp_ms: 100_000,
						offline: vec![],
						recovered_to_online: vec![],
						settle: vec![SettleInfo {
							pubkey: worker_pubkey(1),
							v: 1,
							payout: 0,
							treasury: 0,
						}],
					},
				}));

				let ev = take_events();
				assert_eq!(
					ev[0],
					TestEvent::PhalaComputation(Event::SessionSettlementDropped {
						session: 1,
						v: 1,
						payout: 0,
					})
				);
			});
		}
	}
}
