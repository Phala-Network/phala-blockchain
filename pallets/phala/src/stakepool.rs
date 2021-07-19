pub use self::pallet::*;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use crate::mining;
	use crate::registry;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			Currency, EnsureOrigin, LockIdentifier, LockableCurrency, UnixTime, WithdrawReasons,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	use phala_types::{messaging::SettleInfo, WorkerPublicKey};
	use sp_runtime::{
		traits::{AccountIdConversion, Saturating, TrailingZeroInput, Zero},
		SaturatedConversion,
	};
	use sp_std::collections::vec_deque::VecDeque;
	use sp_std::vec;
	use sp_std::vec::Vec;

	const STAKEPOOL_PALLETID: PalletId = PalletId(*b"phala/sp");
	const STAKING_ID: LockIdentifier = *b"phala/sp";

	#[pallet::config]
	pub trait Config: frame_system::Config + registry::Config + mining::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		type MinDeposit: Get<BalanceOf<Self>>;
		type InsurancePeriod: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Mapping from pool id to PoolInfo
	#[pallet::storage]
	#[pallet::getter(fn mining_pools)]
	pub(super) type MiningPools<T: Config> =
		StorageMap<_, Twox64Concat, u64, PoolInfo<T::AccountId, BalanceOf<T>>>;

	/// Mapping pool to it's UserStakeInfo
	#[pallet::storage]
	#[pallet::getter(fn staking_info)]
	pub(super) type StakingInfo<T: Config> =
		StorageMap<_, Twox64Concat, (u64, T::AccountId), UserStakeInfo<T::AccountId, BalanceOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn pool_count)]
	pub(super) type PoolCount<T> = StorageValue<_, u64, ValueQuery>;

	/// Mapping worker to it's new rewards
	#[pallet::storage]
	#[pallet::getter(fn new_rewards)]
	pub(super) type NewRewards<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, BalanceOf<T>>;

	/// Mapping worker to the pool it belongs to
	#[pallet::storage]
	pub(super) type WorkerInPool<T: Config> = StorageMap<_, Twox64Concat, WorkerPublicKey, u64>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Meh. [n]
		Meh(u32),
		/// [owner, pid]
		PoolCreated(T::AccountId, u64),
		/// [pid, commission]
		PoolCommissionSetted(u64, u16),
		/// [pid, cap]
		PoolCapacitySetted(u64, BalanceOf<T>),
		/// [pid, worker]
		PoolWorkerAdded(u64, WorkerPublicKey),
		/// [pid, user, amount]
		Deposit(u64, T::AccountId, BalanceOf<T>),
		/// [pid, user, amount]
		Withdraw(u64, T::AccountId, BalanceOf<T>),
		/// [pid, user, amount]
		WithdrawRewards(u64, T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		Meh,
		WorkerNotRegistered,
		BenchmarkMissing,
		WorkerHasAdded,
		WorkerHasNotAdded,
		UnauthorizedOperator,
		UnauthorizedPoolOwner,
		InvalidPayoutPerf,
		InvalidCapacity,
		StakeExceedCapacity,
		PoolNotExist,
		PoolIsBusy,
		LessthanMinDeposit,
		InsufficientBalance,
		StakeInfoNotFound,
		InsufficientStake,
		InvalidWithdrawAmount,
		StartMiningCallFailed,
		MinerBindingCallFailed,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();

			// TODO:
			// 1) should we just stop some of workers rather than all of it
			// 2) just iterate pools that contains waitting withdraw rather than all of it
			for pid in 0..PoolCount::<T>::get() {
				let pool_info = Self::ensure_pool(pid).expect("Stake pool doesn't exist; qed.");
				if !pool_info.withdraw_queue.is_empty() {
					// the front withdraw always the oldest one
					if let Some(info) = pool_info.withdraw_queue.front() {
						if (now - info.start_time)
							> T::InsurancePeriod::get().saturated_into::<u64>()
						{
							// stop all worker in this pool
							// TODO: only stop running workers?
							for worker in pool_info.workers {
								let miner: T::AccountId = pool_sub_account(pid, &worker);
								let _ = <mining::pallet::Pallet<T>>::stop_mining(miner);
							}
						}
					}
				}
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
	{
		/// Creates a new stake pool
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let pid = PoolCount::<T>::get();
			MiningPools::<T>::insert(
				pid,
				PoolInfo {
					pid: pid,
					owner: owner.clone(),
					payout_commission: Zero::zero(),
					owner_reward: Zero::zero(),
					cap: None,
					pool_acc: Zero::zero(),
					total_stake: Zero::zero(),
					free_stake: Zero::zero(),
					workers: vec![],
					withdraw_queue: VecDeque::new(),
				},
			);
			PoolCount::<T>::put(pid + 1);
			Self::deposit_event(Event::<T>::PoolCreated(owner, pid));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn add_worker(
			origin: OriginFor<T>,
			pid: u64,
			pubkey: WorkerPublicKey,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let worker_info =
				registry::Worker::<T>::get(&pubkey).ok_or(Error::<T>::WorkerNotRegistered)?;

			// check wheather the owner was bounded as operator
			ensure!(
				worker_info.operator == Some(owner.clone()),
				Error::<T>::UnauthorizedOperator
			);
			// check the worker has finished the benchmark
			ensure!(
				worker_info.intial_score != None,
				Error::<T>::BenchmarkMissing
			);

			// origin must be owner of pool
			let mut pool_info = Self::ensure_pool(pid)?;
			ensure!(pool_info.owner == owner, Error::<T>::UnauthorizedPoolOwner);
			// make sure worker has not been not added
			// TODO: should we set a cap to avoid performance problem
			let workers = &mut pool_info.workers;
			// TODO: limite the number of workers to avoid performance issue.
			ensure!(!workers.contains(&pubkey), Error::<T>::WorkerHasAdded);

			// generate miner account
			let miner: T::AccountId = pool_sub_account(pid, &pubkey);

			// bind worker with minner
			match <mining::pallet::Pallet<T>>::bind(miner.clone(), pubkey.clone()) {
				Ok(()) => {
					// update worker vector
					workers.push(pubkey.clone());
					MiningPools::<T>::insert(&pid, &pool_info);
					NewRewards::<T>::insert(&pubkey, BalanceOf::<T>::zero());
					WorkerInPool::<T>::insert(&pubkey, pid);
					Self::deposit_event(Event::<T>::PoolWorkerAdded(pid, pubkey));
				}
				_ => {
					return Err(Error::<T>::MinerBindingCallFailed.into());
				}
			}

			Ok(())
		}

		/// Destroies a stake pool
		///
		/// Requires:
		/// 1. The sender is the owner
		/// 2. All the miners are stopped
		#[pallet::weight(0)]
		pub fn destroy(origin: OriginFor<T>, id: u64) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Sets the hard cap of the pool
		/// Note: a smaller cap than current total_stake if not allowed.
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::weight(0)]
		pub fn set_cap(origin: OriginFor<T>, pid: u64, cap: BalanceOf<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut pool_info = Self::ensure_pool(pid)?;

			// origin must be owner of pool
			ensure!(pool_info.owner == owner, Error::<T>::UnauthorizedPoolOwner);
			// check cap
			ensure!(pool_info.total_stake <= cap, Error::<T>::InvalidCapacity);

			pool_info.cap = Some(cap);
			MiningPools::<T>::insert(&pid, &pool_info);

			Self::deposit_event(Event::<T>::PoolCapacitySetted(pid, cap));
			Ok(())
		}

		/// Change the pool commission rate
		///
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::weight(0)]
		pub fn set_payout_pref(
			origin: OriginFor<T>,
			pid: u64,
			payout_commission: u16,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			ensure!(payout_commission <= 1000, Error::<T>::InvalidPayoutPerf);
			let mut pool_info = Self::ensure_pool(pid)?;
			// origin must be owner of pool
			ensure!(pool_info.owner == owner, Error::<T>::UnauthorizedPoolOwner);

			pool_info.payout_commission = payout_commission;
			MiningPools::<T>::insert(&pid, &pool_info);

			Self::deposit_event(Event::<T>::PoolCommissionSetted(pid, payout_commission));

			Ok(())
		}

		/// Change the payout perference of a stake pool
		///
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::weight(0)]
		pub fn claim_reward(
			origin: OriginFor<T>,
			pid: u64,
			target: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let info_key = (pid.clone(), who.clone());
			let mut user_info =
				Self::staking_info(&info_key).ok_or(Error::<T>::StakeInfoNotFound)?;
			let mut pool_info = Self::ensure_pool(pid)?;

			// update pool
			Self::update_pool(&mut pool_info);

			// rewards belong to user, including pending rewards and available_rewards
			let rewards = user_info.available_rewards.saturating_add(
				user_info.amount * pool_info.pool_acc / 10u32.pow(6).into() - user_info.user_debt,
			);

			// send reward to user
			<T as Config>::Currency::deposit_into_existing(&who.clone(), rewards.clone())?;

			user_info.user_debt = user_info.amount * pool_info.pool_acc / 10u32.pow(6).into();
			user_info.available_rewards = Zero::zero();
			StakingInfo::<T>::insert(&info_key, &user_info);
			MiningPools::<T>::insert(&pool_info.pid, &pool_info);
			Self::deposit_event(Event::<T>::WithdrawRewards(pid, who, rewards));

			Ok(())
		}

		/// Deposits some funds to a pool
		///
		/// Requires:
		/// 1. The pool exists
		/// 2. After the desposit, the pool doesn't reach the cap
		#[pallet::weight(0)]
		pub fn deposit(origin: OriginFor<T>, pid: u64, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				amount.clone() >= T::MinDeposit::get(),
				Error::<T>::LessthanMinDeposit
			);
			ensure!(
				<T as Config>::Currency::free_balance(&who) >= amount.clone(),
				Error::<T>::InsufficientBalance
			);

			let mut pool_info = Self::ensure_pool(pid)?;
			if let Some(cap) = pool_info.cap {
				ensure!(
					cap.saturating_sub(pool_info.total_stake) >= amount,
					Error::<T>::StakeExceedCapacity
				);
			}

			Self::update_pool(&mut pool_info);

			let info_key = (pid.clone(), who.clone());
			if StakingInfo::<T>::contains_key(&info_key) {
				let mut user_info = Self::staking_info(&info_key).unwrap();
				let pending_rewards = user_info.amount * pool_info.pool_acc
					/ 10u32.pow(6).saturated_into()
					- user_info.user_debt;
				if pending_rewards > Zero::zero() {
					user_info.available_rewards =
						user_info.available_rewards.saturating_add(pending_rewards);
				}

				user_info.amount = user_info.amount.saturating_add(amount.clone());
				user_info.user_debt =
					user_info.amount * pool_info.pool_acc / 10u32.pow(6).saturated_into();

				StakingInfo::<T>::insert(&info_key, &user_info);
			} else {
				// first time deposit to this pool
				StakingInfo::<T>::insert(
					&info_key,
					UserStakeInfo {
						user: who.clone(),
						amount: amount.clone(),
						available_rewards: Zero::zero(),
						user_debt: amount.clone() * pool_info.pool_acc
							/ 10u32.pow(6).saturated_into(),
					},
				);
			}

			<T as Config>::Currency::set_lock(
				STAKING_ID,
				&who,
				amount.clone(),
				WithdrawReasons::all(),
			);

			pool_info.total_stake = pool_info.total_stake.saturating_add(amount.clone());
			pool_info.free_stake = pool_info.free_stake.saturating_add(amount.clone());

			// we have new free stake now, try handle the waitting withdraw queue
			Self::try_handle_waiting_withdraw(&mut pool_info);

			MiningPools::<T>::insert(&pid, &pool_info);

			Self::deposit_event(Event::<T>::Deposit(pid, who, amount));

			Ok(())
		}

		/// Deposits some funds from a pool
		/// Note: there are two scenarios people may meet:
		///     if the pool has free stake and and amount of the free stake greater or equal than withdraw amount
		///     (e.g. pool.free_stake >= amount), the withdraw would take effect immediately.
		///     else the withdraw would be queued and delay untill there have enough free stake in the pool.
		/// Requires:
		/// 1. The pool exists
		#[pallet::weight(0)]
		pub fn withdraw(origin: OriginFor<T>, pid: u64, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let info_key = (pid.clone(), who.clone());
			let mut user_info =
				Self::staking_info(&info_key).ok_or(Error::<T>::StakeInfoNotFound)?;

			ensure!(
				amount > Zero::zero() && user_info.amount >= amount,
				Error::<T>::InvalidWithdrawAmount
			);

			let mut pool_info = Self::ensure_pool(pid)?;
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();

			// if withdraw_queue is not empty, means pool doesn't have free stake now, just add withdraw to queue
			if !pool_info.withdraw_queue.is_empty() {
				pool_info.withdraw_queue.push_back(WithdrawInfo {
					user: who.clone(),
					amount: amount.clone(),
					start_time: now,
				});
			} else {
				Self::try_withdraw(&mut pool_info, &mut user_info, amount.clone());
			}

			StakingInfo::<T>::insert(&info_key, &user_info);
			MiningPools::<T>::insert(&pid, &pool_info);

			Ok(())
		}

		/// Starts a miner on behalf of the stake pool
		///
		/// Requires:
		/// 1. The miner is bounded to the pool and is in Ready state
		/// 2. The remaining stake in the pool can cover the minimal stake requried
		#[pallet::weight(0)]
		pub fn start_mining(
			origin: OriginFor<T>,
			pid: u64,
			worker: WorkerPublicKey,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut pool_info = Self::ensure_pool(pid)?;
			// origin must be owner of pool
			ensure!(pool_info.owner == owner, Error::<T>::UnauthorizedPoolOwner);
			// check free stake
			ensure!(pool_info.free_stake >= stake, Error::<T>::InsufficientStake);
			// check wheather we have add this worker
			ensure!(
				pool_info.workers.contains(&worker),
				Error::<T>::WorkerHasNotAdded
			);
			let miner: T::AccountId = pool_sub_account(pid, &worker);
			<mining::pallet::Pallet<T>>::set_deposit(&miner, stake);
			match <mining::pallet::Pallet<T>>::start_mining(miner.clone()) {
				Ok(()) => {
					pool_info.free_stake = pool_info.free_stake.saturating_sub(stake);
					MiningPools::<T>::insert(&pid, &pool_info);
				}
				_ => {
					// rollback
					<mining::pallet::Pallet<T>>::set_deposit(&miner, Zero::zero());
					return Err(Error::<T>::StartMiningCallFailed.into());
				}
			}

			Ok(())
		}

		/// Stops a miner on behalf of the stake pool
		/// Note: this would let miner enter coolingdown if everything is good
		///
		/// Requires:
		/// 1. There miner is bounded to the pool and is in a stoppable state
		#[pallet::weight(0)]
		pub fn stop_mining(
			origin: OriginFor<T>,
			pid: u64,
			worker: WorkerPublicKey,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pool_info = Self::ensure_pool(pid)?;
			// origin must be owner of pool
			ensure!(pool_info.owner == owner, Error::<T>::UnauthorizedPoolOwner);
			// check wheather we have add this worker
			ensure!(
				pool_info.workers.contains(&worker),
				Error::<T>::WorkerHasNotAdded
			);
			let miner: T::AccountId = pool_sub_account(pid, &worker);
			<mining::pallet::Pallet<T>>::stop_mining(miner)?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			STAKEPOOL_PALLETID.into_account()
		}

		/// Query rewards of user in a specific pool
		pub fn pending_rewards(pid: u64, who: T::AccountId) -> BalanceOf<T> {
			let info_key = (pid.clone(), who.clone());
			let user_info = Self::staking_info(&info_key).expect("Stake info doesn't exist; qed.");
			let pool_info = Self::ensure_pool(pid).expect("Stake pool doesn't exist; qed.");

			// rewards belong to user, including pending rewards and available_rewards
			let rewards = user_info.available_rewards.saturating_add(
				user_info.amount * pool_info.pool_acc / 10u32.pow(6).into() - user_info.user_debt,
			);

			return rewards;
		}

		fn update_pool(pool_info: &mut PoolInfo<T::AccountId, BalanceOf<T>>) {
			let mut new_rewards;

			new_rewards = Self::calculate_reward(&pool_info.workers);
			Self::reward_clear(&pool_info.workers);

			if new_rewards > Zero::zero() && pool_info.total_stake > Zero::zero() {
				pool_info.owner_reward = pool_info.owner_reward.saturating_add(
					new_rewards * pool_info.payout_commission.into() / 1000u32.into(),
				);

				new_rewards =
					new_rewards * (1000 - pool_info.payout_commission).into() / 1000u32.into();
				pool_info.pool_acc = pool_info
					.pool_acc
					.saturating_add(new_rewards * 10u32.pow(6).into() / pool_info.total_stake);
			}
		}

		/// Calculate rewards that belong to this pool.
		/// The rewards here only contains rewards belong to workers in this pool from last pool update
		/// to now. Everytime mining tell us some workers have new rewards(by on_reward callback), we
		/// add it to new_rewards map, when next time the pool do update, we calculate all rewards
		/// belong to this pool, which would be used to update pool_info.pool_acc, then the cached rewards
		/// would be clean.
		fn calculate_reward(workers: &Vec<WorkerPublicKey>) -> BalanceOf<T> {
			let mut pool_new_rewards: BalanceOf<T> = Zero::zero();
			for worker in workers {
				pool_new_rewards =
					pool_new_rewards.saturating_add(Self::new_rewards(&worker).unwrap());
			}
			return pool_new_rewards;
		}

		/// Clear specific miner's reward, only current round rewards can be used to calculate
		/// pool arguments.
		fn reward_clear(workers: &Vec<WorkerPublicKey>) {
			for worker in workers {
				NewRewards::<T>::insert(&worker, BalanceOf::<T>::zero());
			}
		}

		/// Try to withdraw specific amount from pool, would be delayed if the free stake is not enough.
		fn try_withdraw(
			pool_info: &mut PoolInfo<T::AccountId, BalanceOf<T>>,
			user_info: &mut UserStakeInfo<T::AccountId, BalanceOf<T>>,
			amount: BalanceOf<T>,
		) {
			Self::update_pool(pool_info);

			// enough free stake, withdraw directly
			if pool_info.free_stake >= amount {
				pool_info.free_stake = pool_info.free_stake.saturating_sub(amount);
				pool_info.total_stake = pool_info.total_stake.saturating_sub(amount);
				user_info.amount = user_info.amount.saturating_sub(amount);
			} else {
				let now = <T as registry::Config>::UnixTime::now()
					.as_secs()
					.saturated_into::<u64>();
				// all of the free_stake would be withdrew back to user
				let unwithdraw_amount = amount.saturating_sub(pool_info.free_stake);
				pool_info.total_stake = pool_info.total_stake.saturating_sub(pool_info.free_stake);
				user_info.amount = user_info.amount.saturating_sub(unwithdraw_amount);
				pool_info.free_stake = Zero::zero();

				// case some locked asset has not been withdraw(unlock) to user, add it to withdraw queue.
				// when pool has free stake again, the withdraw would be handled
				pool_info.withdraw_queue.push_back(WithdrawInfo {
					user: user_info.user.clone(),
					amount: unwithdraw_amount,
					start_time: now,
				});
			}
			user_info.user_debt =
				user_info.amount * pool_info.pool_acc / 10u32.pow(6).saturated_into();

			// update the lock balance of user
			Self::update_lock(user_info.user.clone(), user_info.amount);
		}

		fn try_handle_waiting_withdraw(pool_info: &mut PoolInfo<T::AccountId, BalanceOf<T>>) {
			while pool_info.free_stake > Zero::zero() {
				if pool_info.withdraw_queue.is_empty() {
					break;
				}

				if let Some(mut withdraw_info) = pool_info.withdraw_queue.pop_front() {
					let info_key = (pool_info.pid.clone(), withdraw_info.user.clone());
					let mut user_info = Self::staking_info(&info_key).unwrap();

					if pool_info.free_stake < withdraw_info.amount {
						pool_info.total_stake =
							pool_info.total_stake.saturating_sub(pool_info.free_stake);
						withdraw_info.amount =
							withdraw_info.amount.saturating_sub(pool_info.free_stake);

						// push front the updated withdraw info
						pool_info.withdraw_queue.push_front(withdraw_info);

						user_info.amount = user_info.amount.saturating_sub(pool_info.free_stake);
						pool_info.free_stake = Zero::zero();
					} else {
						// all of the amount would be withdraw to user and no need to push the popped one back
						pool_info.free_stake =
							pool_info.free_stake.saturating_sub(withdraw_info.amount);
						pool_info.total_stake =
							pool_info.total_stake.saturating_sub(withdraw_info.amount);

						user_info.amount = user_info.amount.saturating_sub(withdraw_info.amount);
					}
					// update user_debt which would determine the user's rewards
					user_info.user_debt =
						user_info.amount * pool_info.pool_acc / 10u32.pow(6).saturated_into();

					StakingInfo::<T>::insert(&info_key, &user_info);

					// update the lock balance of user
					Self::update_lock(user_info.user.clone(), user_info.amount);
				}
			}
		}

		/// Updates a user's locked balance. Doesn't check the amount is less than the free amount!
		fn update_lock(who: T::AccountId, amount: BalanceOf<T>) {
			if amount == Zero::zero() {
				<T as Config>::Currency::remove_lock(STAKING_ID, &who);
			} else {
				<T as Config>::Currency::set_lock(STAKING_ID, &who, amount, WithdrawReasons::all());
			}
		}

		fn ensure_pool(pid: u64) -> Result<PoolInfo<T::AccountId, BalanceOf<T>>, Error<T>> {
			Self::mining_pools(&pid).ok_or(Error::<T>::PoolNotExist)
		}
	}

	impl<T: Config> mining::OnReward for Pallet<T> {
		/// Called when gk send new payout information.
		/// Append specific miner's reward balance of current round,
		/// would be clear once pool was updated
		fn on_reward(settle: &Vec<SettleInfo>) {
			for info in settle {
				let mut balance = Self::new_rewards(&info.pubkey).unwrap();
				balance = balance.saturating_add(info.payout.saturated_into());
				NewRewards::<T>::insert(&info.pubkey, &balance);
			}
		}
	}

	impl<T: Config> mining::OnCleanup<BalanceOf<T>> for Pallet<T>
	where
		T: mining::Config,
	{
		/// Called when worker was cleanuped
		/// After collingdown end, worker was cleanuped, whose deposit balance
		/// would be reset to zero
		fn on_cleanup(worker: &WorkerPublicKey, deposit_balance: BalanceOf<T>) {
			let pid =
				WorkerInPool::<T>::get(worker).expect("Mining workers must be in the pool; qed.");
			let mut pool_info = Self::ensure_pool(pid).expect("Stake pool must exist; qed.");

			// with the worker been cleaned, whose stake now are free
			pool_info.free_stake = pool_info.free_stake.saturating_add(deposit_balance);

			Self::try_handle_waiting_withdraw(&mut pool_info);
			MiningPools::<T>::insert(&pid, &pool_info);
		}
	}

	fn pool_sub_account<T>(pid: u64, pubkey: &WorkerPublicKey) -> T
	where
		T: Encode + Decode + Default,
	{
		let hash = crate::hashing::blake2_256(&(pid, pubkey).encode());
		// stake pool miner
		(b"spm/", hash)
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.unwrap_or_default()
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct PoolInfo<AccountId: Default, Balance> {
		pid: u64,
		owner: AccountId,
		payout_commission: u16,
		owner_reward: Balance,
		cap: Option<Balance>,
		pool_acc: Balance,
		total_stake: Balance,
		free_stake: Balance,
		workers: Vec<WorkerPublicKey>,
		withdraw_queue: VecDeque<WithdrawInfo<AccountId, Balance>>,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct UserStakeInfo<AccountId: Default, Balance> {
		user: AccountId,
		amount: Balance,
		available_rewards: Balance,
		user_debt: Balance,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct WithdrawInfo<AccountId: Default, Balance> {
		user: AccountId,
		amount: Balance,
		start_time: u64,
	}

	pub struct EnsurePool<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> EnsureOrigin<T::Origin> for EnsurePool<T> {
		type Success = T::AccountId;
		fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
			let pool_id = STAKEPOOL_PALLETID.into_account();
			o.into().and_then(|o| match o {
				frame_system::RawOrigin::Signed(who) if who == pool_id => Ok(pool_id),
				r => Err(T::Origin::from(r)),
			})
		}
	}

	#[cfg(test)]
	mod test {
		use assert_matches::assert_matches;
		use frame_support::{assert_noop, assert_ok};
		use hex_literal::hex;
		use sp_runtime::AccountId32;

		use super::*;
		use crate::mock::{
			ecdh_pubkey, take_events, new_test_ext, set_block_1, worker_pubkey, Event as TestEvent,
			Origin, PhalaRegistry, PhalaStakePool, Test, DOLLARS,
		};

		#[test]
		fn test_pool_subaccount() {
			let sub_account: AccountId32 =
				pool_sub_account(1, &WorkerPublicKey::from_raw([0u8; 33]));
			let expected = AccountId32::new(hex!(
				"73706d2f666bf107db95bd7b5b56e2c9b5a008f97361f20d10a7840cf2dfaaf5"
			));
			assert_eq!(sub_account, expected, "Incorrect sub account");
		}

		#[test]
		fn test_create() {
			// Check this fixed: <https://github.com/Phala-Network/phala-blockchain/issues/285>
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				PhalaStakePool::on_finalize(1);
				assert_matches!(
					take_events().as_slice(),
					[TestEvent::PhalaStakePool(Event::PoolCreated(1, 0)),]
				);
				assert_eq!(
					MiningPools::<Test>::get(0),
					Some(PoolInfo {
						pid: 0,
						owner: 1,
						payout_commission: 0,
						owner_reward: 0,
						cap: None,
						pool_acc: 0,
						total_stake: 0,
						free_stake: 0,
						workers: Vec::new(),
						withdraw_queue: VecDeque::new(),
					})
				);
				assert_eq!(PoolCount::<Test>::get(), 1);
			});
		}

		#[test]
		fn test_add_worker() {
			new_test_ext().execute_with(|| {
				set_block_1();
				let worker1 = worker_pubkey(1);
				let worker2 = worker_pubkey(2);

				assert_ok!(PhalaRegistry::force_register_worker(
					Origin::root(),
					worker1.clone(),
					ecdh_pubkey(1),
					Some(1)
				));

				// Create a pool (pid = 0)
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				// Bad inputs
				assert_noop!(
					PhalaStakePool::add_worker(Origin::signed(1), 1, worker2.clone()),
					Error::<Test>::WorkerNotRegistered
				);
				assert_noop!(
					PhalaStakePool::add_worker(Origin::signed(2), 0, worker1.clone()),
					Error::<Test>::UnauthorizedOperator
				);
				assert_noop!(
					PhalaStakePool::add_worker(Origin::signed(1), 0, worker1.clone()),
					Error::<Test>::BenchmarkMissing
				);
				// Add benchmark and retry
				PhalaRegistry::internal_set_benchmark(&worker1, Some(1));
				assert_ok!(PhalaStakePool::add_worker(
					Origin::signed(1),
					0,
					worker1.clone()
				));
				// Other bad cases
				assert_noop!(
					PhalaStakePool::add_worker(Origin::signed(1), 100, worker1.clone()),
					Error::<Test>::PoolNotExist
				);
				// Bind one worker to antoher pool (pid = 1)
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				assert_noop!(
					PhalaStakePool::add_worker(Origin::signed(1), 1, worker1.clone()),
					Error::<Test>::MinerBindingCallFailed
				);
			});
		}

		#[test]
		fn test_full_procedure() {
			new_test_ext().execute_with(|| {
				set_block_1();
				let worker1 = worker_pubkey(1);
				let worker2 = worker_pubkey(2);
				// Register workers
				assert_ok!(PhalaRegistry::force_register_worker(
					Origin::root(),
					worker1.clone(),
					ecdh_pubkey(1),
					Some(1)
				));
				assert_ok!(PhalaRegistry::force_register_worker(
					Origin::root(),
					worker2.clone(),
					ecdh_pubkey(2),
					Some(1)
				));
				PhalaRegistry::internal_set_benchmark(&worker1, Some(1));
				PhalaRegistry::internal_set_benchmark(&worker2, Some(1));
				// Create a pool (pid = 0)
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				assert_ok!(PhalaStakePool::add_worker(
					Origin::signed(1),
					0,
					worker1.clone()
				));
				assert_ok!(PhalaStakePool::add_worker(
					Origin::signed(1),
					0,
					worker2.clone()
				));
				assert_ok!(PhalaStakePool::set_cap(Origin::signed(1), 0, 300 * DOLLARS));
				assert_ok!(PhalaStakePool::deposit(Origin::signed(1), 0, 100 * DOLLARS));
				assert_eq!(
					MiningPools::<Test>::get(0).unwrap().total_stake,
					100 * DOLLARS
				);
				assert_eq!(
					StakingInfo::<Test>::get(&(0, 1)).unwrap().amount,
					100 * DOLLARS
				);

				assert_ok!(PhalaStakePool::deposit(Origin::signed(2), 0, 200 * DOLLARS));
				assert_eq!(
					MiningPools::<Test>::get(0).unwrap().total_stake,
					300 * DOLLARS
				);
				assert_eq!(
					StakingInfo::<Test>::get(&(0, 2)).unwrap().amount,
					200 * DOLLARS
				);

				assert_noop!(
					PhalaStakePool::deposit(Origin::signed(1), 0, 100 * DOLLARS),
					Error::<Test>::StakeExceedCapacity
				);

				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker1.clone(),
					100 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker2.clone(),
					100 * DOLLARS
				));
				// Withdraw free funds
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					100 * DOLLARS
				));
				// TODO: check balance
				// TODO: check queued withdraw
				//   - withdraw 100 PHA
				//   - stop a worker
				//   - wait CD, withdraw succeeded
				//   - withdraw another 100 PHA
				//   - wait 3d, force stop
				//   - wait 7d, withdraw succeeded
			});
		}
	}
}
