pub use self::pallet::*;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use crate::mining;
	use crate::registry;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, EnsureOrigin, LockIdentifier, LockableCurrency, WithdrawReasons},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	use phala_types::{messaging::SettleInfo, WorkerPublicKey};
	use sp_runtime::{
		traits::{AccountIdConversion, Saturating, Zero},
		Permill, SaturatedConversion,
	};
	use sp_std::vec;
	use sp_std::vec::Vec;

	const STAKEPOOL_PALLETID: PalletId = PalletId(*b"phala/sp");
	const STAKING_ID: LockIdentifier = *b"phala/sp";

	#[pallet::config]
	pub trait Config: frame_system::Config + registry::Config + mining::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		type MinDeposit: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Mapping from pool id to PoolInfo
	#[pallet::storage]
	#[pallet::getter(fn mining_pools)]
	pub(super) type MiningPools<T: Config> =
		StorageMap<_, Twox64Concat, u64, PoolInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>;

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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Meh. [n]
		Meh(u32),
		/// [owner, pid]
		PoolCreated(T::AccountId, u64),
		/// [pid, commission]
		PoolCommissionSetted(u64, u16),
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
		PoolIsBusy,
		LessthanMinDeposit,
		InsufficientBalance,
		StakeInfoNotFound,
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
					payout_commission: Zero::zero(),
					owner_reward: Zero::zero(),
					pool_acc: Zero::zero(),
					last_reward_block: T::BlockNumber::zero(),
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
				worker_info.operator == Some(owner.clone()),
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
			NewRewards::<T>::insert(&pubkey, BalanceOf::<T>::zero());
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
			payout_commission: u16,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			ensure!(payout_commission < 1000, Error::<T>::InvalidPayoutPerf);

			let pool_info = Self::mining_pools(pid).ok_or(Error::<T>::PoolNotExist)?;
			// origin must be owner of pool
			ensure!(
				pool_info.owner == owner.clone(),
				Error::<T>::UnauthorizedPoolOwner
			);
			// make sure pool status is not mining
			ensure!(pool_info.state != PoolState::Mining, Error::<T>::PoolIsBusy);

			MiningPools::<T>::mutate(&pid, |pool| {
				if let Some(pool) = pool {
					pool.payout_commission = payout_commission;
				}
			});
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
			let pool_info = Self::mining_pools(&pid).ok_or(Error::<T>::PoolNotExist)?;

			// update pool
			Self::update_pool(pid.clone());

			// rewards belong to user
			let rewards =
				user_info.amount * pool_info.pool_acc / 10u32.pow(6).into() - user_info.user_debt;

			// send reward to user
			<T as Config>::Currency::deposit_into_existing(&who.clone(), rewards.clone())?;

			user_info.user_debt = user_info.amount * pool_info.pool_acc / 10u32.pow(6).into();
			user_info.available_rewards = Zero::zero();
			StakingInfo::<T>::insert(&info_key, &user_info);
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

			let mut pool_info = Self::mining_pools(&pid).ok_or(Error::<T>::PoolNotExist)?;
			Self::update_pool(pid.clone());

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
					&(pid.clone(), who.clone()),
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

			pool_info.total_staked = pool_info.total_staked.saturating_add(amount.clone());
			MiningPools::<T>::insert(&pid, &pool_info);
			Self::deposit_event(Event::<T>::Deposit(pid, who, amount));

			Ok(())
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

		fn update_pool(pid: u64) {
			let mut new_rewards;
			// TODO: check payout block height
			// let currentBlock = <frame_system::Pallet<T>>::block_number();
			let mut pool_info = Self::mining_pools(&pid).unwrap();

			// mining hasn't started, no rewards so far
			// TODO: update last_reward_block in on_reward
			if pool_info.last_reward_block == Zero::zero() {
				return;
			}

			new_rewards = Self::calculate_reward(pid);
			Self::reward_clear(&pool_info.workers);

			if new_rewards > Zero::zero() {
				pool_info.owner_reward = pool_info.owner_reward.saturating_add(
					new_rewards * pool_info.payout_commission.into() / 1000u32.into(),
				);

				new_rewards =
					new_rewards * (1000 - pool_info.payout_commission).into() / 1000u32.into();
				pool_info.pool_acc = pool_info
					.pool_acc
					.saturating_add(new_rewards * 10u32.pow(6).into() / pool_info.total_staked);
				MiningPools::<T>::insert(&pid, &pool_info);
			}
		}

		fn calculate_reward(pid: u64) -> BalanceOf<T> {
			let pool_info = Self::mining_pools(&pid).unwrap();
			let mut pool_new_rewards: BalanceOf<T> = Zero::zero();
			for worker in pool_info.workers {
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
	pub struct PoolInfo<AccountId: Default, Balance, BlockNumber> {
		pid: u64,
		owner: AccountId,
		state: PoolState,
		payout_commission: u16,
		owner_reward: Balance,
		pool_acc: Balance,
		last_reward_block: BlockNumber,
		total_staked: Balance,
		extra_staked: Balance,
		workers: Vec<WorkerPublicKey>,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct UserStakeInfo<AccountId: Default, Balance> {
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
