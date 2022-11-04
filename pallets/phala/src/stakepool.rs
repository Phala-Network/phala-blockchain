//! Pool for collaboratively mining staking

pub use self::pallet::*;

use crate::BalanceOf;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use crate::compute::{computation, basepool};
	use crate::registry;
	use crate::utils::fixed_point::CodecFixedPoint;

	use super::BalanceOf;
	use frame_support::{
		pallet_prelude::*,
		traits::{LockableCurrency, StorageVersion},
	};
	use scale_info::TypeInfo;
	use sp_runtime::Permill;
	use sp_std::{collections::vec_deque::VecDeque, prelude::*};

	use phala_types::WorkerPublicKey;

	pub struct DescMaxLen;

	impl Get<u32> for DescMaxLen {
		fn get() -> u32 {
			4400
		}
	}
	/// The functions to manage user's native currency lock in the Balances pallet
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
	pub trait Config: frame_system::Config + registry::Config + computation::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
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
		StorageMap<_, Twox64Concat, u64, basepool::pallet::DescStr>;

	#[pallet::event]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	/// The state of a pool
	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Default, RuntimeDebug)]
	pub struct PoolInfo<AccountId, Balance> {
		/// Pool ID
		pub pid: u64,
		/// The owner of the pool
		pub owner: AccountId,
		/// The commission the pool owner takes
		///
		/// For example, 10% commission means 10% of the miner reward goes to the pool owner, and
		/// the remaining 90% is distributed to the contributors. Setting to `None` means a
		/// commission of 0%.
		pub payout_commission: Option<Permill>,
		/// Claimable owner reward
		///
		/// Whenver a miner gets some reward, the commission the pool taken goes to here. The owner
		/// can claim their reward at any time.
		pub owner_reward: Balance,
		/// The hard capacity of the pool
		///
		/// When it's set, the totals stake a pool can receive will not exceed this capacity.
		pub cap: Option<Balance>,
		/// The reward [accumulator](crate::utils::accumulator)
		///
		/// An individual user's reward is tracked by [`reward_acc`](PoolInfo::reward_acc), their
		/// [`shares`](UserStakeInfo::shares) and the [`reward_debt`](UserStakeInfo::reward_debt).
		pub reward_acc: CodecFixedPoint,
		/// Total shares
		///
		/// It tracks the total number of shared of all the contributors. Guaranteed to be
		/// non-dust.
		pub total_shares: Balance,
		/// Total stake
		///
		/// It tracks the total number of the stake the pool received. Guaranteed to be non-dust.
		pub total_stake: Balance,
		/// Total free stake
		///
		/// It tracks the total free stake (not used by any miner) in the pool. Can be dust.
		pub free_stake: Balance,
		/// Releasing stake
		///
		/// It tracks the stake that will be unlocked in the future. It's the sum of all the
		/// cooling down miners' remaining stake.
		pub releasing_stake: Balance,
		/// Bound workers
		pub workers: Vec<WorkerPublicKey>,
		/// The queue of withdraw requests
		pub withdraw_queue: VecDeque<WithdrawInfo<AccountId, Balance>>,
	}

	/// A user's staking info
	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct UserStakeInfo<AccountId, Balance> {
		/// User's address
		pub user: AccountId,
		/// The actual locked stake in the pool
		pub locked: Balance,
		/// The share in the pool
		///
		/// Guaranteed to be non-dust. Invariant must hold:
		/// - `StakePools[pid].total_stake == sum(PoolStakers[(pid, user)].shares)`
		pub shares: Balance,
		/// Resolved claimable rewards
		///
		/// It's accumulated by resolving "pending stake" from the reward
		/// [accumulator](crate::utils::accumulator).
		pub available_rewards: Balance,
		/// The debt of a user's stake
		///
		/// It's subject to the pool reward [accumulator](crate::utils::accumulator).
		pub reward_debt: Balance,
	}

	/// A withdraw request, usually stored in the withdrawal queue
	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct WithdrawInfo<AccountId, Balance> {
		/// The withdrawal requester
		pub user: AccountId,
		/// The shares to withdraw. Cannot be dust.
		pub shares: Balance,
		/// The start time of the request
		pub start_time: u64,
	}
}
