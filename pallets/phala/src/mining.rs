pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Currency};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// ///
	// #[pallet::storage]
	// pub type OffchainIngress<T> = StorageMap<_, Twox64Concat, MessageOrigin, u64>;

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
		/// Desposits some tokan as stake
		///
		/// Requires:
		/// 1. Ther miner is in Ready state
		#[pallet::weight(0)]
		pub fn deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Withdraw some tokan from the stake
		///
		/// Requires:
		/// 1. Ther miner is in Ready state
		#[pallet::weight(0)]
		pub fn withdraw(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Starts mining
		///
		/// Requires:
		/// 1. Ther miner is in Ready state
		#[pallet::weight(0)]
		pub fn start_mining(origin: OriginFor<T>) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Stops mining, enterying cooling down state
		///
		/// Requires:
		/// 1. Ther miner is in Idle, Active, or Unresponsive state
		#[pallet::weight(0)]
		pub fn stop_mining(origin: OriginFor<T>) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Turns the miner back to Ready state after cooling down
		///
		/// Requires:
		/// 1. Ther miner is in CoolingDown state and the cooling down period has passed
		#[pallet::weight(0)]
		pub fn cleanup(origin: OriginFor<T>) -> DispatchResult {
			panic!("unimplemented")
		}
	}
}
