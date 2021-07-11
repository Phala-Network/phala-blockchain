pub use self::pallet::*;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, EnsureOrigin},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	use phala_types::WorkerPublicKey;
	use sp_runtime::{traits::AccountIdConversion, Permill};
	use sp_std::vec::Vec;

	const STAKEPOOL_PALLETID: PalletId = PalletId(*b"phala/sp");

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Mapping from pool id to PoolInfo
	#[pallet::storage]
	pub type Pool<T: Config> =
		StorageMap<_, Twox64Concat, u64, PoolInfo<T::AccountId, BalanceOf<T>>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		/// Meh. [n]
		Meh(u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		Meh,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new stake pool
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>, id: u64) -> DispatchResult {
			panic!("unimplemented")
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
			pool_id: u64,
			payout_commission: Option<Permill>,
		) -> DispatchResult {
			panic!("unimplemented")
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

	#[derive(Encode, Decode, Debug, Clone)]
	pub enum PoolState {
		Ready,
		Mining,
	}

	impl Default for PoolState {
		fn default() -> Self {
			PoolState::Ready
		}
	}

	#[derive(Encode, Decode, Debug, Default, Clone)]
	pub struct PoolInfo<AccountId: Default, Balance> {
		owner: AccountId,
		cap: Option<Balance>,
		commission: Permill,
		state: PoolState,
		total_raised: Balance,
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
