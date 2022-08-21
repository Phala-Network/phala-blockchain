//! Pool for collaboratively mining staking

pub use self::pallet::*;
use crate::mining;
use frame_support::traits::Currency;

type BalanceOf<T> =
	<<T as mining::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as mining::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	#[cfg(not(feature = "std"))]
	use alloc::format;
	#[cfg(feature = "std")]
	use std::format;

	use crate::balance_convert::{div as bdiv, mul as bmul, FixedPointConvert};
	use crate::basepool;
	use crate::mining;
	use crate::pawnshop;
	use crate::poolproxy::*;
	use crate::registry;

	use fixed::types::U64F64 as FixedPoint;
	use fixed_macro::types::U64F64 as fp;

	use super::{BalanceOf, NegativeImbalanceOf};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			tokens::fungibles::Mutate, Currency, LockableCurrency, OnUnbalanced, StorageVersion,
			UnixTime,
		},
	};
	use frame_system::{pallet_prelude::*, Origin};

	use sp_runtime::{
		traits::{Saturating, TrailingZeroInput, Zero},
		Permill, SaturatedConversion,
	};
	use sp_std::{collections::vec_deque::VecDeque, fmt::Display, prelude::*, vec};

	use phala_types::{messaging::SettleInfo, WorkerPublicKey};

	pub use rmrk_traits::primitives::{CollectionId, NftId};

	const MAX_WHITELIST_LEN: u32 = 100;

	pub struct DescMaxLen;

	impl Get<u32> for DescMaxLen {
		fn get() -> u32 {
			4400
		}
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ registry::Config
		+ mining::Config
		+ pallet_rmrk_core::Config
		+ basepool::Config
		+ pallet_assets::Config
		+ pallet_democracy::Config
		+ pawnshop::Config
	{
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

	#[pallet::storage]
	pub type VaultAccountAssignments<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u64>;

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
	pub type PoolDescriptions<T: Config> =
		StorageMap<_, Twox64Concat, u64, BoundedVec<u8, super::DescMaxLen>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A pool is created under an owner
		///
		/// Affected states:
		/// - a new entry in [`StakePools`] with the pid
		PoolCreated {
			owner: T::AccountId,
			pid: u64,
		},
		/// The commission of a pool is updated
		///
		/// The commission ratio is represented by an integer. The real value is
		/// `commission / 1_000_000u32`.
		///
		/// Affected states:
		/// - the `payout_commission` field in [`StakePools`] is updated
		PoolCommissionSet {
			pid: u64,
			commission: u32,
		},

		VaultCommissionSet {
			pid: u64,
			commission: u32,
		},
		/// The stake capacity of the pool is updated
		///
		/// Affected states:
		/// - the `cap` field in [`StakePools`] is updated
		PoolCapacitySet {
			pid: u64,
			cap: BalanceOf<T>,
		},
		/// A worker is added to the pool
		///
		/// Affected states:
		/// - the `worker` is added to the vector `workers` in [`StakePools`]
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
		/// - the stake related fields in [`StakePools`]
		/// - the user staking account at [`PoolStakers`]
		/// - the locking ledger of the contributor at [`StakeLedger`]
		/// - when there was any request in the withdraw queue, the action may trigger withdrawals
		///   ([`Withdrawal`](#variant.Withdrawal) event)
		Contribution {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},

		VaultContribution {
			pid: u64,
			user: T::AccountId,
			vault_pid: u64,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},
		/// Some stake was withdrawn from a pool
		///
		/// The lock in [`Balances`](pallet_balances::pallet::Pallet) is updated to release the
		/// locked stake.
		///
		/// Affected states:
		/// - the stake related fields in [`StakePools`]
		/// - the user staking account at [`PoolStakers`]
		/// - the locking ledger of the contributor at [`StakeLedger`]
		Withdrawal {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},

		VaultWithdrawal {
			pid: u64,
			user: T::AccountId,
			vault_pid: u64,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},
		/// Pending rewards were withdrawn by a user
		///
		/// The reward and slash accumulator is resolved, and the reward is sent to the user
		/// account.
		///
		/// Affected states:
		/// - the stake related fields in [`StakePools`]
		/// - the user staking account at [`PoolStakers`]
		RewardsWithdrawn {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
		},
		/// Similar to event `RewardsWithdrawn` but only affected states:
		///  - the stake related fields in [`StakePools`]
		OwnerRewardsWithdrawn {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
		},

		OwnerSharesGained {
			pid: u64,
			shares: BalanceOf<T>,
		},

		OwnerSharesStartWithdraw {
			pid: u64,
			user: T::AccountId,
			shares: BalanceOf<T>,
		},
		/// Similar to event `ewardsWithdrawn` but only affected states:
		///  - the user staking account at [`PoolStakers`]
		StakerRewardsWithdrawn {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
		},
		/// The pool received a slash event from one of its workers (currently disabled)
		///
		/// The slash is accured to the pending slash accumulator.
		PoolSlashed {
			pid: u64,
			amount: BalanceOf<T>,
		},
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
		RewardDismissedNoShare {
			pid: u64,
			amount: BalanceOf<T>,
		},
		/// Some reward is dismissed because the amount is too tiny (dust)
		///
		/// There's no affected state.
		RewardDismissedDust {
			pid: u64,
			amount: BalanceOf<T>,
		},
		/// Some dust stake is removed
		///
		/// Triggered when the remaining stake of a user is too small after withdrawal or slash.
		///
		/// Affected states:
		/// - the balance of the locking ledger of the contributor at [`StakeLedger`] is set to 0
		/// - the user's dust stake is moved to treasury
		DustRemoved {
			user: T::AccountId,
			amount: BalanceOf<T>,
		},
		/// A worker is removed from a pool.
		///
		/// Affected states:
		/// - the worker item in [`WorkerAssignments`] is removed
		/// - the worker is removed from the [`StakePools`] item
		PoolWorkerRemoved {
			pid: u64,
			worker: WorkerPublicKey,
		},
		/// A withdrawal request is inserted to a queue
		///
		/// Affected states:
		/// - a new item is inserted to or an old item is being replaced by the new item in the
		///   withdraw queue in [`StakePools`]
		WithdrawalQueued {
			pid: u64,
			user: T::AccountId,
			shares: BalanceOf<T>,
		},
		/// A pool contribution whitelist is added
		/// - lazy operated when the first staker is added to the whitelist
		PoolWhitelistCreated {
			pid: u64,
		},
		/// The pool contribution whitelist is deleted
		/// - lazy operated when the last staker is removed from the whitelist
		PoolWhitelistDeleted {
			pid: u64,
		},
		/// A staker is added to the pool contribution whitelist
		PoolWhitelistStakerAdded {
			pid: u64,
			staker: T::AccountId,
		},
		/// A staker is removed from the pool contribution whitelist
		PoolWhitelistStakerRemoved {
			pid: u64,
			staker: T::AccountId,
		},
		/// A worker is reclaimed from the pool
		WorkerReclaimed {
			pid: u64,
			worker: WorkerPublicKey,
		},
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
		}
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

		InvaildWithdrawSharesAmount,

		AssetAccountNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		/// Creates a new stake pool
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pid = basepool::PoolCount::<T>::get();
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
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(StakePool::<
					T::AccountId,
					BalanceOf<T>,
				> {
					basepool: basepool::BasePool {
						pid,
						owner: owner.clone(),
						total_shares: Zero::zero(),
						total_value: Zero::zero(),
						free_stake: Zero::zero(),
						withdraw_queue: VecDeque::new(),
						value_subscribers: VecDeque::new(),
						cid: collection_id,
					},
					payout_commission: None,
					owner_reward: Zero::zero(),
					cap: None,
					workers: VecDeque::new(),
					cd_workers: VecDeque::new(),
				}),
			);
			basepool::PoolCount::<T>::put(pid + 1);
			Self::deposit_event(Event::<T>::PoolCreated { owner, pid });
			Ok(())
		}

		/// Creates a new stake pool
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn create_vault(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pid = basepool::PoolCount::<T>::get();
			// TODO(mingxuan): create_collection should return cid
			let collection_id: CollectionId = pallet_rmrk_core::Pallet::<T>::collection_index();
			// Create a NFT collection related to the new stake pool
			let symbol: BoundedVec<u8, <T as pallet_rmrk_core::Config>::CollectionSymbolLimit> =
				format!("VAULT-{}", pid)
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
			let user_id = vault_staker_account(pid, owner.clone());
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(
					Vault::<T::AccountId, BalanceOf<T>> {
						basepool: basepool::BasePool {
							pid,
							owner: owner.clone(),
							total_shares: Zero::zero(),
							total_value: Zero::zero(),
							free_stake: Zero::zero(),
							withdraw_queue: VecDeque::new(),
							value_subscribers: VecDeque::new(),
							cid: collection_id,
						},
						pool_account_id: user_id.clone(),
						delta_price_ratio: None,
						owner_shares: Zero::zero(),
						last_share_price_checkpoint: Zero::zero(),
						invest_pools: VecDeque::new(),
					},
				),
			);
			basepool::PoolCount::<T>::put(pid + 1);
			VaultAccountAssignments::<T>::insert(user_id, pid);
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
				workers.len() + 1 <= T::MaxPoolWorkers::get() as usize,
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
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info),
			);
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
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info),
			);

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
			payout_commission: Permill,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			// origin must be owner of pool
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);

			pool_info.payout_commission = Some(payout_commission);
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info),
			);

			Self::deposit_event(Event::<T>::PoolCommissionSet {
				pid,
				commission: payout_commission.deconstruct(),
			});

			Ok(())
		}

		/// Change the pool commission rate
		///
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::weight(0)]
		pub fn set_vault_payout_pref(
			origin: OriginFor<T>,
			pid: u64,
			payout_commission: Permill,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut pool_info = ensure_vault::<T>(pid)?;
			// origin must be owner of pool
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);

			pool_info.delta_price_ratio = Some(payout_commission);
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(pool_info),
			);

			Self::deposit_event(Event::<T>::VaultCommissionSet {
				pid,
				commission: payout_commission.deconstruct(),
			});

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
			description: BoundedVec<u8, DescMaxLen>,
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
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			// Add pool owner's reward if applicable
			ensure!(
				who == pool_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			let rewards = pool_info.owner_reward;
			ensure!(rewards > Zero::zero(), Error::<T>::NoRewardToClaim);
			pallet_assets::Pallet::<T>::mint_into(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&target,
				rewards,
			)?;
			pool_info.owner_reward = Zero::zero();
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info),
			);
			Self::deposit_event(Event::<T>::OwnerRewardsWithdrawn {
				pid,
				user: who,
				amount: rewards,
			});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn claim_owner_shares(
			origin: OriginFor<T>,
			vault_pid: u64,
			target: T::AccountId,
			shares: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let mut pool_info = ensure_vault::<T>(vault_pid)?;
			// Add pool owner's reward if applicable
			ensure!(
				who == pool_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			ensure!(
				pool_info.owner_shares >= shares,
				Error::<T>::InvaildWithdrawSharesAmount
			);
			ensure!(shares > Zero::zero(), Error::<T>::NoRewardToClaim);
			let price = pool_info
				.basepool
				.share_price()
				.expect("price must exist when owner_shares exist");
			let rewards = bmul(shares, &price);
			let nft_id = basepool::Pallet::<T>::mint_nft(
				pool_info.basepool.cid,
				target.clone(),
				shares.clone(),
				rewards,
			)?;
			Self::withdraw_from_vault(origin, pool_info.basepool.pid, shares)?;
			pool_info.owner_shares -= shares;
			basepool::pallet::Pools::<T>::insert(
				vault_pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(pool_info),
			);
			Self::deposit_event(Event::<T>::OwnerSharesStartWithdraw {
				pid: vault_pid,
				user: who,
				shares,
			});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn maybe_gain_owner_shares(origin: OriginFor<T>, vault_pid: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut pool_info = ensure_vault::<T>(vault_pid)?;
			// Add pool owner's reward if applicable
			ensure!(
				who == pool_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			let current_price = match pool_info.basepool.share_price() {
				Some(price) => BalanceOf::<T>::from_fixed(&price),
				None => return Ok(()),
			};
			if pool_info.last_share_price_checkpoint == Zero::zero() {
				pool_info.last_share_price_checkpoint = current_price;
				basepool::pallet::Pools::<T>::insert(
					vault_pid,
					PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(pool_info),
				);
				return Ok(());
			}
			if current_price <= pool_info.last_share_price_checkpoint {
				return Ok(());
			}
			let delta_price = pool_info.delta_price_ratio.unwrap_or_default()
				* (current_price - pool_info.last_share_price_checkpoint);
			let new_price = current_price - delta_price;
			let adjust_shares = bdiv(pool_info.basepool.total_value, &new_price.to_fixed())
				- pool_info.basepool.total_shares;
			pool_info.basepool.total_shares += adjust_shares;
			pool_info.owner_shares += adjust_shares;
			pool_info.last_share_price_checkpoint = current_price;

			basepool::pallet::Pools::<T>::insert(
				vault_pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(pool_info),
			);
			Self::deposit_event(Event::<T>::OwnerSharesGained {
				pid: vault_pid,
				shares: adjust_shares,
			});

			Ok(())
		}

		/// Let any user to launch a stakepool withdraw. Then check if the pool need to be forced shutdown.
		///
		/// If the shutdown condition is met, all workers in the pool will be forced shutdown.
		/// Note: This function doesn't guarantee no-op when there's error.
		#[pallet::weight(0)]
		pub fn check_and_maybe_force_withdraw(origin: OriginFor<T>, pid: u64) -> DispatchResult {
			ensure_signed(origin)?;
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			let mut pool = ensure_stake_pool::<T>(pid)?;
			Self::try_process_withdraw_queue(&mut pool.basepool);
			let grace_period = T::GracePeriod::get();
			let mut releasing_stake = Zero::zero();
			for worker in pool.cd_workers.iter() {
				let miner: T::AccountId = pool_sub_account(pid, &worker);
				let stakes: BalanceOf<T> = mining::pallet::Stakes::<T>::get(&miner)
					.expect("workers have no stakes recorded; qed.");
				// TODO(mingxuan): handle slash
				releasing_stake += stakes;
			}
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool.clone()),
			);
			if basepool::Pallet::<T>::has_expired_withdrawal(
				&mut pool.basepool,
				now,
				grace_period,
				releasing_stake,
			) {
				for worker in pool.workers.iter() {
					let miner: T::AccountId = pool_sub_account(pid, &worker);
					if !pool.cd_workers.contains(&worker) {
						Self::do_stop_mining(&pool.basepool.owner, pid, worker.clone())?;
					}
				}
			}

			Ok(())
		}

		/// Let any user to launch a stakepool withdraw. Then check if the pool need to be forced shutdown.
		///
		/// If the shutdown condition is met, all workers in the pool will be forced shutdown.
		/// Note: This function doesn't guarantee no-op when there's error.
		#[pallet::weight(0)]
		pub fn vault_check_and_maybe_force_withdraw(
			origin: OriginFor<T>,
			vault_pid: u64,
		) -> DispatchResult {
			ensure_signed(origin.clone())?;
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			let mut vault = ensure_vault::<T>(vault_pid)?;
			Self::try_process_withdraw_queue(&mut vault.basepool);
			let grace_period = T::GracePeriod::get();
			let releasing_stake = Zero::zero();
			for pid in vault.invest_pools.iter() {
				let stake_pool = ensure_stake_pool::<T>(*pid)?;
				let withdraw_vec: VecDeque<_> = stake_pool
					.basepool
					.withdraw_queue
					.iter()
					.filter(|x| x.user == vault.pool_account_id)
					.collect();
				// the length of vec should be 1
				for withdraw in withdraw_vec {
					let nft = basepool::Pallet::<T>::get_nft_attr(
						stake_pool.basepool.cid,
						withdraw.nft_id,
					)?;
					let price = stake_pool
						.basepool
						.share_price()
						.expect("pool must have price: qed.");
					let releasing_stake = bmul(nft.shares.clone(), &price);
				}
			}
			basepool::pallet::Pools::<T>::insert(
				vault_pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(vault.clone()),
			);
			if basepool::Pallet::<T>::has_expired_withdrawal(
				&mut vault.basepool,
				now,
				grace_period,
				releasing_stake,
			) {
				for pid in vault.invest_pools.iter() {
					let stake_pool = ensure_stake_pool::<T>(*pid)?;
					let withdraw_vec: VecDeque<_> = stake_pool
						.basepool
						.withdraw_queue
						.iter()
						.filter(|x| x.user == vault.pool_account_id)
						.collect();
					// the length of vec should be 1
					for withdraw in withdraw_vec {
						let nft = basepool::Pallet::<T>::get_nft_attr(
							stake_pool.basepool.cid,
							withdraw.nft_id,
						)?;
						Self::vault_withdraw(
							origin.clone(),
							stake_pool.basepool.pid,
							vault_pid,
							nft.shares,
						)?;
					}
				}
			}

			Ok(())
		}

		/// Contributes some stake to a vault
		///
		/// Requires:
		/// 1. The pool exists
		/// 2. After the deposit, the pool doesn't reach the cap
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn contribute_to_vault(
			origin: OriginFor<T>,
			pid: u64,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut pool_info = ensure_vault::<T>(pid)?;
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
			let free = pallet_assets::Pallet::<T>::maybe_balance(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&who,
			)
			.ok_or(Error::<T>::AssetAccountNotExist)?;
			ensure!(free >= a, Error::<T>::InsufficientBalance);
			// We don't really want to allow to contribute to a bankrupt StakePool. It can avoid
			// a lot of weird edge cases when dealing with pending slash.
			let shares = basepool::Pallet::<T>::contribute(
				&mut pool_info.basepool,
				who.clone(),
				amount,
				Some(
					|x: &basepool::pallet::BasePool<T::AccountId, BalanceOf<T>>,
					 y: &mut basepool::NftAttr<BalanceOf<T>>,
					 z: T::AccountId| {},
				),
			)?;

			// We have new free stake now, try to handle the waiting withdraw queue

			Self::try_process_withdraw_queue(&mut pool_info.basepool);

			// Persist
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(pool_info.clone()),
			);
			basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			pallet_assets::Pallet::<T>::burn_from(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&who,
				a,
			)?;

			let mut account_status = pawnshop::pallet::StakerAccounts::<T>::get(who.clone())
				.ok_or(pawnshop::Error::<T>::StakerAccountNotFound)?;

			if !account_status
				.invest_pools
				.contains(&(pid, pool_info.basepool.cid))
			{
				account_status
					.invest_pools
					.push((pid, pool_info.basepool.cid));
				pawnshop::pallet::StakerAccounts::<T>::insert(who.clone(), account_status);
			}

			Self::deposit_event(Event::<T>::Contribution {
				pid,
				user: who,
				amount: a,
				shares,
			});
			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn vault_investment(
			origin: OriginFor<T>,
			vault_pid: u64,
			pid: u64,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut vault_info = ensure_vault::<T>(vault_pid)?;
			// Currently, we only allow vault invest to stakepool
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			// Add pool owner's reward if applicable
			ensure!(
				who == vault_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			let a = amount; // Alias to reduce confusion in the code below
				// If the pool has a contribution whitelist in storages, check if the origin is authorized to contribute
			if let Some(whitelist) = PoolContributionWhitelists::<T>::get(&pid) {
				ensure!(
					whitelist.contains(&vault_info.pool_account_id),
					Error::<T>::NotInContributeWhitelist
				);
			}
			ensure!(
				a >= T::MinContribution::get(),
				Error::<T>::InsufficientContribution
			);
			let free = vault_info.basepool.free_stake;
			ensure!(free >= a, Error::<T>::InsufficientBalance);
			// We don't really want to allow to contribute to a bankrupt StakePool. It can avoid
			// a lot of weird edge cases when dealing with pending slash.
			let shares = basepool::Pallet::<T>::contribute(
				&mut pool_info.basepool,
				vault_info.pool_account_id.clone(),
				amount,
				Some(
					|x: &basepool::pallet::BasePool<T::AccountId, BalanceOf<T>>,
					 y: &mut basepool::NftAttr<BalanceOf<T>>,
					 z: T::AccountId| {
						Self::maybe_settle_nft_slash(x, y, z);
					},
				),
			)?;
			if !vault_info.invest_pools.contains(&pid) {
				vault_info.invest_pools.push_back(pid);
			}
			vault_info.basepool.free_stake -= a;
			basepool::pallet::Pools::<T>::insert(
				vault_pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(vault_info.clone()),
			);
			// We have new free stake now, try to handle the waiting withdraw queue

			Self::try_process_withdraw_queue(&mut pool_info.basepool);
			if !pool_info.basepool.value_subscribers.contains(&vault_pid) {
				pool_info.basepool.value_subscribers.push_back(vault_pid);
			}
			// Persist
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info.clone()),
			);
			basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				vault_info.pool_account_id.clone(),
			)?;
			Self::deposit_event(Event::<T>::VaultContribution {
				pid,
				user: who,
				vault_pid,
				amount: a,
				shares,
			});

			Ok(())
		}

		/// Contributes some stake to a stakepool
		///
		/// Requires:
		/// 1. The pool exists
		/// 2. After the deposit, the pool doesn't reach the cap
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn contribute(origin: OriginFor<T>, pid: u64, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
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
			let free = pallet_assets::Pallet::<T>::maybe_balance(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&who,
			)
			.ok_or(Error::<T>::AssetAccountNotExist)?;
			ensure!(free >= a, Error::<T>::InsufficientBalance);
			// We don't really want to allow to contribute to a bankrupt StakePool. It can avoid
			// a lot of weird edge cases when dealing with pending slash.
			let shares = basepool::Pallet::<T>::contribute(
				&mut pool_info.basepool,
				who.clone(),
				amount,
				Some(
					|x: &basepool::pallet::BasePool<T::AccountId, BalanceOf<T>>,
					 y: &mut basepool::NftAttr<BalanceOf<T>>,
					 z: T::AccountId| {
						Self::maybe_settle_nft_slash(x, y, z);
					},
				),
			)?;

			// We have new free stake now, try to handle the waiting withdraw queue

			Self::try_process_withdraw_queue(&mut pool_info.basepool);

			// Post-check to ensure the total stake doesn't exceed the cap
			if let Some(cap) = pool_info.cap {
				ensure!(
					pool_info.basepool.total_value <= cap,
					Error::<T>::StakeExceedsCapacity
				);
			}
			// Persist
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info.clone()),
			);
			basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			pallet_assets::Pallet::<T>::burn_from(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&who,
				a,
			)?;

			let mut account_status = pawnshop::pallet::StakerAccounts::<T>::get(who.clone())
				.ok_or(pawnshop::Error::<T>::StakerAccountNotFound)?;

			if !account_status
				.invest_pools
				.contains(&(pid, pool_info.basepool.cid))
			{
				account_status
					.invest_pools
					.push((pid, pool_info.basepool.cid));
				pawnshop::pallet::StakerAccounts::<T>::insert(who.clone(), account_status);
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
		/// - if the pool has free stake and the amount of the free stake is greater than or equal
		///     to the withdrawal amount (e.g. pool.free_stake >= amount), the withdrawal would
		///     take effect immediately.
		/// - else the withdrawal would be queued and delayed until there is enough free stake.
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn vault_withdraw(
			origin: OriginFor<T>,
			pid: u64,
			vault_pid: u64,
			shares: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			let vault_info = ensure_vault::<T>(vault_pid)?;
			ensure!(
				who == vault_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			let collection_id = pool_info.basepool.cid;
			let nft_id = basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				vault_info.pool_account_id.clone(),
			)?;
			// The nft instance must be wrote to Nft storage at the end of the function
			// this nft's property shouldn't be accessed or wrote again from storage before set_nft_attr
			// is called. Or the property of the nft will be overwrote incorrectly.
			let mut nft = basepool::Pallet::<T>::get_nft_attr(pool_info.basepool.cid, nft_id)?;
			let in_queue_shares = match pool_info
				.basepool
				.withdraw_queue
				.iter()
				.find(|&withdraw| withdraw.user == vault_info.pool_account_id.clone())
			{
				Some(withdraw) => {
					let withdraw_nft = basepool::Pallet::<T>::get_nft_attr(
						pool_info.basepool.cid,
						withdraw.nft_id,
					)
					.expect("get nftattr should always success; qed.");
					withdraw_nft.shares
				}
				None => Zero::zero(),
			};
			ensure!(
				basepool::is_nondust_balance(shares) && (shares <= nft.shares + in_queue_shares),
				Error::<T>::InvalidWithdrawalAmount
			);
			Self::try_withdraw(
				&mut pool_info.basepool,
				&mut nft,
				nft_id,
				vault_info.pool_account_id.clone(),
				shares,
				Some(vault_info.basepool.pid),
			)?;
			basepool::Pallet::<T>::set_nft_attr(pool_info.basepool.cid, nft_id, &nft)
				.expect("set nft attr should always success; qed.");
			let nft_id = basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info.clone()),
			);

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
		#[frame_support::transactional]
		pub fn withdraw_from_vault(
			origin: OriginFor<T>,
			pid: u64,
			shares: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut pool_info = ensure_vault::<T>(pid)?;
			let collection_id = pool_info.basepool.cid;
			let nft_id = basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			// The nft instance must be wrote to Nft storage at the end of the function
			// this nft's property shouldn't be accessed or wrote again from storage before set_nft_attr
			// is called. Or the property of the nft will be overwrote incorrectly.
			let mut nft = basepool::Pallet::<T>::get_nft_attr(pool_info.basepool.cid, nft_id)?;
			let in_queue_shares = match pool_info
				.basepool
				.withdraw_queue
				.iter()
				.find(|&withdraw| withdraw.user == who)
			{
				Some(withdraw) => {
					let withdraw_nft = basepool::Pallet::<T>::get_nft_attr(
						pool_info.basepool.cid,
						withdraw.nft_id,
					)
					.expect("get nftattr should always success; qed.");
					withdraw_nft.shares
				}
				None => Zero::zero(),
			};
			ensure!(
				basepool::is_nondust_balance(shares) && (shares <= nft.shares + in_queue_shares),
				Error::<T>::InvalidWithdrawalAmount
			);
			Self::try_withdraw(
				&mut pool_info.basepool,
				&mut nft,
				nft_id,
				who.clone(),
				shares,
				None,
			)?;
			basepool::Pallet::<T>::set_nft_attr(pool_info.basepool.cid, nft_id, &nft)
				.expect("set nft attr should always success; qed.");
			let nft_id = basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(pool_info.clone()),
			);

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
		#[frame_support::transactional]
		pub fn withdraw(origin: OriginFor<T>, pid: u64, shares: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			let collection_id = pool_info.basepool.cid;
			let nft_id = basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			// The nft instance must be wrote to Nft storage at the end of the function
			// this nft's property shouldn't be accessed or wrote again from storage before set_nft_attr
			// is called. Or the property of the nft will be overwrote incorrectly.
			let mut nft = basepool::Pallet::<T>::get_nft_attr(pool_info.basepool.cid, nft_id)?;
			let in_queue_shares = match pool_info
				.basepool
				.withdraw_queue
				.iter()
				.find(|&withdraw| withdraw.user == who)
			{
				Some(withdraw) => {
					let withdraw_nft = basepool::Pallet::<T>::get_nft_attr(
						pool_info.basepool.cid,
						withdraw.nft_id,
					)
					.expect("get nftattr should always success; qed.");
					withdraw_nft.shares
				}
				None => Zero::zero(),
			};
			ensure!(
				basepool::is_nondust_balance(shares) && (shares <= nft.shares + in_queue_shares),
				Error::<T>::InvalidWithdrawalAmount
			);
			Self::try_withdraw(
				&mut pool_info.basepool,
				&mut nft,
				nft_id,
				who.clone(),
				shares,
				None,
			)?;
			basepool::Pallet::<T>::set_nft_attr(pool_info.basepool.cid, nft_id, &nft)
				.expect("set nft attr should always success; qed.");
			let nft_id = basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info.clone()),
			);

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
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		pub fn do_start_mining(
			owner: &T::AccountId,
			pid: u64,
			worker: WorkerPublicKey,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			// origin must be owner of pool
			ensure!(
				&pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			// check free stake
			ensure!(
				pool_info.basepool.free_stake >= stake,
				Error::<T>::InsufficientFreeStake
			);
			// check wheather we have add this worker
			ensure!(
				pool_info.workers.contains(&worker),
				Error::<T>::WorkerDoesNotExist
			);
			let miner: T::AccountId = pool_sub_account(pid, &worker);
			mining::pallet::Pallet::<T>::start_mining(miner.clone(), stake)?;
			pool_info.basepool.free_stake -= stake;
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info.clone()),
			);
			Self::deposit_event(Event::<T>::MiningStarted {
				pid,
				worker: worker,
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
			pool_info.cd_workers.push_back(worker.clone());
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info.clone()),
			);
			Ok(())
		}
		fn do_reclaim(
			pid: u64,
			sub_account: T::AccountId,
			worker: WorkerPublicKey,
			check_cooldown: bool,
		) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
			let (orig_stake, slashed) =
				mining::Pallet::<T>::reclaim(sub_account.clone(), check_cooldown)?;
			Self::handle_reclaim(pid, orig_stake, slashed);
			Self::deposit_event(Event::<T>::WorkerReclaimed { pid, worker });
			let mut pool_info = ensure_stake_pool::<T>(pid)?;
			pool_info.remove_cd_worker(&worker);
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info.clone()),
			);
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
				pool_info.owner_reward.saturating_accrue(commission);
				let to_distribute = rewards - commission;
				let distributed = if basepool::is_nondust_balance(to_distribute) {
					pool_info
						.basepool
						.distribute_reward::<T>(to_distribute, true);
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
				Self::deposit_event(Event::<T>::PoolSlashed {
					pid,
					amount: slashed,
				});
			}

			// With the worker being cleaned, those stake now are free
			pool_info.basepool.free_stake.saturating_accrue(returned);

			Self::try_process_withdraw_queue(&mut pool_info.basepool);
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info.clone()),
			);
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
			pool_info: &mut basepool::BasePool<T::AccountId, BalanceOf<T>>,
			nft: &mut basepool::NftAttr<BalanceOf<T>>,
			nft_id: NftId,
			userid: T::AccountId,
			shares: BalanceOf<T>,
			maybe_vault_pid: Option<u64>,
		) -> DispatchResult {
			basepool::Pallet::<T>::push_withdraw_in_queue(
				pool_info,
				nft,
				userid.clone(),
				shares.clone(),
				maybe_vault_pid,
			)?;
			Self::deposit_event(Event::<T>::WithdrawalQueued {
				pid: pool_info.pid,
				user: userid.clone(),
				shares: shares,
			});
			Self::try_process_withdraw_queue(pool_info);

			Ok(())
		}

		fn maybe_remove_dust(
			pool_info: &mut basepool::BasePool<T::AccountId, BalanceOf<T>>,
			nft: &basepool::NftAttr<BalanceOf<T>>,
			userid: T::AccountId,
		) -> bool {
			if basepool::is_nondust_balance(nft.shares) {
				return false;
			}
			Self::remove_dust(&userid, nft.get_stake());
			pool_info.total_shares -= nft.shares;
			pool_info.total_value -= nft.get_stake();
			true
		}

		///should be very carful to avoid the vault_info got mutable ref outside the function and saved after this function called
		fn do_withdraw_shares(
			withdrawing_shares: BalanceOf<T>,
			pool_info: &mut basepool::BasePool<T::AccountId, BalanceOf<T>>,
			nft: &mut basepool::NftAttr<BalanceOf<T>>,
			nft_id: NftId,
			userid: T::AccountId,
		) {
			Self::maybe_settle_nft_slash(&pool_info, nft, userid.clone());
			// Overflow warning: remove_stake is carefully written to avoid precision error.
			// (I hope so)
			let (reduced, dust, withdrawn_shares) =
				basepool::Pallet::<T>::remove_stake_from_nft(pool_info, withdrawing_shares, nft)
					.expect("There are enough withdrawing_shares; qed.");
			if let Some(pid) = VaultAccountAssignments::<T>::get(userid.clone()) {
				let mut vault_info =
					ensure_vault::<T>(pid).expect("get vault should success: qed.");
				vault_info.basepool.free_stake += reduced;

				basepool::pallet::Pools::<T>::insert(
					pid,
					PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(vault_info.clone()),
				);
			} else {
				pallet_assets::Pallet::<T>::mint_into(
					<T as pawnshop::Config>::PPhaAssetId::get(),
					&userid,
					reduced + dust,
				)
				.expect("mint asset should not fail");
				Self::deposit_event(Event::<T>::Withdrawal {
					pid: pool_info.pid,
					user: userid,
					amount: reduced,
					shares: withdrawn_shares,
				});
			}
		}

		/// Tries to fulfill the withdraw queue with the newly freed stake
		fn try_process_withdraw_queue(
			pool_info: &mut basepool::BasePool<T::AccountId, BalanceOf<T>>,
		) {
			// The share price shouldn't change at any point in this function. So we can calculate
			// only once at the beginning.
			let price = match pool_info.share_price() {
				Some(price) => price,
				None => return,
			};

			while basepool::is_nondust_balance(pool_info.free_stake) {
				if let Some(withdraw) = pool_info.withdraw_queue.front().cloned() {
					// Must clear the pending reward before any stake change

					let collection_id = pool_info.cid;
					let mut withdraw_nft =
						basepool::Pallet::<T>::get_nft_attr(pool_info.cid, withdraw.nft_id)
							.expect("get nftattr should always success; qed.");
					// Try to fulfill the withdraw requests as much as possible
					let free_shares = if price == fp!(0) {
						withdraw_nft.shares // 100% slashed
					} else {
						bdiv(pool_info.free_stake, &price)
					};
					// This is the shares to withdraw immedately. It should NOT contain any dust
					// because we ensure (1) `free_shares` is not dust earlier, and (2) the shares
					// in any withdraw request mustn't be dust when inserting and updating it.
					let withdrawing_shares = free_shares.min(withdraw_nft.shares);
					debug_assert!(
						basepool::is_nondust_balance(withdrawing_shares),
						"withdrawing_shares must be positive"
					);
					// Actually remove the fulfilled withdraw request. Dust in the user shares is
					// considered but it in the request is ignored.
					Self::do_withdraw_shares(
						withdrawing_shares,
						pool_info,
						&mut withdraw_nft,
						withdraw.nft_id,
						withdraw.user.clone(),
					);
					basepool::Pallet::<T>::set_nft_attr(
						pool_info.cid,
						withdraw.nft_id,
						&withdraw_nft,
					)
					.expect("set nftattr should always success; qed.");
					// Update if the withdraw is partially fulfilled, otherwise pop it out of the
					// queue
					if withdraw_nft.shares == Zero::zero()
						|| Self::maybe_remove_dust(pool_info, &withdraw_nft, withdraw.user.clone())
					{
						pool_info.withdraw_queue.pop_front();
						basepool::Pallet::<T>::burn_nft(pool_info.cid, withdraw.nft_id)
							.expect("burn nft should always success");
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

		/// Removes some dust amount from a user's account by Currency::slash.
		fn remove_dust(who: &T::AccountId, dust: BalanceOf<T>) {
			debug_assert!(dust != Zero::zero());
			if dust != Zero::zero() {
				let actual_removed = pallet_assets::Pallet::<T>::slash(
					<T as pawnshop::Config>::PPhaAssetId::get(),
					who,
					dust.clone(),
				)
				.expect("slash should success with correct amount: qed.");
				let (imbalance, _remaining) = <T as mining::Config>::Currency::slash(who, dust);
				T::OnSlashed::on_unbalanced(imbalance);
				Self::deposit_event(Event::<T>::DustRemoved {
					user: who.clone(),
					amount: actual_removed,
				});
			}
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
						worker: worker.clone(),
					});
					// To adjust the case that skip stakepool::stop_mining when call remove_worker
					// (TODO(mingxuan): should let remove_worker in stakepool call mining directly instead of stakepool -> mining -> stakepool
					// and remove this cover code.)
					if !pool.cd_workers.contains(&worker) {
						pool.cd_workers.push_back(worker.clone());
					}
				}
			});
		}

		fn maybe_settle_nft_slash(
			pool: &basepool::pallet::BasePool<T::AccountId, BalanceOf<T>>,
			nft: &mut basepool::NftAttr<BalanceOf<T>>,
			userid: T::AccountId,
		) {
			match pool.settle_nft_slash(nft) {
				// We don't slash on dust, because the share price is just unstable.
				Some(slashed) if basepool::is_nondust_balance(slashed) => {
					let actual_slashed = pallet_assets::Pallet::<T>::slash(
						<T as pawnshop::Config>::PPhaAssetId::get(),
						&userid,
						slashed.clone(),
					)
					.expect("slash should success with correct amount: qed.");
					let (imbalance, _remaining) =
						<T as mining::Config>::Currency::slash(&userid, slashed);
					T::OnSlashed::on_unbalanced(imbalance);
					// Dust is not considered because it's already merged into the slash if
					// presents.
					pallet_assets::Pallet::<T>::mint_into(
						<T as pawnshop::Config>::PPhaAssetId::get(),
						&userid,
						actual_slashed,
					)
					.expect("mint asset should not fail");
					Self::deposit_event(Event::<T>::SlashSettled {
						pid: pool.pid,
						user: userid,
						amount: actual_slashed,
					});
				}
				_ => (),
			}
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
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
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
				basepool::pallet::Pools::<T>::insert(
					pid,
					PoolProxy::<T::AccountId, BalanceOf<T>>::StakePool(pool_info.clone()),
				);
			}
		}
	}

	impl<T: Config> mining::OnUnbound for Pallet<T>
	where
		BalanceOf<T>: FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
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
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		fn on_stopped(worker: &WorkerPublicKey, orig_stake: BalanceOf<T>, slashed: BalanceOf<T>) {}
	}

	fn pool_sub_account<T>(pid: u64, pubkey: &WorkerPublicKey) -> T
	where
		T: Encode + Decode,
	{
		let hash = crate::hashing::blake2_256(&(pid, pubkey).encode());
		// stake pool miner
		(b"spm/", hash)
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed")
	}

	fn vault_staker_account<T>(pid: u64, owner: T) -> T
	where
		T: Encode + Decode,
	{
		let hash = crate::hashing::blake2_256(&(pid, owner).encode());
		// stake pool miner
		(b"vault/", hash)
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed")
	}

	#[cfg(test)]
	mod test {
		use crate::basepool;
		use crate::poolproxy::*;
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
			Balances, PhalaBasePool, PhalaMining, PhalaRegistry, PhalaStakePool, System, Timestamp,
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
						TestEvent::Uniques(pallet_uniques::Event::Created {
							collection: 0,
							creator: _,
							owner: _
						}),
						TestEvent::RmrkCore(pallet_rmrk_core::Event::CollectionCreated {
							issuer: _,
							collection_id: 0
						}),
						TestEvent::PhalaStakePool(Event::PoolCreated { owner: 1, pid: 0 }),
						TestEvent::Uniques(pallet_uniques::Event::Created {
							collection: 1,
							creator: _,
							owner: _
						}),
						TestEvent::RmrkCore(pallet_rmrk_core::Event::CollectionCreated {
							issuer: _,
							collection_id: 1
						}),
						TestEvent::PhalaStakePool(Event::PoolCreated { owner: 1, pid: 1 }),
					]
				);
				assert_eq!(
					basepool::Pools::<Test>::get(0),
					Some(PoolProxy::<u64, Balance>::StakePool(StakePool::<
						u64,
						Balance,
					> {
						basepool: basepool::BasePool {
							pid: 0,
							owner: 1,
							total_shares: 0,
							total_value: 0,
							free_stake: 0,
							withdraw_queue: VecDeque::new(),
							value_subscribers: VecDeque::new(),
							cid: 0,
						},
						payout_commission: None,
						owner_reward: 0,
						cap: None,
						workers: vec![],
						cd_workers: vec![],
					})),
				);
				assert_eq!(basepool::PoolCount::<Test>::get(), 2);
			});
		}

		#[test]
		fn test_create_vault() {
			// Check this fixed: <https://github.com/Phala-Network/phala-blockchain/issues/285>
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(PhalaStakePool::create_vault(Origin::signed(1)));
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				PhalaStakePool::on_finalize(1);
				assert_matches!(
					take_events().as_slice(),
					[
						TestEvent::Uniques(pallet_uniques::Event::Created {
							collection: 0,
							creator: _,
							owner: _
						}),
						TestEvent::RmrkCore(pallet_rmrk_core::Event::CollectionCreated {
							issuer: _,
							collection_id: 0
						}),
						TestEvent::PhalaStakePool(Event::PoolCreated { owner: 1, pid: 0 }),
						TestEvent::Uniques(pallet_uniques::Event::Created {
							collection: 1,
							creator: _,
							owner: _
						}),
						TestEvent::RmrkCore(pallet_rmrk_core::Event::CollectionCreated {
							issuer: _,
							collection_id: 1
						}),
						TestEvent::PhalaStakePool(Event::PoolCreated { owner: 1, pid: 1 }),
					]
				);
				assert_eq!(
					basepool::Pools::<Test>::get(0),
					Some(PoolProxy::<u64, Balance>::Vault(Vault::<u64, Balance> {
						basepool: basepool::BasePool {
							pid: 0,
							owner: 1,
							total_shares: 0,
							total_value: 0,
							free_stake: 0,
							withdraw_queue: VecDeque::new(),
							value_subscribers: VecDeque::new(),
							cid: 0,
						},
						pool_account_id: 3899606504431772022,
						last_share_price_checkpoint: 0,
						delta_price_ratio: None,
						owner_shares: 0,
						invest_pools: vec![],
					})),
				);
				assert_eq!(basepool::PoolCount::<Test>::get(), 2);
			});
		}

		#[test]
		fn test_mint_nft() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
				let pool_info = ensure_stake_pool::<Test>(0).unwrap();
				assert_ok!(PhalaBasePool::mint_nft(
					pool_info.basepool.cid,
					1,
					1000 * DOLLARS,
					1000 * DOLLARS,
				));

				assert_ok!(PhalaBasePool::get_nft_attr(pool_info.basepool.cid, 0));
				let nft_attr = PhalaBasePool::get_nft_attr(pool_info.basepool.cid, 0).unwrap();
				assert_eq!(nft_attr.shares, 1000 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 1000 * DOLLARS);
			});
		}

		#[test]
		fn test_merge_or_init_nft() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
				let pool_info = ensure_stake_pool::<Test>(0).unwrap();
				assert_ok!(PhalaBasePool::mint_nft(
					pool_info.basepool.cid,
					1,
					1000 * DOLLARS,
					1000 * DOLLARS,
				));
				assert_ok!(PhalaBasePool::mint_nft(
					pool_info.basepool.cid,
					1,
					2000 * DOLLARS,
					2000 * DOLLARS,
				));
				let nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				assert_eq!(nftid_arr.len(), 2);
				assert_ok!(PhalaBasePool::merge_or_init_nft_for_staker(
					pool_info.basepool.cid,
					1
				));
				let nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				assert_eq!(nftid_arr.len(), 1);
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool_info.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(nft_attr.shares, 3000 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 3000 * DOLLARS);
				assert_ok!(PhalaBasePool::merge_or_init_nft_for_staker(
					pool_info.basepool.cid,
					2
				));
				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
					nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
				});
				assert_eq!(nftid_arr.len(), 1);
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool_info.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(nft_attr.shares, 0 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 0 * DOLLARS);
			});
		}

		#[test]
		fn test_set_nft_attr() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
				let pool_info = ensure_stake_pool::<Test>(0).unwrap();
				assert_ok!(PhalaBasePool::mint_nft(
					pool_info.basepool.cid,
					1,
					1000 * DOLLARS,
					1000 * DOLLARS,
				));
				let mut nft_attr = PhalaBasePool::get_nft_attr(pool_info.basepool.cid, 0).unwrap();
				nft_attr.shares = 5000 * DOLLARS;
				nft_attr.set_stake(5000 * DOLLARS);
				assert_ok!(PhalaBasePool::set_nft_attr(
					pool_info.basepool.cid,
					0,
					&nft_attr,
				));
				let nft_attr = PhalaBasePool::get_nft_attr(pool_info.basepool.cid, 0).unwrap();
				assert_eq!(nft_attr.shares, 5000 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 5000 * DOLLARS);
			});
		}

		#[test]
		fn test_remove_stake_from_nft() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					50 * DOLLARS
				));
				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
					nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
				});
				assert_eq!(nftid_arr.len(), 1);
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				let mut nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(pool.basepool.share_price().unwrap(), 1);
				match PhalaBasePool::remove_stake_from_nft(
					&mut pool.basepool,
					40 * DOLLARS,
					&mut nft_attr,
				) {
					Some((amout, user_dust, removed_shares)) => return,
					_ => panic!(),
				}
			});
		}

		#[test]
		fn test_contibute() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					50 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					50 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					30 * DOLLARS
				));

				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
					nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
				});
				assert_eq!(nftid_arr.len(), 1);
				let pool = ensure_stake_pool::<Test>(0).unwrap();
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(nft_attr.shares, 80 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 80 * DOLLARS);
				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
					nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
				});
				assert_eq!(nftid_arr.len(), 1);
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(nft_attr.shares, 50 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 50 * DOLLARS);
			});
		}

		#[test]
		fn test_contibute_to_vault() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_vault(1); // pid = 0
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(1),
					0,
					50 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(2),
					0,
					50 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(1),
					0,
					30 * DOLLARS
				));

				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
					nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
				});
				assert_eq!(nftid_arr.len(), 1);
				let pool = ensure_vault::<Test>(0).unwrap();
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(nft_attr.shares, 80 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 80 * DOLLARS);
				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
					nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
				});
				assert_eq!(nftid_arr.len(), 1);
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(nft_attr.shares, 50 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 50 * DOLLARS);
				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(vault_info.basepool.total_value, 130 * DOLLARS);
				assert_eq!(vault_info.basepool.total_shares, 130 * DOLLARS);
				assert_eq!(vault_info.basepool.free_stake, 130 * DOLLARS);
			});
		}

		#[test]
		fn test_vault_investment() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_vault(3); // pid = 0
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 1
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(3),
					0,
					50 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(2),
					0,
					50 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(3),
					0,
					30 * DOLLARS
				));

				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(vault_info.basepool.total_value, 130 * DOLLARS);
				assert_eq!(vault_info.basepool.total_shares, 130 * DOLLARS);
				assert_eq!(vault_info.basepool.free_stake, 130 * DOLLARS);

				assert_noop!(
					PhalaStakePool::vault_investment(Origin::signed(2), 0, 1, 10 * DOLLARS),
					Error::<Test>::UnauthorizedPoolOwner
				);
				assert_ok!(PhalaStakePool::vault_investment(
					Origin::signed(3),
					0,
					1,
					50 * DOLLARS
				));
				let stakepool_info = ensure_stake_pool::<Test>(1).unwrap();
				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(stakepool_info.basepool.cid)
						.collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(stakepool_info.basepool.cid, x)
						.unwrap();
					nft.owner
						== rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
							vault_info.pool_account_id,
						)
				});
				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(nftid_arr.len(), 1);
				let nft_attr =
					PhalaBasePool::get_nft_attr(stakepool_info.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(nft_attr.shares, 50 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 50 * DOLLARS);
				assert_eq!(vault_info.basepool.total_value, 130 * DOLLARS);
				assert_eq!(vault_info.basepool.total_shares, 130 * DOLLARS);
				assert_eq!(vault_info.basepool.free_stake, 80 * DOLLARS);
				assert_eq!(stakepool_info.basepool.total_value, 50 * DOLLARS);
				assert_eq!(stakepool_info.basepool.total_shares, 50 * DOLLARS);
				assert_eq!(stakepool_info.basepool.free_stake, 50 * DOLLARS);
			});
		}

		#[test]
		fn test_withdraw_from_vault() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_vault(3); // pid = 0
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 1
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(3),
					0,
					50 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(2),
					0,
					50 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(3),
					0,
					30 * DOLLARS
				));
				assert_ok!(PhalaStakePool::vault_investment(
					Origin::signed(3),
					0,
					1,
					80 * DOLLARS
				));
				assert_ok!(PhalaStakePool::withdraw_from_vault(
					Origin::signed(3),
					0,
					80 * DOLLARS
				));
				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(vault_info.basepool.total_value, 80 * DOLLARS);
				assert_eq!(vault_info.basepool.total_shares, 80 * DOLLARS);
				assert_eq!(vault_info.basepool.free_stake, 0 * DOLLARS);
				assert_eq!(vault_info.basepool.withdraw_queue.len(), 1);
				let nft_attr = PhalaBasePool::get_nft_attr(
					vault_info.basepool.cid,
					vault_info.basepool.withdraw_queue[0].nft_id,
				)
				.unwrap();
				assert_eq!(nft_attr.shares, 30 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 30 * DOLLARS);
			});
		}

		#[test]
		fn test_vault_withdraw() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_vault(3); // pid = 0
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 1
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(3),
					0,
					500 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(2),
					0,
					500 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(3),
					0,
					300 * DOLLARS
				));
				assert_ok!(PhalaStakePool::vault_investment(
					Origin::signed(3),
					0,
					1,
					800 * DOLLARS
				));
				let stakepool_info = ensure_stake_pool::<Test>(1).unwrap();
				assert_eq!(stakepool_info.basepool.total_value, 800 * DOLLARS);
				assert_eq!(stakepool_info.basepool.total_shares, 800 * DOLLARS);
				assert_eq!(stakepool_info.basepool.free_stake, 800 * DOLLARS);
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					1,
					worker_pubkey(1),
					400 * DOLLARS
				));
				assert_ok!(PhalaStakePool::vault_withdraw(
					Origin::signed(3),
					1,
					0,
					600 * DOLLARS
				));
				let stakepool_info = ensure_stake_pool::<Test>(1).unwrap();
				assert_eq!(stakepool_info.basepool.total_value, 400 * DOLLARS);
				assert_eq!(stakepool_info.basepool.total_shares, 400 * DOLLARS);
				assert_eq!(stakepool_info.basepool.free_stake, 0 * DOLLARS);
				assert_eq!(stakepool_info.basepool.withdraw_queue.len(), 1);
				let nft_attr = PhalaBasePool::get_nft_attr(
					stakepool_info.basepool.cid,
					stakepool_info.basepool.withdraw_queue[0].nft_id,
				)
				.unwrap();
				assert_eq!(nft_attr.shares, 200 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 200 * DOLLARS);
				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(vault_info.basepool.total_value, 1300 * DOLLARS);
				assert_eq!(vault_info.basepool.total_shares, 1300 * DOLLARS);
				assert_eq!(vault_info.basepool.free_stake, 900 * DOLLARS);
			});
		}

		#[test]
		fn test_on_reward_for_vault() {
			use crate::mining::pallet::OnReward;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_vault(3); // pid = 0
				setup_stake_pool_with_workers(1, &[1]);
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(3),
					0,
					1000 * DOLLARS
				));
				assert_ok!(PhalaStakePool::vault_investment(
					Origin::signed(3),
					0,
					1,
					500 * DOLLARS
				));
				// Staker2 contribute 1000 PHA and start mining
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					1,
					500 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					1,
					worker_pubkey(1),
					500 * DOLLARS
				));
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(1000u32).to_bits(),
					treasury: 0,
				}]);
				let mut pool = ensure_stake_pool::<Test>(1).unwrap();
				assert_eq!(pool.basepool.free_stake, 1500 * DOLLARS);
				assert_eq!(pool.basepool.total_value, 2000 * DOLLARS);
				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(vault_info.basepool.total_value, 1500 * DOLLARS);
				assert_eq!(vault_info.basepool.free_stake, 500 * DOLLARS);
				assert_eq!(vault_info.basepool.total_shares, 1000 * DOLLARS);
			});
		}

		#[test]
		fn test_vault_owner_shares() {
			use crate::mining::pallet::OnReward;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_vault(3); // pid = 0
				assert_ok!(PhalaStakePool::set_vault_payout_pref(
					Origin::signed(3),
					0,
					Permill::from_percent(50)
				));
				setup_stake_pool_with_workers(1, &[1]);
				assert_ok!(PhalaStakePool::contribute_to_vault(
					Origin::signed(3),
					0,
					1000 * DOLLARS
				));
				assert_ok!(PhalaStakePool::vault_investment(
					Origin::signed(3),
					0,
					1,
					500 * DOLLARS
				));
				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(
					vault_info.delta_price_ratio.unwrap(),
					Permill::from_percent(50)
				);
				assert_ok!(PhalaStakePool::maybe_gain_owner_shares(
					Origin::signed(3),
					0
				));
				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(vault_info.owner_shares, 0);
				// Staker2 contribute 1000 PHA and start mining
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					1,
					500 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					1,
					worker_pubkey(1),
					500 * DOLLARS
				));
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(1000u32).to_bits(),
					treasury: 0,
				}]);
				let mut pool = ensure_stake_pool::<Test>(1).unwrap();
				assert_eq!(pool.basepool.free_stake, 1500 * DOLLARS);
				assert_eq!(pool.basepool.total_value, 2000 * DOLLARS);
				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(vault_info.basepool.total_value, 1500 * DOLLARS);
				assert_eq!(vault_info.basepool.free_stake, 500 * DOLLARS);
				assert_eq!(vault_info.basepool.total_shares, 1000 * DOLLARS);
				assert_ok!(PhalaStakePool::maybe_gain_owner_shares(
					Origin::signed(3),
					0
				));
				let vault_info = ensure_vault::<Test>(0).unwrap();
				assert_eq!(vault_info.owner_shares, 200 * DOLLARS);
				assert_eq!(vault_info.basepool.total_shares, 1200 * DOLLARS);
			});
		}

		#[test]
		fn test_withdraw() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
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
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					800 * DOLLARS
				));
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				let mut item = pool
					.basepool
					.withdraw_queue
					.clone()
					.into_iter()
					.find(|x| x.user == 2);
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, item.unwrap().nft_id).unwrap();
				assert_eq!(nft_attr.shares, 300 * DOLLARS);
				assert_eq!(nft_attr.get_stake(), 300 * DOLLARS);
				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
					nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
				});
				let user_nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(user_nft_attr.shares, 200 * DOLLARS);
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(3),
					0,
					1000 * DOLLARS
				));
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool.basepool.withdraw_queue.len(), 0);
				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
					nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(3)
				});
				assert_eq!(nftid_arr.len(), 1);
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(nft_attr.shares, 1000 * DOLLARS);
				assert_eq!(pool.basepool.total_value, 1200 * DOLLARS);
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(3),
					0,
					900 * DOLLARS
				));
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				let mut item = pool
					.basepool
					.withdraw_queue
					.clone()
					.into_iter()
					.find(|x| x.user == 3);
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, item.unwrap().nft_id).unwrap();
				assert_eq!(nft_attr.shares, 200 * DOLLARS);
				assert_ok!(PhalaStakePool::withdraw(Origin::signed(3), 0, 50 * DOLLARS));
				let mut nftid_arr: Vec<NftId> =
					pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
				nftid_arr.retain(|x| {
					let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
					nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(3)
				});
				let user_nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, nftid_arr[0]).unwrap();
				assert_eq!(user_nft_attr.shares, 250 * DOLLARS);
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				let mut item = pool
					.basepool
					.withdraw_queue
					.clone()
					.into_iter()
					.find(|x| x.user == 3);
				let nft_attr =
					PhalaBasePool::get_nft_attr(pool.basepool.cid, item.unwrap().nft_id).unwrap();
				assert_eq!(nft_attr.shares, 50 * DOLLARS);
			});
		}

		#[test]
		fn test_set_pool_description() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_stake_pool_with_workers(1, &[1]);
				let str_hello: BoundedVec<u8, DescMaxLen> =
					("hello").as_bytes().to_vec().try_into().unwrap();
				assert_ok!(PhalaStakePool::set_pool_description(
					Origin::signed(1),
					0,
					str_hello.clone(),
				));
				let list = PhalaStakePool::pool_descriptions(0).unwrap();
				assert_eq!(list, str_hello);
				let str_bye: BoundedVec<u8, DescMaxLen> =
					("bye").as_bytes().to_vec().try_into().unwrap();
				assert_noop!(
					PhalaStakePool::set_pool_description(Origin::signed(2), 0, str_bye,),
					Error::<Test>::UnauthorizedPoolOwner
				);
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
					basepool::Error::<Test>::PoolDoesNotExist
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
				// Cannot start mining without a bound worker
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
				setup_stake_pool_with_workers(1, &[1]); // pid = 0
				setup_stake_pool_with_workers(2, &[2]); // pid = 1
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
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
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
				let mut pool = ensure_stake_pool::<Test>(1).unwrap();
				assert_eq!(pool.workers.contains(&worker_pubkey(2)), false);
				// Check the mining is stopped
				let miner = PhalaMining::miners(&sub_account).unwrap();
				assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
			});
		}

		#[test]
		fn test_stop_mining() {
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				// Cannot start mining without a bound worker
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
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					100 * DOLLARS
				));
				assert_ok!(PhalaStakePool::stop_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
				));
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool.cd_workers, [worker_pubkey(1)]);
			});
		}

		#[test]
		fn test_for_cdworkers() {
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(PhalaStakePool::create(Origin::signed(1)));
				// Cannot start mining without a bound worker
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
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					100 * DOLLARS
				));
				assert_ok!(PhalaStakePool::remove_worker(
					Origin::signed(1),
					0,
					worker_pubkey(1),
				));
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool.cd_workers, [worker_pubkey(1)]);
				elapse_cool_down();
				assert_ok!(PhalaStakePool::reclaim_pool_worker(
					Origin::signed(1),
					0,
					worker_pubkey(1),
				));
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool.cd_workers, []);
			});
		}

		#[test]
		fn test_check_and_maybe_force_withdraw() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(2);
				setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
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
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					800 * DOLLARS
				));
				elapse_seconds(864000);
				assert_ok!(PhalaStakePool::stop_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
				));
				assert_ok!(PhalaStakePool::check_and_maybe_force_withdraw(
					Origin::signed(3),
					0
				));
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool.basepool.free_stake, 0 * DOLLARS);
				assert_eq!(pool.cd_workers, [worker_pubkey(1)]);
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(2),
					0,
					500 * DOLLARS
				));
				elapse_seconds(864000);
				assert_ok!(PhalaStakePool::check_and_maybe_force_withdraw(
					Origin::signed(3),
					0
				));
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool.cd_workers, [worker_pubkey(1), worker_pubkey(2)]);
			});
		}

		#[test]
		fn test_on_reward() {
			use crate::mining::pallet::OnReward;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_stake_pool_with_workers(1, &[1]);

				assert_ok!(PhalaStakePool::set_payout_pref(
					Origin::signed(1),
					0,
					Permill::from_percent(50)
				));
				// Staker2 contribute 1000 PHA and start mining
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					2000 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					1000 * DOLLARS
				));
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(2000u32).to_bits(),
					treasury: 0,
				}]);
				let mut pool = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool.owner_reward, 1000 * DOLLARS);
				assert_eq!(pool.basepool.free_stake, 2000 * DOLLARS);
				assert_eq!(pool.basepool.total_value, 3000 * DOLLARS);
			});
		}

		#[test]
		fn test_pool_cap() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_stake_pool_with_workers(1, &[1]); // pid = 0

				assert_eq!(ensure_stake_pool::<Test>(0).unwrap().cap, None);
				// Pool existence
				assert_noop!(
					PhalaStakePool::set_cap(Origin::signed(2), 100, 1),
					basepool::Error::<Test>::PoolDoesNotExist,
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
					ensure_stake_pool::<Test>(0).unwrap().cap,
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

				// Can stake exceed the cap to swap the withdrawing stake out, as long as the cap
				// can be maintained after the contribution
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					1000 * DOLLARS
				));
				assert_ok!(PhalaStakePool::withdraw(
					Origin::signed(1),
					0,
					1000 * DOLLARS
				));
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(2), 0, 1001 * DOLLARS),
					Error::<Test>::StakeExceedsCapacity
				);
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					1000 * DOLLARS
				));
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
					ensure_stake_pool::<Test>(0).unwrap().basepool.total_value,
					11 * DOLLARS
				);
				assert_eq!(
					ensure_stake_pool::<Test>(1).unwrap().basepool.total_value,
					1100 * DOLLARS
				);

				// Pool existence
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 100, 1 * DOLLARS),
					basepool::Error::<Test>::PoolDoesNotExist
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
		fn test_claim_owner_rewards() {
			use crate::mining::pallet::OnReward;
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_stake_pool_with_workers(1, &[1]); // pid = 0
				assert_ok!(PhalaStakePool::set_payout_pref(
					Origin::signed(1),
					0,
					Permill::from_percent(50)
				));
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
				PhalaStakePool::on_reward(&vec![SettleInfo {
					pubkey: worker_pubkey(1),
					v: FixedPoint::from_num(1u32).to_bits(),
					payout: FixedPoint::from_num(1000u32).to_bits(),
					treasury: 0,
				}]);
				let pool = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool.owner_reward, 500 * DOLLARS);
				assert_ok!(PhalaStakePool::claim_owner_rewards(Origin::signed(1), 0, 1));
				let pool = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool.owner_reward, 0 * DOLLARS);
			});
		}
		#[test]
		fn test_staker_whitelist() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_stake_pool_with_workers(1, &[1]);

				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					40 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					40 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(3),
					0,
					40 * DOLLARS
				));
				assert_ok!(PhalaStakePool::add_staker_to_whitelist(
					Origin::signed(1),
					0,
					2,
				));
				let whitelist = PhalaStakePool::pool_whitelist(0).unwrap();
				assert_eq!(whitelist, [2]);
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					0,
					10 * DOLLARS
				));
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					40 * DOLLARS
				));
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(3), 0, 40 * DOLLARS),
					Error::<Test>::NotInContributeWhitelist
				);
				assert_ok!(PhalaStakePool::add_staker_to_whitelist(
					Origin::signed(1),
					0,
					3,
				));
				let whitelist = PhalaStakePool::pool_whitelist(0).unwrap();
				assert_eq!(whitelist, [2, 3]);
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(3),
					0,
					20 * DOLLARS,
				));
				PhalaStakePool::remove_staker_from_whitelist(Origin::signed(1), 0, 2);
				let whitelist = PhalaStakePool::pool_whitelist(0).unwrap();
				assert_eq!(whitelist, [3]);
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(2), 0, 20 * DOLLARS,),
					Error::<Test>::NotInContributeWhitelist
				);
				PhalaStakePool::remove_staker_from_whitelist(Origin::signed(1), 0, 3);
				assert!(PhalaStakePool::pool_whitelist(0).is_none());
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(3),
					0,
					20 * DOLLARS,
				));
			});
		}

		#[test]
		fn issue_388_double_stake() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_stake_pool_with_workers(1, &[1]);

				let balance = Balances::usable_balance(&1);
				assert_ok!(PhalaStakePool::contribute(Origin::signed(1), 0, balance));
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 0, balance),
					Error::<Test>::InsufficientBalance
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
					[TestEvent::PhalaStakePool(Event::PoolCommissionSet {
						pid: 0,
						commission: 1000_000u32 * 50 / 100
					})]
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
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(1),
					1,
					300 * DOLLARS
				));
				assert_eq!(
					ensure_stake_pool::<Test>(0).unwrap().basepool.total_value,
					100 * DOLLARS
				);

				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					200 * DOLLARS
				));
				assert_eq!(
					ensure_stake_pool::<Test>(0).unwrap().basepool.total_value,
					300 * DOLLARS
				);
				// Shouldn't exceed the pool cap
				assert_noop!(
					PhalaStakePool::contribute(Origin::signed(1), 0, 100 * DOLLARS),
					Error::<Test>::StakeExceedsCapacity
				);
				// Start mining on pool0 (stake 100 for worker1, 100 for worker2)
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
				// 90% stake get returned from pool 0
				let pool0 = ensure_stake_pool::<Test>(0).unwrap();
				// TODO(hangyin): enable when stake is not skipped
				// assert_eq!(pool0.free_stake, 189_999999999999);
				assert_eq!(pool0.basepool.free_stake, 200000000000000);
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
		fn issue500_should_not_restart_worker_in_cool_down() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_workers(1);
				setup_stake_pool_with_workers(1, &[1]); // pid=0
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
		fn subaccount_preimage() {
			new_test_ext().execute_with(|| {
				setup_workers(1);
				setup_stake_pool_with_workers(1, &[1]); // pid=0

				let subaccount: u64 = pool_sub_account(0, &worker_pubkey(1));
				let preimage = SubAccountPreimages::<Test>::get(subaccount);
				assert_eq!(preimage, Some((0, worker_pubkey(1))));
			});
		}

		#[test]
		fn restart_mining_should_work() {
			new_test_ext().execute_with(|| {
				setup_workers(1);
				setup_stake_pool_with_workers(1, &[1]); // pid=0
				assert_ok!(PhalaStakePool::contribute(
					Origin::signed(2),
					0,
					2000 * DOLLARS
				));
				assert_ok!(PhalaStakePool::start_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					1500 * DOLLARS
				));
				// Bad cases
				assert_noop!(
					PhalaStakePool::restart_mining(
						Origin::signed(1),
						0,
						worker_pubkey(1),
						500 * DOLLARS
					),
					Error::<Test>::CannotRestartWithLessStake
				);
				assert_noop!(
					PhalaStakePool::restart_mining(
						Origin::signed(1),
						0,
						worker_pubkey(1),
						1500 * DOLLARS
					),
					Error::<Test>::CannotRestartWithLessStake
				);
				// Happy path
				let pool0 = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool0.basepool.free_stake, 500 * DOLLARS);
				assert_ok!(PhalaStakePool::restart_mining(
					Origin::signed(1),
					0,
					worker_pubkey(1),
					1501 * DOLLARS
				));
				let pool0 = ensure_stake_pool::<Test>(0).unwrap();
				assert_eq!(pool0.basepool.free_stake, 499 * DOLLARS);
			});
		}

		fn setup_stake_pool_with_workers(owner: u64, workers: &[u8]) -> u64 {
			let pid = PhalaBasePool::pool_count();
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

		fn setup_vault(owner: u64) -> u64 {
			let pid = PhalaBasePool::pool_count();
			assert_ok!(PhalaStakePool::create_vault(Origin::signed(owner)));
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
