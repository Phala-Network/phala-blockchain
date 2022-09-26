//! Pool for collaboratively mining staking

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	#[cfg(not(feature = "std"))]
	use alloc::format;
	#[cfg(feature = "std")]
	use std::format;

	use crate::balance_convert::FixedPointConvert;
	use crate::basepool;
	use crate::mining;
	use crate::pawnshop;
	use crate::poolproxy::{ensure_stake_pool, ensure_vault, PoolProxy, StakePool};
	use crate::registry;
	use crate::vault;

	use fixed::types::U64F64 as FixedPoint;

	use crate::BalanceOf;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{tokens::fungibles::Transfer, StorageVersion, UnixTime},
	};
	use frame_system::{pallet_prelude::*, Origin};

	use sp_runtime::{
		traits::{TrailingZeroInput, Zero},
		Permill, SaturatedConversion,
	};
	use sp_std::{collections::vec_deque::VecDeque, fmt::Display, prelude::*, vec};

	use phala_types::{messaging::SettleInfo, WorkerPublicKey};

	pub use rmrk_traits::primitives::{CollectionId, NftId};

	const MAX_WHITELIST_LEN: u32 = 100;

	type DescMaxLen = ConstU32<4400>;

	pub type DescStr = BoundedVec<u8, DescMaxLen>;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ crate::PhalaConfig
		+ registry::Config
		+ mining::Config
		+ pallet_rmrk_core::Config
		+ basepool::Config
		+ pallet_assets::Config
		+ pallet_democracy::Config
		+ pawnshop::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

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

		/// The origin that can turn on or off mining
		type MiningSwitchOrigin: EnsureOrigin<Self::Origin>;

		/// The origin that can trigger backfill tasks.
		type BackfillOrigin: EnsureOrigin<Self::Origin>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(5);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Mapping from workers to the pool they belong to
	///
	/// The map entry lasts from `add_worker()` to `remove_worker()` or force unbinding.
	#[pallet::storage]
	pub type WorkerAssignments<T: Config> = StorageMap<_, Twox64Concat, WorkerPublicKey, u64>;

	/// (Deprecated)
	// TODO: remove it
	#[pallet::storage]
	pub type SubAccountAssignments<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u64>;

	/// Switch to enable the stake pool pallet (disabled by default)
	#[pallet::storage]
	#[pallet::getter(fn mining_enabled)]
	pub type MiningEnabled<T> = StorageValue<_, bool, ValueQuery, MiningEnabledByDefault<T>>;

	#[pallet::type_value]
	pub fn MiningEnabledByDefault<T: Config>() -> bool {
		T::MiningEnabledByDefault::get()
	}

	/// Helper storage to track the preimage of the mining sub-accounts. Not used in consensus.
	#[pallet::storage]
	pub type SubAccountPreimages<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (u64, WorkerPublicKey)>;

	/// Mapping for pools that specify certain stakers to contribute stakes
	#[pallet::storage]
	#[pallet::getter(fn pool_whitelist)]
	pub type PoolContributionWhitelists<T: Config> =
		StorageMap<_, Twox64Concat, u64, Vec<T::AccountId>>;

	/// Mapping for pools that store their descriptions set by owner
	#[pallet::storage]
	#[pallet::getter(fn pool_descriptions)]
	pub type PoolDescriptions<T: Config> = StorageMap<_, Twox64Concat, u64, DescStr>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A stake pool is created by `owner`
		///
		/// Affected states:
		/// - a new entry in [`Pools`] with the pid
		PoolCreated { owner: T::AccountId, pid: u64 },

		/// The commission of a pool is updated
		///
		/// The commission ratio is represented by an integer. The real value is
		/// `commission / 1_000_000u32`.
		///
		/// Affected states:
		/// - the `payout_commission` field in [`Pools`] is updated
		PoolCommissionSet { pid: u64, commission: u32 },

		/// The stake capacity of the pool is updated
		///
		/// Affected states:
		/// - the `cap` field in [`Pools`] is updated
		PoolCapacitySet { pid: u64, cap: BalanceOf<T> },

		/// A worker is added to the pool
		///
		/// Affected states:
		/// - the `worker` is added to the vector `workers` in [`Pools`]
		/// - the worker in the [`WorkerAssignments`] is pointed to `pid`
		/// - the worker-miner binding is updated in `mining` pallet ([`WorkerBindings`](mining::pallet::WorkerBindings),
		///   [`MinerBindings`](mining::pallet::MinerBindings))
		PoolWorkerAdded {
			pid: u64,
			worker: WorkerPublicKey,
			miner: T::AccountId,
		},

		/// Someone contributed to a pool
		///
		/// Affected states:
		/// - the stake related fields in [`Pools`]
		/// - the user P-PHA balance reduced
		/// - the user recive ad share NFT once contribution succeeded
		/// - when there was any request in the withdraw queue, the action may trigger withdrawals
		///   ([`Withdrawal`](#variant.Withdrawal) event)
		Contribution {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},

		/// Some stake was withdrawn from a pool
		///
		/// Affected states:
		/// - the stake related fields in [`Pools`]
		/// - the user asset account
		Withdrawal {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},

		/// Owner rewards were withdrawn by pool owner
		/// Affected states:
		/// - the stake related fields in [`Pools`]
		/// - the owner asset account
		OwnerRewardsWithdrawn {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
		},

		/// The pool received a slash event from one of its workers (currently disabled)
		///
		/// The slash is accured to the pending slash accumulator.
		PoolSlashed { pid: u64, amount: BalanceOf<T> },

		/// Some slash is actually settled to a contributor (currently disabled)
		SlashSettled {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
		},

		/// Some reward is dismissed because the worker is no longer bound to a pool
		///
		/// There's no affected state.
		RewardDismissedNotInPool {
			worker: WorkerPublicKey,
			amount: BalanceOf<T>,
		},

		/// Some reward is dismissed because the pool doesn't have any share
		///
		/// There's no affected state.
		RewardDismissedNoShare { pid: u64, amount: BalanceOf<T> },

		/// Some reward is dismissed because the amount is too tiny (dust)
		///
		/// There's no affected state.
		RewardDismissedDust { pid: u64, amount: BalanceOf<T> },

		/// A worker is removed from a pool.
		///
		/// Affected states:
		/// - the worker item in [`WorkerAssignments`] is removed
		/// - the worker is removed from the [`Pools`] item
		PoolWorkerRemoved { pid: u64, worker: WorkerPublicKey },

		/// A withdrawal request is inserted to a queue
		///
		/// Affected states:
		/// - a new item is inserted to or an old item is being replaced by the new item in the
		///   withdraw queue in [`Pools`]
		WithdrawalQueued {
			pid: u64,
			user: T::AccountId,
			shares: BalanceOf<T>,
		},

		/// A pool contribution whitelist is added
		/// - lazy operated when the first staker is added to the whitelist
		PoolWhitelistCreated { pid: u64 },

		/// The pool contribution whitelist is deleted
		/// - lazy operated when the last staker is removed from the whitelist
		PoolWhitelistDeleted { pid: u64 },

		/// A staker is added to the pool contribution whitelist
		PoolWhitelistStakerAdded { pid: u64, staker: T::AccountId },

		/// A staker is removed from the pool contribution whitelist
		PoolWhitelistStakerRemoved { pid: u64, staker: T::AccountId },

		/// A worker is reclaimed from the pool
		WorkerReclaimed { pid: u64, worker: WorkerPublicKey },

		/// The amount of reward that distributed to owner and stakers
		RewardReceived {
			pid: u64,
			to_owner: BalanceOf<T>,
			to_stakers: BalanceOf<T>,
		},

		/// The amount of stakes for a worker to start mine
		MiningStarted {
			pid: u64,
			worker: WorkerPublicKey,
			amount: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The worker is not registered in the registry when adding to the pool
		WorkerNotRegistered,
		/// The worker doesn't have a valid benchmark when adding to the pool
		BenchmarkMissing,
		/// The worker is already added to the pool
		WorkerExists,
		/// The worker is already in cd_workers
		WorkerAlreadyStopped,
		/// The target worker is not in the pool
		WorkerDoesNotExist,
		/// The worker is already added to another pool
		WorkerInAnotherPool,
		/// The owner of the pool doesn't have the access to the worker
		///
		/// The access to a worker is granted by it's `operator` parameter set by `register_worker`
		UnauthorizedOperator,
		/// The caller is not the owner of the pool
		UnauthorizedPoolOwner,
		/// The stake capacity is set too low to cover the existing stake
		InadequateCapacity,
		/// The stake added to a pool exceeds its capacity
		StakeExceedsCapacity,
		/// The specified pool doesn't exist
		PoolDoesNotExist,
		_PoolIsBusy,
		/// The contributed stake is smaller than the minimum threshold
		InsufficientContribution,
		/// Trying to contribute more than the available balance
		InsufficientBalance,
		/// The user doesn't have stake in a pool
		PoolStakeNotFound,
		/// Cannot start mining because there's no enough free stake
		InsufficientFreeStake,
		/// The withdrawal amount is too small (considered as dust)
		InvalidWithdrawalAmount,
		/// Couldn't bind worker and the pool mining subaccount
		FailedToBindMinerAndWorker,
		/// Internal error: Cannot withdraw from the subsidy pool. This should never happen.
		InternalSubsidyPoolCannotWithdraw,
		/// The pool has already got all the stake completely slashed.
		///
		/// In this case, no more funds can be contributed to the pool until all the pending slash
		/// has been resolved.
		PoolBankrupt,
		/// There's no pending reward to claim
		NoRewardToClaim,
		/// The StakePool is not enabled yet.
		FeatureNotEnabled,
		/// Failed to add a worker because the number of the workers exceeds the upper limit.
		WorkersExceedLimit,
		/// Restarted with a less stake is not allowed in the tokenomic.
		CannotRestartWithLessStake,
		/// Invalid amount of balance input when force reward.
		InvalidForceRewardAmount,
		/// Invalid staker to contribute because origin isn't in Pool's contribution whitelist.
		NotInContributeWhitelist,
		/// Can not add the staker to whitelist because the staker is already in whitelist.
		AlreadyInContributeWhitelist,
		/// Too many stakers in contribution whitelist that exceed the limit
		ExceedWhitelistMaxLen,
		/// The pool hasn't have a whitelist created
		NoWhitelistCreated,
		/// Too long for pool description length
		ExceedMaxDescriptionLen,
		/// Withdraw queue is not empty so that we can't restart mining
		WithdrawQueueNotEmpty,
		/// Stakepool's collection_id isn't founded
		MissingCollectionId,
		/// Vault is forced locked for it has some expired withdrawal
		VaultIsLocked,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + vault::Config,
	{
		/// Creates a new stake pool
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pid = basepool::Pallet::<T>::consume_new_pid();
			// TODO(mingxuan): create_collection should return cid
			let collection_id: CollectionId = pallet_rmrk_core::Pallet::<T>::collection_index();
			// Create a NFT collection related to the new stake pool
			let symbol: BoundedVec<u8, <T as pallet_rmrk_core::Config>::CollectionSymbolLimit> =
				format!("STAKEPOOL-{}", pid)
					.as_bytes()
					.to_vec()
					.try_into()
					.expect("create a bvec from string should never fail; qed.");
			pallet_rmrk_core::Pallet::<T>::create_collection(
				Origin::<T>::Signed(basepool::pallet_id::<T::AccountId>()).into(),
				Default::default(),
				None,
				symbol,
			)?;
			let account_id =
				basepool::pallet::create_staker_account::<T::AccountId>(pid, owner.clone());
			let (owner_reward_account, lock_account) =
				create_owner_and_lock_account::<T::AccountId>(pid, owner.clone());
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::StakePool(StakePool {
					basepool: basepool::BasePool {
						pid,
						owner: owner.clone(),
						total_shares: Zero::zero(),
						total_value: Zero::zero(),
						withdraw_queue: VecDeque::new(),
						value_subscribers: VecDeque::new(),
						cid: collection_id,
						pool_account_id: account_id,
					},
					payout_commission: None,
					cap: None,
					workers: VecDeque::new(),
					cd_workers: VecDeque::new(),
					lock_account,
					owner_reward_account,
				}),
			);
			Self::deposit_event(Event::<T>::PoolCreated { owner, pid });
			Ok(())
		}

		/// Adds a worker to a pool
		///
		/// This will bind a worker to the corresponding pool sub-account. The binding will not be
		/// released until the worker is removed gracefully by `remove_worker()`, or a force unbind
		/// by the worker operator via `Mining::unbind()`.
		///
		/// Requires:
		/// 1. The worker is registered and benchmarked
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
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			// make sure worker has not been not added
			let workers = &mut pool_info.workers;
			ensure!(!workers.contains(&pubkey), Error::<T>::WorkerExists);
			// too many workers may cause performance regression
			ensure!(
				workers.len() < T::MaxPoolWorkers::get() as usize,
				Error::<T>::WorkersExceedLimit
			);

			// generate miner account
			let miner: T::AccountId = pool_sub_account(pid, &pubkey);

			// bind worker with miner
			mining::pallet::Pallet::<T>::bind(miner.clone(), pubkey)
				.or(Err(Error::<T>::FailedToBindMinerAndWorker))?;

			// Save the preimage of the sub-account,
			// the lifecycle of the preimage should be the same with the miner record,
			// current implementation we don't delete miner records even its no longer in-use,
			// so we won't delete preimages for now.
			SubAccountPreimages::<T>::insert(miner.clone(), (pid, pubkey));

			// update worker vector
			workers.push_back(pubkey);
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info));
			WorkerAssignments::<T>::insert(&pubkey, pid);
			Self::deposit_event(Event::<T>::PoolWorkerAdded {
				pid,
				worker: pubkey,
				miner,
			});

			Ok(())
		}

		/// Removes a worker from a pool
		///
		/// Requires:
		/// 1. The worker is registered
		/// 2. The worker is associated with a pool
		/// 3. The worker is removable (not in mining)
		#[pallet::weight(0)]
		pub fn remove_worker(
			origin: OriginFor<T>,
			pid: u64,
			worker: WorkerPublicKey,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// The sender is the pool owner
			let pool = ensure_stake_pool::<T>(pid)?;
			ensure!(
				pool.basepool.owner == who,
				Error::<T>::UnauthorizedPoolOwner
			);
			// The worker is in this pool. It implies:
			// - The worker is already in `PoolInfo::worker` list
			let lookup_pid =
				WorkerAssignments::<T>::get(worker).ok_or(Error::<T>::WorkerDoesNotExist)?;
			ensure!(pid == lookup_pid, Error::<T>::WorkerInAnotherPool);
			// Remove the worker from the pool (notification suspended)
			let sub_account: T::AccountId = pool_sub_account(pid, &worker);
			mining::pallet::Pallet::<T>::unbind_miner(&sub_account, false)?;
			// Manually clean up the worker, including the pool worker list, and the assignment
			// indices. (Theoretically we can enable the unbinding notification, and follow the
			// same path as a force unbinding, but it doesn't sounds graceful.)
			Self::remove_worker_from_pool(&worker);
			Ok(())
		}

		// /// Destroys a stake pool
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
		/// Note: a smaller cap than current total_value if not allowed.
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::weight(0)]
		pub fn set_cap(origin: OriginFor<T>, pid: u64, cap: BalanceOf<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut pool_info = ensure_stake_pool::<T>(pid)?;

			// origin must be owner of pool
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			// check cap
			ensure!(
				pool_info.basepool.total_value <= cap,
				Error::<T>::InadequateCapacity
			);

			pool_info.cap = Some(cap);
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info));

			Self::deposit_event(Event::<T>::PoolCapacitySet { pid, cap });
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
			payout_commission: Option<Permill>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			// origin must be owner of pool
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);

			pool_info.payout_commission = payout_commission;
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info));

			let mut commission: u32 = 0;
			if let Some(ratio) = payout_commission {
				commission = ratio.deconstruct();
			}
			Self::deposit_event(Event::<T>::PoolCommissionSet { pid, commission });

			Ok(())
		}

		/// Add a staker accountid to contribution whitelist.
		///
		/// Calling this method will forbide stakers contribute who isn't in the whitelist.
		/// The caller must be the owner of the pool.
		/// If a pool hasn't registed in the wihtelist map, any staker could contribute as what they use to do.
		/// The whitelist has a lmit len of 100 stakers.
		#[pallet::weight(0)]
		pub fn add_staker_to_whitelist(
			origin: OriginFor<T>,
			pid: u64,
			staker: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pool_info = ensure_stake_pool::<T>(pid)?;
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			if let Some(mut whitelist) = PoolContributionWhitelists::<T>::get(&pid) {
				ensure!(
					!whitelist.contains(&staker),
					Error::<T>::AlreadyInContributeWhitelist
				);
				ensure!(
					(whitelist.len() as u32) < MAX_WHITELIST_LEN,
					Error::<T>::ExceedWhitelistMaxLen
				);
				whitelist.push(staker.clone());
				PoolContributionWhitelists::<T>::insert(&pid, &whitelist);
			} else {
				let new_list = vec![staker.clone()];
				PoolContributionWhitelists::<T>::insert(&pid, &new_list);
				Self::deposit_event(Event::<T>::PoolWhitelistCreated { pid });
			}
			Self::deposit_event(Event::<T>::PoolWhitelistStakerAdded { pid, staker });

			Ok(())
		}

		/// Add a description to the pool
		///
		/// The caller must be the owner of the pool.
		#[pallet::weight(0)]
		pub fn set_pool_description(
			origin: OriginFor<T>,
			pid: u64,
			description: DescStr,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pool_info = ensure_stake_pool::<T>(pid)?;
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			PoolDescriptions::<T>::insert(&pid, description);

			Ok(())
		}

		/// Remove a staker accountid to contribution whitelist.
		///
		/// The caller must be the owner of the pool.
		/// If the last staker in the whitelist is removed, the pool will return back to a normal pool that allow anyone to contribute.
		#[pallet::weight(0)]
		pub fn remove_staker_from_whitelist(
			origin: OriginFor<T>,
			pid: u64,
			staker: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pool_info = ensure_stake_pool::<T>(pid)?;
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			let mut whitelist =
				PoolContributionWhitelists::<T>::get(&pid).ok_or(Error::<T>::NoWhitelistCreated)?;
			ensure!(
				whitelist.contains(&staker),
				Error::<T>::NotInContributeWhitelist
			);
			whitelist.retain(|accountid| accountid != &staker);
			if whitelist.is_empty() {
				PoolContributionWhitelists::<T>::remove(&pid);
				Self::deposit_event(Event::<T>::PoolWhitelistStakerRemoved {
					pid,
					staker: staker.clone(),
				});
				Self::deposit_event(Event::<T>::PoolWhitelistDeleted { pid });
			} else {
				PoolContributionWhitelists::<T>::insert(&pid, &whitelist);
				Self::deposit_event(Event::<T>::PoolWhitelistStakerRemoved {
					pid,
					staker: staker.clone(),
				});
			}

			Ok(())
		}

		/// Claims pool-owner's pending rewards of the sender and send to the `target`
		///
		/// The rewards associate to sender's "staker role" will not be claimed
		///
		/// Requires:
		/// 1. The sender is a pool owner
		#[pallet::weight(0)]
		pub fn claim_owner_rewards(
			origin: OriginFor<T>,
			pid: u64,
			target: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let pool_info = ensure_stake_pool::<T>(pid)?;
			// Add pool owner's reward if applicable
			ensure!(
				who == pool_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			let rewards = pool_info.get_owner_stakes::<T>();
			ensure!(rewards > Zero::zero(), Error::<T>::NoRewardToClaim);
			<pallet_assets::pallet::Pallet<T> as Transfer<T::AccountId>>::transfer(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&pool_info.owner_reward_account,
				&target,
				rewards,
				false,
			)?;
			Self::deposit_event(Event::<T>::OwnerRewardsWithdrawn {
				pid,
				user: who,
				amount: rewards,
			});

			Ok(())
		}

		/// Let any user to launch a stakepool withdraw. Then check if the pool need to be forced shutdown.
		///
		/// If the shutdown condition is met, all workers in the pool will be forced shutdown.
		/// Note: This function doesn't guarantee no-op when there's error.
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn check_and_maybe_force_withdraw(origin: OriginFor<T>, pid: u64) -> DispatchResult {
			ensure_signed(origin)?;
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			let mut pool = ensure_stake_pool::<T>(pid)?;
			basepool::Pallet::<T>::try_process_withdraw_queue(&mut pool.basepool);
			let grace_period = T::GracePeriod::get();
			let mut releasing_stake = Zero::zero();
			for worker in pool.cd_workers.iter() {
				let miner: T::AccountId = pool_sub_account(pid, worker);
				let stakes: BalanceOf<T> = mining::pallet::Stakes::<T>::get(&miner)
					.expect("workers have no stakes recorded; qed.");
				// TODO(mingxuan): handle slash
				releasing_stake += stakes;
			}
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool.clone()));
			if basepool::Pallet::<T>::has_expired_withdrawal(
				&pool.basepool,
				now,
				grace_period,
				releasing_stake,
			) {
				for worker in pool.workers.iter() {
					let miner: T::AccountId = pool_sub_account(pid, worker);
					let miner_info = match mining::pallet::Pallet::<T>::miners(&miner) {
						Some(miner) => miner,
						None => continue, // Skip non-existing miners
					};
					if !miner_info.state.is_mining() {
						continue;
					}
					if !pool.cd_workers.contains(worker) {
						Self::do_stop_mining(&pool.basepool.owner, pid, *worker)?;
					}
				}
			}

			Ok(())
		}

		/// Contributes some stake to a stakepool
		///
		/// Requires:
		/// 1. The pool exists
		/// 2. After the deposit, the pool doesn't reach the cap
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn contribute(
			origin: OriginFor<T>,
			pid: u64,
			amount: BalanceOf<T>,
			as_vault: Option<u64>,
		) -> DispatchResult {
			let mut who = ensure_signed(origin)?;
			let mut maybe_vault = None;
			if let Some(vault_pid) = as_vault {
				let vault_info = ensure_vault::<T>(vault_pid)?;
				ensure!(
					who == vault_info.basepool.owner,
					Error::<T>::UnauthorizedPoolOwner
				);
				who = vault_info.basepool.pool_account_id.clone();
				maybe_vault = Some((vault_pid, vault_info));
			}
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			let a = amount; // Alias to reduce confusion in the code below
				// If the pool has a contribution whitelist in storages, check if the origin is authorized to contribute
			if let Some(whitelist) = PoolContributionWhitelists::<T>::get(&pid) {
				ensure!(
					whitelist.contains(&who) || pool_info.basepool.owner == who,
					Error::<T>::NotInContributeWhitelist
				);
			}
			ensure!(
				a >= T::MinContribution::get(),
				Error::<T>::InsufficientContribution
			);
			let free = match &maybe_vault {
				Some((_, vault_info)) => vault_info.basepool.get_free_stakes::<T>(),
				_ => pallet_assets::Pallet::<T>::balance(
					<T as pawnshop::Config>::PPhaAssetId::get(),
					&who,
				),
			};
			ensure!(free >= a, Error::<T>::InsufficientBalance);
			// a lot of weird edge cases when dealing with pending slash.
			let shares =
				basepool::Pallet::<T>::contribute(&mut pool_info.basepool, who.clone(), amount)?;
			if let Some((vault_pid, vault_info)) = &mut maybe_vault {
				if !vault_info.invest_pools.contains(&pid) {
					vault_info.invest_pools.push_back(pid);
				}
				basepool::pallet::Pools::<T>::insert(
					*vault_pid,
					PoolProxy::Vault(vault_info.clone()),
				);
				if !pool_info.basepool.value_subscribers.contains(vault_pid) {
					pool_info.basepool.value_subscribers.push_back(*vault_pid);
				}
			}
			// We have new free stake now, try to handle the waiting withdraw queue

			basepool::Pallet::<T>::try_process_withdraw_queue(&mut pool_info.basepool);

			// Post-check to ensure the total stake doesn't exceed the cap
			if let Some(cap) = pool_info.cap {
				ensure!(
					pool_info.basepool.total_value <= cap,
					Error::<T>::StakeExceedsCapacity
				);
			}
			// Persist
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info.clone()));
			basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			if as_vault.is_none() {
				pawnshop::Pallet::<T>::maybe_subscribe_to_pool(&who, pid, pool_info.basepool.cid)?;
			}

			Self::deposit_event(Event::<T>::Contribution {
				pid,
				user: who,
				amount: a,
				shares,
			});
			Ok(())
		}

		/// Demands the return of some stake from a pool.
		///
		/// Note: there are two scenarios people may meet
		///
		/// - The withdrawal would be queued and delayed until there is enough free stake.
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn withdraw(
			origin: OriginFor<T>,
			pid: u64,
			shares: BalanceOf<T>,
			as_vault: Option<u64>,
		) -> DispatchResult {
			let mut who = ensure_signed(origin)?;
			if let Some(vault_pid) = as_vault {
				let vault_info = ensure_vault::<T>(vault_pid)?;
				ensure!(
					!vault::pallet::VaultLocks::<T>::contains_key(vault_pid),
					Error::<T>::VaultIsLocked
				);
				ensure!(
					who == vault_info.basepool.owner,
					Error::<T>::UnauthorizedPoolOwner
				);
				who = vault_info.basepool.pool_account_id;
			}
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			let nft_id = basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			// The nft instance must be wrote to Nft storage at the end of the function
			// this nft's property shouldn't be accessed or wrote again from storage before set_nft_attr
			// is called. Or the property of the nft will be overwrote incorrectly.
			let mut nft_guard =
				basepool::Pallet::<T>::get_nft_attr_guard(pool_info.basepool.cid, nft_id)?;
			let nft = &mut nft_guard.attr;
			let in_queue_shares = match pool_info
				.basepool
				.withdraw_queue
				.iter()
				.find(|&withdraw| withdraw.user == who)
			{
				Some(withdraw) => {
					let withdraw_nft_guard = basepool::Pallet::<T>::get_nft_attr_guard(
						pool_info.basepool.cid,
						withdraw.nft_id,
					)
					.expect("get nftattr should always success; qed.");
					withdraw_nft_guard.attr.shares
				}
				None => Zero::zero(),
			};
			ensure!(
				basepool::is_nondust_balance(shares) && (shares <= nft.shares + in_queue_shares),
				Error::<T>::InvalidWithdrawalAmount
			);
			basepool::Pallet::<T>::try_withdraw(&mut pool_info.basepool, nft, who.clone(), shares)?;
			nft_guard.save()?;
			let _nft_id =
				basepool::Pallet::<T>::merge_or_init_nft_for_staker(pool_info.basepool.cid, who)?;
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info.clone()));

			Ok(())
		}

		/// Starts a miner on behalf of the stake pool
		///
		/// Requires:
		/// 1. The miner is bound to the pool and is in Ready state
		/// 2. The remaining stake in the pool can cover the minimal stake required
		#[pallet::weight(0)]
		pub fn start_mining(
			origin: OriginFor<T>,
			pid: u64,
			worker: WorkerPublicKey,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::do_start_mining(&owner, pid, worker, stake)
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
			Self::do_stop_mining(&owner, pid, worker)
		}

		/// Reclaims the releasing stake of a miner in a pool.
		#[pallet::weight(0)]
		pub fn reclaim_pool_worker(
			origin: OriginFor<T>,
			pid: u64,
			worker: WorkerPublicKey,
		) -> DispatchResult {
			ensure_signed(origin)?;
			ensure_stake_pool::<T>(pid)?;
			let sub_account: T::AccountId = pool_sub_account(pid, &worker);
			Self::do_reclaim(pid, sub_account, worker, true).map(|_| ())
		}

		/// Enables or disables mining. Must be called with the council or root permission.
		#[pallet::weight(0)]
		pub fn set_mining_enable(origin: OriginFor<T>, enable: bool) -> DispatchResult {
			T::MiningSwitchOrigin::ensure_origin(origin)?;
			MiningEnabled::<T>::put(enable);
			Ok(())
		}

		/// Restart the miner with a higher stake
		#[pallet::weight(195_000_000)]
		#[frame_support::transactional]
		pub fn restart_mining(
			origin: OriginFor<T>,
			pid: u64,
			worker: WorkerPublicKey,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			// Make sure the withdraw queue is empty to avoid troubles
			let pool = ensure_stake_pool::<T>(pid)?;
			ensure!(
				pool.basepool.withdraw_queue.len() as u32 <= 0,
				Error::<T>::WithdrawQueueNotEmpty
			);
			// Stop and instantly reclaim the worker
			Self::do_stop_mining(&owner, pid, worker)?;
			let miner: T::AccountId = pool_sub_account(pid, &worker);
			let (orig_stake, slashed) = Self::do_reclaim(pid, miner, worker, false)?;
			let released = orig_stake - slashed;
			ensure!(stake > released, Error::<T>::CannotRestartWithLessStake);
			// Simply start mining. Rollback if there's no enough stake,
			Self::do_start_mining(&owner, pid, worker, stake)
		}
	}

	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + vault::Config,
	{
		pub fn do_start_mining(
			owner: &T::AccountId,
			pid: u64,
			worker: WorkerPublicKey,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let pool_info = ensure_stake_pool::<T>(pid)?;
			// origin must be owner of pool
			ensure!(
				&pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			// check free stake
			ensure!(
				pool_info.basepool.get_free_stakes::<T>() >= stake,
				Error::<T>::InsufficientFreeStake
			);
			// check wheather we have add this worker
			ensure!(
				pool_info.workers.contains(&worker),
				Error::<T>::WorkerDoesNotExist
			);
			let miner: T::AccountId = pool_sub_account(pid, &worker);
			mining::pallet::Pallet::<T>::start_mining(miner, stake)?;
			<pallet_assets::pallet::Pallet<T> as Transfer<T::AccountId>>::transfer(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&pool_info.basepool.pool_account_id,
				&pool_info.lock_account,
				stake,
				false,
			)?;
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info));
			Self::deposit_event(Event::<T>::MiningStarted {
				pid,
				worker,
				amount: stake,
			});

			Ok(())
		}
		fn do_stop_mining(
			owner: &T::AccountId,
			pid: u64,
			worker: WorkerPublicKey,
		) -> DispatchResult {
			ensure!(Self::mining_enabled(), Error::<T>::FeatureNotEnabled);
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			// origin must be owner of pool
			ensure!(
				&pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			// check whether we have add this worker
			ensure!(
				pool_info.workers.contains(&worker),
				Error::<T>::WorkerDoesNotExist
			);
			ensure!(
				!pool_info.cd_workers.contains(&worker),
				Error::<T>::WorkerAlreadyStopped
			);
			let miner: T::AccountId = pool_sub_account(pid, &worker);
			// Mining::stop_mining will notify us how much it will release by `on_stopped`
			<mining::pallet::Pallet<T>>::stop_mining(miner)?;
			pool_info.cd_workers.push_back(worker);
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info.clone()));
			Ok(())
		}
		fn do_reclaim(
			pid: u64,
			sub_account: T::AccountId,
			worker: WorkerPublicKey,
			check_cooldown: bool,
		) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
			let (orig_stake, slashed) = mining::Pallet::<T>::reclaim(sub_account, check_cooldown)?;
			Self::handle_reclaim(pid, orig_stake, slashed);
			Self::deposit_event(Event::<T>::WorkerReclaimed { pid, worker });
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			pool_info.remove_cd_worker(&worker);
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info.clone()));
			Ok((orig_stake, slashed))
		}

		/// Adds up the newly received reward to `reward_acc`
		fn handle_pool_new_reward(
			pool_info: &mut StakePool<T::AccountId, BalanceOf<T>>,
			rewards: BalanceOf<T>,
		) {
			if rewards > Zero::zero() {
				mining::Pallet::<T>::withdraw_subsidy_pool(
					&<T as pawnshop::Config>::PawnShopAccountId::get(),
					rewards,
				)
				.expect("this should not happen");
				if basepool::balance_close_to_zero(pool_info.basepool.total_shares) {
					Self::deposit_event(Event::<T>::RewardDismissedNoShare {
						pid: pool_info.basepool.pid,
						amount: rewards,
					});
					return;
				}
				let commission = pool_info.payout_commission.unwrap_or_default() * rewards;

				pawnshop::Pallet::<T>::mint_into(&pool_info.owner_reward_account, commission)
					.expect("mint into should be success");
				let to_distribute = rewards - commission;
				pawnshop::Pallet::<T>::mint_into(
					&pool_info.basepool.pool_account_id,
					to_distribute,
				)
				.expect("mint into should be success");
				let distributed = if basepool::is_nondust_balance(to_distribute) {
					pool_info.basepool.distribute_reward::<T>(to_distribute);
					true
				} else if to_distribute > Zero::zero() {
					Self::deposit_event(Event::<T>::RewardDismissedDust {
						pid: pool_info.basepool.pid,
						amount: to_distribute,
					});
					false
				} else {
					false
				};
				if distributed || commission > Zero::zero() {
					Self::deposit_event(Event::<T>::RewardReceived {
						pid: pool_info.basepool.pid,
						to_owner: commission,
						to_stakers: to_distribute,
					});
				}
			}
		}

		/// Called when worker was reclaimed.
		///
		/// After the cool down ends, worker was cleaned up, whose contributed balance would be
		/// reset to zero.
		fn handle_reclaim(pid: u64, orig_stake: BalanceOf<T>, slashed: BalanceOf<T>) {
			let mut pool_info = ensure_stake_pool::<T>(pid).expect("Stake pool must exist; qed.");

			let returned = orig_stake - slashed;
			if slashed != Zero::zero() {
				// Remove some slashed value from `total_value`, causing the share price to reduce
				// and creating a logical pending slash. The actual slash happens with the pending
				// slash to individuals is settled.
				pool_info.basepool.slash(slashed);
				//TODO(mingxuan): Burn the PPHA and transfer the amount to treasury when slash is active
				Self::deposit_event(Event::<T>::PoolSlashed {
					pid,
					amount: slashed,
				});
			}

			// With the worker being cleaned, those stake now are free
			<pallet_assets::pallet::Pallet<T> as Transfer<T::AccountId>>::transfer(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&pool_info.lock_account,
				&pool_info.basepool.pool_account_id,
				returned,
				false,
			)
			.expect("transfer should not fail");

			basepool::Pallet::<T>::try_process_withdraw_queue(&mut pool_info.basepool);
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info.clone()));
		}

		/// Removes a worker from a pool, either intentionally or unintentionally.
		///
		/// It assumes the worker is already in a pool.
		fn remove_worker_from_pool(worker: &WorkerPublicKey) {
			let pid = WorkerAssignments::<T>::take(worker).expect("Worker must be in a pool; qed.");
			basepool::pallet::Pools::<T>::mutate(pid, |value| {
				if let Some(PoolProxy::StakePool(pool)) = value {
					pool.remove_worker(worker);
					Self::deposit_event(Event::<T>::PoolWorkerRemoved {
						pid,
						worker: *worker,
					});
					// To adjust the case that skip stakepool::stop_mining when call remove_worker
					// (TODO(mingxuan): should let remove_worker in stakepool call mining directly instead of stakepool -> mining -> stakepool
					// and remove this cover code.)
					if !pool.cd_workers.contains(worker) {
						pool.cd_workers.push_back(*worker);
					}
				}
			});
		}

		pub(crate) fn migration_remove_assignments() -> Weight {
			let writes = SubAccountAssignments::<T>::drain().count();
			T::DbWeight::get().writes(writes as _)
		}
	}

	impl<T: Config> mining::OnReward for Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + vault::Config,
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
						Self::deposit_event(Event::<T>::RewardDismissedNotInPool {
							worker: info.pubkey,
							amount: reward,
						});
						return;
					}
				};
				let mut pool_info =
					ensure_stake_pool::<T>(pid).expect("Stake pool must exist; qed.");
				Self::handle_pool_new_reward(&mut pool_info, reward);
				basepool::pallet::Pools::<T>::insert(pid, PoolProxy::StakePool(pool_info));
			}
		}
	}

	impl<T: Config> mining::OnUnbound for Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + vault::Config,
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
		BalanceOf<T>: FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + vault::Config,
	{
		fn on_stopped(
			_worker: &WorkerPublicKey,
			_orig_stake: BalanceOf<T>,
			_slashed: BalanceOf<T>,
		) {
		}
	}

	pub fn pool_sub_account<T>(pid: u64, pubkey: &WorkerPublicKey) -> T
	where
		T: Encode + Decode,
	{
		let hash = crate::hashing::blake2_256(&(pid, pubkey).encode());
		// stake pool miner
		(b"spm/", hash)
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed")
	}

	pub fn create_owner_and_lock_account<T>(pid: u64, owner: T) -> (T, T)
	where
		T: Encode + Decode,
	{
		let hash = crate::hashing::blake2_256(&(pid, owner).encode());
		let owner_reward_account = (b"so/", hash)
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed");
		let lock_account = (b"sl/", hash)
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed");
		(owner_reward_account, lock_account)
	}
}
