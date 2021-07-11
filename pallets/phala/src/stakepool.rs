pub use self::pallet::*;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use crate::mining;
	use crate::registry;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, EnsureOrigin},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	use phala_types::WorkerPublicKey;
	use sp_runtime::{
		traits::{AccountIdConversion, Saturating, Zero},
		Permill,
	};
	use sp_std::vec;
	use sp_std::vec::Vec;

	const STAKEPOOL_PALLETID: PalletId = PalletId(*b"phala/sp");

	#[pallet::config]
	pub trait Config: frame_system::Config + registry::Config + mining::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Meh. [n]
		Meh(u32),
		/// [owner, pid]
		PoolCreated(T::AccountId, u64),
		/// [pid, commission]
		PoolCommissionSetted(u64, Permill),
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
		WorkerHasAdded,
		UnauthorizedOperator,
		UnauthorizedPoolOwner,
		InvalidPayoutPerf,
		PoolNotExist,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new stake pool
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let pid = PoolCount::<T>::get();
			PoolCount::<T>::mutate(|id| *id = pid + 1);
			MiningPools::<T>::insert(
				pid + 1,
				PoolInfo {
					pid: pid + 1,
					owner: owner.clone(),
					state: PoolState::default(),
					payout_commission: None,
					pool_acc: Zero::zero(),
					total_staked: Zero::zero(),
					extra_staked: Zero::zero(),
					workers: vec![],
				},
			);
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

			// make sure the worker has registered
			ensure!(
				registry::Worker::<T>::contains_key(&pubkey),
				Error::<T>::WorkerNotRegistered
			);
			// check the wheather the owner was bounded as operator
			let worker_info = registry::Worker::<T>::get(&pubkey).unwrap();
			ensure!(
				worker_info.operator.unwrap() == owner.clone(),
				Error::<T>::UnauthorizedOperator
			);

			// origin must be owner of pool
			let mut pool_info = Self::mining_pools(pid).ok_or(Error::<T>::PoolNotExist)?;
			ensure!(
				pool_info.owner == owner.clone(),
				Error::<T>::UnauthorizedPoolOwner
			);
			// make sure worker has not been not added
			let mut workers = pool_info.workers;
			ensure!(workers.contains(&pubkey), Error::<T>::WorkerHasAdded);

			// generate miner account
			let miner: T::AccountId =
				STAKEPOOL_PALLETID.into_sub_account((pid, owner, pubkey.clone()).encode());

			// bind worker with minner
			<mining::pallet::Pallet<T>>::bind(
				frame_system::RawOrigin::Signed(Self::account_id()).into(),
				miner.clone(),
				pubkey.clone(),
			)?;

			// update worker vector
			workers.push(pubkey.clone());
			pool_info.workers = workers;
			MiningPools::<T>::insert(&pid, &pool_info);
			Self::deposit_event(Event::<T>::PoolWorkerAdded(pid, pubkey));

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
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::weight(0)]
		pub fn set_cap(origin: OriginFor<T>, id: u64, cap: BalanceOf<T>) -> DispatchResult {
			panic!("unimplemented")
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

			let pool_info = Self::mining_pools(pid).ok_or(Error::<T>::PoolNotExist)?;
			// origin must be owner of pool
			ensure!(
				pool_info.owner == owner.clone(),
				Error::<T>::UnauthorizedPoolOwner
			);
			// make sure pool status is not mining
			ensure!(
				pool_info.state != PoolState::Mining,
				Error::<T>::UnauthorizedPoolOwner
			);

			if let Some(commission) = payout_commission {
				MiningPools::<T>::mutate(&pid, |pool| {
					if let Some(pool) = pool {
						pool.payout_commission = Some(commission);
					}
				});
				Self::deposit_event(Event::<T>::PoolCommissionSetted(pid, commission));
			}

			Ok(())
		}

		/// Change the payout perference of a stake pool
		///
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::weight(0)]
		pub fn claim_reward(
			origin: OriginFor<T>,
			pool_id: u64,
			target: T::AccountId,
		) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Sets target miners
		///
		/// Requires:
		/// 1. The pool exists
		/// 2. The miners are bounded to the pool
		#[pallet::weight(0)]
		pub fn set_target_miners(
			origin: OriginFor<T>,
			pool_id: u64,
			targets: Vec<WorkerPublicKey>,
		) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Deposits some funds to a pool
		///
		/// Requires:
		/// 1. The pool exists
		/// 2. After the desposit, the pool doesn't reach the cap
		#[pallet::weight(0)]
		pub fn deposit(origin: OriginFor<T>, pool_id: u64, amount: BalanceOf<T>) -> DispatchResult {
			panic!("unimplemented")
		}

		// TODO(h4x): Should we allow cancellation of a withdraw plan?

		/// Starts a withdraw plan
		///
		/// This action will create a withdraw plan (allocating a `withdraw_id`), and store the
		/// start time of the withdraw. After the waiting time, it can be executed by calling
		/// `execute_withdraw()`.
		///
		/// Requires:
		/// 1. The sender is the owner of a certain contribution to the pool
		/// 2. `amount` mustn't be larger than owner's deposit
		#[pallet::weight(0)]
		pub fn start_withdraw(
			origin: OriginFor<T>,
			pool_id: u64,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Executes a withdraw request
		///
		/// Requires:
		/// 1. `withdraw_id` is valid and finished withdraw plan
		#[pallet::weight(0)]
		pub fn execute_withdraw(
			origin: OriginFor<T>,
			pool_id: u64,
			withdraw_id: u32,
		) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Starts a miner on behalf of the stake pool
		///
		/// Requires:
		/// 1. The miner is bounded to the pool and is in Ready state
		/// 2. The remaining stake in the pool can cover the minimal stake requried
		#[pallet::weight(0)]
		pub fn start_mining(
			origin: OriginFor<T>,
			pool_id: u64,
			worker: WorkerPublicKey,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Stops a miner on behalf of the stake pool
		///
		/// Requires:
		/// 1. There miner is bounded to the pool and is in a stoppable state
		#[pallet::weight(0)]
		pub fn stop_mining(
			origin: OriginFor<T>,
			pool_id: u64,
			worker: WorkerPublicKey,
		) -> DispatchResult {
			panic!("unimplemented")
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			STAKEPOOL_PALLETID.into_account()
		}
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub enum PoolState {
		Ready,
		Mining,
	}

	impl Default for PoolState {
		fn default() -> Self {
			PoolState::Ready
		}
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct PoolInfo<AccountId: Default, Balance> {
		pid: u64,
		owner: AccountId,
		state: PoolState,
		payout_commission: Option<Permill>,
		pool_acc: Balance,
		total_staked: Balance,
		extra_staked: Balance,
		workers: Vec<WorkerPublicKey>,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct UserStakeInfo<AccountId: Default, Balance> {
		has_deposited: bool,
		user: AccountId,
		amount: Balance,
		available_rewards: Balance,
		user_debt: Balance,
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
}
