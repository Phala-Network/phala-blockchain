pub use self::pallet::*;
use crate::mining;

use frame_support::traits::Currency;

pub type BalanceOf<T> =
	<<T as mining::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type NegativeImbalanceOf<T> = <<T as mining::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
	use crate::basepool;
	use crate::mining;
	use crate::poolproxy::PoolProxy;
	use crate::registry;
	use crate::vault;

	pub use rmrk_traits::primitives::{CollectionId, NftId};

	use super::{BalanceOf, NegativeImbalanceOf};

	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::fungibles::Mutate, tokens::nonfungibles::InspectEnumerable, Currency,
			ExistenceRequirement::KeepAlive, LockableCurrency, OnUnbalanced, StorageVersion,
		},
	};

	use pallet_democracy::{AccountVote, ReferendumIndex, ReferendumInfo};

	use crate::balance_convert::{mul as bmul, FixedPointConvert};

	use sp_std::{fmt::Display, prelude::*, result::Result};

	use sp_runtime::traits::Zero;

	use frame_system::pallet_prelude::*;

	use scale_info::TypeInfo;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ registry::Config
		+ pallet_rmrk_core::Config
		+ mining::Config
		+ pallet_assets::Config
		+ pallet_democracy::Config
		+ basepool::Config
		+ Encode
		+ Decode
		+ pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		#[pallet::constant]
		type PPhaAssetId: Get<u32>;

		#[pallet::constant]
		type PawnShopAccountId: Get<Self::AccountId>;

		/// The handler to absorb the slashed amount.
		type OnSlashed: OnUnbalanced<NegativeImbalanceOf<Self>>;
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct FinanceAccount<Balance> {
		pub invest_pools: Vec<(u64, CollectionId)>,
		pub locked: Balance,
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(5);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type VoteAccountMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ReferendumIndex,
		Blake2_128Concat,
		T::AccountId,
		(BalanceOf<T>, BalanceOf<T>),
	>;

	#[pallet::storage]
	pub type AccountVoteMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, ReferendumIndex, ()>;

	#[pallet::storage]
	#[pallet::getter(fn staker_account)]
	pub type StakerAccounts<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, FinanceAccount<BalanceOf<T>>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
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
	}

	#[pallet::error]
	pub enum Error<T> {
		StakerAccountNotFound,

		RedeemAmountExceedsAvaliableStake,

		VoteAmountLargerThanTotalStakes,

		ReferendumInvalid,

		ReferendumOngoing,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: pallet_democracy::Config<Currency = <T as mining::Config>::Currency>,
		T: Config + vault::Config,
	{
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn pawn(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			<T as mining::Config>::Currency::transfer(
				&user,
				&T::PawnShopAccountId::get(),
				amount,
				KeepAlive,
			)?;
			Self::mint_into(T::PPhaAssetId::get(), &user, amount)?;

			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn redeem(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			is_at_most: bool,
		) -> DispatchResult {
			let mut actural_amount = amount;
			let user = ensure_signed(origin)?;
			let active_stakes = Self::get_net_value(user.clone())?;
			let staker_status =
				StakerAccounts::<T>::get(&user).ok_or(Error::<T>::StakerAccountNotFound)?;
			if is_at_most {
				if actural_amount + staker_status.locked > active_stakes {
					actural_amount = active_stakes - staker_status.locked;
				}
			} else {
				ensure!(
					actural_amount + staker_status.locked <= active_stakes,
					Error::<T>::RedeemAmountExceedsAvaliableStake,
				);
			}
			<T as mining::Config>::Currency::transfer(
				&T::PawnShopAccountId::get(),
				&user,
				actural_amount,
				KeepAlive,
			)?;
			Self::burn_from(T::PPhaAssetId::get(), &user, actural_amount)?;

			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn vote(
			origin: OriginFor<T>,
			aye_amount: BalanceOf<T>,
			nay_amount: BalanceOf<T>,
			vote_id: ReferendumIndex,
		) -> DispatchResult {
			let user = ensure_signed(origin.clone())?;
			if !Self::is_ongoing(vote_id) {
				return Err(Error::<T>::ReferendumInvalid.into());
			}

			let active_stakes = Self::get_net_value(user.clone())?;
			ensure!(
				active_stakes >= aye_amount + nay_amount,
				Error::<T>::VoteAmountLargerThanTotalStakes,
			);
			VoteAccountMap::<T>::insert(vote_id, &user, (aye_amount, nay_amount));
			AccountVoteMap::<T>::insert(&user, vote_id, ());
			let account_vote = Self::accumulate_account_vote(vote_id);
			pallet_democracy::Pallet::<T>::vote(origin, vote_id, account_vote)?;
			Self::update_user_locked(user)?;
			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn unlock(
			origin: OriginFor<T>,
			vote_id: ReferendumIndex,
			max_iterations: u32,
		) -> DispatchResult {
			ensure_signed(origin)?;
			if Self::is_ongoing(vote_id) {
				return Err(Error::<T>::ReferendumOngoing.into());
			}
			let mut iter = VoteAccountMap::<T>::iter_prefix(vote_id).drain();
			let mut i = 0;
			loop {
				if let Some((user, _)) = iter.next() {
					AccountVoteMap::<T>::remove(user.clone(), vote_id);
					Self::update_user_locked(user.clone()).expect("useraccount should exist: qed.");
					if i >= max_iterations {
						break;
					}
					i += 1;
				} else {
					break;
				}
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + vault::Config,
	{
		pub fn get_asset_id() -> u32 {
			T::PPhaAssetId::get()
		}

		pub fn remove_dust(who: &T::AccountId, dust: BalanceOf<T>) {
			debug_assert!(dust != Zero::zero());
			if dust != Zero::zero() {
				let actual_removed =
					pallet_assets::Pallet::<T>::slash(T::PPhaAssetId::get(), who, dust)
						.expect("slash should success with correct amount: qed.");
				let (imbalance, _remaining) = <T as mining::Config>::Currency::slash(
					&<mining::pallet::Pallet<T>>::account_id(),
					dust,
				);
				T::OnSlashed::on_unbalanced(imbalance);
				Self::deposit_event(Event::<T>::DustRemoved {
					user: who.clone(),
					amount: actual_removed,
				});
			}
		}

		pub fn mint_into(
			asset_id: u32,
			target: &T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			pallet_assets::Pallet::<T>::mint_into(asset_id, &target, amount)?;
			Ok(())
		}

		pub fn burn_from(
			asset_id: u32,
			target: &T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			pallet_assets::Pallet::<T>::burn_from(asset_id, &target, amount)?;
			Ok(())
		}

		pub fn maybe_update_account_status(
			who: &T::AccountId,
			pid: u64,
			cid: CollectionId,
		) -> DispatchResult {
			let mut account_status =
				StakerAccounts::<T>::get(&who).ok_or(Error::<T>::StakerAccountNotFound)?;

			if !account_status.invest_pools.contains(&(pid, cid)) {
				account_status.invest_pools.push((pid, cid));
				StakerAccounts::<T>::insert(who, account_status);
			}
			Ok(())
		}

		fn get_net_value(who: T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
			let account_status =
				StakerAccounts::<T>::get(&who).ok_or(Error::<T>::StakerAccountNotFound)?;
			let mut total_active_stakes: BalanceOf<T> = Zero::zero();
			for (pid, cid) in &account_status.invest_pools {
				pallet_uniques::Pallet::<T>::owned_in_collection(&cid, &who).for_each(|nftid| {
					let property_guard = basepool::Pallet::<T>::get_nft_attr_guard(*cid, nftid)
						.expect("get nft should not fail: qed.");
					let property = &property_guard.attr;
					let pool_proxy = basepool::Pallet::<T>::pool_collection(pid)
						.expect("get pool should not fail: qed.");
					let basepool = &match pool_proxy {
						PoolProxy::Vault(p) => p.basepool,
						PoolProxy::StakePool(p) => p.basepool,
					};
					if let Some(price) = basepool.share_price() {
						total_active_stakes += bmul(property.shares, &price);
					}
				});
			}
			Ok(total_active_stakes)
		}

		// TODO(mingxuan): Optimize to O(1) in the future.
		fn accumulate_account_vote(vote_id: ReferendumIndex) -> AccountVote<BalanceOf<T>> {
			let mut total_aye_amount: BalanceOf<T> = Zero::zero();
			let mut total_nay_amount: BalanceOf<T> = Zero::zero();
			VoteAccountMap::<T>::iter_prefix(vote_id).for_each(|(_, (aye_amount, nay_amount))| {
				total_aye_amount += aye_amount;
				total_nay_amount += nay_amount;
			});

			AccountVote::<BalanceOf<T>>::Split {
				aye: total_aye_amount,
				nay: total_nay_amount,
			}
		}

		fn update_user_locked(user: T::AccountId) -> DispatchResult {
			let mut max_lock: BalanceOf<T> = Zero::zero();
			AccountVoteMap::<T>::iter_prefix(user.clone()).for_each(|(vote_id, ())| {
				let (aye_amount, nay_amount) = VoteAccountMap::<T>::get(vote_id, user.clone())
					.expect("reverse indexing must success: qed.");
				max_lock = max_lock.max(aye_amount + nay_amount);
			});
			let mut account_status =
				StakerAccounts::<T>::get(&user).ok_or(Error::<T>::StakerAccountNotFound)?;
			account_status.locked = max_lock;
			StakerAccounts::<T>::insert(user, account_status);

			Ok(())
		}

		fn is_ongoing(vote_id: ReferendumIndex) -> bool {
			let vote_info = pallet_democracy::Pallet::<T>::referendum_info(vote_id.clone());
			matches!(vote_info, Some(ReferendumInfo::Ongoing(_)))
		}
	}
}
