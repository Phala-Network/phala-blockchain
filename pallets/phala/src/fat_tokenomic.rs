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
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Stake of user to contract
	#[pallet::storage]
	pub type ContractUserStakes<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		ContractId,
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Map of contracts to their total stakes received
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
		UserStakeChanged {
			account: T::AccountId,
			contract: ContractId,
			stake: BalanceOf<T>,
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
		/// Adjust stake to given contract.
		///
		/// Fat contracts accept depoits from accounts. The deposit info would be sent the cluster's
		/// system contract. Then the system contract would invoke the driver contract (if installed)
		/// to process the deposit info. A public good cluster usually would set the contracts' scheduling
		/// weights according to the total depoit on contracts. More weights means it would get more
		/// compute resource to run the contract. The weights are applied on contract query and Sidevm
		/// CPU round scheduling.
		///
		/// If users stake on a contract doesn't deployed yet. The deposit would send to the cluster
		/// even if the contract is deployed later. User can re-stake with or without changing the amount
		/// to sync the depoit the the cluster after the contract is actually deployed.
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

			let mut total = ContractTotalStakes::<T>::get(contract);
			let orig = ContractUserStakes::<T>::get(&user, contract);
			if amount > orig {
				let delta = amount - orig;
				total += delta;
				<T as Config>::Currency::transfer(&user, &Self::pallet_id(), delta, KeepAlive)?;
			} else {
				let delta = orig - amount;
				total -= delta;
				<T as Config>::Currency::transfer(&Self::pallet_id(), &user, delta, AllowDeath)?;
			}
			ContractUserStakes::<T>::insert(&user, contract, amount);
			ContractTotalStakes::<T>::insert(contract, total);

			Self::deposit_event(Event::ContractDepositChanged {
				contract,
				deposit: total,
			});

			Self::deposit_event(Event::UserStakeChanged {
				account: user,
				contract,
				stake: amount,
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
