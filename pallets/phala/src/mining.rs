pub use self::pallet::*;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use crate::mq::{self, MessageOriginInfo};
	use crate::registry;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::KeepAlive, Randomness, UnixTime},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use phala_types::{
		messaging::{
			DecodedMessage, GatekeeperEvent, HeartbeatChallenge, MessageOrigin,
			MiningInfoUpdateEvent, SettleInfo, SystemEvent, TokenomicParameters as TokenomicParams,
			WorkerEvent,
		},
		WorkerPublicKey,
	};
	use sp_core::U256;
	use sp_runtime::{traits::AccountIdConversion, SaturatedConversion};
	use sp_std::cmp;
	use sp_std::vec::Vec;

	use crate::balance_convert::FixedPointConvert;
	use fixed::types::U64F64 as FixedPoint;
	use fixed_sqrt::FixedSqrt;

	const DEFAULT_EXPECTED_HEARTBEAT_COUNT: u32 = 20;
	const MINING_PALLETID: PalletId = PalletId(*b"phala/pp");

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
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
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct Benchmark {
		iterations: u64,
		mining_start_time: u64,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct MinerInfo {
		pub state: MinerState,
		ve: u128,
		v: u128,
		v_updated_at: u64,
		p_instant: u64,
		benchmark: Benchmark,
		cool_down_start: u64,
	}

	pub trait OnReward {
		fn on_reward(settle: &Vec<SettleInfo>) {}
	}

	pub trait OnUnbound {
		/// Called wthen a worker was unbound from a miner.
		///
		/// `force` is set if the unbinding caused an unexpected miner shutdown.
		fn on_unbound(worker: &WorkerPublicKey, force: bool) {}
	}

	pub trait OnReclaim<AccountId, Balance> {
		/// Called when the miner has finished reclaiming and a given amount of the stake should be
		/// returned.
		///
		/// When called, it's not guaranteed there's still a worker associated to the miner.
		fn on_reclaim(worker: &AccountId, stake: Balance) {}
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct WorkerStat<Balance> {
		total_reward: Balance,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + mq::Config + registry::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ExpectedBlockTimeSec: Get<u32>;

		type Currency: Currency<Self::AccountId>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type OnReward: OnReward;
		type OnUnbound: OnUnbound;
		type OnReclaim: OnReclaim<Self::AccountId, BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Tokenomic parameters used by Gatekeepers to compute the V promote.
	#[pallet::storage]
	pub type TokenomicParameters<T> = StorageValue<_, TokenomicParams>;

	/// Total online miners
	///
	/// Increased when a miner is turned to MininIdle; decreased when turned to CoolingDown
	#[pallet::storage]
	#[pallet::getter(fn online_miners)]
	pub type OnlineMiners<T> = StorageValue<_, u32, ValueQuery>;

	/// The expected heartbeat count (default: 20)
	#[pallet::storage]
	pub type ExpectedHeartbeatCount<T> = StorageValue<_, u32>;

	/// The miner state.
	///
	/// The miner state is created when a miner is bounded with a worker, but it will be kept even
	/// if the worker is force unbounded. A re-bind of a worker will reset the mining state.
	#[pallet::storage]
	#[pallet::getter(fn miners)]
	pub(super) type Miners<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, MinerInfo>;

	/// The bound worker for a miner account
	#[pallet::storage]
	pub(super) type MinerBindings<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, WorkerPublicKey>;

	/// The bound miner account for a worker
	#[pallet::storage]
	pub(super) type WorkerBindings<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, T::AccountId>;

	/// The statistics for a worker
	#[pallet::storage]
	#[pallet::getter(fn worker_stats)]
	pub(super) type WorkerStats<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, WorkerStat<BalanceOf<T>>>;

	/// The cool down period (in blocks)
	#[pallet::storage]
	#[pallet::getter(fn cool_down_period)]
	pub(super) type CoolDownPeriod<T> = StorageValue<_, u64, ValueQuery>;

	/// The next id to assign to a mining session
	#[pallet::storage]
	pub(super) type NextSessionId<T> = StorageValue<_, u32, ValueQuery>;

	/// The stakes of miner accounts.
	///
	/// Only presents for mining and cooling down miners.
	#[pallet::storage]
	#[pallet::getter(fn stakes)]
	pub(super) type Stakes<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [period]
		CoolDownExpirationChanged(u64),
		/// [miner]
		MinerStarted(T::AccountId),
		/// [miner]
		MinerStopped(T::AccountId),
		/// [miner]
		MinerReclaimed(T::AccountId),
		/// [miner, worker]
		MinerBound(T::AccountId, WorkerPublicKey),
		/// [miner, worker]
		MinerUnbound(T::AccountId, WorkerPublicKey),
		/// [miner]
		MinerEnterUnresponsive(T::AccountId),
		/// [miner]
		MinerExitUnresponive(T::AccountId),
		/// [miner, amount]
		_MinerStaked(T::AccountId, BalanceOf<T>),
		/// [miner, amount]
		_MinerWithdrew(T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		BadSender,
		InvalidMessage,
		WorkerNotRegistered,
		GatekeeperNotRegistered,
		DuplicateBoundMiner,
		BenchmarkMissing,
		MinerNotFound,
		MinerNotBound,
		MinerNotReady,
		MinerNotMining,
		WorkerNotBound,
		CoolDownNotReady,
		InsufficientStake,
		TooMuchStake,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert,
	{
		#[pallet::weight(0)]
		pub fn set_cool_down_expiration(origin: OriginFor<T>, period: u64) -> DispatchResult {
			ensure_root(origin)?;

			CoolDownPeriod::<T>::mutate(|p| *p = period);
			Self::deposit_event(Event::<T>::CoolDownExpirationChanged(period));
			Ok(())
		}

		/// Unbinds a worker from the given miner (or pool sub-account).
		///
		/// It will trigger a force stop of mining if the miner is still in mining state.
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

		/// Turns the miner back to Ready state after cooling down and trigger stake releasing.
		///
		/// Note: anyone can trigger cleanup
		/// Requires:
		/// 1. Ther miner is in CoolingDown state and the cool down period has passed
		#[pallet::weight(0)]
		pub fn reclaim(origin: OriginFor<T>, miner: T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;
			let mut miner_info = Miners::<T>::get(&miner).ok_or(Error::<T>::MinerNotFound)?;
			ensure!(Self::can_reclaim(&miner_info), Error::<T>::CoolDownNotReady);
			miner_info.state = MinerState::Ready;
			miner_info.cool_down_start = 0u64;
			Miners::<T>::insert(&miner, &miner_info);

			let stake = Stakes::<T>::take(&miner).unwrap_or_default();
			// TODO: clean up based on V
			T::OnReclaim::on_reclaim(&miner, stake);
			Self::deposit_event(Event::<T>::MinerReclaimed(miner));
			Ok(())
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
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert,
	{
		fn on_finalize(_n: T::BlockNumber) {
			Self::heartbeat_challenge();
		}
	}

	// - Properly handle heartbeat message.
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

		pub fn on_gk_message_received(
			message: DecodedMessage<MiningInfoUpdateEvent<T::BlockNumber>>,
		) -> DispatchResult {
			if !matches!(message.sender, MessageOrigin::Gatekeeper) {
				return Err(Error::<T>::BadSender.into());
			}

			let event = message.payload;
			if !event.is_empty() {
				let now = <T as registry::Config>::UnixTime::now()
					.as_secs()
					.saturated_into::<u64>();

				// worker offline, update bound miner state to unresponsive
				for worker in event.offline {
					if let Some(binding_miner) = WorkerBindings::<T>::get(&worker) {
						let mut miner_info =
							Self::miners(&binding_miner).ok_or(Error::<T>::MinerNotFound)?;
						miner_info.state = MinerState::MiningUnresponsive;
						Miners::<T>::insert(&binding_miner, &miner_info);
						Self::deposit_event(Event::<T>::MinerEnterUnresponsive(binding_miner));
					}
				}

				// worker recovered to online, update bound miner state to idle
				for worker in event.recovered_to_online {
					if let Some(binding_miner) = WorkerBindings::<T>::get(&worker) {
						let mut miner_info =
							Self::miners(&binding_miner).ok_or(Error::<T>::MinerNotFound)?;
						miner_info.state = MinerState::MiningIdle;
						Miners::<T>::insert(&binding_miner, &miner_info);
						Self::deposit_event(Event::<T>::MinerExitUnresponive(binding_miner));
					}
				}

				for info in event.settle.clone() {
					if let Some(binding_miner) = WorkerBindings::<T>::get(&info.pubkey) {
						let mut miner_info =
							Self::miners(&binding_miner).ok_or(Error::<T>::MinerNotFound)?;
						miner_info.v = info.v; // in bits
						miner_info.v_updated_at = now;
						Miners::<T>::insert(&binding_miner, &miner_info);
					}
				}

				T::OnReward::on_reward(&event.settle);
			}

			Ok(())
		}

		fn can_reclaim(miner_info: &MinerInfo) -> bool {
			if miner_info.state != MinerState::MiningCoolingDown {
				return false;
			}
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			now - miner_info.cool_down_start >= Self::cool_down_period()
		}

		/// Binding miner with worker
		///
		/// Requires:
		/// 1. The worker is alerady registered
		/// 2. The worker has an initial benchmark
		/// 3. The worker is not bounded
		pub fn bind(miner: T::AccountId, pubkey: WorkerPublicKey) -> DispatchResult {
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();

			ensure!(
				registry::Workers::<T>::contains_key(&pubkey),
				Error::<T>::WorkerNotRegistered
			);

			ensure!(
				Self::ensure_miner_bound(&miner).is_err(),
				Error::<T>::DuplicateBoundMiner
			);
			ensure!(
				Self::ensure_worker_bound(&pubkey).is_err(),
				Error::<T>::DuplicateBoundMiner
			);

			MinerBindings::<T>::insert(&miner, &pubkey);
			WorkerBindings::<T>::insert(&pubkey, &miner);

			Miners::<T>::insert(
				&miner,
				MinerInfo {
					state: MinerState::Ready,
					ve: 0,
					v: 0,
					v_updated_at: now,
					p_instant: 0u64,
					benchmark: Benchmark {
						iterations: 0u64,
						mining_start_time: now,
					},
					cool_down_start: 0u64,
				},
			);

			Self::deposit_event(Event::<T>::MinerBound(miner, pubkey));
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
				Self::stop_mining(miner.clone())?;
				// TODO: consider the final state sync (could cause slash) when stopping mining
			}
			MinerBindings::<T>::remove(miner);
			WorkerBindings::<T>::remove(&worker);
			Self::deposit_event(Event::<T>::MinerUnbound(miner.clone(), worker.clone()));
			if notify {
				T::OnUnbound::on_unbound(&worker, force);
			}

			Ok(())
		}

		/// Starts mining with the given `stake`, assuming the stake is already locked externally
		pub fn start_mining(miner: T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
			let worker = MinerBindings::<T>::get(&miner).ok_or(Error::<T>::MinerNotFound)?;

			ensure!(
				Miners::<T>::get(&miner).unwrap().state == MinerState::Ready,
				Error::<T>::MinerNotReady
			);

			let worker_info =
				registry::Workers::<T>::get(&worker).expect("Bounded worker must exist; qed.");
			let p = worker_info
				.initial_score
				.ok_or(Error::<T>::BenchmarkMissing)?;

			let tokenomic = Tokenomic::<T>::new(
				TokenomicParameters::<T>::get().expect("TokenomicParameters must exist; qed."),
			);
			let min_stake = tokenomic.minimal_stake(p);
			ensure!(stake >= min_stake, Error::<T>::InsufficientStake);

			let ve = tokenomic.ve(stake, p, worker_info.confidence_level);
			let v_max = tokenomic.v_max();
			ensure!(ve <= v_max, Error::<T>::TooMuchStake);

			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();

			Stakes::<T>::insert(&miner, stake);
			Miners::<T>::mutate(&miner, |info| {
				if let Some(info) = info {
					info.state = MinerState::MiningIdle;
					info.ve = ve.to_bits();
					info.v = ve.to_bits();
					info.v_updated_at = now;
				}
			});
			OnlineMiners::<T>::mutate(|v| *v += 1);

			let session_id = NextSessionId::<T>::get();
			NextSessionId::<T>::put(session_id + 1);
			Self::push_message(SystemEvent::new_worker_event(
				worker,
				WorkerEvent::MiningStart {
					session_id: session_id,
					init_v: ve.to_bits(),
				},
			));
			Self::deposit_event(Event::<T>::MinerStarted(miner));
			Ok(())
		}

		/// Stops mining, enterying cool down state
		///
		/// Requires:
		/// 1. Ther miner is in Idle, MiningActive, or MiningUnresponsive state
		pub fn stop_mining(miner: T::AccountId) -> DispatchResult {
			let worker = MinerBindings::<T>::get(&miner).ok_or(Error::<T>::MinerNotBound)?;
			let mut miner_info = Miners::<T>::get(&miner).ok_or(Error::<T>::MinerNotFound)?;

			ensure!(
				miner_info.state != MinerState::Ready
					&& miner_info.state != MinerState::MiningCoolingDown,
				Error::<T>::MinerNotMining
			);

			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			miner_info.state = MinerState::MiningCoolingDown;
			miner_info.cool_down_start = now;
			Miners::<T>::insert(&miner, &miner_info);
			OnlineMiners::<T>::mutate(|v| *v -= 1); // v cannot be 0

			Self::push_message(SystemEvent::new_worker_event(
				worker,
				WorkerEvent::MiningStop,
			));
			Self::deposit_event(Event::<T>::MinerStopped(miner));
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

		#[allow(unused)]
		fn update_tokenomic_parameters(params: TokenomicParams) {
			TokenomicParameters::<T>::put(params.clone());
			Self::push_message(GatekeeperEvent::TokenomicParametersChanged(params));
		}

		pub fn withdraw_subsidy_pool(target: &T::AccountId, value: BalanceOf<T>) -> DispatchResult {
			let wallet = Self::account_id();
			T::Currency::transfer(&wallet, &target, value, KeepAlive)
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

		/// Calcuates the initial Ve
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
			(cost_k * p + cost_b) / pha_rate
		}

		/// Gets the operating cost per sec
		#[cfg(test)]
		fn op_cost(&self, p: u32) -> FixedPoint {
			let cost_k = FixedPoint::from_bits(self.params.cost_k);
			let cost_b = FixedPoint::from_bits(self.params.cost_b);
			let pha_rate = FixedPoint::from_bits(self.params.pha_rate);
			let p = FixedPoint::from_num(p);
			(cost_k * p + cost_b) / pha_rate
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
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub tokenomic_parameters: TokenomicParams,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		/// Default tokenoic parameters for Phala
		fn default() -> Self {
			use fixed_macro::types::U64F64 as fp;
			let pha_rate = fp!(1);
			let rho = fp!(1.00000099985); // hourly: 1.00020,  1.0002 ** (1/300)
			let slash_rate = fp!(0.001) / 300; // hourly rate: 0.001, convert to per-block rate
			let budget_per_sec = fp!(720000) / 24 / 3600;
			let v_max = fp!(30000);
			let cost_k = fp!(0.0415625) / 3600 / 24 / 365; // annual 0.0415625, convert to per sec
			let cost_b = fp!(88.59375) / 3600 / 24 / 365; // annual 88.59375, convert to per sec
			let heartbeat_window = 10; // 10 blocks
			let rig_k = fp!(0.3);
			let rig_b = fp!(0);
			let re = fp!(1.5);
			let k = fp!(100);

			Self {
				tokenomic_parameters: TokenomicParams {
					pha_rate: pha_rate.to_bits(),
					rho: rho.to_bits(),
					budget_per_sec: budget_per_sec.to_bits(),
					v_max: v_max.to_bits(),
					cost_k: cost_k.to_bits(),
					cost_b: cost_b.to_bits(),
					slash_rate: slash_rate.to_bits(),
					heartbeat_window: 10,
					rig_k: rig_k.to_bits(),
					rig_b: rig_b.to_bits(),
					re: re.to_bits(),
					k: k.to_bits(),
				},
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
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
			new_test_ext, set_block_1, setup_workers, take_events, take_messages, worker_pubkey,
			Event as TestEvent, Origin, Test, DOLLARS,
		};
		// Pallets
		use crate::mock::{PhalaMining, System};

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
						TestEvent::PhalaMining(Event::MinerBound(1, worker_pubkey(1))),
						TestEvent::PhalaMining(Event::MinerUnbound(1, worker_pubkey(1)))
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
					Error::<Test>::DuplicateBoundMiner
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
					fp!(1950.00000000000000001626)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 2),
					fp!(1950.00000000000000001626)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 3),
					fp!(1950.00000000000000001626)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 4),
					fp!(1819.99999999999999998694)
				);
				assert_eq!(
					tokenomic.ve(1000 * DOLLARS, 1000, 5),
					fp!(1754.9999999999999999723)
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
				// Budget per day
				let budger_per_sec = FixedPoint::from_bits(tokenomic.params.budget_per_sec);
				assert_eq!(budger_per_sec * 3600 * 24, fp!(719999.99999999999999843875));
				// Cost estimation per year
				assert_eq!(
					tokenomic.op_cost(2000) * 3600 * 24 * 365,
					fp!(171.71874999890369452304)
				);
			});
		}
	}
}
