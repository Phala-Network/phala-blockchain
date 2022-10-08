//! The Fat Contract tokenomic module

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::mq::MessageOriginInfo;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::*, StorageVersion},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use phala_types::messaging::ContractId;
	use sp_runtime::traits::{AccountIdConversion, Zero};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	const PALLET_ID: PalletId = PalletId(*b"phat/tok");

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(5);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// (contract, user) -> stake
	#[pallet::storage]
	pub type ContractUserStakes<T: Config> =
		StorageMap<_, Twox64Concat, (ContractId, T::AccountId), BalanceOf<T>, ValueQuery>;

	/// contract -> stake
	#[pallet::storage]
	pub type ContractTotalStakes<T: Config> =
		StorageMap<_, Twox64Concat, ContractId, BalanceOf<T>, ValueQuery>;

	/// Minimum allowed stake
	#[pallet::storage]
	pub type MinStake<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ContractDepositChanged {
			contract: ContractId,
			deposit: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidAmountOfStake,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config,
		T: crate::fat::Config,
	{
		#[pallet::weight(0)]
		pub fn adjust_stake(
			origin: OriginFor<T>,
			contract: ContractId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;
			ensure!(
				amount == Zero::zero() || amount >= MinStake::<T>::get(),
				Error::<T>::InvalidAmountOfStake
			);

			let mut total = ContractTotalStakes::<T>::get(&contract);
			let orig = ContractUserStakes::<T>::get((&contract, &user));
			if amount > orig {
				let delta = amount - orig;
				total += delta;
				<T as Config>::Currency::transfer(&user, &Self::pallet_id(), delta, KeepAlive)?;
			} else {
				let delta = orig - amount;
				total -= delta;
				<T as Config>::Currency::transfer(&Self::pallet_id(), &user, delta, AllowDeath)?;
			}
			ContractUserStakes::<T>::insert((&contract, &user), amount);
			ContractTotalStakes::<T>::insert(contract, total);

			Self::deposit_event(Event::ContractDepositChanged {
				contract,
				deposit: total,
			});

			if let Some(the_system_contract) =
				crate::fat::Pallet::<T>::get_system_contract(&contract)
			{
				let selector: u32 = 0xa24bcb44; // ContractDeposit::change_deposit()
				let message = (selector.to_be_bytes(), contract, total).encode();
				Self::push_ink_message(the_system_contract, message);
			}
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn pallet_id() -> T::AccountId {
			PALLET_ID.into_account_truncating()
		}
	}

	impl<T: Config + crate::mq::Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}
}

#[cfg(test)]
mod tests;
