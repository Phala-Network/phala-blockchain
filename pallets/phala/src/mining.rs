pub use self::pallet::*;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use crate::mq::{self, MessageOriginInfo};
	use crate::registry;
	use frame_support::traits::WithdrawReasons;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			Currency, ExistenceRequirement::KeepAlive, OnUnbalanced, Randomness, StorageVersion,
			UnixTime,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use phala_types::{
		messaging::{
			DecodedMessage, GatekeeperEvent, HeartbeatChallenge, MessageOrigin,
			MiningInfoUpdateEvent, MiningReportEvent, SettleInfo, SystemEvent,
			TokenomicParameters as TokenomicParams, WorkerEvent,
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
	use fixed::types::U64F64 as FixedPoint;
	use fixed_macro::types::U64F64 as fp;
	use fixed_sqrt::FixedSqrt;

	const DEFAULT_EXPECTED_HEARTBEAT_COUNT: u32 = 20;
	const MINING_PALLETID: PalletId = PalletId(*b"phala/pp");

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub enum MinerState {
		Ready,
		MiningIdle,
		MiningActive,
		MiningUnresponsive,
		MiningCoolingDown,
	}

	impl MinerState {
		fn can_unbind(&self) -> bool {
			matches!(self, MinerState::Ready | MinerState::MiningCoolingDown)
		}
		fn can_settle(&self) -> bool {
			// TODO(hangyin):
			//
			// We don't allow a settlement in MiningCoolingDown or Ready. After a miner is stopped,
			// it's released immediately and the slash is pre-settled (to make sure the force
			// withdrawal can be processed correctly).
			//
			// We have to either figure out how to allow settlement in CoolingDown state, or
			// complete disable it as we do now. Note that when CoolingDown settle is not allowed,
			// we still have to make sure the slashed V is periodically updated on the blockchain.
			matches!(
				self,
				MinerState::MiningIdle | MinerState::MiningActive | MinerState::MiningUnresponsive
			)
		}
		fn is_mining(&self) -> bool {
			matches!(
				self,
				MinerState::MiningIdle | MinerState::MiningUnresponsive
			)
		}
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct Benchmark {
		p_init: u32,
		p_instant: u32,
		iterations: u64,
		mining_start_time: u64,
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

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct MinerInfo {
		pub state: MinerState,
		/// The intiial V, in U64F64 bits
		pub ve: u128,
		/// The last updated V, in U64F64 bits
		pub v: u128,
		v_updated_at: u64,
		benchmark: Benchmark,
		cool_down_start: u64,
		stats: MinerStats,
	}

	impl MinerInfo {
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
		fn on_reward(settle: &[SettleInfo]) {}
	}

	pub trait OnUnbound {
		/// Called when a worker was unbound from a miner.
		///
		/// `force` is set if the unbinding caused an unexpected miner shutdown.
		fn on_unbound(worker: &WorkerPublicKey, force: bool) {}
	}

	pub trait OnStopped<Balance> {
		/// Called with a miner is stopped and can already calculate the final slash and stake.
		///
		/// It guarantees the number will be the same as the return value of `reclaim()`
		fn on_stopped(worker: &WorkerPublicKey, orig_stake: Balance, slashed: Balance) {}
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Default, RuntimeDebug)]
	pub struct MinerStats {
		total_reward: u128,
	}

	impl MinerStats {
		fn on_reward(&mut self, payout_bits: u128) {
			let payout: u128 = FixedPointConvert::from_bits(payout_bits);
			self.total_reward += payout;
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + mq::Config + registry::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ExpectedBlockTimeSec: Get<u32>;
		type MinInitP: Get<u32>;

		type Currency: Currency<Self::AccountId>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type OnReward: OnReward;
		type OnUnbound: OnUnbound;
		type OnStopped: OnStopped<BalanceOf<Self>>;
		type OnTreasurySettled: OnUnbalanced<NegativeImbalanceOf<Self>>;
		// Let the StakePool to take over the slash events.

		/// The origin to update tokenomic.
		type UpdateTokenomicOrigin: EnsureOrigin<Self::Origin>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(5);

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

	/// Total online miners including MiningIdle and MiningUnresponsive workers.
	///
	/// Increased when a miner is turned to MiningIdle; decreased when turned to CoolingDown.
	#[pallet::storage]
	#[pallet::getter(fn online_miners)]
	pub type OnlineMiners<T> = StorageValue<_, u32, ValueQuery>;

	/// The expected heartbeat count at every block (default: 20)
	#[pallet::storage]
	pub type ExpectedHeartbeatCount<T> = StorageValue<_, u32>;

	/// The miner state.
	///
	/// The miner state is created when a miner is bounded with a worker, but it will be kept even
	/// if the worker is force unbound. A re-bind of a worker will reset the mining state.
	#[pallet::storage]
	#[pallet::getter(fn miners)]
	pub type Miners<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, MinerInfo>;

	/// The bound worker for a miner account
	#[pallet::storage]
	pub type MinerBindings<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, WorkerPublicKey>;

	/// The bound miner account for a worker
	#[pallet::storage]
	pub type WorkerBindings<T: Config> = StorageMap<_, Twox64Concat, WorkerPublicKey, T::AccountId>;

	/// The cool down period (in sec)
	#[pallet::storage]
	#[pallet::getter(fn cool_down_period)]
	pub type CoolDownPeriod<T> = StorageValue<_, u64, ValueQuery>;

	/// The next id to assign to a mining session
	#[pallet::storage]
	pub type NextSessionId<T> = StorageValue<_, u32, ValueQuery>;

	/// The block number when the mining starts. Used to calculate halving.
	#[pallet::storage]
	pub type MiningStartBlock<T: Config> = StorageValue<_, T::BlockNumber>;

	/// The interval of halving (75% decay) in block number.
	#[pallet::storage]
	pub type MiningHalvingInterval<T: Config> = StorageValue<_, T::BlockNumber>;

	/// The stakes of miner accounts.
	///
	/// Only presents for mining and cooling down miners.
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
		/// A miner starts mining.
		///
		/// Affected states:
		/// - the miner info at [`Miners`] is updated with `MiningIdle` state
		/// - [`NextSessionId`] for the miner is incremented
		/// - [`Stakes`] for the miner is updated
		/// - [`OnlineMiners`] is incremented
		MinerStarted { miner: T::AccountId },
		/// Miner stops mining.
		///
		/// Affected states:
		/// - the miner info at [`Miners`] is updated with `MiningCoolingDown` state
		/// - [`OnlineMiners`] is decremented
		MinerStopped { miner: T::AccountId },
		/// Miner is reclaimed, with its slash settled.
		MinerReclaimed {
			miner: T::AccountId,
			original_stake: BalanceOf<T>,
			slashed: BalanceOf<T>,
		},
		/// Miner & worker are bound.
		///
		/// Affected states:
		/// - [`MinerBindings`] for the miner account is pointed to the worker
		/// - [`WorkerBindings`] for the worker is pointed to the miner account
		/// - the miner info at [`Miners`] is updated with `Ready` state
		MinerBound {
			miner: T::AccountId,
			worker: WorkerPublicKey,
		},
		/// Miner & worker are unbound.
		///
		/// Affected states:
		/// - [`MinerBindings`] for the miner account is removed
		/// - [`WorkerBindings`] for the worker is removed
		MinerUnbound {
			miner: T::AccountId,
			worker: WorkerPublicKey,
		},
		/// Miner enters unresponsive state.
		///
		/// Affected states:
		/// - the miner info at [`Miners`] is updated from `MiningIdle` to `MiningUnresponsive`
		MinerEnterUnresponsive { miner: T::AccountId },
		/// Miner returns to responsive state.
		///
		/// Affected states:
		/// - the miner info at [`Miners`] is updated from `MiningUnresponsive` to `MiningIdle`
		MinerExitUnresponsive { miner: T::AccountId },
		/// Miner settled successfully.
		///
		/// It results in the v in [`Miners`] being updated. It also indicates the downstream
		/// stake pool has received the mining reward (payout), and the treasury has received the
		/// tax.
		MinerSettled {
			miner: T::AccountId,
			v_bits: u128,
			payout_bits: u128,
		},
		/// Some internal error happened when settling a miner's ledger.
		InternalErrorMinerSettleFailed { worker: WorkerPublicKey },
		/// Block subsidy halved by 25%.
		///
		/// This event will be followed by a [`TokenomicParametersChanged`] event indicating the
		/// change of the block subsidy budget in the parameter.
		SubsidyBudgetHalved,
		/// Some internal error happened when trying to halve the subsidy
		InternalErrorWrongHalvingConfigured,
		/// Tokenomic parameter changed.
		///
		/// Affected states:
		/// - [`TokenomicParameters`] is updated.
		TokenomicParametersChanged,
		/// A miner settlement was dropped because the on-chain version is more up-to-date.
		///
		/// This is a temporary walk-around of the mining staking design. Will be fixed by
		/// StakePool v2.
		MinerSettlementDropped {
			miner: T::AccountId,
			v: u128,
			payout: u128,
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
		/// Not permitted because the miner is already bound with another worker.
		DuplicateBoundMiner,
		/// There's no benchmark result on the blockchain.
		BenchmarkMissing,
		/// Miner not found.
		MinerNotFound,
		/// Not permitted because the miner is not bound with a worker.
		MinerNotBound,
		/// Miner is not in `Ready` state to proceed.
		MinerNotReady,
		/// Miner is not in `Mining` state to stop mining.
		MinerNotMining,
		/// Not permitted because the worker is not bound with a miner account.
		WorkerNotBound,
		/// Cannot reclaim the worker because it's still in cooldown period.
		CoolDownNotReady,
		/// Cannot start mining because there's too little stake.
		InsufficientStake,
		/// Cannot start mining because there's too much stake (exceeds Vmax).
		TooMuchStake,
		/// Internal error. The tokenomic parameter is not set.
		InternalErrorBadTokenomicParameters,
		/// Not permitted because the worker is already bound with another miner account.
		DuplicateBoundWorker,
		/// Indicating the initial benchmark score is too low to start mining.
		BenchmarkTooLow,
		/// Internal error. A miner should never start with existing stake in the storage.
		InternalErrorCannotStartWithExistingStake,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;

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

		/// Unbinds a worker from the given miner (or pool sub-account).
		///
		/// It will trigger a force stop of mining if the miner is still in mining state. Anyone
		/// can call it.
		#[pallet::weight(0)]
		pub fn unbind(origin: OriginFor<T>, miner: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let pubkey = Self::ensure_miner_bound(&miner)?;
			let worker =
				registry::Workers::<T>::get(&pubkey).ok_or(Error::<T>::WorkerNotRegistered)?;
			ensure!(worker.operator == Some(who), Error::<T>::BadSender);
			// Always notify the subscriber. Please note that even if the miner is not mining, we
			// still have to notify the subscriber that an unbinding operation has just happened.
			Self::unbind_miner(&miner, true)
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

		/// Start mining
		///
		/// Only for integration test.
		#[pallet::weight(1)]
		pub fn force_start_mining(
			origin: OriginFor<T>,
			miner: T::AccountId,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::start_mining(miner, stake)?;
			Ok(())
		}

		/// Stop mining
		///
		/// Only for integration test.
		#[pallet::weight(1)]
		pub fn force_stop_mining(origin: OriginFor<T>, miner: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Self::stop_mining(miner)?;
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
			if let Some(interval) = MiningHalvingInterval::<T>::get() {
				let block_elapsed = n - MiningStartBlock::<T>::get().unwrap_or_default();
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
			MINING_PALLETID.into_account()
		}

		fn heartbeat_challenge() {
			// Random seed for the heartbeat challenge
			let seed_hash = T::Randomness::random(crate::constants::RANDOMNESS_SUBJECT).0;
			let seed: U256 = AsRef::<[u8]>::as_ref(&seed_hash).into();
			// PoW target for the random sampling
			let online_miners = OnlineMiners::<T>::get();
			let num_tx =
				ExpectedHeartbeatCount::<T>::get().unwrap_or(DEFAULT_EXPECTED_HEARTBEAT_COUNT);
			let online_target = pow_target(num_tx, online_miners, T::ExpectedBlockTimeSec::get());
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

		pub fn on_mining_message_received(
			message: DecodedMessage<MiningReportEvent>,
		) -> DispatchResult {
			if let MessageOrigin::Worker(worker) = message.sender {
				match message.payload {
					MiningReportEvent::Heartbeat {
						iterations,
						challenge_time,
						..
					} => {
						// Handle with great care!
						//
						// In some cases, a message can be delayed, but the worker has been already
						// unbound or removed (possible?). So when we receive a message, don't
						// assume the worker is always there and the miner state is complete. So
						// far it sounds safe to just discard this message, but not interrupt the
						// entire message queue.
						//
						// So we call `ensure_worker_bound` here, and return an error if the worker
						// is not bound. However if the worker is indeed bound, the rest of the
						// code assumes the Miners, Workers, and worker score must exist.
						let miner = Self::ensure_worker_bound(&worker)?;
						let mut miner_info = Self::miners(&miner).expect("Bound miner; qed.");
						let worker =
							registry::Workers::<T>::get(&worker).expect("Bound worker; qed.");
						let now = Self::now_sec();
						let challenge_time_sec = challenge_time / 1000;
						miner_info
							.benchmark
							.update(now, iterations, challenge_time_sec)
							.expect("Benchmark report must be valid; qed.");
						Miners::<T>::insert(&miner, miner_info);
					}
				};
			}
			Ok(())
		}

		pub fn on_gk_message_received(
			message: DecodedMessage<MiningInfoUpdateEvent<T::BlockNumber>>,
		) -> DispatchResult {
			if !matches!(message.sender, MessageOrigin::Gatekeeper) {
				return Err(Error::<T>::BadSender.into());
			}

			let event = message.payload;
			if !event.is_empty() {
				let emit_ts = event.timestamp_ms / 1000;
				let now = Self::now_sec();

				// worker offline, update bound miner state to unresponsive
				for worker in event.offline {
					if let Some(account) = WorkerBindings::<T>::get(&worker) {
						let mut miner_info = match Self::miners(&account) {
							Some(miner) => miner,
							None => continue, // Skip non-existing miners
						};
						// Skip non-mining miners
						if !miner_info.state.is_mining() {
							continue;
						}
						miner_info.state = MinerState::MiningUnresponsive;
						Miners::<T>::insert(&account, &miner_info);
						Self::deposit_event(Event::<T>::MinerEnterUnresponsive { miner: account });
						Self::push_message(SystemEvent::new_worker_event(
							worker,
							WorkerEvent::MiningEnterUnresponsive,
						));
					}
				}

				// worker recovered to online, update bound miner state to idle
				for worker in event.recovered_to_online {
					if let Some(account) = WorkerBindings::<T>::get(&worker) {
						let mut miner_info = match Self::miners(&account) {
							Some(miner) => miner,
							None => continue, // Skip non-existing miners
						};
						// Skip non-mining miners
						if !miner_info.state.is_mining() {
							continue;
						}
						miner_info.state = MinerState::MiningIdle;
						Miners::<T>::insert(&account, &miner_info);
						Self::deposit_event(Event::<T>::MinerExitUnresponsive { miner: account });
						Self::push_message(SystemEvent::new_worker_event(
							worker,
							WorkerEvent::MiningExitUnresponsive,
						));
					}
				}

				for info in &event.settle {
					// Do not crash here
					if Self::try_handle_settle(info, now, emit_ts).is_err() {
						Self::deposit_event(Event::<T>::InternalErrorMinerSettleFailed {
							worker: info.pubkey,
						})
					}
				}

				T::OnReward::on_reward(&event.settle);
			}

			Ok(())
		}

		/// Tries to handle settlement of a miner.
		///
		/// We really don't want to crash the interrupt the message processing. So when there's an
		/// error we return it, and let the caller to handle it gracefully.
		///
		/// `now` and `emit_ts` are both in second.
		fn try_handle_settle(info: &SettleInfo, now: u64, emit_ts: u64) -> DispatchResult {
			if let Some(account) = WorkerBindings::<T>::get(&info.pubkey) {
				let mut miner_info = Self::miners(&account).ok_or(Error::<T>::MinerNotFound)?;
				debug_assert!(miner_info.state.can_settle(), "Miner cannot settle now");
				if miner_info.v_updated_at >= emit_ts {
					// Received a late update of the settlement. For now we just drop it.
					Self::deposit_event(Event::<T>::MinerSettlementDropped {
						miner: account,
						v: info.v,
						payout: info.payout,
					});
					return Ok(());
				}
				// Otherwise it's a normal update. Let's proceed.
				miner_info.v = info.v; // in bits
				miner_info.v_updated_at = now;
				miner_info.stats.on_reward(info.payout);
				Miners::<T>::insert(&account, &miner_info);
				// Handle treasury deposit
				let treasury_deposit = FixedPointConvert::from_bits(info.treasury);
				let imbalance = Self::withdraw_imbalance_from_subsidy_pool(treasury_deposit)?;
				T::OnTreasurySettled::on_unbalanced(imbalance);
				Self::deposit_event(Event::<T>::MinerSettled {
					miner: account,
					v_bits: info.v,
					payout_bits: info.payout,
				});
			}
			Ok(())
		}

		fn can_reclaim(miner_info: &MinerInfo, check_cooldown: bool) -> bool {
			if miner_info.state != MinerState::MiningCoolingDown {
				return false;
			}
			if !check_cooldown {
				return true;
			}
			let now = Self::now_sec();
			now - miner_info.cool_down_start >= Self::cool_down_period()
		}

		/// Turns the miner back to Ready state after cooling down and trigger stake releasing.
		///
		/// Requires:
		/// 1. The miner is in CoolingDown state and the cool down period has passed
		pub fn reclaim(
			miner: T::AccountId,
			check_cooldown: bool,
		) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
			let mut miner_info = Miners::<T>::get(&miner).ok_or(Error::<T>::MinerNotFound)?;
			ensure!(
				Self::can_reclaim(&miner_info, check_cooldown),
				Error::<T>::CoolDownNotReady
			);
			miner_info.state = MinerState::Ready;
			miner_info.cool_down_start = 0u64;
			Miners::<T>::insert(&miner, &miner_info);

			let orig_stake = Stakes::<T>::take(&miner).unwrap_or_default();
			let (_returned, slashed) = miner_info.calc_final_stake(orig_stake);

			Self::deposit_event(Event::<T>::MinerReclaimed {
				miner,
				original_stake: orig_stake,
				slashed,
			});
			Ok((orig_stake, slashed))
		}

		/// Binds a miner to a worker
		///
		/// This will bind the miner account to the worker, and then create a `Miners` entry to
		/// track the mining session in the future. The mining session will exist until the miner
		/// and the worker is unbound.
		///
		/// Requires:
		/// 1. The worker is already registered
		/// 2. The worker has an initial benchmark
		/// 3. Both the worker and the miner are not bound
		/// 4. There's no stake in CD associated with the miner
		pub fn bind(miner: T::AccountId, pubkey: WorkerPublicKey) -> DispatchResult {
			let worker =
				registry::Workers::<T>::get(&pubkey).ok_or(Error::<T>::WorkerNotRegistered)?;
			// Check the worker has finished the benchmark
			ensure!(worker.initial_score != None, Error::<T>::BenchmarkMissing);
			// Check miner and worker not bound
			ensure!(
				Self::ensure_miner_bound(&miner).is_err(),
				Error::<T>::DuplicateBoundMiner
			);
			ensure!(
				Self::ensure_worker_bound(&pubkey).is_err(),
				Error::<T>::DuplicateBoundWorker
			);
			// Make sure we are not overriding a running miner (even if the worker is unbound)
			let can_bind = match Miners::<T>::get(&miner) {
				Some(info) => info.state == MinerState::Ready,
				None => true,
			};
			ensure!(can_bind, Error::<T>::MinerNotReady);

			let now = Self::now_sec();
			MinerBindings::<T>::insert(&miner, &pubkey);
			WorkerBindings::<T>::insert(&pubkey, &miner);
			Miners::<T>::insert(
				&miner,
				MinerInfo {
					state: MinerState::Ready,
					ve: 0,
					v: 0,
					v_updated_at: now,
					benchmark: Benchmark {
						p_init: 0u32,
						p_instant: 0u32,
						iterations: 0u64,
						mining_start_time: now,
						challenge_time_last: 0u64,
					},
					cool_down_start: 0u64,
					stats: Default::default(),
				},
			);

			Self::deposit_event(Event::<T>::MinerBound {
				miner,
				worker: pubkey,
			});
			Ok(())
		}

		/// Unbinds a miner from a worker
		///
		/// - `notify`: whether to notify the subscribe the unbinding event.
		///
		/// Requires:
		/// 1. The miner is bounded with a worker
		pub fn unbind_miner(miner: &T::AccountId, notify: bool) -> DispatchResult {
			let worker = Self::ensure_miner_bound(miner)?;
			let miner_info = Miners::<T>::get(miner)
				.expect("A bounded miner must has the associated MinerInfo; qed.");

			let force = !miner_info.state.can_unbind();
			if force {
				// Force unbinding. Stop the miner first.
				// Note that `stop_mining` will notify the subscribers with the slashed value.
				Self::stop_mining(miner.clone())?;
				// TODO: consider the final state sync (could cause slash) when stopping mining
			}
			MinerBindings::<T>::remove(miner);
			WorkerBindings::<T>::remove(&worker);
			Self::deposit_event(Event::<T>::MinerUnbound {
				miner: miner.clone(),
				worker,
			});
			if notify {
				T::OnUnbound::on_unbound(&worker, force);
			}

			Ok(())
		}

		/// Starts mining with the given `stake`, assuming the stake is already locked externally
		///
		/// A minimal P is required to avoid some edge case (e.g. miner not getting full benchmark
		/// with a close-to-zero score).
		pub fn start_mining(miner: T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
			let worker = MinerBindings::<T>::get(&miner).ok_or(Error::<T>::MinerNotFound)?;
			ensure!(
				Miners::<T>::get(&miner).unwrap().state == MinerState::Ready,
				Error::<T>::MinerNotReady
			);
			// Double check the Stake shouldn't be overrode
			ensure!(
				Stakes::<T>::get(&miner) == None,
				Error::<T>::InternalErrorCannotStartWithExistingStake,
			);

			let worker_info =
				registry::Workers::<T>::get(&worker).expect("Bounded worker must exist; qed.");
			let p = worker_info
				.initial_score
				.ok_or(Error::<T>::BenchmarkMissing)?;
			// Disallow some weird benchmark score.
			ensure!(p >= T::MinInitP::get(), Error::<T>::BenchmarkTooLow);

			let tokenomic = Self::tokenomic()?;
			let min_stake = tokenomic.minimal_stake(p);
			ensure!(stake >= min_stake, Error::<T>::InsufficientStake);

			let ve = tokenomic.ve(stake, p, worker_info.confidence_level);
			let v_max = tokenomic.v_max();
			ensure!(ve <= v_max, Error::<T>::TooMuchStake);

			let now = Self::now_sec();

			Stakes::<T>::insert(&miner, stake);
			Miners::<T>::mutate(&miner, |info| {
				if let Some(info) = info {
					info.state = MinerState::MiningIdle;
					info.ve = ve.to_bits();
					info.v = ve.to_bits();
					info.v_updated_at = now;
					info.benchmark.p_init = p;
				}
			});
			OnlineMiners::<T>::mutate(|v| *v += 1);

			let session_id = NextSessionId::<T>::get();
			NextSessionId::<T>::put(session_id + 1);
			Self::push_message(SystemEvent::new_worker_event(
				worker,
				WorkerEvent::MiningStart {
					session_id,
					init_v: ve.to_bits(),
					init_p: p,
				},
			));
			Self::deposit_event(Event::<T>::MinerStarted { miner });
			Ok(())
		}

		/// Stops mining, entering cool down state
		///
		/// Requires:
		/// 1. The miner is in Idle, MiningActive, or MiningUnresponsive state
		pub fn stop_mining(miner: T::AccountId) -> DispatchResult {
			let worker = MinerBindings::<T>::get(&miner).ok_or(Error::<T>::MinerNotBound)?;
			let mut miner_info = Miners::<T>::get(&miner).ok_or(Error::<T>::MinerNotFound)?;

			ensure!(
				miner_info.state != MinerState::Ready
					&& miner_info.state != MinerState::MiningCoolingDown,
				Error::<T>::MinerNotMining
			);

			let now = Self::now_sec();
			miner_info.state = MinerState::MiningCoolingDown;
			miner_info.cool_down_start = now;
			Miners::<T>::insert(&miner, &miner_info);
			OnlineMiners::<T>::mutate(|v| *v -= 1); // v cannot be 0

			// Calculate remaining stake (assume there's no more slash after calling `stop_mining`)
			let orig_stake = Stakes::<T>::get(&miner).unwrap_or_default();
			let (_returned, slashed) = miner_info.calc_final_stake(orig_stake);
			// Notify the subscriber with the slash prediction
			T::OnStopped::on_stopped(&worker, orig_stake, slashed);

			Self::push_message(SystemEvent::new_worker_event(
				worker,
				WorkerEvent::MiningStop,
			));
			Self::deposit_event(Event::<T>::MinerStopped { miner });
			Ok(())
		}

		/// Returns if the worker is already bounded to a miner
		pub fn ensure_worker_bound(pubkey: &WorkerPublicKey) -> Result<T::AccountId, Error<T>> {
			WorkerBindings::<T>::get(&pubkey).ok_or(Error::<T>::WorkerNotBound)
		}

		/// Returns if the miner is already bounded to a worker
		pub fn ensure_miner_bound(miner: &T::AccountId) -> Result<WorkerPublicKey, Error<T>> {
			MinerBindings::<T>::get(&miner).ok_or(Error::<T>::MinerNotBound)
		}

		fn update_tokenomic_parameters(params: TokenomicParams) {
			TokenomicParameters::<T>::put(params.clone());
			Self::push_message(GatekeeperEvent::TokenomicParametersChanged(params));
			Self::deposit_event(Event::<T>::TokenomicParametersChanged);
		}

		pub fn withdraw_subsidy_pool(target: &T::AccountId, value: BalanceOf<T>) -> DispatchResult {
			let wallet = Self::account_id();
			<T as Config>::Currency::transfer(&wallet, target, value, KeepAlive)
		}

		pub fn withdraw_imbalance_from_subsidy_pool(
			value: BalanceOf<T>,
		) -> Result<NegativeImbalanceOf<T>, DispatchError> {
			let wallet = Self::account_id();
			<T as Config>::Currency::withdraw(
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
			let pha_rate = FixedPoint::from_bits(self.params.pha_rate);
			let p = FixedPoint::from_num(p);
			cost_k * p + cost_b
		}

		/// Gets the operating cost per block
		#[cfg(test)]
		fn op_cost(&self, p: u32) -> FixedPoint {
			let cost_k = FixedPoint::from_bits(self.params.cost_k);
			let cost_b = FixedPoint::from_bits(self.params.cost_b);
			let pha_rate = FixedPoint::from_bits(self.params.pha_rate);
			let p = FixedPoint::from_num(p);
			cost_k * p + cost_b
		}

		/// Converts confidence level to score
		fn confidence_score(confidence_level: u8) -> FixedPoint {
			use fixed_macro::types::U64F64 as fp;
			const SCORES: [FixedPoint; 5] = [fp!(1), fp!(1), fp!(1), fp!(0.8), fp!(0.7)];
			if 1 <= confidence_level && confidence_level <= 5 {
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
					heartbeat_window: 10,
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

	pub(crate) mod migrations {
		use super::{Config, CoolDownPeriod, Pallet, TokenomicParameters};
		use fixed_macro::types::U64F64 as fp;
		use frame_support::pallet_prelude::*;

		use phala_types::messaging::TokenomicParameters as TokenomicParams;

		pub fn initialize<T: Config>() -> Weight {
			log::info!("phala_pallet::mining: initialize()");
			let block_sec = 12;
			let hour_sec = 3600;
			let day_sec = 24 * hour_sec;
			let year_sec = 365 * day_sec;
			// Initialize with Khala tokenomic parameters
			let pha_rate = fp!(0.84);
			let rho = fp!(1.000000666600231); // hourly: 1.00020,  1.0002 ** (1/300) convert to per-block
			let slash_rate = fp!(0.001) * block_sec / hour_sec; // hourly rate: 0.001, convert to per-block
			let budget_per_block = fp!(60000) * block_sec / day_sec;
			let v_max = fp!(30000);
			let cost_k = fp!(0.0415625) * block_sec / year_sec / pha_rate; // annual 0.0415625, convert to per-block
			let cost_b = fp!(88.59375) * block_sec / year_sec / pha_rate; // annual 88.59375, convert to per-block
			let treasury_ratio = fp!(0.2);
			let heartbeat_window = 10; // 10 blocks
			let rig_k = fp!(0.3) / pha_rate;
			let rig_b = fp!(0) / pha_rate;
			let re = fp!(1.5);
			let k = fp!(50);
			let kappa = fp!(1);
			// Write storage
			CoolDownPeriod::<T>::put(604800); // 7 days
			TokenomicParameters::<T>::put(TokenomicParams {
				pha_rate: pha_rate.to_bits(),
				rho: rho.to_bits(),
				budget_per_block: budget_per_block.to_bits(),
				v_max: v_max.to_bits(),
				cost_k: cost_k.to_bits(),
				cost_b: cost_b.to_bits(),
				slash_rate: slash_rate.to_bits(),
				treasury_ratio: treasury_ratio.to_bits(),
				heartbeat_window: 10,
				rig_k: rig_k.to_bits(),
				rig_b: rig_b.to_bits(),
				re: re.to_bits(),
				k: k.to_bits(),
				kappa: kappa.to_bits(),
			});
			T::DbWeight::get().writes(2)
		}

		pub(crate) fn signal_phala_launch<T: Config>() -> Weight {
			use crate::mq::pallet::MessageOriginInfo;
			use phala_types::messaging::GatekeeperEvent;
			Pallet::<T>::queue_message(GatekeeperEvent::PhalaLaunched);
			T::DbWeight::get().writes(1)
		}

		pub(crate) fn trigger_unresp_fix<T: Config>() -> Weight {
			use crate::mq::pallet::MessageOriginInfo;
			use phala_types::messaging::GatekeeperEvent;
			Pallet::<T>::queue_message(GatekeeperEvent::UnrespFix);
			T::DbWeight::get().writes(1)
		}

		pub(crate) fn enable_phala_tokenomic<T: Config>() -> Weight
		where
			super::BalanceOf<T>: crate::balance_convert::FixedPointConvert,
		{
			let encoded_params = hex_literal::hex!["b81e85eb51b81e450000000000000000fd7eb4062f0b00000100000000000000482aa913d044d8e0bb0000000000000000000000000000003075000000000000255a8ed66500000000000000000000009d3473f8f34f030000000000000000000000000000000000000000000000000033333333333333330000000000000000140000003b1c318036762473000000000000000000000000000000000000000000000000000000000000008001000000000000000000000000000000320000000000000000000000000000000100000000000000"];
			let phala_params = TokenomicParams::decode(&mut &encoded_params[..])
				.expect("Hardcoded TokenomicParams is valid; qed.");
			super::ScheduledTokenomicUpdate::<T>::put(phala_params);
			// Update the halving schedule
			let now = frame_system::Pallet::<T>::block_number();
			let interval: T::BlockNumber = 1296000_u32.into(); // 180 days in 12s
			super::MiningStartBlock::<T>::put(now);
			super::MiningHalvingInterval::<T>::put(interval);
			T::DbWeight::get().reads_writes(1, 3)
		}
	}

	fn pow_target(num_tx: u32, num_workers: u32, secs_per_block: u32) -> U256 {
		use fixed::types::U32F32;
		if num_workers == 0 {
			return U256::zero();
		}
		let num_workers = U32F32::from_num(num_workers);
		let num_tx = U32F32::from_num(num_tx);
		// Limit tx per block for a single miner
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
			worker_pubkey, BlockNumber, Event as TestEvent, Origin, Test, DOLLARS,
		};
		// Pallets
		use crate::mock::{PhalaMining, PhalaRegistry, System};

		use fixed_macro::types::U64F64 as fp;
		use frame_support::{assert_noop, assert_ok};

		#[test]
		fn test_mining_wallet_setup() {
			new_test_ext().execute_with(|| {
				assert!(System::account_exists(&PhalaMining::account_id()));
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
				OnlineMiners::<Test>::put(20);
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
				assert_ok!(PhalaMining::bind(1, worker_pubkey(1)));
				assert_ok!(PhalaMining::unbind_miner(&1, false));
				assert_eq!(
					take_events().as_slice(),
					[
						TestEvent::PhalaMining(Event::MinerBound {
							miner: 1,
							worker: worker_pubkey(1)
						}),
						TestEvent::PhalaMining(Event::MinerUnbound {
							miner: 1,
							worker: worker_pubkey(1)
						})
					]
				);
				// Checks edge cases
				assert_noop!(
					PhalaMining::bind(1, worker_pubkey(100)),
					Error::<Test>::WorkerNotRegistered,
				);
				assert_noop!(
					PhalaMining::unbind_miner(&1, false),
					Error::<Test>::MinerNotBound,
				);
				// No double binding
				assert_ok!(PhalaMining::bind(2, worker_pubkey(2)));
				assert_noop!(
					PhalaMining::bind(2, worker_pubkey(1)),
					Error::<Test>::DuplicateBoundMiner
				);
				assert_noop!(
					PhalaMining::bind(1, worker_pubkey(2)),
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
				assert_ok!(PhalaMining::bind(1, worker_pubkey(1)));
				assert_ok!(PhalaMining::start_mining(1, 1000 * DOLLARS));
				// Force unbind without StakePool registration will cause a panic
				let _ = PhalaMining::unbind(Origin::signed(1), 1);
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
				assert_ok!(PhalaMining::trigger_subsidy_halving());
				let params = TokenomicParameters::<Test>::get().unwrap();
				assert_eq!(FixedPoint::from_bits(params.budget_per_block), fp!(75));
				assert_ok!(PhalaMining::trigger_subsidy_halving());
				let params = TokenomicParameters::<Test>::get().unwrap();
				assert_eq!(FixedPoint::from_bits(params.budget_per_block), fp!(56.25));
				let msgs = take_messages();
				assert_eq!(msgs.len(), 2);
				let ev = take_events();
				assert_eq!(
					ev,
					vec![
						TestEvent::PhalaMining(Event::<Test>::SubsidyBudgetHalved),
						TestEvent::PhalaMining(Event::<Test>::TokenomicParametersChanged),
						TestEvent::PhalaMining(Event::<Test>::SubsidyBudgetHalved),
						TestEvent::PhalaMining(Event::<Test>::TokenomicParametersChanged),
					]
				);
			});
		}

		#[test]
		fn tokenomic_update_is_postponed() {
			new_test_ext().execute_with(|| {
				set_block_1();
				let tokenomic = TokenomicParameters::<Test>::get().unwrap();
				assert_ok!(PhalaMining::update_tokenomic(Origin::root(), tokenomic));
				let ev = take_events();
				assert_eq!(ev.len(), 0);
				PhalaMining::on_finalize(1);
				let ev = take_events();
				assert_eq!(ev.len(), 1);
				assert_eq!(ScheduledTokenomicUpdate::<Test>::get(), None);
			});
		}

		#[test]
		fn khala_tokenomics() {
			new_test_ext().execute_with(|| {
				migrations::initialize::<Test>();
				let params = TokenomicParameters::<Test>::get().unwrap();
				let tokenomic = Tokenomic::<Test>::new(params);

				assert_eq!(tokenomic.minimal_stake(1000), 1581_138830073177);

				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 1),
					fp!(2035.71428571428571430895)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 2),
					fp!(2035.71428571428571430895)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 3),
					fp!(2035.71428571428571430895)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 4),
					fp!(1899.99999999999999999225)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 5),
					fp!(1832.14285714285714283387)
				);
				assert_eq!(
					tokenomic.ve(5000 * DOLLARS, 2000, 4),
					fp!(7999.99999999999999991944)
				);
			});
		}

		#[test]
		fn test_benchmark_report() {
			use phala_types::messaging::{DecodedMessage, MessageOrigin, MiningReportEvent, Topic};
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				// 100 iters per sec
				PhalaRegistry::internal_set_benchmark(&worker_pubkey(1), Some(600));
				assert_ok!(PhalaMining::bind(1, worker_pubkey(1)));
				assert_ok!(PhalaMining::start_mining(1, 3000 * DOLLARS));
				// Though only the mining workers can send heartbeat, but we don't enforce it in
				// the pallet, but just by pRuntime. Therefore we can directly throw a heartbeat
				// response to test benchmark report.

				// 110% boost
				elapse_seconds(100);
				assert_ok!(PhalaMining::on_mining_message_received(DecodedMessage::<
					MiningReportEvent,
				> {
					sender: MessageOrigin::Worker(worker_pubkey(1)),
					destination: Topic::new(*b"phala/mining/report"),
					payload: MiningReportEvent::Heartbeat {
						session_id: 0,
						challenge_block: 2,
						challenge_time: 100_000,
						iterations: 11000,
					},
				}));
				let miner = PhalaMining::miners(1).unwrap();
				assert_eq!(
					miner.benchmark,
					Benchmark {
						p_init: 600,
						p_instant: 660,
						iterations: 11000,
						mining_start_time: 0,
						challenge_time_last: 100,
					}
				);

				// 150% boost (capped)
				elapse_seconds(100);
				assert_ok!(PhalaMining::on_mining_message_received(DecodedMessage::<
					MiningReportEvent,
				> {
					sender: MessageOrigin::Worker(worker_pubkey(1)),
					destination: Topic::new(*b"phala/mining/report"),
					payload: MiningReportEvent::Heartbeat {
						session_id: 0,
						challenge_block: 3,
						challenge_time: 200_000,
						iterations: 11000 + 15000,
					},
				}));
				let miner = PhalaMining::miners(1).unwrap();
				assert_eq!(
					miner.benchmark,
					Benchmark {
						p_init: 600,
						p_instant: 720,
						iterations: 26000,
						mining_start_time: 0,
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
				mining_start_time: 0,
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
					mining_start_time: 0,
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
					mining_start_time: 0,
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
				assert_ok!(PhalaMining::bind(1, worker_pubkey(1)));
				elapse_seconds(100);
				assert_ok!(PhalaMining::start_mining(1, 3000 * DOLLARS));
				take_events();
				assert_ok!(PhalaMining::on_gk_message_received(DecodedMessage::<
					MiningInfoUpdateEvent<BlockNumber>,
				> {
					sender: MessageOrigin::Gatekeeper,
					destination: Topic::new(*b"^phala/mining/update"),
					payload: MiningInfoUpdateEvent::<BlockNumber> {
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
					TestEvent::PhalaMining(Event::MinerSettlementDropped {
						miner: 1,
						v: 1,
						payout: 0,
					})
				);
			});
		}

		#[test]
		fn phala_params_migration_not_crash() {
			new_test_ext().execute_with(|| {
				migrations::enable_phala_tokenomic::<Test>();
			});
		}
	}
}
