//! # OneshotTransfer Pallet
//!
//! This pallet is the utility to enable one-time transfer. It allows to transfer from any
//! non-blacklisted accounts to up to 100 accounts. However after a successful transfer, both the
//! sender account and the destination accounts will be blacklisted, preveting them from any
//! transferring again in the lifetime.
//!
//! This pallet is useful when token transfer is disabled due to security concerns in the
//! typical "progressive launch process" of Substrate-based blockchains, while users still need
//! subaccounts to pay the transaction fee for the already enabled features (e.g. Phala mining).

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::KeepAlive, StorageVersion},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// The accounts forbidden to transfer
	#[pallet::storage]
	pub type BlacklistedAccounts<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AccountsBlacklisted(Vec<T::AccountId>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Operation not permitted because the sender is already in the blacklist
		SenderAlreadyBlacklisted,
		/// Operation not permitted because one of the destination is already in the blacklist
		DestinationAlreadyBlacklisted,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Distributes some amounts to each specified accounts and mark the sender and destination
		/// accounts as blacklisted.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		#[transactional]
		pub fn distribute(
			origin: OriginFor<T>,
			transfers: Vec<(T::AccountId, BalanceOf<T>)>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Check blacklist
			ensure!(
				BlacklistedAccounts::<T>::get(&who).is_none(),
				Error::<T>::SenderAlreadyBlacklisted
			);
			for (dest, _) in &transfers {
				ensure!(
					BlacklistedAccounts::<T>::get(dest).is_none(),
					Error::<T>::DestinationAlreadyBlacklisted
				);
			}
			// Try to transfer (the entire call will be rolledback if there's no enough funds)
			let mut blacklisted = Vec::<T::AccountId>::new();
			for (dest, amount) in &transfers {
				T::Currency::transfer(&who, dest, *amount, KeepAlive)?;
				BlacklistedAccounts::<T>::insert(dest, ());
				blacklisted.push(dest.clone());
			}
			BlacklistedAccounts::<T>::insert(&who, ());
			blacklisted.push(who);

			Self::deposit_event(Event::<T>::AccountsBlacklisted(blacklisted));
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			let mut w = Weight::zero();
			let old = Self::on_chain_storage_version();
			w += T::DbWeight::get().reads(1);

			if old == 0 {
				STORAGE_VERSION.put::<super::Pallet<T>>();
				w += T::DbWeight::get().writes(1);
			}

			w
		}
	}

	#[cfg(test)]
	mod test {
		use super::*;
		use crate::mock::{
			new_test_ext, set_block_1, take_events,
			RuntimeEvent as TestEvent, RuntimeOrigin as Origin, Test, DOLLARS,
		};
		// Pallets
		use crate::mock::PhalaOneshotTransfer;
		use frame_support::{assert_noop, assert_ok};

		#[test]
		fn can_distribute_once() {
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(PhalaOneshotTransfer::distribute(
					Origin::signed(1),
					vec![(2, DOLLARS), (3, DOLLARS)]
				));
				// Blacklist works
				assert_eq!(BlacklistedAccounts::<Test>::get(0), None);
				assert_eq!(BlacklistedAccounts::<Test>::get(1), Some(()));
				assert_eq!(BlacklistedAccounts::<Test>::get(2), Some(()));
				assert_eq!(BlacklistedAccounts::<Test>::get(3), Some(()));
				// Transfer happened
				assert_eq!(
					take_events(),
					vec![
						TestEvent::Balances(pallet_balances::Event::Transfer {
							from: 1,
							to: 2,
							amount: DOLLARS
						}),
						TestEvent::Balances(pallet_balances::Event::Transfer {
							from: 1,
							to: 3,
							amount: DOLLARS
						}),
						TestEvent::PhalaOneshotTransfer(Event::AccountsBlacklisted(vec![2, 3, 1]))
					]
				);

				assert_noop!(
					PhalaOneshotTransfer::distribute(Origin::signed(1), vec![(4, DOLLARS)]),
					Error::<Test>::SenderAlreadyBlacklisted
				);
				assert_noop!(
					PhalaOneshotTransfer::distribute(
						Origin::signed(5),
						vec![(1, DOLLARS), (6, DOLLARS)]
					),
					Error::<Test>::DestinationAlreadyBlacklisted
				);
			});
		}

		#[test]
		fn insufficient_funds() {
			new_test_ext().execute_with(|| {
				set_block_1();

				assert_noop!(
					PhalaOneshotTransfer::distribute(
						Origin::signed(1),
						vec![(2, 10000000 * DOLLARS)]
					),
					pallet_balances::Error::<Test>::InsufficientBalance
				);
			});
		}

		#[test]
		fn no_enough_existence_deposit() {
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_noop!(
					PhalaOneshotTransfer::distribute(Origin::signed(1), vec![(10, 1)]),
					pallet_balances::Error::<Test>::ExistentialDeposit
				);
			});
		}
	}
}
