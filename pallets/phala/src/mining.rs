pub use self::pallet::*;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use crate::mq::{self, MessageOriginInfo};
	use crate::registry;
	use crate::stakepool;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, EnsureOrigin, Randomness, UnixTime},
	};
	use frame_system::pallet_prelude::*;
	use phala_types::{
		messaging::{
			DecodedMessage, HeartbeatChallenge, MessageOrigin, MiningInfoUpdateEvent, SettleInfo,
			SystemEvent, WorkerEvent,
		},
		WorkerPublicKey,
	};
	use sp_core::U256;
	use sp_runtime::{
		traits::{Saturating, Zero},
		SaturatedConversion,
	};
	use sp_std::cmp;
	use sp_std::vec::Vec;

	const DEFAULT_EXPECTED_HEARTBEAT_COUNT: u32 = 10;
	const INITIAL_V: u64 = 1;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub enum MinerState {
		Ready,
		MiningIdle,
		MiningActive,
		MiningUnresponsive,
		MiningCoolingDown,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct Benchmark {
		iterations: u64,
		mining_start_time: u64,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct MinerInfo {
		state: MinerState,
		ve: u64,
		v: u64,
		v_updated_at: u64,
		p_instant: u64,
		benchmark: Benchmark,
		cooling_down_start: u64,
	}

	pub trait OnReward {
		fn on_reward(settle: &Vec<SettleInfo>) {}
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
		type PoolOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
		type MinStaking: Get<BalanceOf<Self>>;
		type OnReward: OnReward;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Total online miners
	#[pallet::storage]
	pub type OnlineMiners<T> = StorageValue<_, u32>;

	/// The expected heartbeat count (default: 20)
	#[pallet::storage]
	pub type ExpectedHeartbeatCount<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn miner)]
	pub(super) type Miner<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, MinerInfo>;

	#[pallet::storage]
	#[pallet::getter(fn miner_binding)]
	pub(super) type MinerBinding<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, WorkerPublicKey>;

	#[pallet::storage]
	#[pallet::getter(fn worker_binding)]
	pub(super) type WorkerBinding<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn worker_stats)]
	pub(super) type WorkerStats<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, WorkerStat<BalanceOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn coolingdown_expire)]
	pub(super) type CoolingDownExpire<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn mining_sessionid)]
	pub(super) type MiningSessionId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn deposit_balance)]
	pub(super) type DepositBalance<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn reward_balance)]
	pub(super) type RewardBalance<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Meh. [n]
		Meh(u32),
		/// [period]
		CoplingDownExpireChanged(u64),
		/// [miner]
		MiningStarted(T::AccountId),
		/// [miner]
		MiningStoped(T::AccountId),
		/// [miner]
		MiningCleanup(T::AccountId),
		/// [miner, worker]
		MinerBounded(T::AccountId, WorkerPublicKey),
		/// [miner]
		MinerEnterUnresponsive(T::AccountId),
		/// [miner]
		MinerExitUnresponive(T::AccountId),
		/// [miner, amount]
		MinerDeposited(T::AccountId, BalanceOf<T>),
		/// [miner, amount]
		MinerWithdrawed(T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		BadSender,
		InvalidMessage,
		WorkerNotRegistered,
		GatekeeperNotRegistered,
		DuplicatedBoundedMiner,
		BenchmarkMissing,
		MinerNotFounded,
		MinerNotBounded,
		MinerNotInReadyState,
		MinerNotMining,
		CoolingDownNotPassed,
		InsufficientStaking,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn sudo_set_coolingdownexpire(origin: OriginFor<T>, period: u64) -> DispatchResult {
			ensure_root(origin)?;

			CoolingDownExpire::<T>::mutate(|p| *p = period);
			Self::deposit_event(Event::<T>::CoplingDownExpireChanged(period));
			Ok(())
		}

		/// Binding miner with worker
		///
		/// Requires:
		/// 1. Ther worker is alerady registered
		#[pallet::weight(0)]
		pub fn bind(
			origin: OriginFor<T>,
			miner: T::AccountId,
			pubkey: WorkerPublicKey,
		) -> DispatchResult {
			T::PoolOrigin::ensure_origin(origin)?;

			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();

			ensure!(
				registry::Worker::<T>::contains_key(&pubkey),
				Error::<T>::WorkerNotRegistered
			);

			ensure!(
				!MinerBinding::<T>::contains_key(&miner),
				Error::<T>::DuplicatedBoundedMiner
			);
			ensure!(
				!WorkerBinding::<T>::contains_key(&pubkey),
				Error::<T>::DuplicatedBoundedMiner
			);

			MinerBinding::<T>::insert(&miner, &pubkey);
			WorkerBinding::<T>::insert(&pubkey, &miner);

			Miner::<T>::insert(
				&miner,
				MinerInfo {
					state: MinerState::Ready,
					ve: 0u64,
					v: 0u64,
					v_updated_at: now,
					p_instant: 0u64,
					benchmark: Benchmark {
						iterations: 0u64,
						mining_start_time: now,
					},
					cooling_down_start: 0u64,
				},
			);

			Self::deposit_event(Event::<T>::MinerBounded(miner, pubkey));
			Ok(())
		}

		/// Desposits some tokan as stake
		///
		/// Requires:
		/// 1. Ther miner is in Ready state
		#[pallet::weight(0)]
		pub fn deposit(
			origin: OriginFor<T>,
			miner: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			T::PoolOrigin::ensure_origin(origin)?;

			ensure!(
				MinerBinding::<T>::contains_key(&miner),
				Error::<T>::MinerNotFounded
			);
			ensure!(
				Miner::<T>::get(&miner).unwrap().state == MinerState::Ready,
				Error::<T>::MinerNotInReadyState
			);

			let already_reserved = DepositBalance::<T>::get(&miner).unwrap_or_default();
			DepositBalance::<T>::insert(&miner, already_reserved.saturating_add(amount));

			Ok(())
		}

		/// Withdraw some token from the stake
		///
		/// Requires:
		/// 1. Ther miner is in Ready state
		#[pallet::weight(0)]
		pub fn withdraw(
			origin: OriginFor<T>,
			miner: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			T::PoolOrigin::ensure_origin(origin)?;

			ensure!(
				MinerBinding::<T>::contains_key(&miner),
				Error::<T>::MinerNotFounded
			);
			// mining should be stopped before withdraw
			ensure!(
				Miner::<T>::get(&miner).unwrap().state == MinerState::Ready,
				Error::<T>::MinerNotInReadyState
			);

			let already_reserved = DepositBalance::<T>::get(&miner).unwrap_or_default();
			DepositBalance::<T>::insert(&miner, already_reserved.saturating_sub(amount));

			Ok(())
		}

		/// Starts mining
		///
		/// Requires:
		/// 1. Ther miner is in Ready state
		#[pallet::weight(0)]
		pub fn start_mining(origin: OriginFor<T>) -> DispatchResult {
			let miner = ensure_signed(origin)?;
			let worker = MinerBinding::<T>::get(&miner).ok_or(Error::<T>::MinerNotFounded)?;

			ensure!(
				Miner::<T>::get(&miner).unwrap().state == MinerState::Ready,
				Error::<T>::MinerNotInReadyState
			);

			let worker_info =
				registry::Worker::<T>::get(&worker).expect("Bounded worker must exist; qed.");
			ensure!(
				worker_info.intial_score != None,
				Error::<T>::BenchmarkMissing
			);

			let already_reserved = DepositBalance::<T>::get(&miner).unwrap_or_default();
			ensure!(
				// TODO: dynamic compute MinStaking according to worker
				already_reserved >= T::MinStaking::get(),
				Error::<T>::InsufficientStaking
			);

			Miner::<T>::mutate(&miner, |info| {
				if let Some(info) = info {
					info.state = MinerState::MiningIdle;
				}
			});

			let session_id = MiningSessionId::<T>::get();
			MiningSessionId::<T>::put(session_id + 1);
			Self::push_message(SystemEvent::new_worker_event(
				worker,
				WorkerEvent::MiningStart {
					session_id: session_id,
					init_v: INITIAL_V as _,
				},
			));
			Self::deposit_event(Event::<T>::MiningStarted(miner));
			Ok(())
		}

		/// Stops mining, enterying cooling down state
		///
		/// Requires:
		/// 1. Ther miner is in Idle, MiningActive, or MiningUnresponsive state
		#[pallet::weight(0)]
		pub fn stop_mining(origin: OriginFor<T>) -> DispatchResult {
			let miner = ensure_signed(origin)?;
			let worker = MinerBinding::<T>::get(&miner).ok_or(Error::<T>::MinerNotBounded)?;
			let mut miner_info = Miner::<T>::get(&miner).ok_or(Error::<T>::MinerNotFounded)?;

			ensure!(
				miner_info.state != MinerState::Ready
					&& miner_info.state != MinerState::MiningCoolingDown,
				Error::<T>::MinerNotMining
			);

			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			miner_info.state = MinerState::MiningCoolingDown;
			miner_info.cooling_down_start = now;
			Miner::<T>::insert(&miner, &miner_info);

			Self::push_message(SystemEvent::new_worker_event(
				worker,
				WorkerEvent::MiningStop,
			));
			Self::deposit_event(Event::<T>::MiningStoped(miner));
			Ok(())
		}

		/// Turns the miner back to Ready state after cooling down
		///
		/// Requires:
		/// 1. Ther miner is in CoolingDown state and the cooling down period has passed
		#[pallet::weight(0)]
		pub fn cleanup(origin: OriginFor<T>) -> DispatchResult {
			let miner = ensure_signed(origin)?;
			let mut miner_info = Miner::<T>::get(&miner).ok_or(Error::<T>::MinerNotFounded)?;
			ensure!(
				Self::can_cleanup(&miner_info),
				Error::<T>::CoolingDownNotPassed
			);
			miner_info.state = MinerState::Ready;
			miner_info.cooling_down_start = 0u64;
			Miner::<T>::insert(&miner, &miner_info);

			// clear deposit balance
			DepositBalance::<T>::insert(&miner, BalanceOf::<T>::zero());

			Self::deposit_event(Event::<T>::MiningCleanup(miner));
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
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {
			Self::heartbeat_challenge();
		}
	}

	// - Properly handle heartbeat message.
	impl<T: Config> Pallet<T> {
		fn heartbeat_challenge() {
			// Random seed for the heartbeat challenge
			let seed_hash = T::Randomness::random(crate::constants::RANDOMNESS_SUBJECT).0;
			let seed: U256 = AsRef::<[u8]>::as_ref(&seed_hash).into();
			// PoW target for the random sampling
			let online_miners = OnlineMiners::<T>::get().unwrap_or(0);
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

				// worker offline, update bounded miner state to unresponsive
				for worker in event.offline {
					if let Some(binding_miner) = WorkerBinding::<T>::get(&worker) {
						let mut miner_info =
							Self::miner(&binding_miner).ok_or(Error::<T>::MinerNotFounded)?;
						miner_info.state = MinerState::MiningUnresponsive;
						Miner::<T>::insert(&binding_miner, &miner_info);
						Self::deposit_event(Event::<T>::MinerEnterUnresponsive(binding_miner));
					}
				}

				// worker recovered to online, update bounded miner state to idle
				for worker in event.recovered_to_online {
					if let Some(binding_miner) = WorkerBinding::<T>::get(&worker) {
						let mut miner_info =
							Self::miner(&binding_miner).ok_or(Error::<T>::MinerNotFounded)?;
						miner_info.state = MinerState::MiningIdle;
						Miner::<T>::insert(&binding_miner, &miner_info);
						Self::deposit_event(Event::<T>::MinerExitUnresponive(binding_miner));
					}
				}

				for info in event.settle.clone() {
					if let Some(binding_miner) = WorkerBinding::<T>::get(&info.pubkey) {
						let mut miner_info =
							Self::miner(&binding_miner).ok_or(Error::<T>::MinerNotFounded)?;
						miner_info.v = info.v as _; //TODO(wenfeng)
						miner_info.v_updated_at = now;
						Miner::<T>::insert(&binding_miner, &miner_info);

						let reward_reserved =
							RewardBalance::<T>::get(&binding_miner).unwrap_or_default();
						RewardBalance::<T>::insert(
							&binding_miner,
							reward_reserved.saturating_add(info.payout.saturated_into()),
						);
					}
				}

				T::OnReward::on_reward(&event.settle);
			}

			Ok(())
		}

		fn can_cleanup(miner_info: &MinerInfo) -> bool {
			if miner_info.state != MinerState::MiningCoolingDown {
				return false;
			}

			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			if (now - miner_info.cooling_down_start) > Self::coolingdown_expire() {
				true
			} else {
				false
			}
		}
	}

	fn pow_target(num_tx: u32, num_workers: u32, secs_per_block: u32) -> U256 {
		use substrate_fixed::types::U32F32;
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
		use crate::mock::{events, new_test_ext, set_block_1, Event as TestEvent, Test};

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
				let ev = events();
				let message = match ev.as_slice() {
					[TestEvent::PhalaMq(mq::Event::OutboundMessage(m))] => m,
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
	}
}
