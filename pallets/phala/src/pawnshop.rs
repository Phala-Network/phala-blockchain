pub use self::pallet::*;
use crate::mining;

use frame_support::traits::Currency;

pub type BalanceOf<T> =
	<<T as mining::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use crate::basepool;
	use crate::mining;
	use crate::registry;

	pub use rmrk_traits::primitives::{CollectionId, NftId};

	use super::BalanceOf;

	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::fungibles::Mutate, tokens::nonfungibles::InspectEnumerable, Currency,
			ExistenceRequirement::KeepAlive, LockableCurrency, StorageVersion,
		},
	};

	use pallet_democracy::{AccountVote, ReferendumIndex, ReferendumInfo};

	use crate::balance_convert::FixedPointConvert;

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
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		StakerAccountNotFound,

		RedemptAmountLargerThanTotalStakes,

		NotEnoughFreeStakesToRedempt,

		VoteAmountLargerThanTotalStakes,

		NotEnoughFreeStakesToVote,

		ReferendumInvalid,

		ReferendumOngoing,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
		T: pallet_democracy::Config<Currency = <T as mining::Config>::Currency>,
	{
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn pawn(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			<T as mining::Config>::Currency::transfer(
				&user,
				&T::PawnShopAccountId::get(),
				amount.clone(),
				KeepAlive,
			)?;
			pallet_assets::Pallet::<T>::mint_into(T::PPhaAssetId::get(), &user, amount)?;

			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn redempt(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let active_stakes = Self::get_current_active_stakes(user.clone())?;
			let account_status =
				StakerAccounts::<T>::get(user.clone()).ok_or(Error::<T>::StakerAccountNotFound)?;
			ensure!(
				amount <= active_stakes,
				Error::<T>::RedemptAmountLargerThanTotalStakes,
			);
			ensure!(
				active_stakes - amount >= account_status.locked,
				Error::<T>::NotEnoughFreeStakesToRedempt,
			);
			<T as mining::Config>::Currency::transfer(
				&T::PawnShopAccountId::get(),
				&user,
				amount.clone(),
				KeepAlive,
			)?;
			pallet_assets::Pallet::<T>::burn_from(T::PPhaAssetId::get(), &user, amount)?;

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
			if !Self::ensure_ongoing(vote_id) {
				return Err(Error::<T>::ReferendumInvalid.into());
			}

			let active_stakes = Self::get_current_active_stakes(user.clone())?;
			let account_status =
				StakerAccounts::<T>::get(user.clone()).ok_or(Error::<T>::StakerAccountNotFound)?;
			ensure!(
				active_stakes >= aye_amount + nay_amount,
				Error::<T>::VoteAmountLargerThanTotalStakes,
			);
			ensure!(
				active_stakes - aye_amount - nay_amount >= account_status.locked,
				Error::<T>::NotEnoughFreeStakesToVote,
			);
			VoteAccountMap::<T>::insert(
				vote_id,
				user.clone(),
				(aye_amount.clone(), nay_amount.clone()),
			);
			AccountVoteMap::<T>::insert(user.clone(), vote_id, ());
			let account_vote = Self::accumulate_account_vote(vote_id);
			pallet_democracy::Pallet::<T>::vote(origin, vote_id, account_vote)?;
			Self::settle_user_locked(user)?;
			Ok(())
		}

		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn unlock(origin: OriginFor<T>, vote_id: ReferendumIndex) -> DispatchResult {
			ensure_signed(origin)?;
			if Self::ensure_ongoing(vote_id) {
				return Err(Error::<T>::ReferendumOngoing.into());
			}
			VoteAccountMap::<T>::iter_prefix(vote_id).for_each(|(user, _)| {
				AccountVoteMap::<T>::remove(user.clone(), vote_id);
				Self::settle_user_locked(user).expect("useraccount should exist: qed.");
			});
			VoteAccountMap::<T>::remove_prefix(vote_id, None);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		fn get_current_active_stakes(who: T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
			let account_status =
				StakerAccounts::<T>::get(who.clone()).ok_or(Error::<T>::StakerAccountNotFound)?;
			let mut total_active_stakes: BalanceOf<T> = Zero::zero();
			for (_, cid) in &account_status.invest_pools {
				pallet_uniques::Pallet::<T>::owned_in_collection(&cid, &who).for_each(|nftid| {
					let property = basepool::Pallet::<T>::get_nft_attr(*cid, nftid)
						.expect("get nft should not fail: qed.");
					total_active_stakes += property.shares;
				});
			}
			Ok(total_active_stakes)
		}

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

		fn settle_user_locked(user: T::AccountId) -> DispatchResult {
			let mut max_lock = Zero::zero();
			AccountVoteMap::<T>::iter_prefix(user.clone()).for_each(|(vote_id, ())| {
				let (aye_amount, nay_amount) = VoteAccountMap::<T>::get(vote_id, user.clone())
					.expect("reverse indexing must success: qed.");
				if max_lock < aye_amount + nay_amount {
					max_lock = aye_amount + nay_amount;
				}
			});
			let mut account_status =
				StakerAccounts::<T>::get(user.clone()).ok_or(Error::<T>::StakerAccountNotFound)?;
			account_status.locked = max_lock;
			StakerAccounts::<T>::insert(user, account_status);

			Ok(())
		}

		fn ensure_ongoing(vote_id: ReferendumIndex) -> bool {
			let vote_info = pallet_democracy::Pallet::<T>::referendum_info(vote_id.clone());
			match vote_info {
				Some(info) => match info {
					ReferendumInfo::Ongoing(_) => true,
					_ => false,
				},
				None => false,
			}
		}
	}
}
