pub use self::pallet::*;

use frame_support::traits::Currency;
use sp_runtime::traits::Zero;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use crate::accumulator::Accumulator;
	use crate::balance_convert::{div as bdiv, mul as bmul, FixedPointConvert};
	use crate::fixed_point::CodecFixedPoint;
	use crate::mining;
	use crate::registry;

	use fixed::types::U64F64 as FixedPoint;
	use fixed_macro::types::U64F64 as fp;

	use super::{
		balance_close_to_zero, balances_nearly_equal, extract_dust, is_nondust_balance, BalanceOf,
		NegativeImbalanceOf,
	};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			Currency, Imbalance, LockIdentifier, LockableCurrency, OnUnbalanced, StorageVersion,
			UnixTime, WithdrawReasons,
		},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{Saturating, TrailingZeroInput, Zero},
		Permill, SaturatedConversion,
	};
	use sp_std::{collections::vec_deque::VecDeque, fmt::Display, prelude::*, vec};

	use phala_types::{messaging::SettleInfo, WorkerPublicKey};

	const STAKING_ID: LockIdentifier = *b"phala/sp";

	pub trait Ledger<AccountId, Balance> {
		/// Increases the locked amount for a user
		///
		/// Unsafe: it assumes there's enough free `amount`
		fn ledger_accrue(who: &AccountId, amount: Balance);
		/// Decreases the locked amount for a user
		///
		/// Optionally remove some dust by `Currency::slash` and move it to the Treasury.
		/// Unsafe: it assumes there's enough locked `amount`
		fn ledger_reduce(who: &AccountId, amount: Balance, dust: Balance);
		/// Gets the locked amount of `who`
		fn ledger_query(who: &AccountId) -> Balance;
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + registry::Config + mining::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

		#[pallet::constant]
		type MinContribution: Get<BalanceOf<Self>>;

		/// The grace period for force withdraw request, in seconds.
		#[pallet::constant]
		type GracePeriod: Get<u64>;

		/// If mining is enabled by default.
		#[pallet::constant]
		type MiningEnabledByDefault: Get<bool>;

		/// The max allowed workers in a pool
		#[pallet::constant]
		type MaxPoolWorkers: Get<u32>;

		/// The handler to absorb the slashed amount.
		type OnSlashed: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// The origin that can turn on or off mining
		type MiningSwitchOrigin: EnsureOrigin<Self::Origin>;

		/// The origin that can trigger backfill tasks.
		type BackfillOrigin: EnsureOrigin<Self::Origin>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Mapping from pool id to PoolInfo
	#[pallet::storage]
	#[pallet::getter(fn stake_pools)]
	pub type StakePools<T: Config> =
		StorageMap<_, Twox64Concat, u64, PoolInfo<T::AccountId, BalanceOf<T>>>;

	/// Mapping from (pid, staker) to UserStakeInfo
	#[pallet::storage]
	#[pallet::getter(fn pool_stakers)]
	pub type PoolStakers<T: Config> =
		StorageMap<_, Twox64Concat, (u64, T::AccountId), UserStakeInfo<T::AccountId, BalanceOf<T>>>;

	/// The number of total pools
	#[pallet::storage]
	#[pallet::getter(fn pool_count)]
	pub type PoolCount<T> = StorageValue<_, u64, ValueQuery>;

	/// Mapping from workers to the pool they belong to
	///
	/// The map entry lasts from `add_worker()` to `remove_worker()` or force unbinding.
	#[pallet::storage]
	pub type WorkerAssignments<T: Config> = StorageMap<_, Twox64Concat, WorkerPublicKey, u64>;

	/// (Deprecated)
	// TODO: remove it
	#[pallet::storage]
	pub type SubAccountAssignments<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u64>;

	/// Mapping staker to it's the balance locked in all pools
	#[pallet::storage]
	#[pallet::getter(fn stake_ledger)]
	pub type StakeLedger<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

	/// Mapping from the block timestamp to pools that has withdrawal requests queued in that block
	#[pallet::storage]
	#[pallet::getter(fn withdrawal_queued_pools)]
	pub type WithdrawalQueuedPools<T: Config> = StorageMap<_, Twox64Concat, u64, Vec<u64>>;

	/// Queue that contains all block's timestamp, in that block contains the waiting withdraw reqeust.
	/// This queue has a max size of (T::GracePeriod * 8) bytes
	#[pallet::storage]
	#[pallet::getter(fn withdrawal_timestamps)]
	pub type WithdrawalTimestamps<T> = StorageValue<_, VecDeque<u64>, ValueQuery>;

	/// Switch to enable the stake pool pallet (disabled by default)
	#[pallet::storage]
	#[pallet::getter(fn mining_enabled)]
	pub type MiningEnabled<T> = StorageValue<_, bool, ValueQuery, MiningEnabledByDefault<T>>;

	#[pallet::type_value]
	pub fn MiningEnabledByDefault<T: Config>() -> bool {
		T::MiningEnabledByDefault::get()
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// \[owner, pid\]
		PoolCreated(T::AccountId, u64),
		/// The real commission ratio is commission/1_000_000u32. \[pid, commission\]
		PoolCommissionSet(u64, u32),
		/// \[pid, cap\]
		PoolCapacitySet(u64, BalanceOf<T>),
		/// \[pid, worker\]
		PoolWorkerAdded(u64, WorkerPublicKey),
		/// \[pid, user, amount\]
		Contribution(u64, T::AccountId, BalanceOf<T>),
		/// \[pid, user, amount\]
		Withdrawal(u64, T::AccountId, BalanceOf<T>),
		/// \[pid, user, amount\]
		RewardsWithdrawn(u64, T::AccountId, BalanceOf<T>),
		/// \[pid, amount\]
		PoolSlashed(u64, BalanceOf<T>),
		/// \[pid, account, amount\]
		SlashSettled(u64, T::AccountId, BalanceOf<T>),
		/// Some reward is dismissed because the worker is no longer bound to a pool. \[worker, amount\]
		RewardDismissedNotInPool(WorkerPublicKey, BalanceOf<T>),
		/// Some reward is dismissed because the pool doesn't have any share. \[pid, amount\]
		RewardDismissedNoShare(u64, BalanceOf<T>),
		/// Some reward is dismissed because the amount is too tiny (dust). \[pid, amount\]
		RewardDismissedDust(u64, BalanceOf<T>),
		/// Some dust stake is removed. \[user, amount\]
		DustRemoved(T::AccountId, BalanceOf<T>),
		/// A worker is removed from a pool.
		PoolWorkerRemoved { pid: u64, worker: WorkerPublicKey },
		/// A withdrawal request is queued by adding or replacing an old one.
		WithdrawalQueued {
			pid: u64,
			user: T::AccountId,
			shares: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		WorkerNotRegistered,
		BenchmarkMissing,
		WorkerExists,
		WorkerDoesNotExist,
		WorkerInAnotherPool,
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
		/// The pool has already got all the stake completely slashed.
		///
		/// In this case, no more funds can be contributed to the pool until all the pending slash
		/// has been resolved.
		PoolBankrupt,
		NoRewardToClaim,
		/// The StakePool is not enabled yet.
		FeatureNotEnabled,
		/// Failed to add a worker because the number of the workers exceeds the upper limit.
		WorkersExceedLimit,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
		BalanceOf<T>: FixedPointConvert + Display,
	{
		fn on_finalize(_n: T::BlockNumber) {
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			Self::maybe_force_withdraw(now);
		}

		fn on_runtime_upgrade() -> Weight {
			let mut w = 0;
			let old = Self::on_chain_storage_version();
			w += T::DbWeight::get().reads(1);

			if old == 0 {
				w += super::migrations::migrate_to_v1::<T>();
				STORAGE_VERSION.put::<super::Pallet<T>>();
				w += T::DbWeight::get().writes(1);
			}
			w
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
		BalanceOf<T>: FixedPointConvert + Display,
	{
		/// Creates a new stake pool
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let pid = PoolCount::<T>::get();
			StakePools::<T>::insert(
				pid,
				PoolInfo {
					pid,
					owner: owner.clone(),
					payout_commission: None,
					owner_reward: Zero::zero(),
					cap: None,
					reward_acc: CodecFixedPoint::zero(),
					total_shares: Zero::zero(),
					total_stake: Zero::zero(),
					free_stake: Zero::zero(),
					releasing_stake: Zero::zero(),
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

			// origin must be the owner of the pool
			let mut pool_info = Self::ensure_pool(pid)?;
			ensure!(pool_info.owner == owner, Error::<T>::UnauthorizedPoolOwner);
			// make sure worker has not been not added
			let workers = &mut pool_info.workers;
			ensure!(!workers.contains(&pubkey), Error::<T>::WorkerExists);
			// too many workers may cause performance regression
			ensure!(
				workers.len() + 1 <= T::MaxPoolWorkers::get() as usize,
				Error::<T>::WorkersExceedLimit
			);

			// generate miner account
			let miner: T::AccountId = pool_sub_account(pid, &pubkey);

			// bind worker with minner
			mining::pallet::Pallet::<T>::bind(miner.clone(), pubkey)
				.or(Err(Error::<T>::FailedToBindMinerAndWorker))?;

			// update worker vector
			workers.push(pubkey);
			StakePools::<T>::insert(&pid, &pool_info);
			WorkerAssignments::<T>::insert(&pubkey, pid);
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
			let lookup_pid =
				WorkerAssignments::<T>::get(worker).ok_or(Error::<T>::WorkerDoesNotExist)?;
			ensure!(pid == lookup_pid, Error::<T>::WorkerInAnotherPool);
			// Remove the worker from the pool (notification suspended)
			let sub_account: T::AccountId = pool_sub_account(pid, &worker);
			mining::pallet::Pallet::<T>::unbind_miner(&sub_account, false)?;
			// Manually clean up the worker, including the pool worker list, and the assignment
			// indices. (Theoritically we can enable the unbinding notification, and follow the
			// same path as a force unbinding, but it doesn't sounds graceful.)
			Self::remove_worker_from_pool(&worker);
			Ok(())
		}

		// /// Destroies a stake pool
		// ///
		// /// Requires:
		// /// 1. The sender is the owner
		// /// 2. All the miners are stopped
		// #[pallet::weight(0)]
		// pub fn destroy(origin: OriginFor<T>, id: u64) -> DispatchResult {
		// 	panic!("unimplemented")
		// }

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
		/// 1. The sender is a pool owner or staker
		#[pallet::weight(0)]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			pid: u64,
			target: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut pool_info = Self::ensure_pool(pid)?;
			let mut rewards = BalanceOf::<T>::zero();
			// Add pool owner's reward if applicable
			if who == pool_info.owner {
				rewards += pool_info.owner_reward;
				pool_info.owner_reward = Zero::zero();
			}
			// Settle the pending reward, and calculate the rewards belong to user
			let info_key = (pid, who.clone());
			let mut user_info = Self::pool_stakers(&info_key);
			if let Some(ref mut user_info) = user_info {
				pool_info.settle_user_pending_reward(user_info);
				rewards += user_info.available_rewards;
				user_info.available_rewards = Zero::zero();
			}
			ensure!(rewards > Zero::zero(), Error::<T>::NoRewardToClaim);
			mining::Pallet::<T>::withdraw_subsidy_pool(&target, rewards)
				.or(Err(Error::<T>::InternalSubsidyPoolCannotWithdraw))?;
			// Update ledger
			StakePools::<T>::insert(pid, &pool_info);
			if let Some(user_info) = user_info {
				PoolStakers::<T>::insert(&info_key, &user_info);
			}
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
			let free = <T as Config>::Currency::free_balance(&who);
			let locked = Self::ledger_query(&who);
			ensure!(free - locked >= a, Error::<T>::InsufficientBalance);

			let mut pool_info = Self::ensure_pool(pid)?;
			if let Some(cap) = pool_info.cap {
				ensure!(
					cap.saturating_sub(pool_info.total_stake) >= a,
					Error::<T>::StakeExceedsCapacity
				);
			}

			// We don't really want to allow to contribute to a bankrupt StakePool. It can avoid
			// a lot of weird edge cases when dealing with pending slash.
			ensure!(
				// There's no share, meaning the pool is empty;
				pool_info.total_shares == Zero::zero()
				// or there's no trivial `total_stake`, meaning it's still operating normally
				|| pool_info.total_stake > Zero::zero(),
				Error::<T>::PoolBankrupt
			);

			let info_key = (pid, who.clone());
			// Clear the pending reward before adding stake, if applies
			let mut user_info = match Self::pool_stakers(&info_key) {
				Some(mut user_info) => {
					pool_info.settle_user_pending_reward(&mut user_info);
					Self::maybe_settle_slash(&pool_info, &mut user_info);
					user_info
				}
				None => UserStakeInfo {
					user: who.clone(),
					locked: Zero::zero(),
					shares: Zero::zero(),
					available_rewards: Zero::zero(),
					reward_debt: Zero::zero(),
				},
			};
			pool_info.add_stake(&mut user_info, a);

			// Persist
			PoolStakers::<T>::insert(&info_key, &user_info);
			// Lock the funds
			Self::ledger_accrue(&who, a);

			// We have new free stake now, try to handle the waiting withdraw queue
			Self::try_process_withdraw_queue(&mut pool_info);

			// Persist
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
		pub fn withdraw(origin: OriginFor<T>, pid: u64, shares: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let info_key = (pid, who.clone());
			let mut user_info =
				Self::pool_stakers(&info_key).ok_or(Error::<T>::PoolStakeNotFound)?;

			ensure!(
				is_nondust_balance(shares) && shares <= user_info.shares,
				Error::<T>::InvalidWithdrawalAmount
			);
			// TODO(hangyin): consider the amounts in the withdraw request
			// https://github.com/Phala-Network/phala-blockchain/issues/490

			let mut pool_info = Self::ensure_pool(pid)?;
			Self::try_withdraw(&mut pool_info, &mut user_info, shares);

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
			ensure!(Self::mining_enabled(), Error::<T>::FeatureNotEnabled);
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
			mining::pallet::Pallet::<T>::start_mining(miner, stake)?;
			pool_info.free_stake -= stake;
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
			ensure!(Self::mining_enabled(), Error::<T>::FeatureNotEnabled);
			let pool_info = Self::ensure_pool(pid)?;
			// origin must be owner of pool
			ensure!(pool_info.owner == owner, Error::<T>::UnauthorizedPoolOwner);
			// check wheather we have add this worker
			ensure!(
				pool_info.workers.contains(&worker),
				Error::<T>::WorkerDoesNotExist
			);
			let miner: T::AccountId = pool_sub_account(pid, &worker);
			// Mining::stop_mining will notify us how much it will release by `on_stopped`
			<mining::pallet::Pallet<T>>::stop_mining(miner)?;

			Ok(())
		}

		/// Reclaims the releasing stake of a miner in a pool.
		#[pallet::weight(0)]
		pub fn reclaim_pool_worker(
			origin: OriginFor<T>,
			pid: u64,
			worker: WorkerPublicKey,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::ensure_pool(pid)?;
			let sub_account: T::AccountId = pool_sub_account(pid, &worker);
			let (orig_stake, slashed) = mining::Pallet::<T>::reclaim(sub_account)?;
			Self::handle_reclaim(pid, orig_stake, slashed);
			Ok(())
		}

		/// Enables or disables mining. Must be called with the council or root permission.
		#[pallet::weight(0)]
		pub fn set_mining_enable(origin: OriginFor<T>, enable: bool) -> DispatchResult {
			T::MiningSwitchOrigin::ensure_origin(origin)?;
			MiningEnabled::<T>::put(enable);
			Ok(())
		}

		// TODO(hangyin): remove once after issue 527 is closed.
		/// Temporary function to reconcile incorrect withdraw queue (issue 527).
		#[pallet::weight(195_000_000)]
		pub fn reconcile_withdraw_queue(
			origin: OriginFor<T>,
			pid: u64,
			account: T::AccountId,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let mut pool_info = Self::ensure_pool(pid)?;
			let info_key = (pid, account.clone());
			let user_info = Self::pool_stakers(&info_key).ok_or(Error::<T>::PoolStakeNotFound)?;
			// We don't care about dust
			let (available_shares, _dust) = extract_dust(user_info.shares);
			// Update the withdraw request if it exceeds the actual available shares, or remove it
			// if there's no share at all.
			if let Some(idx) = pool_info
				.withdraw_queue
				.iter()
				.position(|req| req.user == account && req.shares > available_shares)
			{
				if available_shares == Zero::zero() {
					pool_info.withdraw_queue.remove(idx);
				} else {
					pool_info.withdraw_queue.get_mut(idx).unwrap().shares = available_shares;
				}
				StakePools::<T>::insert(pid, pool_info);
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
		BalanceOf<T>: FixedPointConvert + Display,
	{
		/// Adds up the newly received reward to `reward_acc`
		fn handle_pool_new_reward(
			pool_info: &mut PoolInfo<T::AccountId, BalanceOf<T>>,
			rewards: BalanceOf<T>,
		) {
			if rewards > Zero::zero() {
				if balance_close_to_zero(pool_info.total_shares) {
					Self::deposit_event(Event::<T>::RewardDismissedNoShare(pool_info.pid, rewards));
					return;
				}
				let commission = pool_info.payout_commission.unwrap_or_default() * rewards;
				pool_info.owner_reward.saturating_accrue(commission);
				let to_distribute = rewards - commission;
				if is_nondust_balance(to_distribute) {
					pool_info.distribute_reward(to_distribute);
				} else if to_distribute > Zero::zero() {
					Self::deposit_event(Event::<T>::RewardDismissedDust(
						pool_info.pid,
						to_distribute,
					));
				}
			}
		}

		/// Called when worker was reclaimed.
		///
		/// After the cool down ends, worker was cleaned up, whose contributed balance would be
		/// reset to zero.
		fn handle_reclaim(pid: u64, orig_stake: BalanceOf<T>, slashed: BalanceOf<T>) {
			let mut pool_info = Self::ensure_pool(pid).expect("Stake pool must exist; qed.");

			let returned = orig_stake - slashed;
			if slashed != Zero::zero() {
				// Remove some slashed value from `total_stake`, causing the share price to reduce
				// and creating a logical pending slash. The actual slash happens with the pending
				// slash to individuals is settled.
				pool_info.slash(slashed);
				Self::deposit_event(Event::<T>::PoolSlashed(pid, slashed));
			}

			// With the worker being cleaned, those stake now are free
			debug_assert!(
				pool_info.releasing_stake >= returned,
				"More return then expected"
			);
			pool_info.free_stake.saturating_accrue(returned);
			pool_info.releasing_stake.saturating_reduce(returned);

			Self::try_process_withdraw_queue(&mut pool_info);
			StakePools::<T>::insert(&pid, &pool_info);
		}

		/// Tries to withdraw a specific amount from a pool.
		///
		/// The withdraw request would be delayed if the free stake is not enough, otherwise
		/// withdraw from the free stake immediately.
		///
		/// The updates are made in `pool_info` and `user_info`. It's up to the caller to persist
		/// the data.
		///
		/// Requires:
		/// 1. The user's pending slash is already settled.
		/// 2. The pool must has shares and stake (or it can cause division by zero error)
		fn try_withdraw(
			pool_info: &mut PoolInfo<T::AccountId, BalanceOf<T>>,
			user_info: &mut UserStakeInfo<T::AccountId, BalanceOf<T>>,
			shares: BalanceOf<T>,
		) {
			pool_info.settle_user_pending_reward(user_info);
			let free_shares = match pool_info.share_price() {
				Some(price) if price != fp!(0) => bdiv(pool_info.free_stake, &price),
				// LOL, 100% slashed. We allow to withdraw all any number of shares with zero token
				// in return.
				_ => shares,
			};
			// The user is requesting to withdraw `shares`. So we compare the maximal free shares
			// with that of the request. The `shares` in the request is splitted to:
			// - `withdraw_shares`: can be withdrawn immedately
			// - `queued_shares`: enqueued in the withdrawl queue
			// We remove the dust in both values.
			let withdrawing_shares = shares.min(free_shares);
			let (withdrawing_shares, _) = extract_dust(withdrawing_shares);
			let queued_shares = shares - withdrawing_shares;
			let (queued_shares, _) = extract_dust(queued_shares);
			// Try withdraw immediately if we can
			if withdrawing_shares > Zero::zero() {
				Self::maybe_settle_slash(pool_info, user_info);
				// Overflow warning: remove_stake is carefully written to avoid precision error.
				// (I hope so)
				let (reduced, dust) = pool_info
					.remove_stake(user_info, withdrawing_shares)
					.expect("There are enough withdrawing_shares; qed.");
				Self::ledger_reduce(&user_info.user, reduced, dust);
				Self::deposit_event(Event::<T>::Withdrawal(
					pool_info.pid,
					user_info.user.clone(),
					reduced,
				));
			}
			// Some locked assets haven't been withdrawn (unlocked) to user, add it to the withdraw
			// queue. When the pool has free stake again, the withdrawal will be fulfilled.
			if queued_shares > Zero::zero() {
				// Remove the existing withdraw request in the queue if there is any.
				pool_info
					.withdraw_queue
					.retain(|withdraw| withdraw.user != user_info.user);
				// Push the request
				let now = <T as registry::Config>::UnixTime::now()
					.as_secs()
					.saturated_into::<u64>();
				pool_info.withdraw_queue.push_back(WithdrawInfo {
					user: user_info.user.clone(),
					shares: queued_shares,
					start_time: now,
				});
				Self::maybe_add_withdraw_queue(now, pool_info.pid);
				Self::deposit_event(Event::<T>::WithdrawalQueued {
					pid: pool_info.pid,
					user: user_info.user.clone(),
					shares: queued_shares,
				});
			}
			// Update the pending reward after changing the staked amount
			pool_info.reset_pending_reward(user_info);
		}

		/// Tries to fulfill the withdraw queue with the newly freed stake
		fn try_process_withdraw_queue(pool_info: &mut PoolInfo<T::AccountId, BalanceOf<T>>) {
			// The share price shouldn't change at any point in this function. So we can calculate
			// only once at the beginning.
			let price = match pool_info.share_price() {
				Some(price) => price,
				None => return,
			};

			while is_nondust_balance(pool_info.free_stake) {
				if let Some(mut withdraw) = pool_info.withdraw_queue.front().cloned() {
					// Must clear the pending reward before any stake change
					let info_key = (pool_info.pid, withdraw.user.clone());
					let mut user_info = match Self::pool_stakers(&info_key) {
						Some(user) => user,
						// Usually it shouldn't be the case but we still check as a safe-guard
						None => {
							pool_info.withdraw_queue.pop_front();
							continue;
						}
					};
					pool_info.settle_user_pending_reward(&mut user_info);
					// Try to fulfill the withdraw requests as much as possible
					let free_shares = if price == fp!(0) {
						withdraw.shares // 100% slashed
					} else {
						bdiv(pool_info.free_stake, &price)
					};
					// This is the shares to withdraw immedately. It should NOT contain any dust
					// because we ensure (1) `free_shares` is not dust earlier, and (2) the shares
					// in any withdraw request mustn't be dust when inserting and updating it.
					let withdrawing_shares = free_shares.min(withdraw.shares);
					debug_assert!(
						is_nondust_balance(withdrawing_shares),
						"withdrawing_shares must be positive"
					);
					// Actually remove the fulfilled withdraw request. Dust in the user shares is
					// considered but it in the request is ignored.
					Self::maybe_settle_slash(pool_info, &mut user_info);
					let (reduced, dust) = pool_info
						.remove_stake(&mut user_info, withdrawing_shares)
						.expect("Remove only what we have; qed.");
					let (shares, _) = extract_dust(withdraw.shares - withdrawing_shares);
					withdraw.shares = shares;
					// Withdraw the funds
					Self::ledger_reduce(&user_info.user, reduced, dust);
					Self::deposit_event(Event::<T>::Withdrawal(
						pool_info.pid,
						user_info.user.clone(),
						reduced,
					));
					// Update the pending reward after changing the staked amount
					pool_info.reset_pending_reward(&mut user_info);
					PoolStakers::<T>::insert(&info_key, &user_info);
					// Update if the withdraw is partially fulfilled, otherwise pop it out of the
					// queue
					if withdraw.shares == Zero::zero() {
						pool_info.withdraw_queue.pop_front();
					} else {
						*pool_info
							.withdraw_queue
							.front_mut()
							.expect("front exists as just checked; qed.") = withdraw;
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

		/// Removes some dust amount from a user's account by Currency::slash.
		fn remove_dust(who: &T::AccountId, dust: BalanceOf<T>) {
			debug_assert!(dust != Zero::zero());
			if dust != Zero::zero() {
				let (imbalance, _remaining) = <T as Config>::Currency::slash(who, dust);
				let actual_removed = imbalance.peek();
				T::OnSlashed::on_unbalanced(imbalance);
				Self::deposit_event(Event::<T>::DustRemoved(who.clone(), actual_removed));
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
			if let Some(mut pool_list) = WithdrawalQueuedPools::<T>::get(&start_time) {
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
			StakePools::<T>::mutate(pid, |value| {
				if let Some(pool) = value {
					pool.remove_worker(worker);
					Self::deposit_event(Event::<T>::PoolWorkerRemoved {
						pid,
						worker: worker.clone(),
					});
				}
			});
		}

		fn maybe_settle_slash(
			pool: &PoolInfo<T::AccountId, BalanceOf<T>>,
			user: &mut UserStakeInfo<T::AccountId, BalanceOf<T>>,
		) {
			match pool.settle_slash(user) {
				// We don't slash on dust, because the share price is just unstable.
				Some(slashed) if is_nondust_balance(slashed) => {
					let (imbalance, _remaining) =
						<T as Config>::Currency::slash(&user.user, slashed);
					let actual_slashed = imbalance.peek();
					T::OnSlashed::on_unbalanced(imbalance);
					// Dust is not considered because it's already merged into the slash if
					// presents.
					Self::ledger_reduce(&user.user, actual_slashed, Zero::zero());
					Self::deposit_event(Event::<T>::SlashSettled(
						pool.pid,
						user.user.clone(),
						actual_slashed,
					));
				}
				_ => (),
			}
		}

		/// Tries to enforce expired withdraw requests
		///
		/// TODO: carefully examine the caveat in this function
		fn maybe_force_withdraw(now: u64) {
			// TODO: review again!
			let mut t = WithdrawalTimestamps::<T>::get();
			if t.is_empty() {
				return;
			}
			// Handle timeout requests at every block
			let grace_period = T::GracePeriod::get();
			while let Some(start_time) = t.front().cloned() {
				if now - start_time <= grace_period {
					break;
				}
				let pools = WithdrawalQueuedPools::<T>::take(start_time)
					.expect("Pool list must exist; qed.");
				for &pid in pools.iter() {
					let pool = Self::ensure_pool(pid).expect("Stake pool must exist; qed.");
					if pool.has_expired_withdrawal(now, grace_period) {
						// Force shutdown all miners
						for worker in pool.workers {
							let miner: T::AccountId = pool_sub_account(pid, &worker);
							// TODO: avoid stop mining multiple times?
							let _ = <mining::pallet::Pallet<T>>::stop_mining(miner);
						}
					}
				}
				// pop front timestamp
				t.pop_front();
			}
			WithdrawalTimestamps::<T>::put(&t);
		}
	}

	impl<T: Config> mining::OnReward for Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
		BalanceOf<T>: FixedPointConvert + Display,
	{
		/// Called when gk send new payout information.
		/// Append specific miner's reward balance of current round,
		/// would be clear once pool was updated
		fn on_reward(settle: &[SettleInfo]) {
			for info in settle {
				let payout_fixed = FixedPoint::from_bits(info.payout);
				let reward = BalanceOf::<T>::from_fixed(&payout_fixed);

				let pid = match WorkerAssignments::<T>::get(&info.pubkey) {
					Some(pid) => pid,
					None => {
						Self::deposit_event(Event::<T>::RewardDismissedNotInPool(
							info.pubkey,
							reward,
						));
						return;
					}
				};
				let mut pool_info = Self::ensure_pool(pid).expect("Stake pool must exist; qed.");
				Self::handle_pool_new_reward(&mut pool_info, reward);
				StakePools::<T>::insert(&pid, &pool_info);
			}
		}
	}

	impl<T: Config> mining::OnUnbound for Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
		BalanceOf<T>: FixedPointConvert + Display,
	{
		fn on_unbound(worker: &WorkerPublicKey, _force: bool) {
			// Usually called on worker force unbinding (force == true), but it's also possible
			// that the user unbind from the mining pallet directly.

			// Warning: when using Mining & StakePool pallets together, here we assume all the
			// miners are only registered by StakePool. So we don't bother to double check if the
			// worker exists.

			// In case of slash, `Mining::stop_mining()` will notify us a slash happened and we do
			// bookkeeping stuff (i.e. updating releasing_stake), and eventually the slash will
			// be enacted at `on_reclaim`.
			Self::remove_worker_from_pool(worker);
		}
	}

	impl<T: Config> mining::OnStopped<BalanceOf<T>> for Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
		BalanceOf<T>: FixedPointConvert + Display,
	{
		/// Called when a worker is stopped and there is releasing stake
		fn on_stopped(worker: &WorkerPublicKey, orig_stake: BalanceOf<T>, slashed: BalanceOf<T>) {
			let pid = WorkerAssignments::<T>::get(worker)
				.expect("Stopping workers have assignment; qed.");
			let mut pool_info = Self::ensure_pool(pid).expect("Stake pool must exist; qed.");
			let returned = orig_stake - slashed;
			pool_info.releasing_stake.saturating_accrue(returned);
			StakePools::<T>::insert(pid, pool_info);
		}
	}

	impl<T: Config> Ledger<T::AccountId, BalanceOf<T>> for Pallet<T>
	where
		T: mining::Config<Currency = <T as Config>::Currency>,
		BalanceOf<T>: FixedPointConvert + Display,
	{
		fn ledger_accrue(who: &T::AccountId, amount: BalanceOf<T>) {
			let b: BalanceOf<T> = StakeLedger::<T>::get(who).unwrap_or_default();
			let new_b = b.saturating_add(amount);
			StakeLedger::<T>::insert(who, new_b);
			Self::update_lock(who, new_b);
		}

		fn ledger_reduce(who: &T::AccountId, amount: BalanceOf<T>, dust: BalanceOf<T>) {
			let b: BalanceOf<T> = StakeLedger::<T>::get(who).unwrap_or_default();
			let to_remove = amount + dust;
			debug_assert!(b >= to_remove, "Cannot reduce lock more than it has");
			let new_b = b.saturating_sub(to_remove);
			StakeLedger::<T>::insert(who, new_b);
			Self::update_lock(who, new_b);
			if dust != Zero::zero() {
				Self::remove_dust(who, dust);
			}
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

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Default, RuntimeDebug)]
	pub struct PoolInfo<AccountId: Default, Balance> {
		/// Pool ID
		pub pid: u64,
		/// The owner of the pool
		pub owner: AccountId,
		/// The commission the pool owner takes
		pub payout_commission: Option<Permill>,
		/// Claimalbe owner reward
		pub owner_reward: Balance,
		/// The hard cap of the pool
		pub cap: Option<Balance>,
		/// The reward accumulator
		pub reward_acc: CodecFixedPoint,
		/// Total shares. Cannot be dust.
		pub total_shares: Balance,
		/// Total stake. Cannot be dust.
		pub total_stake: Balance,
		/// Total free stake. Can be dust.
		pub free_stake: Balance,
		/// Releasing stake (will be unlocked after worker reclaiming)
		pub releasing_stake: Balance,
		/// Bound workers
		pub workers: Vec<WorkerPublicKey>,
		/// The queue of withdraw requests
		pub withdraw_queue: VecDeque<WithdrawInfo<AccountId, Balance>>,
	}

	impl<AccountId, Balance> PoolInfo<AccountId, Balance>
	where
		AccountId: Default,
		Balance: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
	{
		/// Adds some stake to a user.
		///
		/// No dirty slash allowed. Usually it doesn't change the price of the share, unless the
		/// share price is zero (all slashed), which is a really a disrupting case that we don't
		/// even bother to deal with.
		fn add_stake(&mut self, user: &mut UserStakeInfo<AccountId, Balance>, amount: Balance) {
			debug_assert!(is_nondust_balance(amount));
			self.assert_slash_clean(user);
			self.assert_reward_clean(user);
			// Calcuate shares to add
			let shares = match self.share_price() {
				Some(price) if price != fp!(0) => bdiv(amount, &price),
				_ => amount, // adding new stake (share price = 1)
			};
			// Add the stake
			user.shares.saturating_accrue(shares);
			user.locked.saturating_accrue(amount);
			self.reset_pending_reward(user);
			// Update self
			self.total_shares += shares;
			self.total_stake.saturating_accrue(amount);
			self.free_stake.saturating_accrue(amount);
		}

		/// Removes some shares from a user and returns the removed stake amount.
		///
		/// This function can deal with fixed point precision issue (I hope so). However it also
		/// requires:
		///
		/// - There's no dirty slash
		/// - `shares` mustn't exceed the user shares.
		///
		/// It returns `None` and makes no change if there's any error. Otherwise it returns a
		/// tuple with the amount of the actual removed stake, and potentially the dust stake to
		/// remove.
		fn remove_stake(
			&mut self,
			user: &mut UserStakeInfo<AccountId, Balance>,
			shares: Balance,
		) -> Option<(Balance, Balance)> {
			debug_assert!(is_nondust_balance(shares));
			self.assert_slash_clean(user);
			self.assert_reward_clean(user);

			// It's tricky to deal with the fixed point precision loss. Generally we drop the dust
			// shares, because shares are just a virtual value representing the ownership of a
			// pool. However for balances, even the dust of 1 pico PHA must be absorbed.
			//
			// In StakePool, all the balances are PHA locked in user's account. Therefore once we
			// want to absorb some dust, we have to withdraw the funds from the users' stake, and
			// deposit it to the Treasury.
			let price = self.share_price()?;
			let amount = bmul(shares, &price);
			// In case `amount` is a little bit larger than `free_stake` or `user.locked`. Note
			// that it should never really exceed `user.locked` because we ask the caller to ensure
			// the share to remove is not greater than user's shares.
			let amount = amount.min(self.free_stake).min(user.locked);
			// Remove shares and stake from the user record
			let user_shares = user.shares.checked_sub(&shares)?;
			let (user_shares, shares_dust) = extract_dust(user_shares);
			let user_locked = user.locked.checked_sub(&amount)?;
			let (user_locked, user_dust) = extract_dust(user_locked);
			// Whenver we remove dust shares in user's account, we remove it in the total_shares
			// as well. It keeps the invariant:
			//   pool.total_shares == sum(pool_user.shares)
			let total_shares = self.total_shares.checked_sub(&(shares + shares_dust))?;
			debug_assert!(
				extract_dust(total_shares).1 == Zero::zero(),
				"total_shares should never have dust"
			);
			// Apply updates
			//
			// Note that ideally we should maintain the invariant:
			//   pool.total_stake == sum(pool_user.stake)
			//
			// When we extracted some dust, we should also deduct it from the total_stake. However,
			// the invarant will be broken anyway when a pool got any slash settled. For example,
			// when the total stake got slashed by 1 pico-PHA, the sum of pool users' stake will
			// be always larger than total_stake.
			//
			// So we'd rather just don't deduct the the removed dust at all. Instead, we drop the
			// dust in pool.total_stake after the user stake is removed. In other words, we always
			// drop the dust in total_stake without more bookkeeping.
			let (total_stake, _) = extract_dust(self.total_stake - amount);
			if total_stake > Zero::zero() {
				self.free_stake -= amount;
				self.total_stake -= amount;
			} else {
				self.free_stake = Zero::zero();
				self.total_stake = Zero::zero();
			}
			self.total_shares = total_shares;
			user.shares = user_shares;
			user.locked = user_locked;
			self.reset_pending_reward(user);
			Some((amount, user_dust))
		}

		/// Slashes the pool with dust removed.
		fn slash(&mut self, amount: Balance) {
			debug_assert!(
				is_nondust_balance(self.total_shares),
				"No share in the pool. This shouldn't happen."
			);
			debug_assert!(
				self.total_stake >= amount,
				"No enough stake to slash (total = {}, slash = {})",
				self.total_stake,
				amount
			);
			let amount = self.total_stake.min(amount);
			// Note that once the stake reaches zero by slashing (implying the share is non-zero),
			// the pool goes banckrupt. In such case, the pool becomes frozen.
			// (TODO: maybe can be recovered by removing all the miners from the pool? How to take
			// care of PoolUsers?)
			let (new_stake, _) = extract_dust(self.total_stake - amount);
			self.total_stake = new_stake;
		}

		/// Asserts there's no dirty slash (in debug profile only)
		fn assert_slash_clean(&self, user: &UserStakeInfo<AccountId, Balance>) {
			debug_assert!(
				self.total_shares == Zero::zero()
				// Due to the unstable fixed point share price, we cannot compare them directly
					|| balances_nearly_equal(bmul(user.shares, &self.share_price().unwrap()), user.locked),
				"There shouldn't be any dirty slash (user shares = {}, price = {:?}, user locked = {}, delta = {})",
				user.shares, self.share_price(), user.locked,
				bmul(user.shares, &self.share_price().unwrap()) - user.locked
			);
		}

		/// Asserts there's no pending reward (in debug profile only)
		fn assert_reward_clean(&self, user: &UserStakeInfo<AccountId, Balance>) {
			debug_assert!(
				self.pending_reward(user) == Zero::zero(),
				"The pending reward should be zero (user share = {}, user debt = {}, accumulator = {:?}, delta = {}))",
				user.shares, user.reward_debt, self.reward_acc,
				self.pending_reward(user)
			);
		}

		/// Settles the pending slash for a pool user.
		///
		/// Returns the slashed amount if succeeded, otherwise None.
		fn settle_slash(&self, user: &mut UserStakeInfo<AccountId, Balance>) -> Option<Balance> {
			let price = self.share_price()?;
			let locked = user.locked;
			let new_locked = bmul(user.shares, &price);
			// Double check the new_locked won't exceed the original locked
			let new_locked = new_locked.min(locked);
			// When only dust remaining in the pool, we include the dust in the slash amount
			let (new_locked, _) = extract_dust(new_locked);
			user.locked = new_locked;
			// The actual slashed amount. Usually slash will only cause the share price decreasing.
			// However in some edge case (i.e. the pool got slashed to 0 and then new contribution
			// added), the locked amount may even become larger
			Some(locked - new_locked)
		}

		/// Returns the price of one share, or None if no share at all.
		fn share_price(&self) -> Option<FixedPoint> {
			self.total_stake
				.to_fixed()
				.checked_div(self.total_shares.to_fixed())
		}

		/// Settles all the pending rewards of a user and move to `available_rewards` for claiming
		fn settle_user_pending_reward(&self, user: &mut UserStakeInfo<AccountId, Balance>) {
			let pending_reward = self.pending_reward(user);
			user.available_rewards.saturating_accrue(pending_reward);
			self.reset_pending_reward(user);
		}

		// Distributes additinoal rewards to the current share holders.
		//
		// Additional rewards contribute to the face value of the pool shares. The vaue of each
		// share effectively grows by (rewards / total_shares).
		//
		// Warning: `total_reward` mustn't be zero.
		fn distribute_reward(&mut self, rewards: Balance) {
			assert!(
				is_nondust_balance(self.total_shares),
				"Divide by zero at distribute_reward"
			);
			Accumulator::<Balance>::distribute(
				self.total_shares,
				self.reward_acc.get_mut(),
				rewards,
			);
		}

		/// Calculates the pending reward a user is holding
		fn pending_reward(&self, user: &UserStakeInfo<AccountId, Balance>) -> Balance {
			Accumulator::<Balance>::pending(user.shares, &self.reward_acc.into(), user.reward_debt)
		}

		/// Resets user's `reward_debt` to remove all the pending rewards
		fn reset_pending_reward(&self, user: &mut UserStakeInfo<AccountId, Balance>) {
			Accumulator::<Balance>::clear_pending(
				user.shares,
				&self.reward_acc.into(),
				&mut user.reward_debt,
			);
		}

		/// Removes a worker from the pool's worker list
		fn remove_worker(&mut self, worker: &WorkerPublicKey) {
			self.workers.retain(|w| w != worker);
		}

		/// Returns if the pool has expired withdrawal requests
		fn has_expired_withdrawal(&self, now: u64, grace_period: u64) -> bool {
			debug_assert!(
				self.free_stake == Zero::zero(),
				"We really don't want to have free stake and withdraw requests at the same time"
			);
			// If we check the pool withdraw_queue here, we don't have to remove a pool from
			// WithdrawalQueuedPools when a pool has handled their waiting withdraw requests before
			// timeout. Compare the IO performance we think removing pool from
			// WithdrawalQueuedPools would have more resource cost.

			// If the pool is bankrupt, or there's no share, we just skip this pool.
			let price = match self.share_price() {
				Some(price) if price != fp!(0) => price,
				_ => return false,
			};
			let mut budget = self.free_stake + self.releasing_stake;
			for request in &self.withdraw_queue {
				let amount = bmul(request.shares, &price);
				if amount > budget {
					// Run out of budget, let's check if the request is still in the grace period
					return now - request.start_time > grace_period;
				} else {
					// Otherwise we allocate some budget to virtually fulfill the request
					budget -= amount;
				}
			}
			false
		}
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct UserStakeInfo<AccountId: Default, Balance> {
		/// User account
		pub user: AccountId,
		/// The actual locked stake
		pub locked: Balance,
		/// The share in the pool. Cannot be dust.
		///
		/// Invariant must hold:
		///   StakePools[pid].total_stake == sum(PoolStakers[(pid, user)].shares)
		pub shares: Balance,
		/// Claimable rewards
		pub available_rewards: Balance,
		/// The debt of a user's stake subject to the pool reward accumulator
		pub reward_debt: Balance,
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct WithdrawInfo<AccountId: Default, Balance> {
		/// The withdrawal requester
		pub user: AccountId,
		/// The shares to withdraw. Cannot be dust.
		pub shares: Balance,
		/// The start time of the request
		pub start_time: u64,
	}

	#[cfg(test)]
	mod test {
		use assert_matches::assert_matches;
		use fixed_macro::types::U64F64 as fp;
		use frame_support::{assert_noop, assert_ok};
		use hex_literal::hex;
		use sp_runtime::AccountId32;

		use super::*;
		use crate::mock::{
			ecdh_pubkey, elapse_cool_down, elapse_seconds, new_test_ext, set_block_1,
			setup_workers, setup_workers_linked_operators, take_events, teleport_to_block,
			worker_pubkey, Balance, BlockNumber, Event as TestEvent, Origin, Test, DOLLARS,
		};
		// Pallets
		use crate::mock::{
			Balances, PhalaMining, PhalaRegistry, PhalaStakePool, System, Timestamp,
		};

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
						reward_acc: CodecFixedPoint::zero(),
						total_shares: 0,
						total_stake: 0,
						free_stake: 0,
						releasing_stake: 0,
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
				// Check worker assignments cleared, and the worker removed from the pool
				assert!(!WorkerAssignments::<Test>::contains_key(&worker_pubkey(1)));
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
				// Check worker assignments cleared, and the worker removed from the pool
				assert!(!WorkerAssignments::<Test>::contains_key(&worker_pubkey(2)));
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
					PhalaStakePool::contribute(Origin::signed(2), 0, 900 * DOLLARS),
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
					PhalaStakePool::contribute(
						Origin::signed(1),
						0,
						Balances::usable_balance(1) + 1
					),
					Error::<Test>::InsufficientBalance,
				);
			});
		}

		#[test]
		fn test_slash() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid = 0

				// Account1 contributes 100 PHA, account2 contributes 400 PHA
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
				// Start a miner
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					500 * DOLLARS
				));
				let sub_account1: u64 = pool_sub_account(0, &worker_pubkey(1));
				let miner = PhalaMining::miners(sub_account1).unwrap();
				let ve = FixedPoint::from_bits(miner.ve);
				assert_eq!(ve, fp!(650.3900000000000000054));
				// Simulate a slash of 50%
				let _ = take_events();
				simulate_v_update(1, (ve / 2).to_bits());
				// Stop & settle
				assert_ok!(PhalaStakePool::stop_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				elapse_cool_down();
				assert_ok!(PhalaStakePool::reclaim_pool_worker(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				let ev = take_events();
				assert_matches!(
					ev.as_slice(),
					[
						TestEvent::PhalaMining(mining::Event::MinerSettled(_, v, 0)),
						TestEvent::PhalaMining(mining::Event::MinerStopped(_)),
						TestEvent::PhalaMining(mining::Event::MinerReclaimed(_, _, _)),
						TestEvent::PhalaStakePool(Event::PoolSlashed(0, slashed)),
					]
					if FixedPoint::from_bits(*v) == ve / 2
						&& *slashed == 250000000000000
				);
				// Settle the pending slash
				let _ = take_events();
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				assert_eq!(pool.total_stake, 250000000000000);
				let mut staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let mut staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				PhalaStakePool::maybe_settle_slash(&pool, &mut staker1);
				PhalaStakePool::maybe_settle_slash(&pool, &mut staker2);
				StakePools::<Test>::insert(0, pool);
				PoolStakers::<Test>::insert((0, 1), staker1);
				PoolStakers::<Test>::insert((0, 2), staker2);
				let ev = take_events();
				assert_eq!(
					ev,
					vec![
						TestEvent::Balances(pallet_balances::Event::Slashed {
							who: 1,
							amount: 50000000000000
						}),
						TestEvent::PhalaStakePool(Event::SlashSettled(0, 1, 50000000000000)),
						TestEvent::Balances(pallet_balances::Event::Slashed {
							who: 2,
							amount: 200000000000000
						}),
						TestEvent::PhalaStakePool(Event::SlashSettled(0, 2, 200000000000000))
					]
				);
				// Check slash settled. Remaining: 50 PHA, 200 PHA
				assert_eq!(PhalaStakePool::stake_ledger(1), Some(50000000000000));
				assert_eq!(PhalaStakePool::stake_ledger(2), Some(200000000000000));
				assert_eq!(Balances::locks(1), vec![the_lock(50000000000000)]);
				assert_eq!(Balances::locks(2), vec![the_lock(200000000000000)]);
				assert_eq!(Balances::free_balance(1), 950000000000000);
				assert_eq!(Balances::free_balance(2), 1800000000000000);
				// Account3 contributes 250 PHA. Now: 50 : 200 : 250
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(3),
					0,
					250 * DOLLARS + 1 // Round up to 500 PHA again
				));
				// Slash 50% again
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					500 * DOLLARS
				));
				let miner = PhalaMining::miners(sub_account1).unwrap();
				let ve = FixedPoint::from_bits(miner.ve);
				let _ = take_events();
				simulate_v_update(1, (ve / 2).to_bits());
				// Full stop & settle
				assert_ok!(PhalaStakePool::stop_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				elapse_cool_down();
				assert_ok!(PhalaStakePool::reclaim_pool_worker(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				let ev = take_events();
				assert_matches!(
					ev.as_slice(),
					[
						TestEvent::PhalaMining(mining::Event::MinerSettled(_, _, 0)),
						TestEvent::PhalaMining(mining::Event::MinerStopped(_)),
						TestEvent::PhalaMining(mining::Event::MinerReclaimed(
							_,
							500000000000000,
							250000000000000
						)),
						TestEvent::PhalaStakePool(Event::PoolSlashed(0, 250000000000000)),
					]
				);
				// Withdraw & check amount
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				let staker3 = PhalaStakePool::pool_stakers((0, 3)).unwrap();
				let _ = take_events();
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(1),
					0,
					staker1.shares
				));
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					staker2.shares
				));
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(3),
					0,
					staker3.shares
				));
				let ev = take_events();
				assert_eq!(
					ev,
					vec![
						// Account1: ~25 PHA remaining
						TestEvent::Balances(pallet_balances::Event::Slashed {
							who: 1,
							amount: 25000000000000
						}),
						TestEvent::PhalaStakePool(Event::SlashSettled(0, 1, 25000000000000)),
						TestEvent::PhalaStakePool(Event::Withdrawal(0, 1, 25000000000000)),
						// Account2: ~100 PHA remaining
						TestEvent::Balances(pallet_balances::Event::Slashed {
							who: 2,
							amount: 100000000000000
						}),
						TestEvent::PhalaStakePool(Event::SlashSettled(0, 2, 100000000000000)),
						TestEvent::PhalaStakePool(Event::Withdrawal(0, 2, 100000000000000)),
						// Account1: ~125 PHA remaining
						TestEvent::Balances(pallet_balances::Event::Slashed {
							who: 3,
							amount: 125000000000001
						}),
						TestEvent::PhalaStakePool(Event::SlashSettled(0, 3, 125000000000001)),
						TestEvent::PhalaStakePool(Event::Withdrawal(0, 3, 125000000000000))
					]
				);
			});
		}

		#[test]
		fn test_no_contribution_to_bankrupt_pool() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid = 0
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					100 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					100 * DOLLARS
				));
				// Slash 100% and stop
				simulate_v_update(1, fp!(0).to_bits());
				assert_ok!(PhalaStakePool::stop_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				elapse_cool_down();
				assert_ok!(PhalaStakePool::reclaim_pool_worker(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				// Check cannot contribute
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 0, 10 * DOLLARS),
					Error::<Test>::PoolBankrupt,
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
				assert_eq!(pool.reward_acc, CodecFixedPoint::zero());
				assert_eq!(pool.total_stake, 500 * DOLLARS);

				// Mined 500 PHA
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(500u32).to_bits(),
					treasury: 0,
				}]);
				// Should result in 100, 400 PHA pending reward for staker1 & 2
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.reward_acc.get(), fp!(1));
				assert_eq!(pool.pending_reward(&staker1), 100 * DOLLARS);
				assert_eq!(pool.pending_reward(&staker2), 400 * DOLLARS);

				// Staker1 claims 100 PHA rewrad, left 100 debt & no pending reward
				let _ = take_events();
				assert_ok!(PhalaStakePool::claim_rewards(Origin::signed(1), 0, 1));
				assert_eq!(
					take_events().as_slice(),
					[
						TestEvent::Balances(pallet_balances::Event::<Test>::Transfer {
							from: PhalaMining::account_id(),
							to: 1,
							amount: 100 * DOLLARS
						}),
						TestEvent::PhalaStakePool(Event::RewardsWithdrawn(0, 1, 100 * DOLLARS))
					]
				);
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				assert_eq!(pool.reward_acc.get(), fp!(1), "reward_acc shouldn't change");
				assert_eq!(staker1.reward_debt, 100 * DOLLARS);
				assert_eq!(pool.pending_reward(&staker1), 0);

				// Mined 500 PHA
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(500u32).to_bits(),
					treasury: 0,
				}]);
				// Should result in 100, 800 PHA pending reward for staker1 & 2
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.reward_acc.get(), fp!(2));
				assert_eq!(pool.pending_reward(&staker1), 100 * DOLLARS);
				assert_eq!(pool.pending_reward(&staker2), 800 * DOLLARS);

				// Staker2 claims 800 PHA rewrad, left 800 debt
				let _ = take_events();
				assert_ok!(PhalaStakePool::claim_rewards(Origin::signed(2), 0, 2));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(staker2.reward_debt, 800 * DOLLARS);

				// Staker1 contribute another 300 PHA (now 50:50), causing a passive reward settlement
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					300 * DOLLARS
				));
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				assert_eq!(staker1.shares, 400 * DOLLARS);
				assert_eq!(staker1.reward_debt, 800 * DOLLARS);

				// Mined 800 PHA
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(800u32).to_bits(),
					treasury: 0,
				}]);
				assert_ok!(PhalaStakePool::claim_rewards(Origin::signed(1), 0, 1));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.reward_acc.get(), fp!(3));
				assert_eq!(pool.pending_reward(&staker1), 0);
				assert_eq!(pool.pending_reward(&staker2), 400 * DOLLARS);

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
				assert_eq!(staker1.shares, 0);
				assert_eq!(staker1.reward_debt, 0);
				assert_eq!(staker2.shares, 400 * DOLLARS);
			});
		}

		#[test]
		fn dismiss_dust_reward() {
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
				let _ = take_events();
				// Inject 100 pico PHA payout to trigger dust removal (99 after convering to fp)
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: 100u128.to_fixed().to_bits(),
					treasury: 0,
				}]);
				let ev = take_events();
				assert_eq!(
					ev,
					vec![TestEvent::PhalaStakePool(Event::RewardDismissedDust(0, 99))]
				);
			});
		}

		#[test]
		fn test_late_reward_report() {
			use crate::mining::pallet::OnReward;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid = 0

				// Simulate no share in the pool.
				let _ = take_events();
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(500u32).to_bits(),
					treasury: 0,
				}]);
				let ev = take_events();
				assert_eq!(
					ev,
					vec![TestEvent::PhalaStakePool(Event::RewardDismissedNoShare(
						0,
						500 * DOLLARS
					))]
				);
				// Simulate the worker is already unbound
				assert_ok!(PhalaStakePool::remove_worker(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				let _ = take_events();
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(500u32).to_bits(),
					treasury: 0,
				}]);
				let ev = take_events();
				assert_eq!(
					ev,
					vec![TestEvent::PhalaStakePool(Event::RewardDismissedNotInPool(
						worker_pubkey(1),
						500 * DOLLARS
					))]
				);
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
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(500u32).to_bits(),
					treasury: 0,
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
			use crate::mining::pallet::OnStopped;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_pool_with_workers(1, &[1, 2]); // pid = 0

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
				assert_eq!(staker2.shares, 1000 * DOLLARS);
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
				assert_eq!(staker2.shares, 501 * DOLLARS);
				assert_eq!(Balances::locks(2), vec![the_lock(501 * DOLLARS)]);
				// Withdraw 2 PHA will only fulfill 1 PHA from the free stake, leaving 1 PHA in the
				// withdraw queue
				assert_ok!(PhalaStakePool::withdraw(Origin::signed(2), 0, 2 * DOLLARS));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				assert_eq!(pool.free_stake, 0);
				assert_eq!(pool.total_stake, 500 * DOLLARS);
				assert_eq!(staker2.shares, 500 * DOLLARS);
				assert_eq!(Balances::locks(2), vec![the_lock(500 * DOLLARS)]);
				// Check the queue
				assert_eq!(
					pool.withdraw_queue,
					vec![WithdrawInfo {
						user: 2,
						shares: 1 * DOLLARS,
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
				assert_eq!(staker1.shares, 1 * DOLLARS);
				assert_eq!(staker2.shares, 499 * DOLLARS);
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
							shares: 199 * DOLLARS,
							start_time: 0
						},
						WithdrawInfo {
							user: 1,
							shares: 1 * DOLLARS,
							start_time: 0
						}
					]
				);
				assert_eq!(staker1.shares, 1 * DOLLARS);
				assert_eq!(staker2.shares, 499 * DOLLARS);
				// Trigger a force clear by `on_reclaim()`, releasing 100 PHA stake to partially
				// fulfill staker2's withdraw request, but leaving staker1's untouched.
				let _ = take_events();
				PhalaStakePool::on_stopped(&worker_pubkey(2), 100 * DOLLARS, 0);
				PhalaStakePool::handle_reclaim(0, 100 * DOLLARS, 0);
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
				assert_eq!(staker1.shares, 1 * DOLLARS);
				assert_eq!(staker2.shares, 399 * DOLLARS);
				// Trigger another force clear with 100 PHA slashed, releasing all 400 PHA stake
				// (100 slashed & 300 free), fulfilling stakers' requests.
				let _ = take_events();
				PhalaStakePool::on_stopped(&worker_pubkey(1), 400 * DOLLARS, 100 * DOLLARS);
				PhalaStakePool::handle_reclaim(0, 400 * DOLLARS, 100 * DOLLARS);
				assert_eq!(
					take_events().as_slice(),
					[
						TestEvent::PhalaStakePool(Event::PoolSlashed(0, 100 * DOLLARS)),
						// Staker 2 got 75% * 99 PHA back
						TestEvent::Balances(pallet_balances::Event::Slashed {
							who: 2,
							amount: 99_750000000000
						}),
						TestEvent::PhalaStakePool(Event::SlashSettled(0, 2, 99_750000000000)),
						TestEvent::PhalaStakePool(Event::Withdrawal(0, 2, 74_250000000000)),
						// Staker 1 got 75% * 1 PHA back
						TestEvent::Balances(pallet_balances::Event::Slashed {
							who: 1,
							amount: 250000000000
						}),
						TestEvent::PhalaStakePool(Event::SlashSettled(0, 1, 250000000000)),
						TestEvent::PhalaStakePool(Event::Withdrawal(0, 1, 750000000000)),
					]
				);
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				let staker1 = PhalaStakePool::pool_stakers((0, 1)).unwrap();
				let staker2 = PhalaStakePool::pool_stakers((0, 2)).unwrap();
				// After fulfill all the withdraw requests (100 shares, 75 PHA), there are 100 PHA
				// slashed and 75 PHA withdrawn, leaving 225 PHA in the pool, all belong to
				// staker2.
				assert_eq!(pool.total_stake, 225 * DOLLARS);
				assert_eq!(pool.free_stake, 225 * DOLLARS);
				assert_eq!(staker1.shares, 0);
				assert_eq!(staker2.shares, 300 * DOLLARS);
				assert_eq!(Balances::locks(1), vec![]);
				assert_eq!(Balances::locks(2), vec![the_lock(225 * DOLLARS)]);
			});
		}

		#[test]
		fn test_pool_has_expired_withdraw() {
			// Default pool setup
			let mut pool: PoolInfo<u64, Balance> = Default::default();
			pool.total_shares = 1000 * DOLLARS;
			pool.total_stake = 900 * DOLLARS; // 90% stake returned
			pool.withdraw_queue.push_back(WithdrawInfo {
				user: 1,
				shares: 100 * DOLLARS,
				start_time: 0,
			});
			pool.withdraw_queue.push_back(WithdrawInfo {
				user: 2,
				shares: 200 * DOLLARS,
				start_time: 100,
			});
			pool.withdraw_queue.push_back(WithdrawInfo {
				user: 3,
				shares: 400 * DOLLARS,
				start_time: 200,
			});
			// No releasing stake
			let pool1 = PoolInfo::<u64, Balance> {
				releasing_stake: 0,
				..pool.clone()
			};
			assert!(!pool1.has_expired_withdrawal(0, 100), "All in grace period");
			assert!(
				!pool1.has_expired_withdrawal(100, 100),
				"Still all in grace period"
			);
			assert!(
				pool1.has_expired_withdrawal(101, 100),
				"First withdraw request expired"
			);
			// Releaing stake to cover the first request
			let pool2 = PoolInfo::<u64, Balance> {
				releasing_stake: 90 * DOLLARS,
				..pool.clone()
			};
			assert!(
				!pool2.has_expired_withdrawal(101, 100),
				"First withdraw request fulfilled"
			);
			assert!(
				pool2.has_expired_withdrawal(201, 100),
				"Second withdraw request expired"
			);
			let pool3 = PoolInfo::<u64, Balance> {
				releasing_stake: 630 * DOLLARS - 10,
				..pool.clone()
			};
			assert!(
				pool3.has_expired_withdrawal(1000, 100),
				"No enought releasing stake to fulfill all"
			);
			let pool4 = PoolInfo::<u64, Balance> {
				releasing_stake: 630 * DOLLARS,
				..pool.clone()
			};
			assert!(!pool4.has_expired_withdrawal(1000, 100), "Enough stake");
		}

		#[test]
		fn test_force_withdraw() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid = 0
				let sub_account1: u64 = pool_sub_account(0, &worker_pubkey(1));

				// Stake 1000 PHA, and start two miners with 400 & 100 PHA as stake
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					900 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(3),
					0,
					100 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					900 * DOLLARS
				));
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					900 * DOLLARS
				));
				// Now: 100 already withdrawl, 800 in queue
				// Then we make the withdraw request expired.
				let grace_period = <Test as Config>::GracePeriod::get();
				elapse_seconds(grace_period + 1);
				teleport_to_block(2);
				// Check stake releasing
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				assert_eq!(pool.releasing_stake, 900 * DOLLARS);
				assert_eq!(pool.total_stake, 900 * DOLLARS);
				let user2 = PhalaStakePool::pool_stakers(&(0, 2)).unwrap();
				assert_eq!(user2.locked, 800 * DOLLARS);
				assert_eq!(user2.shares, 800 * DOLLARS);
				// Check worker is shutting down
				let miner = PhalaMining::miners(sub_account1).unwrap();
				assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
				// Reclaim, triggering the return of the stake
				elapse_cool_down();
				assert_ok!(PhalaStakePool::reclaim_pool_worker(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				// Check worker is is reclaimed
				let miner = PhalaMining::miners(sub_account1).unwrap();
				assert_eq!(miner.state, mining::MinerState::Ready);
				// Check settled
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				assert_eq!(pool.releasing_stake, 0);
				assert_eq!(pool.total_stake, 100 * DOLLARS);
				let user2 = PhalaStakePool::pool_stakers(&(0, 2)).unwrap();
				assert_eq!(user2.locked, 0);
				assert_eq!(user2.shares, 0);
				assert_eq!(Balances::locks(2), vec![]);
			});
		}

		#[test]
		fn double_withdraw_cancel_the_first() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_pool_with_workers(1, &[1, 2]); // pid = 0

				// Stake 1000 PHA, and start a miner with 500 PHA as stake
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					500 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					500 * DOLLARS
				));
				// Request to withdraw 100 PHA
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					100 * DOLLARS
				));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				assert_eq!(pool.withdraw_queue.len(), 1);
				assert_eq!(pool.withdraw_queue.get(0).unwrap().shares, 100 * DOLLARS);
				// Request to withdraw 200 PHA again
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					200 * DOLLARS
				));
				let pool = PhalaStakePool::stake_pools(0).unwrap();
				assert_eq!(pool.withdraw_queue.len(), 1);
				assert_eq!(pool.withdraw_queue.get(0).unwrap().shares, 200 * DOLLARS);
			});
		}

		#[test]
		fn issue_388_double_stake() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]);

				let balance = Balances::usable_balance(&1);
				assert_ok!(PhalaStakePool::contribute(Origin::signed(1), 0, balance));
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 0, balance),
					Error::<Test>::InsufficientBalance
				);
			});
		}

		#[test]
		fn test_pool_owner_reward() {
			use crate::mining::pallet::OnReward;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]);

				assert_ok!(PhalaStakePool::set_payout_pref(
					Origin::signed(1),
					0,
					Permill::from_percent(50)
				));
				// Staker2 contribute 1000 PHA and start mining
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					1000 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					1000 * DOLLARS
				));
				// Mined 100 PHA
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(100u32).to_bits(),
					treasury: 0,
				}]);
				// Both owner and staker2 can claim 50 PHA
				let _ = take_events();
				assert_ok!(PhalaStakePool::claim_rewards(Origin::signed(1), 0, 1));
				assert_ok!(PhalaStakePool::claim_rewards(Origin::signed(2), 0, 2));
				let ev = take_events();
				assert_matches!(
					ev.as_slice(),
					[
						TestEvent::Balances(pallet_balances::Event::Transfer {
							from: _,
							to: 1,
							amount: 50000000000000
						}),
						TestEvent::PhalaStakePool(Event::RewardsWithdrawn(0, 1, 50000000000000)),
						TestEvent::Balances(pallet_balances::Event::Transfer {
							from: _,
							to: 2,
							amount: 49999999999999
						}),
						TestEvent::PhalaStakePool(Event::RewardsWithdrawn(0, 2, 49999999999999))
					]
				);
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
					PoolStakers::<Test>::get(&(0, 1)).unwrap().shares,
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
					PoolStakers::<Test>::get(&(0, 2)).unwrap().shares,
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

				let sub_account1: u64 = pool_sub_account(0, &worker1);
				let sub_account2: u64 = pool_sub_account(0, &worker2);

				// Slash pool 0 to 90%
				let miner0 = PhalaMining::miners(sub_account1).unwrap();
				let ve = FixedPoint::from_bits(miner0.ve);
				simulate_v_update(1, (ve * fp!(0.9)).to_bits());

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
				let miner1 = PhalaMining::miners(&sub_account1).unwrap();
				let miner2 = PhalaMining::miners(&sub_account2).unwrap();
				assert_eq!(miner1.state, mining::MinerState::MiningCoolingDown);
				assert_eq!(miner2.state, mining::MinerState::MiningCoolingDown);
				// Wait the cool down period
				elapse_cool_down();
				assert_ok!(PhalaStakePool::reclaim_pool_worker(
					Origin::signed(1),
					0,
					worker1
				));
				assert_ok!(PhalaStakePool::reclaim_pool_worker(
					Origin::signed(1),
					0,
					worker2
				));
				// 90% stake get returend from pool 0
				let pool0 = PhalaStakePool::stake_pools(0).unwrap();
				assert_eq!(pool0.free_stake, 189_999999999999);
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
				// Remove worker from the pools
				assert_ok!(PhalaStakePool::remove_worker(
					Origin::signed(1),
					0,
					worker1.clone()
				));
				assert_ok!(PhalaStakePool::remove_worker(
					Origin::signed(1),
					0,
					worker2.clone()
				));
			});
		}

		#[test]
		fn issue487_eps_should_not_cause_dead_loop() {
			new_test_ext().execute_with(|| {
				set_block_1();
				StakePools::<Test>::insert(
					0,
					PoolInfo {
						pid: 0,
						owner: 1,
						payout_commission: None,
						owner_reward: 0,
						cap: None,
						reward_acc: CodecFixedPoint::zero(),
						/// Total shares
						total_shares: 47_9299_9999_9999,
						total_stake: 47_9300_0000_0000,
						free_stake: 1,
						releasing_stake: 0,
						workers: vec![],
						/// The queue of withdraw requests
						withdraw_queue: {
							let mut q = VecDeque::<WithdrawInfo<u64, u128>>::new();
							q.push_back(WithdrawInfo {
								user: 2,
								shares: 298_9080_0000_0000,
								start_time: 100,
							});
							q
						},
					},
				);
				PoolStakers::<Test>::insert(
					(0, 2),
					UserStakeInfo::<u64, u128> {
						user: 2,
						locked: 299_9000_0000_0000,
						shares: 299_9000_0000_0000,
						available_rewards: 0,
						reward_debt: 0,
					},
				);
				PhalaStakePool::ledger_accrue(&2, 299_9000_0000_0000);
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(3),
					0,
					1 * DOLLARS
				));
			});
		}

		#[test]
		fn issue490_limit_one_withdraw_per_user() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid=0
								  // Start a worker as usual
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					1500 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					1500 * DOLLARS
				));

				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					100 * DOLLARS
				));
				let queue = PhalaStakePool::stake_pools(0).unwrap().withdraw_queue;
				assert_eq!(queue[0].shares, 100 * DOLLARS);
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					200 * DOLLARS
				));
				let queue = PhalaStakePool::stake_pools(0).unwrap().withdraw_queue;
				assert_eq!(queue[0].shares, 200 * DOLLARS);
			});
		}

		#[test]
		fn issue500_should_not_restart_worker_in_cool_down() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid=0
								  // Start a worker as usual
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					1500 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					1500 * DOLLARS
				));
				assert_ok!(PhalaStakePool::stop_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				let subaccount: u64 = pool_sub_account(0, &worker_pubkey(1));
				let miner = PhalaMining::miners(subaccount).unwrap();
				assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
				// Remove the worker
				assert_ok!(PhalaStakePool::remove_worker(
					Origin::signed(1),
					0,
					worker_pubkey(1)
				));
				let miner = PhalaMining::miners(subaccount).unwrap();
				assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
				// Now the stake is still in CD state. We cannot add it back.
				assert_noop!(
					PhalaStakePool::add_worker(Origin::signed(1), 0, worker_pubkey(1)),
					Error::<Test>::FailedToBindMinerAndWorker,
				);
				let miner = PhalaMining::miners(subaccount).unwrap();
				assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
			});
		}

		#[test]
		fn issue257_reconcile_reduced_withdraw() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid=0
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					500 * DOLLARS
				));
				let orig_pool = StakePools::<Test>::get(0);
				StakePools::<Test>::mutate(0, |pool_info| {
					pool_info
						.as_mut()
						.unwrap()
						.withdraw_queue
						.push_back(WithdrawInfo {
							user: 2,
							shares: 1000 * DOLLARS,
							start_time: 1u64,
						});
				});
				// Should reduce the requested shares to 500
				assert_ok!(PhalaStakePool::reconcile_withdraw_queue(
					Origin::signed(1),
					0,
					2
				));
				let req = StakePools::<Test>::get(0)
					.unwrap()
					.withdraw_queue
					.get(0)
					.cloned()
					.unwrap();
				assert_eq!(req.shares, 500 * DOLLARS);
			});
		}

		#[test]
		fn issue257_reconcile_removed_withdraw() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_pool_with_workers(1, &[1]); // pid=0
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					500 * DOLLARS
				));
				// Remove all the stake, leaving zero share in PoolStakers
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					500 * DOLLARS
				));
				let orig_pool = StakePools::<Test>::get(0);
				StakePools::<Test>::mutate(0, |pool_info| {
					pool_info
						.as_mut()
						.unwrap()
						.withdraw_queue
						.push_back(WithdrawInfo {
							user: 2,
							shares: 1000 * DOLLARS,
							start_time: 1u64,
						});
				});
				// Should reduce the requested shares to 500
				assert_ok!(PhalaStakePool::reconcile_withdraw_queue(
					Origin::signed(1),
					0,
					2
				));
				assert!(StakePools::<Test>::get(0)
					.unwrap()
					.withdraw_queue
					.is_empty());
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

		fn simulate_v_update(worker: u8, v_bits: u128) {
			use phala_types::messaging::{
				DecodedMessage, MessageOrigin, MiningInfoUpdateEvent, SettleInfo, Topic,
			};
			let block = System::block_number();
			let now = Timestamp::now();
			assert_ok!(PhalaMining::on_gk_message_received(DecodedMessage::<
				MiningInfoUpdateEvent<BlockNumber>,
			> {
				sender: MessageOrigin::Gatekeeper,
				destination: Topic::new(*b"^phala/mining/update"),
				payload: MiningInfoUpdateEvent::<BlockNumber> {
					block_number: block,
					timestamp_ms: now,
					offline: vec![],
					recovered_to_online: vec![],
					settle: vec![SettleInfo {
						pubkey: worker_pubkey(worker),
						v: v_bits,
						payout: 0,
						treasury: 0,
					}],
				},
			}));
		}
	}
}

mod migrations;

use sp_runtime::traits::AtLeast32BitUnsigned;

/// Returns true if `n` is close to zero (1000 pico, or 1e-8).
fn balance_close_to_zero<B: AtLeast32BitUnsigned + Copy>(n: B) -> bool {
	n <= B::from(1000u32)
}

/// Returns true if `a` and `b` are close enough (1000 pico, or 1e-8)
fn balances_nearly_equal<B: AtLeast32BitUnsigned + Copy>(a: B, b: B) -> bool {
	if a > b {
		balance_close_to_zero(a - b)
	} else {
		balance_close_to_zero(b - a)
	}
}

/// Returns true if `n` is a non-trivial positive balance
fn is_nondust_balance<B: AtLeast32BitUnsigned + Copy>(n: B) -> bool {
	!balance_close_to_zero(n)
}

/// Normalizes `n` to zero if it's a dust balance.
///
/// Returns type  `n` itself if it's a non-trivial positive balance, otherwise zero.
fn extract_dust<B: AtLeast32BitUnsigned + Copy>(n: B) -> (B, B) {
	if balance_close_to_zero(n) {
		(Zero::zero(), n)
	} else {
		(n, Zero::zero())
	}
}
