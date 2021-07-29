pub use self::pallet::*;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use crate::mining;
	use crate::registry;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, LockIdentifier, LockableCurrency, UnixTime, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;

	use crate::balance_convert::FixedPointConvert;
	use fixed::types::U64F64 as FixedPoint;
	use phala_types::{messaging::SettleInfo, WorkerPublicKey};
	use sp_runtime::{
		traits::{Saturating, TrailingZeroInput, Zero},
		Permill, SaturatedConversion,
	};
	use sp_std::collections::vec_deque::VecDeque;
	use sp_std::vec;
	use sp_std::vec::Vec;

	const STAKING_ID: LockIdentifier = *b"phala/sp";

	pub trait Ledger<AccountId, Balance> {
		/// Increases the locked amount for a user
		///
		/// Unsafe: it assumes there's enough free `amount`
		fn ledger_accrue(who: &AccountId, amount: Balance);
		/// Decreases the locked amount for a user
		///
		/// Unsafe: it assumes there's enough locked `amount`
		fn ledger_reduce(who: &AccountId, amount: Balance);
		/// Gets the locked amount of `who`
		fn ledger_query(who: &AccountId) -> Balance;
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + registry::Config + mining::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		type MinContribution: Get<BalanceOf<Self>>;
		type InsurancePeriod: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Mapping from pool id to PoolInfo
	#[pallet::storage]
	#[pallet::getter(fn stake_pools)]
	pub(super) type StakePools<T: Config> =
		StorageMap<_, Twox64Concat, u64, PoolInfo<T::AccountId, BalanceOf<T>>>;

	/// Mapping from (pid, staker) to UserStakeInfo
	#[pallet::storage]
	#[pallet::getter(fn pool_stakers)]
	pub(super) type PoolStakers<T: Config> =
		StorageMap<_, Twox64Concat, (u64, T::AccountId), UserStakeInfo<T::AccountId, BalanceOf<T>>>;

	/// The number of total pools
	#[pallet::storage]
	#[pallet::getter(fn pool_count)]
	pub(super) type PoolCount<T> = StorageValue<_, u64, ValueQuery>;

	/// Mapping from workers to the pool they belong to
	#[pallet::storage]
	pub(super) type WorkerAssignments<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, u64>;

	/// Mapping a miner sub-account to the pool it belongs to.
	///
	/// The map entry lasts from `start_mining()` to `on_cleanup()`.
	#[pallet::storage]
	pub(super) type SubAccountAssignments<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, u64>;

	/// Mapping staker to it's the balance locked in all pools
	#[pallet::storage]
	#[pallet::getter(fn stake_ledger)]
	pub(super) type StakeLedger<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

	/// Mapping from the block timestamp to pools that has withdrawal requests queued in that block
	#[pallet::storage]
	#[pallet::getter(fn withdrawal_queued_pools)]
	pub(super) type WithdrawalQueuedPools<T: Config> = StorageMap<_, Twox64Concat, u64, Vec<u64>>;

	/// Queue that contains all block's timestamp, in that block contains the waiting withdraw reqeust.
	/// This queue has a max size of (T::InsurancePeriod * 8) bytes
	#[pallet::storage]
	#[pallet::getter(fn withdrawal_timestamps)]
	pub(super) type WithdrawalTimestamps<T> = StorageValue<_, VecDeque<u64>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [owner, pid]
		PoolCreated(T::AccountId, u64),
		/// [pid, commission]. The real commission ratio is commission/1_000_000u32
		PoolCommissionSet(u64, u32),
		/// [pid, cap]
		PoolCapacitySet(u64, BalanceOf<T>),
		/// [pid, worker]
		PoolWorkerAdded(u64, WorkerPublicKey),
		/// [pid, user, amount]
		Contribution(u64, T::AccountId, BalanceOf<T>),
		/// [pid, user, amount]
		Withdrawal(u64, T::AccountId, BalanceOf<T>),
		/// [pid, user, amount]
		RewardsWithdrawn(u64, T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		WorkerNotRegistered,
		BenchmarkMissing,
		WorkerExists,
		WorkerDoesNotExist,
		WorerInAnotherPool,
		UnauthorizedOperator,
		UnauthorizedPoolOwner,
		/// The stake capacity is set too low for the existing stake
		InadequateCapacity,
		StakeExceedsCapacity,
		PoolDoesNotExist,
		_PoolIsBusy,
		InsufficientContribution,
		InsufficientBalance,
		PoolStakeNotFound,
		InsufficientFreeStake,
		InvalidWithdrawalAmount,
		FailedToBindMinerAndWorker,
		/// Internal error: Cannot withdraw from the subsidy pool. This should never happen.
		InternalSubsidyPoolCannotWithdraw,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
		BalanceOf<T>: FixedPointConvert,
	{
		fn on_finalize(_n: T::BlockNumber) {
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();

			let mut t = WithdrawalTimestamps::<T>::get();
			// has waiting withdraw request
			if !t.is_empty() {
				// we just handle timeout request every block
				while let Some(start_time) = t.front().cloned() {
					if now - start_time <= T::InsurancePeriod::get().saturated_into::<u64>() {
						break;
					}
					let pools = WithdrawalQueuedPools::<T>::take(start_time)
						.expect("Pool list must exist; qed.");
					for &pid in pools.iter() {
						let pool_info =
							Self::ensure_pool(pid).expect("Stake pool must exist; qed.");
						// if we check the pool withdraw_queue here, we don't have to remove a pool from WithdrawalQueuedPools when
						// a pool has handled their waiting withdraw request before timeout. Compare the IO performance we
						// think remove pool from WithdrawalQueuedPools would have more resource cost.
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
										// TODO: avoid stop mining multiple times
										let _ = <mining::pallet::Pallet<T>>::stop_mining(miner);
									}
								}
							}
						}
					}
					// pop front timestamp
					t.pop_front();
				}
				WithdrawalTimestamps::<T>::put(&t);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
		BalanceOf<T>: FixedPointConvert,
	{
		/// Creates a new stake pool
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let pid = PoolCount::<T>::get();
			StakePools::<T>::insert(
				pid,
				PoolInfo {
					pid: pid,
					owner: owner.clone(),
					payout_commission: None,
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

		/// Adds a worker to a pool
		///
		/// This will bind a worker to the corresponding pool sub-account. The binding will not be
		/// released until the worker is removed gracefully by `remove_worker()`, or a force unbind
		/// by the worker opreator via `Mining::unbind()`.
		///
		/// Requires:
		/// 1. The worker is registered and benchmakred
		/// 2. The worker is not bound a pool
		#[pallet::weight(0)]
		pub fn add_worker(
			origin: OriginFor<T>,
			pid: u64,
			pubkey: WorkerPublicKey,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let worker_info =
				registry::Workers::<T>::get(&pubkey).ok_or(Error::<T>::WorkerNotRegistered)?;

			// check wheather the owner was bound as operator
			ensure!(
				worker_info.operator == Some(owner.clone()),
				Error::<T>::UnauthorizedOperator
			);
			// check the worker has finished the benchmark
			ensure!(
				worker_info.initial_score != None,
				Error::<T>::BenchmarkMissing
			);

			// origin must be owner of pool
			let mut pool_info = Self::ensure_pool(pid)?;
			ensure!(pool_info.owner == owner, Error::<T>::UnauthorizedPoolOwner);
			// make sure worker has not been not added
			// TODO: should we set a cap to avoid performance problem
			let workers = &mut pool_info.workers;
			// TODO: limit the number of workers to avoid performance issue.
			ensure!(!workers.contains(&pubkey), Error::<T>::WorkerExists);

			// generate miner account
			let miner: T::AccountId = pool_sub_account(pid, &pubkey);

			// bind worker with minner
			mining::pallet::Pallet::<T>::bind(miner.clone(), pubkey.clone())
				.or(Err(Error::<T>::FailedToBindMinerAndWorker))?;

			// update worker vector
			workers.push(pubkey.clone());
			StakePools::<T>::insert(&pid, &pool_info);
			WorkerAssignments::<T>::insert(&pubkey, pid);
			SubAccountAssignments::<T>::insert(&miner, pid);
			Self::deposit_event(Event::<T>::PoolWorkerAdded(pid, pubkey));

			Ok(())
		}

		/// Removes a worker from a pool
		///
		/// Requires:
		/// 1. The worker is registered
		/// 2. The worker is associated with a pool
		/// 3. The worker is removalbe (not in mining)
		#[pallet::weight(0)]
		pub fn remove_worker(
			origin: OriginFor<T>,
			pid: u64,
			worker: WorkerPublicKey,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// The sender is the pool owner
			let pool = Self::ensure_pool(pid)?;
			ensure!(pool.owner == who, Error::<T>::UnauthorizedPoolOwner);
			// The worker is in this pool. It implies:
			// - The worker is already in `PoolInfo::worker` list
			// - The sub-account assignment exists (because they are created & killed together)
			let lookup_pid =
				WorkerAssignments::<T>::get(worker).ok_or(Error::<T>::WorkerDoesNotExist)?;
			ensure!(pid == lookup_pid, Error::<T>::WorerInAnotherPool);
			// Remove the worker from the pool (notification suspended)
			let sub_account: T::AccountId = pool_sub_account(pid, &worker);
			mining::pallet::Pallet::<T>::unbind_miner(&sub_account, false)?;
			// Manually clean up the worker, including the pool worker list, and the assignment
			// indices. (Theoritically we can enable the unbinding notification, and follow the
			// same path as a force unbinding, but it doesn't sounds graceful.)
			Self::remove_worker_from_pool(&worker);
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
		///
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
			ensure!(pool_info.total_stake <= cap, Error::<T>::InadequateCapacity);

			pool_info.cap = Some(cap);
			StakePools::<T>::insert(&pid, &pool_info);

			Self::deposit_event(Event::<T>::PoolCapacitySet(pid, cap));
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
			payout_commission: Permill,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut pool_info = Self::ensure_pool(pid)?;
			// origin must be owner of pool
			ensure!(pool_info.owner == owner, Error::<T>::UnauthorizedPoolOwner);

			pool_info.payout_commission = Some(payout_commission);
			StakePools::<T>::insert(&pid, &pool_info);

			Self::deposit_event(Event::<T>::PoolCommissionSet(
				pid,
				payout_commission.deconstruct(),
			));

			Ok(())
		}

		/// Claims all the pending rewards of the sender and send to the `target`
		///
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::weight(0)]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			pid: u64,
			target: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let info_key = (pid.clone(), who.clone());
			let mut user_info =
				Self::pool_stakers(&info_key).ok_or(Error::<T>::PoolStakeNotFound)?;
			let pool_info = Self::ensure_pool(pid)?;

			// Clear the pending reward, and calculate the rewards belong to user
			pool_info.clear_user_pending_reward(&mut user_info);
			let rewards = user_info.available_rewards;
			user_info.available_rewards = Zero::zero();
			mining::Pallet::<T>::withdraw_subsidy_pool(&target, rewards)
				.or(Err(Error::<T>::InternalSubsidyPoolCannotWithdraw))?;
			PoolStakers::<T>::insert(&info_key, &user_info);
			Self::deposit_event(Event::<T>::RewardsWithdrawn(pid, who, rewards));

			Ok(())
		}

		/// Contributes some stake to a pool
		///
		/// Requires:
		/// 1. The pool exists
		/// 2. After the desposit, the pool doesn't reach the cap
		#[pallet::weight(0)]
		pub fn contribute(origin: OriginFor<T>, pid: u64, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let a = amount; // Alias to reduce confusion in the code below

			ensure!(
				a >= T::MinContribution::get(),
				Error::<T>::InsufficientContribution
			);
			ensure!(
				<T as Config>::Currency::free_balance(&who) >= a,
				Error::<T>::InsufficientBalance
			);

			let mut pool_info = Self::ensure_pool(pid)?;
			if let Some(cap) = pool_info.cap {
				ensure!(
					cap.saturating_sub(pool_info.total_stake) >= a,
					Error::<T>::StakeExceedsCapacity
				);
			}

			let info_key = (pid.clone(), who.clone());
			// Clear the pending reward before adding stake, if applies
			let mut user_info = match Self::pool_stakers(&info_key) {
				Some(mut user_info) => {
					pool_info.clear_user_pending_reward(&mut user_info);
					user_info
				}
				None => UserStakeInfo {
					user: who.clone(),
					amount: Zero::zero(),
					available_rewards: Zero::zero(),
					user_debt: Zero::zero(),
				},
			};
			// Add the stake
			user_info.amount.saturating_accrue(a);
			user_info.clear_pending_reward(pool_info.pool_acc);
			PoolStakers::<T>::insert(&info_key, &user_info);
			// Lock the funds
			Self::ledger_accrue(&who, a);
			// Update pool info
			pool_info.total_stake = pool_info.total_stake.saturating_add(a);
			pool_info.free_stake = pool_info.free_stake.saturating_add(a);

			// we have new free stake now, try handle the waitting withdraw queue
			Self::try_process_withdraw_queue(&mut pool_info);

			StakePools::<T>::insert(&pid, &pool_info);

			Self::deposit_event(Event::<T>::Contribution(pid, who, a));
			Ok(())
		}

		/// Demands the return of some stake from a pool.
		///
		/// Note: there are two scenarios people may meet
		///
		/// - if the pool has free stake and the amount of the free stake is greater than or equal
		///     to the withdrawal amount (e.g. pool.free_stake >= amount), the withdrawal would
		///     take effect immediately.
		/// - else the withdrawal would be queued and delayed until there is enough free stake.
		#[pallet::weight(0)]
		pub fn withdraw(origin: OriginFor<T>, pid: u64, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let info_key = (pid.clone(), who.clone());
			let mut user_info =
				Self::pool_stakers(&info_key).ok_or(Error::<T>::PoolStakeNotFound)?;

			ensure!(
				amount > Zero::zero() && user_info.amount >= amount,
				Error::<T>::InvalidWithdrawalAmount
			);

			let mut pool_info = Self::ensure_pool(pid)?;
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();

			// if withdraw_queue is not empty, means pool doesn't have free stake now, just add withdraw to queue
			if !pool_info.withdraw_queue.is_empty() {
				pool_info.withdraw_queue.push_back(WithdrawInfo {
					user: who.clone(),
					amount: amount,
					start_time: now,
				});
				Self::maybe_add_withdraw_queue(now, pool_info.pid);
			} else {
				Self::try_withdraw(&mut pool_info, &mut user_info, amount);
			}

			PoolStakers::<T>::insert(&info_key, &user_info);
			StakePools::<T>::insert(&pid, &pool_info);

			Ok(())
		}

		/// Starts a miner on behalf of the stake pool
		///
		/// Requires:
		/// 1. The miner is bound to the pool and is in Ready state
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
			ensure!(
				pool_info.free_stake >= stake,
				Error::<T>::InsufficientFreeStake
			);
			// check wheather we have add this worker
			ensure!(
				pool_info.workers.contains(&worker),
				Error::<T>::WorkerDoesNotExist
			);
			let miner: T::AccountId = pool_sub_account(pid, &worker);
			mining::pallet::Pallet::<T>::start_mining(miner.clone(), stake)?;
			pool_info.free_stake = pool_info.free_stake.saturating_sub(stake);
			StakePools::<T>::insert(&pid, &pool_info);
			Ok(())
		}

		/// Stops a miner on behalf of the stake pool
		/// Note: this would let miner enter CoolingDown if everything is good
		///
		/// Requires:
		/// 1. There miner is bound to the pool and is in a stoppable state
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
				Error::<T>::WorkerDoesNotExist
			);
			let miner: T::AccountId = pool_sub_account(pid, &worker);
			<mining::pallet::Pallet<T>>::stop_mining(miner)?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Adds up the newly received reward to `pool_acc`
		fn handle_pool_new_reward(
			pool_info: &mut PoolInfo<T::AccountId, BalanceOf<T>>,
			rewards: BalanceOf<T>,
		) {
			if rewards > Zero::zero() && pool_info.total_stake > Zero::zero() {
				let commission = pool_info.payout_commission.unwrap_or_default() * rewards;
				pool_info.owner_reward.saturating_accrue(commission);
				pool_info.add_reward(rewards - commission);
			}
		}

		/// Tries to withdraw a specific amount from a pool.
		///
		/// The withdraw request would be delayed if the free stake is not enough, otherwise
		/// withdraw from the free stake immediately.
		///
		/// WARNING:
		/// 1. The method assumes user pending reward is already cleared.
		/// 2. The updates are made in `pool_info` and `user_info`. It's up to the caller to
		///     persist the data.
		fn try_withdraw(
			pool_info: &mut PoolInfo<T::AccountId, BalanceOf<T>>,
			user_info: &mut UserStakeInfo<T::AccountId, BalanceOf<T>>,
			amount: BalanceOf<T>,
		) {
			// enough free stake, withdraw directly
			if pool_info.free_stake >= amount {
				pool_info.free_stake = pool_info.free_stake.saturating_sub(amount);
				pool_info.total_stake = pool_info.total_stake.saturating_sub(amount);
				user_info.amount = user_info.amount.saturating_sub(amount);
				Self::ledger_reduce(&user_info.user, amount);
				Self::deposit_event(Event::<T>::Withdrawal(
					pool_info.pid,
					user_info.user.clone(),
					amount,
				));
			} else {
				let now = <T as registry::Config>::UnixTime::now()
					.as_secs()
					.saturated_into::<u64>();
				// all of the free_stake would be withdrew back to user
				let delta = pool_info.free_stake;
				let unwithdraw_amount = amount.saturating_sub(pool_info.free_stake);
				pool_info.total_stake = pool_info.total_stake.saturating_sub(delta);
				user_info.amount.saturating_reduce(delta);
				Self::ledger_reduce(&user_info.user, pool_info.free_stake);
				Self::deposit_event(Event::<T>::Withdrawal(
					pool_info.pid,
					user_info.user.clone(),
					pool_info.free_stake,
				));
				pool_info.free_stake = Zero::zero();

				// case some locked asset has not been withdraw(unlock) to user, add it to withdraw queue.
				// when pool has free stake again, the withdraw would be handled
				pool_info.withdraw_queue.push_back(WithdrawInfo {
					user: user_info.user.clone(),
					amount: unwithdraw_amount,
					start_time: now,
				});
				Self::maybe_add_withdraw_queue(now, pool_info.pid);
			}
			// Update the pending reward after changing the staked amount
			user_info.clear_pending_reward(pool_info.pool_acc);
		}

		/// Tries to fulfill the withdraw queue with the newly freed stake
		fn try_process_withdraw_queue(pool_info: &mut PoolInfo<T::AccountId, BalanceOf<T>>) {
			while pool_info.free_stake > Zero::zero() {
				if let Some(mut withdraw) = pool_info.withdraw_queue.front().cloned() {
					// Must clear the pending reward before any stake change
					let info_key = (pool_info.pid.clone(), withdraw.user.clone());
					let mut user_info = Self::pool_stakers(&info_key).unwrap();
					pool_info.clear_user_pending_reward(&mut user_info);
					// Try to fulfill the withdraw requests as much as possible
					let delta = sp_std::cmp::min(pool_info.free_stake, withdraw.amount);
					pool_info.free_stake.saturating_reduce(delta);
					pool_info.total_stake.saturating_reduce(delta);
					withdraw.amount.saturating_reduce(delta);
					user_info.amount.saturating_reduce(delta);
					// Actually withdraw the funds
					Self::ledger_reduce(&user_info.user, delta);
					Self::deposit_event(Event::<T>::Withdrawal(
						pool_info.pid,
						user_info.user.clone(),
						delta,
					));
					// Update the pending reward after changing the staked amount
					user_info.clear_pending_reward(pool_info.pool_acc);
					PoolStakers::<T>::insert(&info_key, &user_info);
					// Update if the withdraw is partially fulfilled, otherwise pop it out of the
					// queue
					if withdraw.amount == Zero::zero() {
						pool_info.withdraw_queue.pop_front();
					} else {
						*pool_info.withdraw_queue.front_mut().unwrap() = withdraw;
					}
				} else {
					break;
				}
			}
		}

		/// Updates a user's locked balance. Doesn't check the amount is less than the free amount!
		fn update_lock(who: &T::AccountId, amount: BalanceOf<T>) {
			if amount == Zero::zero() {
				<T as Config>::Currency::remove_lock(STAKING_ID, who);
			} else {
				<T as Config>::Currency::set_lock(STAKING_ID, who, amount, WithdrawReasons::all());
			}
		}

		/// Gets the pool record by `pid`. Returns error if not exist
		fn ensure_pool(pid: u64) -> Result<PoolInfo<T::AccountId, BalanceOf<T>>, Error<T>> {
			Self::stake_pools(&pid).ok_or(Error::<T>::PoolDoesNotExist)
		}

		/// Adds the givin pool (`pid`) to the withdraw queue if not present
		fn maybe_add_withdraw_queue(start_time: u64, pid: u64) {
			let mut t = WithdrawalTimestamps::<T>::get();
			if let Some(last_start_time) = t.back().cloned() {
				// the last_start_time == start_time means already have a withdraw request added early of this block,
				// last_start_time > start_time is impossible
				if last_start_time < start_time {
					t.push_back(start_time);
				}
			} else {
				// first time add withdraw pool
				t.push_back(start_time);
			}
			WithdrawalTimestamps::<T>::put(&t);

			// push pool to the pool list, if the pool was added in this pool, means it has waiting withdraw request
			// in current block(if they have the same timestamp, we think they are in the same block)
			if WithdrawalQueuedPools::<T>::contains_key(&start_time) {
				let mut pool_list = WithdrawalQueuedPools::<T>::get(&start_time).unwrap();
				// if pool has already been added, ignore it
				if !pool_list.contains(&pid) {
					pool_list.push(pid);
					WithdrawalQueuedPools::<T>::insert(&start_time, &pool_list);
				}
			} else {
				WithdrawalQueuedPools::<T>::insert(&start_time, vec![pid]);
			}
		}

		/// Removes a worker from a pool, either intentially or unintentially.
		///
		/// It assumes the worker is already in a pool.
		fn remove_worker_from_pool(worker: &WorkerPublicKey) {
			let pid = WorkerAssignments::<T>::take(worker).expect("Worker must be in a pool; qed.");
			let sub_account: T::AccountId = pool_sub_account(pid, worker);
			SubAccountAssignments::<T>::remove(sub_account);
			StakePools::<T>::mutate(pid, |value| {
				if let Some(pool) = value {
					pool.remove_worker(&worker);
				}
			});
		}
	}

	impl<T: Config> mining::OnReward for Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert,
	{
		/// Called when gk send new payout information.
		/// Append specific miner's reward balance of current round,
		/// would be clear once pool was updated
		fn on_reward(settle: &Vec<SettleInfo>) {
			for info in settle {
				let pid = WorkerAssignments::<T>::get(&info.pubkey)
					.expect("Mining workers must be in the pool; qed.");
				let mut pool_info = Self::ensure_pool(pid).expect("Stake pool must exist; qed.");

				let payout_fixed = FixedPoint::from_bits(info.payout);
				let reward = BalanceOf::<T>::from_fixed(&payout_fixed);
				Self::handle_pool_new_reward(&mut pool_info, reward);
				StakePools::<T>::insert(&pid, &pool_info);
			}
		}
	}

	impl<T: Config> mining::OnUnbound for Pallet<T>
	where
		T: mining::Config,
	{
		fn on_unbound(worker: &WorkerPublicKey, _force: bool) {
			// Assuming force == true?
			Self::remove_worker_from_pool(worker);
		}
	}

	impl<T: Config> mining::OnReclaim<T::AccountId, BalanceOf<T>> for Pallet<T>
	where
		T: mining::Config,
	{
		/// Called when worker was cleanuped
		/// After collingdown end, worker was cleanuped, whose contributed balance
		/// would be reset to zero
		fn on_reclaim(miner: &T::AccountId, stake: BalanceOf<T>) {
			let pid =
				SubAccountAssignments::<T>::take(miner).expect("Sub-acocunt must exist; qed.");
			let mut pool_info = Self::ensure_pool(pid).expect("Stake pool must exist; qed.");

			// with the worker been cleaned, whose stake now are free
			pool_info.free_stake = pool_info.free_stake.saturating_add(stake);

			Self::try_process_withdraw_queue(&mut pool_info);
			StakePools::<T>::insert(&pid, &pool_info);
		}
	}

	impl<T: Config> Ledger<T::AccountId, BalanceOf<T>> for Pallet<T> {
		fn ledger_accrue(who: &T::AccountId, amount: BalanceOf<T>) {
			let b: BalanceOf<T> = StakeLedger::<T>::get(who).unwrap_or_default();
			let new_b = b.saturating_add(amount);
			StakeLedger::<T>::insert(who, new_b);
			Self::update_lock(who, new_b);
		}

		fn ledger_reduce(who: &T::AccountId, amount: BalanceOf<T>) {
			let b: BalanceOf<T> = StakeLedger::<T>::get(who).unwrap_or_default();
			let new_b = b.saturating_sub(amount);
			StakeLedger::<T>::insert(who, new_b);
			Self::update_lock(who, new_b);
		}

		fn ledger_query(who: &T::AccountId) -> BalanceOf<T> {
			StakeLedger::<T>::get(who).unwrap_or_default()
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
		payout_commission: Option<Permill>,
		owner_reward: Balance,
		cap: Option<Balance>,
		pool_acc: Balance,
		total_stake: Balance,
		free_stake: Balance,
		workers: Vec<WorkerPublicKey>,
		withdraw_queue: VecDeque<WithdrawInfo<AccountId, Balance>>,
	}

	impl<AccountId, Balance> PoolInfo<AccountId, Balance>
	where
		AccountId: Default,
		Balance: sp_runtime::traits::AtLeast32BitUnsigned + Copy,
	{
		/// Clears the pending rewards of a user and move to `available_rewards` for claiming
		fn clear_user_pending_reward(&self, user_info: &mut UserStakeInfo<AccountId, Balance>) {
			let pending_reward = user_info.pending_reward(self.pool_acc);
			user_info
				.available_rewards
				.saturating_accrue(pending_reward);
			user_info.clear_pending_reward(self.pool_acc);
		}

		// Distributes additinoal rewards to the current share holders.
		//
		// Additional rewards contribute to the face value of the pool shares. The vaue of each
		// share effectively grows by (rewards / total_shares).
		fn add_reward(&mut self, rewards: Balance) {
			self.pool_acc
				.saturating_accrue(rewards * 10u32.pow(6).into() / self.total_stake);
		}

		fn remove_worker(&mut self, worker: &WorkerPublicKey) {
			self.workers.retain(|w| w != worker);
		}
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

	impl<AccountId, Balance> UserStakeInfo<AccountId, Balance>
	where
		AccountId: Default,
		Balance: sp_runtime::traits::AtLeast32BitUnsigned + Copy,
	{
		/// Calculates the pending reward this user holds
		///
		/// - `acc_per_share`: accumulated reward per share
		fn pending_reward(&self, acc_per_share: Balance) -> Balance {
			self.amount * acc_per_share / 1_000_000u32.into() - self.user_debt
		}

		/// Resets the `user_debt` to remove all the pending rewards
		fn clear_pending_reward(&mut self, acc_per_share: Balance) {
			self.user_debt = self.amount * acc_per_share / 1_000_000u32.into();
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
			ecdh_pubkey, elapse_cool_down, new_test_ext, set_block_1, setup_workers,
			setup_workers_linked_operators, take_events, worker_pubkey, Balance,
			Event as TestEvent, Origin, Test, DOLLARS,
		};
		// Pallets
		use crate::mock::{Balances, PhalaMining, PhalaRegistry, PhalaStakePool};

		#[test]
		fn test_pool_subaccount() {
			let sub_account: AccountId32 =
				pool_sub_account(1, &WorkerPublicKey::from_raw([0u8; 32]));
			let expected = AccountId32::new(hex!(
				"73706d2f02ab4d74c86ec3b3997a4fadf33e55e8279650c8539ea67e053c02dc"
			));
			assert_eq!(sub_account, expected, "Incorrect sub account");
		}

		#[test]
		fn test_create() {
			// Check this fixed: <https://github.com/Phala-Network/phala-blockchain/issues/285>
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				PhalaStakePool::on_finalize(1);
				assert_matches!(
					take_events().as_slice(),
					[
						TestEvent::PhalaStakePool(Event::PoolCreated(1, 0)),
						TestEvent::PhalaStakePool(Event::PoolCreated(1, 1)),
					]
				);
				assert_eq!(
					StakePools::<Test>::get(0),
					Some(PoolInfo {
						pid: 0,
						owner: 1,
						payout_commission: None,
						owner_reward: 0,
						cap: None,
						pool_acc: 0,
						total_stake: 0,
						free_stake: 0,
						workers: Vec::new(),
						withdraw_queue: VecDeque::new(),
					})
				);
				assert_eq!(PoolCount::<Test>::get(), 2);
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
				// Check binding
				let subaccount = pool_sub_account(0, &worker_pubkey(1));
				assert_eq!(
					PhalaMining::ensure_worker_bound(&worker_pubkey(1)).unwrap(),
					subaccount,
				);
				assert_eq!(
					PhalaMining::ensure_miner_bound(&subaccount).unwrap(),
					worker_pubkey(1),
				);
				// Check assignments
				assert_eq!(WorkerAssignments::<Test>::get(&worker_pubkey(1)), Some(0));
				assert_eq!(SubAccountAssignments::<Test>::get(&subaccount), Some(0));
				// Other bad cases
				assert_noop!(
					PhalaStakePool::add_worker(Origin::signed(1), 100, worker1.clone()),
					Error::<Test>::PoolDoesNotExist
				);
				// Bind one worker to antoher pool (pid = 1)
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				assert_noop!(
					PhalaStakePool::add_worker(Origin::signed(1), 1, worker1.clone()),
					Error::<Test>::FailedToBindMinerAndWorker
				);
			});
		}

		#[test]
		fn test_start_mining() {
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				// Cannot start mining wihtout a bound worker
				assert_noop!(
					PhalaStakePool::start_mining(Origin::signed(1), 0, worker_pubkey(1), 0),
					Error::<Test>::WorkerDoesNotExist
				);
				// Basic setup
				setup_workers(2);
				assert_ok!(PhalaStakePool::add_worker(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					100 * DOLLARS
				));
				// No enough stake
				assert_noop!(
					PhalaStakePool::start_mining(Origin::signed(1), 0, worker_pubkey(1), 0),
					mining::Error::<Test>::InsufficientStake
				);
				// Too much stake
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(99),
					0,
					30000 * DOLLARS
				));
				assert_noop!(
					PhalaStakePool::start_mining(
						Origin::signed(1),
						0,
						worker_pubkey(1),
						30000 * DOLLARS
					),
					mining::Error::<Test>::TooMuchStake
				);
				// Can start mining normally
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					100 * DOLLARS
				));
				assert_eq!(PhalaMining::online_miners(), 1);
			});
		}

		#[test]
		fn test_force_unbind() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers_linked_operators(2);
				setup_pool_with_workers(1, &[1]); // pid = 0
				setup_pool_with_workers(2, &[2]); // pid = 1
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					100 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					1,
					100 * DOLLARS
				));

				// Pool0: Change the operator to account101 and force unbind (not mining)
				assert_ok!(PhalaRegistry::force_register_worker(
					Origin::root(),
					worker_pubkey(1),
					ecdh_pubkey(1),
					Some(101)
				));
				let sub_account = pool_sub_account(0, &worker_pubkey(1));
				assert_ok!(PhalaMining::unbind(Origin::signed(101), sub_account));
				// Check assignments cleared, and the worker removed from the pool
				assert_eq!(
					WorkerAssignments::<Test>::contains_key(&worker_pubkey(1)),
					false
				);
				assert_eq!(
					SubAccountAssignments::<Test>::contains_key(&sub_account),
					false
				);
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				assert_eq!(pool.workers.contains(&worker_pubkey(1)), false);
				// Check the mining is ready
				let miner = PhalaMining::miners(&sub_account).unwrap();
				assert_eq!(miner.state, mining::MinerState::Ready);

				// Pool1: Change the operator to account102 and force unbind (mining)
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(2),
					1,
					worker_pubkey(2),
					100 * DOLLARS
				));
				assert_ok!(PhalaRegistry::force_register_worker(
					Origin::root(),
					worker_pubkey(2),
					ecdh_pubkey(2),
					Some(102)
				));
				let sub_account = pool_sub_account(1, &worker_pubkey(2));
				assert_ok!(PhalaMining::unbind(Origin::signed(102), sub_account));
				// Check assignments cleared, and the worker removed from the pool
				assert_eq!(
					WorkerAssignments::<Test>::contains_key(&worker_pubkey(2)),
					false
				);
				assert_eq!(
					SubAccountAssignments::<Test>::contains_key(&sub_account),
					false
				);
				let pool = PhalaStakePool::stake_pools(1).unwrap();
				assert_eq!(pool.workers.contains(&worker_pubkey(2)), false);
				// Check the mining is stopped
				let miner = PhalaMining::miners(&sub_account).unwrap();
				assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
			});
		}

		#[test]
		fn test_pool_cap() {
			new_test_ext().execute_with(|| {
				set_block_1();
				let worker1 = worker_pubkey(1);
				assert_ok!(PhalaRegistry::force_register_worker(
					Origin::root(),
					worker1.clone(),
					ecdh_pubkey(1),
					Some(1)
				));

				assert_ok!(PhalaStakePool::create(Origin::signed(1))); // pid = 0
				assert_eq!(PhalaStakePool::stake_pools(0).unwrap().cap, None);
				// Pool existence
				assert_noop!(
					PhalaStakePool::set_cap(Origin::signed(2), 100, 1),
					Error::<Test>::PoolDoesNotExist,
				);
				// Owner only
				assert_noop!(
					PhalaStakePool::set_cap(Origin::signed(2), 0, 1),
					Error::<Test>::UnauthorizedPoolOwner,
				);
				// Cap to 1000 PHA
				assert_ok!(PhalaStakePool::set_cap(
					Origin::signed(1),
					0,
					1000 * DOLLARS
				));
				assert_eq!(
					PhalaStakePool::stake_pools(0).unwrap().cap,
					Some(1000 * DOLLARS)
				);
				// Check cap shouldn't be less than the current stake
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					100 * DOLLARS
				));
				assert_noop!(
					PhalaStakePool::set_cap(Origin::signed(1), 0, 99 * DOLLARS),
					Error::<Test>::InadequateCapacity,
				);
				// Stake to the cap
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					900 * DOLLARS
				));
				// Exceed the cap
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 0, 900 * DOLLARS),
					Error::<Test>::StakeExceedsCapacity,
				);
			});
		}

		#[test]
		fn test_stake() {
			new_test_ext().execute_with(|| {
				set_block_1();
				let worker1 = worker_pubkey(1);
				assert_ok!(PhalaRegistry::force_register_worker(
					Origin::root(),
					worker1.clone(),
					ecdh_pubkey(1),
					Some(1)
				));

				assert_ok!(PhalaStakePool::create(Origin::signed(1))); // pid = 0
				assert_ok!(PhalaStakePool::create(Origin::signed(2))); // pid = 1

				// Stake normally
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					1 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					10 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					1,
					100 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					1,
					1000 * DOLLARS
				));
				// Check total stake
				assert_eq!(
					PhalaStakePool::stake_pools(0).unwrap().total_stake,
					11 * DOLLARS
				);
				assert_eq!(
					PhalaStakePool::stake_pools(1).unwrap().total_stake,
					1100 * DOLLARS
				);
				// Check total locks
				assert_eq!(PhalaStakePool::stake_ledger(1), Some(101 * DOLLARS));
				assert_eq!(PhalaStakePool::stake_ledger(2), Some(1010 * DOLLARS));
				assert_eq!(Balances::locks(1), vec![the_lock(101 * DOLLARS)]);
				assert_eq!(Balances::locks(2), vec![the_lock(1010 * DOLLARS)]);

				// Pool existence
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 100, 1 * DOLLARS),
					Error::<Test>::PoolDoesNotExist
				);
				// Dust contribution
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 0, 1),
					Error::<Test>::InsufficientContribution
				);
				// Stake more than account1 has
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 0, Balances::free_balance(1) + 1,),
					Error::<Test>::InsufficientBalance,
				);
			});
		}

		#[test]
		fn test_reward_management() {
			use crate::mining::pallet::OnReward;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid = 0

				// Check stake before receiving any rewards
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					100 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					400 * DOLLARS
				));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				assert_eq!(pool.pool_acc, 0);
				assert_eq!(pool.total_stake, 500 * DOLLARS);

				// Mined 500 PHA
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1).to_bits(),
					payout: FixedPoint::from_num(500).to_bits(),
				}]);
				// Should result in 100, 400 PHA pending reward for staker1 & 2
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.pool_acc, 1_000_000);
				assert_eq!(staker1.pending_reward(pool.pool_acc), 100 * DOLLARS);
				assert_eq!(staker2.pending_reward(pool.pool_acc), 400 * DOLLARS);

				// Staker1 claims 100 PHA rewrad, left 100 debt & no pending reward
				let _ = take_events();
				assert_ok!(PhalaStakePool::claim_rewards(Origin::signed(1), 0, 1));
				assert_eq!(
					take_events().as_slice(),
					[
						TestEvent::Balances(pallet_balances::Event::<Test>::Transfer(
							PhalaMining::account_id(),
							1,
							100 * DOLLARS
						)),
						TestEvent::PhalaStakePool(Event::RewardsWithdrawn(0, 1, 100 * DOLLARS))
					]
				);
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				assert_eq!(pool.pool_acc, 1_000_000, "pool_acc shouldn't change");
				assert_eq!(staker1.user_debt, 100 * DOLLARS);
				assert_eq!(staker1.pending_reward(pool.pool_acc), 0);

				// Mined 500 PHA
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1).to_bits(),
					payout: FixedPoint::from_num(500).to_bits(),
				}]);
				// Should result in 100, 800 PHA pending reward for staker1 & 2
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.pool_acc, 2_000_000);
				assert_eq!(staker1.pending_reward(pool.pool_acc), 100 * DOLLARS);
				assert_eq!(staker2.pending_reward(pool.pool_acc), 800 * DOLLARS);

				// Staker2 claims 800 PHA rewrad, left 800 debt
				let _ = take_events();
				assert_ok!(PhalaStakePool::claim_rewards(Origin::signed(2), 0, 2));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(staker2.user_debt, 800 * DOLLARS);

				// Staker1 contribute another 300 PHA (now 50:50), causing a passive reward settlement
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					300 * DOLLARS
				));
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				assert_eq!(staker1.amount, 400 * DOLLARS);
				assert_eq!(staker1.user_debt, 800 * DOLLARS);

				// Mined 800 PHA
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1).to_bits(),
					payout: FixedPoint::from_num(800).to_bits(),
				}]);
				assert_ok!(PhalaStakePool::claim_rewards(Origin::signed(1), 0, 1));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.pool_acc, 3_000_000);
				assert_eq!(staker1.pending_reward(pool.pool_acc), 0);
				assert_eq!(staker2.pending_reward(pool.pool_acc), 400 * DOLLARS);

				// Staker1 withdraw all
				let _ = take_events();
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(1),
					0,
					400 * DOLLARS
				));
				assert_eq!(
					take_events().as_slice(),
					[TestEvent::PhalaStakePool(Event::Withdrawal(
						0,
						1,
						400 * DOLLARS
					))]
				);
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(staker1.amount, 0);
				assert_eq!(staker1.user_debt, 0);
				assert_eq!(staker2.amount, 400 * DOLLARS);
			});
		}

		#[test]
		fn test_drained_subsidy_pool_noop() {
			use crate::mining::pallet::OnReward;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid = 0
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					100 * DOLLARS
				));
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1).to_bits(),
					payout: FixedPoint::from_num(500).to_bits(),
				}]);
				assert_ok!(Balances::set_balance(
					Origin::root(),
					PhalaMining::account_id(),
					1 * DOLLARS,
					0
				));
				assert_noop!(
					PhalaStakePool::claim_rewards(Origin::signed(1), 0, 1),
					Error::<Test>::InternalSubsidyPoolCannotWithdraw
				);
			});
		}

		#[test]
		fn test_withdraw() {
			use crate::mining::pallet::OnReclaim;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_pool_with_workers(1, &[1, 2]); // pid = 0
				let sub_account1 = pool_sub_account(0, &worker_pubkey(1));
				let sub_account2 = pool_sub_account(0, &worker_pubkey(2));

				// Stake 1000 PHA, and start two miners with 400 & 100 PHA as stake
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					1000 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					400 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(2),
					100 * DOLLARS
				));
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(staker2.amount, 1000 * DOLLARS);
				assert_eq!(Balances::locks(2), vec![the_lock(1000 * DOLLARS)]);
				// Cannot withdraw more than one's stake
				assert_noop!(
					PhalaStakePool::withdraw(Origin::signed(2), 0, 9999 * DOLLARS),
					Error::<Test>::InvalidWithdrawalAmount
				);
				// Immediate withdraw 499 PHA from the free stake
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					499 * DOLLARS
				));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.free_stake, 1 * DOLLARS);
				assert_eq!(pool.total_stake, 501 * DOLLARS);
				assert_eq!(staker2.amount, 501 * DOLLARS);
				assert_eq!(Balances::locks(2), vec![the_lock(501 * DOLLARS)]);
				// Withdraw 2 PHA will only fulfill 1 PHA from the free stake, leaving 1 PHA in the
				// withdraw queue
				assert_ok!(PhalaStakePool::withdraw(Origin::signed(2), 0, 2 * DOLLARS));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.free_stake, 0);
				assert_eq!(pool.total_stake, 500 * DOLLARS);
				assert_eq!(staker2.amount, 500 * DOLLARS);
				assert_eq!(Balances::locks(2), vec![the_lock(500 * DOLLARS)]);
				// Check the queue
				assert_eq!(
					pool.withdraw_queue,
					vec![WithdrawInfo {
						user: 2,
						amount: 1 * DOLLARS,
						start_time: 0
					}]
				);
				let ts_queue = PhalaStakePool::withdrawal_timestamps();
				assert_eq!(ts_queue.len(), 1);
				assert_eq!(
					PhalaStakePool::withdrawal_queued_pools(ts_queue.front().unwrap()),
					Some(vec![0])
				);

				// Contribute 1 PHA to trigger instant withdraw, fulfilling the withdraw request.
				// Then staker1 has 1PHA in stake, and staker2 only has 499 PHA in stake.
				let _ = take_events();
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					1 * DOLLARS
				));
				assert_eq!(
					take_events().as_slice(),
					[
						TestEvent::PhalaStakePool(Event::Withdrawal(0, 2, 1 * DOLLARS)),
						TestEvent::PhalaStakePool(Event::Contribution(0, 1, 1 * DOLLARS))
					]
				);
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.free_stake, 0);
				assert_eq!(pool.total_stake, 500 * DOLLARS);
				assert_eq!(pool.withdraw_queue.is_empty(), true);
				assert_eq!(staker1.amount, 1 * DOLLARS);
				assert_eq!(staker2.amount, 499 * DOLLARS);
				assert_eq!(Balances::locks(2), vec![the_lock(499 * DOLLARS)]);
				// Staker2 and 1 withdraw 199 PHA, 1 PHA, queued, and then wait for force clear
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					199 * DOLLARS
				));
				assert_ok!(PhalaStakePool::withdraw(Origin::signed(1), 0, 1 * DOLLARS));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(
					pool.withdraw_queue,
					vec![
						WithdrawInfo {
							user: 2,
							amount: 199 * DOLLARS,
							start_time: 0
						},
						WithdrawInfo {
							user: 1,
							amount: 1 * DOLLARS,
							start_time: 0
						}
					]
				);
				assert_eq!(staker1.amount, 1 * DOLLARS);
				assert_eq!(staker2.amount, 499 * DOLLARS);
				// Trigger a force clear by `on_reclaim()`, releasing 100 PHA stake to partially
				// fulfill staker2's withdraw request, but leaving staker1's untouched.
				let _ = take_events();
				PhalaStakePool::on_reclaim(&sub_account2, 100 * DOLLARS);
				assert_eq!(
					take_events().as_slice(),
					[TestEvent::PhalaStakePool(Event::Withdrawal(
						0,
						2,
						100 * DOLLARS
					)),]
				);
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.total_stake, 400 * DOLLARS);
				assert_eq!(pool.free_stake, 0);
				assert_eq!(staker1.amount, 1 * DOLLARS);
				assert_eq!(staker2.amount, 399 * DOLLARS);
				// Trigger another force clear, releasing all the remaining 400 PHA stake,
				// fulfilling staker2's request.
				// Then all 300 PHA becomes free, and there are 1 & 300 PHA loced by the stakers.
				let _ = take_events();
				PhalaStakePool::on_reclaim(&sub_account1, 400 * DOLLARS);
				assert_eq!(
					take_events().as_slice(),
					[
						TestEvent::PhalaStakePool(Event::Withdrawal(0, 2, 99 * DOLLARS)),
						TestEvent::PhalaStakePool(Event::Withdrawal(0, 1, 1 * DOLLARS))
					]
				);
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.total_stake, 300 * DOLLARS);
				assert_eq!(pool.free_stake, 300 * DOLLARS);
				assert_eq!(staker1.amount, 0);
				assert_eq!(staker2.amount, 300 * DOLLARS);
				assert_eq!(Balances::locks(1), vec![]);
				assert_eq!(Balances::locks(2), vec![the_lock(300 * DOLLARS)]);

				// TODO: handle slash at on_reclaim()
			});
		}

		#[test]
		fn test_full_procedure() {
			new_test_ext().execute_with(|| {
				set_block_1();
				let worker1 = worker_pubkey(1);
				let worker2 = worker_pubkey(2);
				let worker3 = worker_pubkey(3);
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
				assert_ok!(PhalaRegistry::force_register_worker(
					Origin::root(),
					worker3.clone(),
					ecdh_pubkey(3),
					Some(1)
				));
				PhalaRegistry::internal_set_benchmark(&worker1, Some(1));
				PhalaRegistry::internal_set_benchmark(&worker2, Some(1));
				PhalaRegistry::internal_set_benchmark(&worker3, Some(1));

				// Create a pool (pid = 0)
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				let _ = take_events();
				assert_ok!(PhalaStakePool::set_payout_pref(
					Origin::signed(1),
					0,
					Permill::from_percent(50)
				));
				assert_eq!(
					take_events().as_slice(),
					[TestEvent::PhalaStakePool(Event::PoolCommissionSet(
						0,
						1000_000u32 * 50 / 100
					))]
				);
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
				// Create a pool (pid = 1)
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				assert_ok!(PhalaStakePool::add_worker(
					Origin::signed(1),
					1,
					worker3.clone()
				));
				// Contribute 300 PHA to pool0, 300 to pool1
				assert_ok!(PhalaStakePool::set_cap(Origin::signed(1), 0, 300 * DOLLARS));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					100 * DOLLARS
				));
				assert_eq!(StakeLedger::<Test>::get(1).unwrap(), 100 * DOLLARS);
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					1,
					300 * DOLLARS
				));
				assert_eq!(StakeLedger::<Test>::get(1).unwrap(), 400 * DOLLARS);
				assert_eq!(
					StakePools::<Test>::get(0).unwrap().total_stake,
					100 * DOLLARS
				);
				assert_eq!(
					PoolStakers::<Test>::get(&(0, 1)).unwrap().amount,
					100 * DOLLARS
				);

				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					200 * DOLLARS
				));
				assert_eq!(
					StakePools::<Test>::get(0).unwrap().total_stake,
					300 * DOLLARS
				);
				assert_eq!(
					PoolStakers::<Test>::get(&(0, 2)).unwrap().amount,
					200 * DOLLARS
				);
				// Shouldn't exceed the pool cap
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 0, 100 * DOLLARS),
					Error::<Test>::StakeExceedsCapacity
				);
				// Start mining on pool0 (stake 100 for worker1, 100 for worke2)
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
				assert_eq!(PhalaMining::online_miners(), 2);
				// Withdraw 100 free funds
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(1),
					0,
					100 * DOLLARS
				));
				assert_eq!(StakeLedger::<Test>::get(1).unwrap(), 300 * DOLLARS);

				// TODO: check queued withdraw
				//   - withdraw 100 PHA
				//   - stop a worker
				//   - wait CD, withdraw succeeded
				//   - withdraw another 100 PHA
				//   - wait 3d, force stop
				//   - wait 7d, withdraw succeeded

				// Stop mining
				assert_ok!(PhalaStakePool::stop_mining(
					Origin::signed(1),
					0,
					worker1.clone()
				));
				assert_ok!(PhalaStakePool::stop_mining(
					Origin::signed(1),
					0,
					worker2.clone()
				));
				assert_eq!(PhalaMining::online_miners(), 0);
				let sub_account1: u64 = pool_sub_account(0, &worker1);
				let sub_account2: u64 = pool_sub_account(0, &worker2);
				let miner1 = PhalaMining::miners(&sub_account1).unwrap();
				let miner2 = PhalaMining::miners(&sub_account2).unwrap();
				assert_eq!(miner1.state, mining::MinerState::MiningCoolingDown);
				assert_eq!(miner2.state, mining::MinerState::MiningCoolingDown);
				// Wait the cool down period
				elapse_cool_down();
				assert_ok!(PhalaMining::reclaim(
					Origin::signed(1),
					sub_account1.clone()
				));
				assert_ok!(PhalaMining::reclaim(
					Origin::signed(1),
					sub_account2.clone()
				));
				// All stake get returend
				let pool0 = PhalaStakePool::stake_pools(0).unwrap();
				assert_eq!(pool0.free_stake, 200 * DOLLARS);
				// Withdraw the stakes
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					200 * DOLLARS
				));
				// Stop pool1 and withdraw stake as well
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(1),
					1,
					300 * DOLLARS
				));
				// Settle everything
				assert_eq!(StakeLedger::<Test>::get(1).unwrap(), 0);
				assert_eq!(StakeLedger::<Test>::get(1).unwrap(), 0);
				assert!(Balances::locks(1).is_empty());
				assert!(Balances::locks(2).is_empty());
			});
		}

		fn the_lock(amount: Balance) -> pallet_balances::BalanceLock<Balance> {
			pallet_balances::BalanceLock {
				id: STAKING_ID,
				amount,
				reasons: pallet_balances::Reasons::All,
			}
		}

		/// Sets up a stakepool with the given workers added.
		///
		/// Returns the pool id.
		fn setup_pool_with_workers(owner: u64, workers: &[u8]) -> u64 {
			let pid = PhalaStakePool::pool_count();
			assert_ok!(PhalaStakePool::create(Origin::signed(owner)));
			for id in workers {
				assert_ok!(PhalaStakePool::add_worker(
					Origin::signed(owner),
					pid,
					worker_pubkey(*id),
				));
			}
			pid
		}
	}
}
