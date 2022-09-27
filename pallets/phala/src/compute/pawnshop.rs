pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::basepool;
	use crate::mining;
	use crate::poolproxy::PoolProxy;
	use crate::registry;
	use crate::vault;

	pub use rmrk_traits::primitives::{CollectionId, NftId};

	use crate::{BalanceOf, NegativeImbalanceOf, PhalaConfig};

	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::fungibles::{Inspect, Mutate},
			tokens::nonfungibles::InspectEnumerable,
			Currency,
			ExistenceRequirement::{AllowDeath, KeepAlive},
			OnUnbalanced, StorageVersion,
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
		+ crate::PhalaConfig
		+ registry::Config
		+ pallet_rmrk_core::Config
		+ mining::Config
		+ pallet_assets::Config
		+ pallet_democracy::Config
		+ basepool::Config
		+ pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// P-PHA's asset id
		#[pallet::constant]
		type PPhaAssetId: Get<u32>;
		/// Pha's global fund pool
		#[pallet::constant]
		type PawnShopAccountId: Get<Self::AccountId>;
		/// The handler to absorb the slashed amount.
		type OnSlashed: OnUnbalanced<NegativeImbalanceOf<Self>>;
	}

	/// User's asset status proxy
	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct FinanceAccount<Balance> {
		/// The pools and their pool collection id the user delegated
		pub invest_pools: Vec<(u64, CollectionId)>,
		/// The locked ppha amount used to vote
		pub locked: Balance,
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(5);

	const MAX_ITERARTIONS: u32 = 100;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Mapping from the vote ids and accounts to the amounts of ppha used to approve or oppose to the vote
	#[pallet::storage]
	pub type VoteAccountMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ReferendumIndex,
		Blake2_128Concat,
		T::AccountId,
		(BalanceOf<T>, BalanceOf<T>),
	>;

	/// Mapping from the accounts and vote ids to the amounts of ppha used to approve or oppose to the vote
	#[pallet::storage]
	pub type AccountVoteMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, ReferendumIndex, ()>;

	/// Mapping for users to their asset status proxys
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
		/// user's `FinanceAccount` does not exist in storage: `StakerAccounts`
		StakerAccountNotFound,
		/// Trying to redeem more than the available balance
		RedeemAmountExceedsAvaliableStake,
		/// Trying to vote more than the available balance
		VoteAmountLargerThanTotalStakes,
		/// The vote is not currently on going
		ReferendumInvalid,
		/// The vote is now on going and the ppha used in voting can not be unlocked
		ReferendumOngoing,
		/// The Iteration exceed the max limitaion
		IterationsIsNotVaild,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: pallet_democracy::Config<Currency = <T as crate::PhalaConfig>::Currency>,
		T: Config + vault::Config,
	{
		/// Pawns some pha and gain equal amount of ppha
		///
		/// The pawned pha is stored in `PawnShopAccountId`'s wallet and can not be taken away
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn pawn(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			<T as PhalaConfig>::Currency::transfer(
				&user,
				&T::PawnShopAccountId::get(),
				amount,
				KeepAlive,
			)?;
			Self::mint_into(&user, amount)?;
			StakerAccounts::<T>::insert(
				&user,
				FinanceAccount::<BalanceOf<T>> {
					invest_pools: vec![],
					locked: Zero::zero(),
				},
			);
			Ok(())
		}

		/// Burns the amount of all free ppha and redeems equal amount of pha
		///
		/// The redeemed pha is transfered from `PawnShopAccountId` to the user's wallet
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn redeem_all(origin: OriginFor<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let active_stakes = Self::get_net_value(user.clone())?;
			let free_stakes: BalanceOf<T> = <pallet_assets::pallet::Pallet<T> as Inspect<
				T::AccountId,
			>>::balance(T::PPhaAssetId::get(), &user);
			let staker_status =
				StakerAccounts::<T>::get(&user).ok_or(Error::<T>::StakerAccountNotFound)?;
			let withdraw_amount = (active_stakes - staker_status.locked).min(free_stakes);
			<T as PhalaConfig>::Currency::transfer(
				&T::PawnShopAccountId::get(),
				&user,
				withdraw_amount,
				AllowDeath,
			)?;
			Self::burn_from(&user, withdraw_amount)?;
			Ok(())
		}

		/// Redeems some pha by burning equal amount of ppha
		///
		/// The redeemed pha is transfered from `PawnShopAccountId` to the user's wallet
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn redeem(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let free_stakes: BalanceOf<T> = <pallet_assets::pallet::Pallet<T> as Inspect<
				T::AccountId,
			>>::balance(T::PPhaAssetId::get(), &user);
			ensure!(
				amount <= free_stakes,
				Error::<T>::RedeemAmountExceedsAvaliableStake
			);
			let active_stakes = Self::get_net_value(user.clone())?;
			let staker_status =
				StakerAccounts::<T>::get(&user).ok_or(Error::<T>::StakerAccountNotFound)?;
			ensure!(
				amount + staker_status.locked <= active_stakes,
				Error::<T>::RedeemAmountExceedsAvaliableStake,
			);
			<T as PhalaConfig>::Currency::transfer(
				&T::PawnShopAccountId::get(),
				&user,
				amount,
				AllowDeath,
			)?;
			Self::burn_from(&user, amount)?;

			Ok(())
		}

		/// Uses some ppha to approve or oppose a vote
		///
		/// Can both approve and oppose a vote at the same time
		/// The pphas used in vote will be locked until the vote is finished or canceled
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

		/// Tries to unlock pphas used in vote after the vote finished or canceled
		///
		/// Must assign the max iterations to avoid computing complexity overwhelm
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
			ensure!(
				max_iterations > 0 && max_iterations <= MAX_ITERARTIONS,
				Error::<T>::IterationsIsNotVaild
			);
			let mut iter = VoteAccountMap::<T>::iter_prefix(vote_id).drain();
			let mut i = 0;
			for (user, _) in iter.by_ref() {
				AccountVoteMap::<T>::remove(user.clone(), vote_id);
				Self::update_user_locked(user.clone()).expect("useraccount should exist: qed.");
				i += 1;
				if i >= max_iterations {
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
		/// Gets ppha's asset id
		pub fn get_asset_id() -> u32 {
			T::PPhaAssetId::get()
		}

		/// Removes slash dust
		pub fn remove_dust(who: &T::AccountId, dust: BalanceOf<T>) {
			debug_assert!(dust != Zero::zero());
			if dust != Zero::zero() {
				let actual_removed =
					pallet_assets::Pallet::<T>::slash(T::PPhaAssetId::get(), who, dust)
						.expect("slash should success with correct amount: qed.");
				let (imbalance, _remaining) = <T as PhalaConfig>::Currency::slash(
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

		/// Mints some ppha
		pub fn mint_into(target: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			pallet_assets::Pallet::<T>::mint_into(T::PPhaAssetId::get(), target, amount)?;
			Ok(())
		}

		/// Burns some ppha
		pub fn burn_from(target: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			pallet_assets::Pallet::<T>::burn_from(T::PPhaAssetId::get(), target, amount)?;
			Ok(())
		}

		/// Push a pid into invest pools if it is not included yet
		pub fn maybe_subscribe_to_pool(
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

		/// Caculates the net ppha value of a user
		///
		/// The net ppha value includes:
		/// 1. Free stakes in user's asset account
		/// 2. The current value of shares owned by the user
		/// Note: shares in withdraw queues are not included
		fn get_net_value(who: T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
			let mut total_active_stakes: BalanceOf<T> =
				<pallet_assets::pallet::Pallet<T> as Inspect<T::AccountId>>::balance(
					T::PPhaAssetId::get(),
					&who,
				);
			let account_status = match StakerAccounts::<T>::get(&who) {
				Some(account_status) => account_status,
				None => return Ok(total_active_stakes),
			};
			for (pid, cid) in &account_status.invest_pools {
				pallet_uniques::Pallet::<T>::owned_in_collection(cid, &who).for_each(|nftid| {
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

		/// Sums up all amounts of ppha approves or opposes to the vote
		// TODO(mingxuan): Optimize to O(1) in the future.
		pub fn accumulate_account_vote(vote_id: ReferendumIndex) -> AccountVote<BalanceOf<T>> {
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

		/// Tries to update locked ppha amount of the user
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

		/// Checks if the vote is ongoing
		fn is_ongoing(vote_id: ReferendumIndex) -> bool {
			let vote_info = pallet_democracy::Pallet::<T>::referendum_info(vote_id);
			matches!(vote_info, Some(ReferendumInfo::Ongoing(_)))
		}
	}
}
