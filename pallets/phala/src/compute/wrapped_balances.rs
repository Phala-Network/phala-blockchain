pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::balance_convert::{mul as bmul, FixedPointConvert};
	use crate::base_pool;
	use crate::computation;
	use crate::pool_proxy::PoolProxy;
	use crate::registry;
	use crate::vault;
	use crate::{BalanceOf, NegativeImbalanceOf, PhalaConfig};
	use frame_support::traits::tokens::{Fortitude, Precision};
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
	use frame_system::{pallet_prelude::*, RawOrigin};
	use pallet_democracy::{AccountVote, ReferendumIndex, ReferendumInfo};
	pub use rmrk_traits::primitives::{CollectionId, NftId};
	use scale_info::TypeInfo;
	use sp_runtime::traits::Zero;
	use sp_std::{fmt::Display, prelude::*, result::Result};
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ crate::PhalaConfig
		+ registry::Config
		+ pallet_rmrk_core::Config
		+ computation::Config
		+ pallet_assets::Config
		+ pallet_democracy::Config
		+ base_pool::Config
		+ pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// W-PHA's asset id
		#[pallet::constant]
		type WPhaAssetId: Get<u32>;
		/// Pha's global fund pool
		#[pallet::constant]
		type WrappedBalancesAccountId: Get<Self::AccountId>;
		/// The handler to absorb the slashed amount.
		type OnSlashed: OnUnbalanced<NegativeImbalanceOf<Self>>;
	}

	/// User's asset status proxy
	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug, Default)]
	pub struct FinanceAccount<Balance> {
		/// The pools and their pool collection id the user delegated
		pub invest_pools: Vec<(u64, CollectionId)>,
		/// The locked W-PHA amount used to vote
		pub locked: Balance,
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

	const MAX_ITERRATIONS: u32 = 100;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Mapping from the vote ids and accounts to the amounts of W-PHA used to approve or oppose to the vote
	#[pallet::storage]
	pub type VoteAccountMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ReferendumIndex,
		Blake2_128Concat,
		T::AccountId,
		(BalanceOf<T>, BalanceOf<T>),
	>;

	/// Mapping from the accounts and vote ids to the amounts of W-PHA used to approve or oppose to the vote
	#[pallet::storage]
	pub type AccountVoteMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, ReferendumIndex, ()>;

	/// Mapping for users to their asset status proxys
	#[pallet::storage]
	pub type StakerAccounts<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, FinanceAccount<BalanceOf<T>>>;

	/// Collect the unmintable dust
	// TODO: since this is the imbalance, consider to mint it in the future.
	#[pallet::storage]
	pub type UnmintableDust<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

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
		Wrapped {
			user: T::AccountId,
			amount: BalanceOf<T>,
		},
		Unwrapped {
			user: T::AccountId,
			amount: BalanceOf<T>,
		},
		Voted {
			user: T::AccountId,
			vote_id: ReferendumIndex,
			aye_amount: BalanceOf<T>,
			nay_amount: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// user's `FinanceAccount` does not exist in storage: `StakerAccounts`
		StakerAccountNotFound,
		/// Trying to unwrap more than the available balance
		UnwrapAmountExceedsAvaliableStake,
		/// Trying to vote more than the available balance
		VoteAmountLargerThanTotalStakes,
		/// The vote is not currently on going
		ReferendumInvalid,
		/// The vote is now on going and the W-PHA used in voting can not be unlocked
		ReferendumOngoing,
		/// The Iteration exceed the max limitaion
		IterationsIsNotVaild,
	}

	impl<T: Config> rmrk_traits::TransferHooks<T::AccountId, u32, u32> for Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_rmrk_core::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: pallet_democracy::Config<Currency = <T as crate::PhalaConfig>::Currency>,
		T: Config + vault::Config,
	{
		fn pre_check(
			_sender: &T::AccountId,
			_recipient: &T::AccountId,
			collection_id: &CollectionId,
			_nft_id: &NftId,
		) -> bool {
			if base_pool::pallet::PoolCollections::<T>::get(collection_id).is_some() {
				// Forbid any delegation transfer before delegation nft transfer and sell is fully prepared.
				// TODO(mingxuan): reopen pre_check function.
				return false;
			}

			true
		}
		fn post_transfer(
			_sender: &T::AccountId,
			recipient: &T::AccountId,
			collection_id: &CollectionId,
			_nft_id: &NftId,
		) -> bool {
			if let Some(pid) = base_pool::pallet::PoolCollections::<T>::get(collection_id) {
				base_pool::Pallet::<T>::merge_nft_for_staker(
					*collection_id,
					recipient.clone(),
					pid,
				)
				.expect("mrege or init should not fail");
			}
			true
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_rmrk_core::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: pallet_democracy::Config<Currency = <T as crate::PhalaConfig>::Currency>,
		T: Config + vault::Config,
	{
		/// Wraps some pha and gain equal amount of W-PHA
		///
		/// The wrapped pha is stored in `WrappedBalancesAccountId`'s wallet and can not be taken away
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		#[frame_support::transactional]
		pub fn wrap(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			<T as PhalaConfig>::Currency::transfer(
				&user,
				&T::WrappedBalancesAccountId::get(),
				amount,
				KeepAlive,
			)?;
			Self::mint_into(&user, amount)?;
			if !StakerAccounts::<T>::contains_key(&user) {
				StakerAccounts::<T>::insert(
					&user,
					FinanceAccount::<BalanceOf<T>> {
						invest_pools: vec![],
						locked: Zero::zero(),
					},
				);
			}
			Self::deposit_event(Event::<T>::Wrapped { user, amount });
			Ok(())
		}

		/// Burns the amount of all free W-PHA and unwraps equal amount of pha
		///
		/// The unwrapped pha is transfered from `WrappedBalancesAccountId` to the user's wallet
		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		#[frame_support::transactional]
		pub fn unwrap_all(origin: OriginFor<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let active_stakes = Self::get_net_value(user.clone())?;
			let free_stakes: BalanceOf<T> = <pallet_assets::pallet::Pallet<T> as Inspect<
				T::AccountId,
			>>::balance(T::WPhaAssetId::get(), &user);
			let locked =
				StakerAccounts::<T>::get(&user).map_or(Zero::zero(), |status| status.locked);
			let withdraw_amount = (active_stakes - locked).min(free_stakes);
			<T as PhalaConfig>::Currency::transfer(
				&T::WrappedBalancesAccountId::get(),
				&user,
				withdraw_amount,
				AllowDeath,
			)?;
			Self::burn_from(&user, withdraw_amount)?;
			Ok(())
		}

		/// Unwraps some pha by burning equal amount of W-PHA
		///
		/// The unwrapped pha is transfered from `WrappedBalancesAccountId` to the user's wallet
		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		#[frame_support::transactional]
		pub fn unwrap(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let free_stakes: BalanceOf<T> = <pallet_assets::pallet::Pallet<T> as Inspect<
				T::AccountId,
			>>::balance(T::WPhaAssetId::get(), &user);
			ensure!(
				amount <= free_stakes,
				Error::<T>::UnwrapAmountExceedsAvaliableStake
			);
			let active_stakes = Self::get_net_value(user.clone())?;
			let locked =
				StakerAccounts::<T>::get(&user).map_or(Zero::zero(), |status| status.locked);
			ensure!(
				amount + locked <= active_stakes,
				Error::<T>::UnwrapAmountExceedsAvaliableStake,
			);
			<T as PhalaConfig>::Currency::transfer(
				&T::WrappedBalancesAccountId::get(),
				&user,
				amount,
				AllowDeath,
			)?;
			Self::burn_from(&user, amount)?;
			Self::deposit_event(Event::<T>::Unwrapped { user, amount });
			Ok(())
		}

		/// Uses some W-PHA to approve or oppose a vote
		///
		/// Can both approve and oppose a vote at the same time
		/// The W-PHA used in vote will be locked until the vote is finished or canceled
		#[pallet::call_index(3)]
		#[pallet::weight({0})]
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
			pallet_democracy::Pallet::<T>::vote(
				RawOrigin::Signed(T::WrappedBalancesAccountId::get()).into(),
				vote_id,
				account_vote,
			)?;
			Self::update_user_locked(user.clone())?;
			Self::deposit_event(Event::<T>::Voted {
				user,
				vote_id,
				aye_amount,
				nay_amount,
			});
			Ok(())
		}

		/// Tries to unlock W-PHAs used in vote after the vote finished or canceled
		///
		/// Must assign the max iterations to avoid computing complexity overwhelm
		#[pallet::call_index(4)]
		#[pallet::weight({0})]
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
				max_iterations > 0 && max_iterations <= MAX_ITERRATIONS,
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

		// Reserved: #[pallet::call_index(5)]
	}
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_rmrk_core::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + vault::Config,
	{
		/// Gets W-PHA's asset id
		pub fn get_asset_id() -> u32 {
			T::WPhaAssetId::get()
		}

		/// Removes slash dust
		pub fn remove_dust(who: &T::AccountId, dust: BalanceOf<T>) {
			debug_assert!(dust != Zero::zero());
			if dust != Zero::zero() {
				let actual_removed = pallet_assets::Pallet::<T>::burn_from(
					T::WPhaAssetId::get(),
					who,
					dust,
					Precision::BestEffort,
					Fortitude::Force,
				)
				.expect("slash should success with correct amount: qed.");
				let (imbalance, _remaining) = <T as PhalaConfig>::Currency::slash(
					&<computation::pallet::Pallet<T>>::account_id(),
					dust,
				);
				T::OnSlashed::on_unbalanced(imbalance);
				Self::deposit_event(Event::<T>::DustRemoved {
					user: who.clone(),
					amount: actual_removed,
				});
			}
		}

		/// Mints some W-PHA. If the amount is below ED, it returns Ok(false) and adds the dust
		/// to `UnmintableDust`.
		pub fn mint_into(
			target: &T::AccountId,
			amount: BalanceOf<T>,
		) -> Result<bool, DispatchError> {
			let wpha = T::WPhaAssetId::get();
			let result = pallet_assets::Pallet::<T>::mint_into(wpha, target, amount);
			if result == Err(sp_runtime::TokenError::BelowMinimum.into()) {
				UnmintableDust::<T>::mutate(|value| *value += amount);
				return Ok(false);
			}
			result.and(Ok(true))
		}

		/// Burns some W-PHA
		pub fn burn_from(target: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			pallet_assets::Pallet::<T>::burn_from(
				T::WPhaAssetId::get(),
				target,
				amount,
				Precision::BestEffort,
				Fortitude::Force,
			)?;
			Ok(())
		}

		/// Push a pid into invest pools if it is not included yet
		pub fn maybe_subscribe_to_pool(
			who: &T::AccountId,
			pid: u64,
			cid: CollectionId,
		) -> DispatchResult {
			let mut account_status = StakerAccounts::<T>::get(who).unwrap_or_default();

			if !account_status.invest_pools.contains(&(pid, cid)) {
				account_status.invest_pools.push((pid, cid));
			}
			StakerAccounts::<T>::insert(who, account_status);
			Ok(())
		}

		/// Caculates the net W-PHA value of a user
		///
		/// The net W-PHA value includes:
		/// 1. Free stakes in user's asset account
		/// 2. The current value of shares owned by the user
		/// Note: shares in withdraw queues are not included
		fn get_net_value(who: T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
			let mut total_active_stakes: BalanceOf<T> =
				<pallet_assets::pallet::Pallet<T> as Inspect<T::AccountId>>::balance(
					T::WPhaAssetId::get(),
					&who,
				);
			let account_status = match StakerAccounts::<T>::get(&who) {
				Some(account_status) => account_status,
				None => return Ok(total_active_stakes),
			};
			for (pid, cid) in &account_status.invest_pools {
				pallet_uniques::Pallet::<T>::owned_in_collection(cid, &who).for_each(|nftid| {
					let property_guard = base_pool::Pallet::<T>::get_nft_attr_guard(*cid, nftid)
						.expect("get nft should not fail: qed.");
					let property = &property_guard.attr;
					let pool_proxy = base_pool::Pallet::<T>::pool_collection(pid)
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

		/// Sums up all amounts of W-PHA approves or opposes to the vote
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

		/// Tries to update locked W-PHA amount of the user
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

		/// Returns the minimum balance of WPHA
		pub fn min_balance() -> BalanceOf<T> {
			if !<pallet_assets::pallet::Pallet<T> as Inspect<T::AccountId>>::asset_exists(
				T::WPhaAssetId::get(),
			) {
				panic!("WPHA does not exist");
			}
			<pallet_assets::pallet::Pallet<T> as Inspect<T::AccountId>>::minimum_balance(
				T::WPhaAssetId::get(),
			)
		}
	}
}
