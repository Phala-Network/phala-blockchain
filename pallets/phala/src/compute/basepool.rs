pub use self::pallet::*;

use sp_runtime::traits::{AtLeast32BitUnsigned, Zero};

#[frame_support::pallet]
pub mod pallet {
	use crate::computation;
	use crate::pawnshop;
	use crate::poolproxy::*;
	use crate::registry;
	use crate::vault;
	use crate::BalanceOf;

	pub use rmrk_traits::{
		primitives::{CollectionId, NftId},
		Nft, Property,
	};

	use super::{extract_dust, is_nondust_balance};

	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::fungibles::Transfer, tokens::nonfungibles::InspectEnumerable, StorageVersion,
			UnixTime,
		},
	};

	use fixed::types::U64F64 as FixedPoint;
	use fixed_macro::types::U64F64 as fp;

	use crate::balance_convert::{div as bdiv, mul as bmul, FixedPointConvert};

	use sp_std::{collections::vec_deque::VecDeque, fmt::Display, prelude::*, result::Result};

	use sp_runtime::{
		traits::{CheckedSub, Member, TrailingZeroInput, Zero},
		SaturatedConversion,
	};

	use frame_system::{pallet_prelude::*, Origin};

	use scale_info::TypeInfo;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

	const NFT_PROPERTY_KEY: &str = "stake-info";

	const MAX_RECURSIONS: u32 = 100;

	const MAX_WHITELIST_LEN: u32 = 100;

	type DescMaxLen = ConstU32<4400>;

	pub type DescStr = BoundedVec<u8, DescMaxLen>;

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct WithdrawInfo<AccountId> {
		/// The withdrawal requester
		pub user: AccountId,
		/// The start time of the request
		pub start_time: u64,
		/// The nft_id of the withdraw request
		pub nft_id: NftId,
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ crate::PhalaConfig
		+ registry::Config
		+ pallet_rmrk_core::Config
		+ computation::Config
		+ pallet_assets::Config
		+ pallet_democracy::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct NftAttr<Balance> {
		/// Shares that the Nft contains
		pub shares: Balance,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Mapping from the next self-increased nft ids to collections
	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub type NextNftId<T: Config> = StorageMap<_, Twox64Concat, CollectionId, NftId, ValueQuery>;

	/// The number of total pools
	#[pallet::storage]
	#[pallet::getter(fn pool_count)]
	pub type PoolCount<T> = StorageValue<_, u64, ValueQuery>;

	/// Mapping from pids to pools (including stake pools and vaults)
	#[pallet::storage]
	#[pallet::getter(fn pool_collection)]
	pub type Pools<T: Config> =
		StorageMap<_, Twox64Concat, u64, PoolProxy<T::AccountId, BalanceOf<T>>>;

	/// Mapping from the NftId to its internal locking status
	#[pallet::storage]
	pub(super) type NftLocks<T: Config> = StorageMap<_, Twox64Concat, (CollectionId, NftId), ()>;

	/// Mapping for pools that specify certain stakers to contribute stakes
	#[pallet::storage]
	#[pallet::getter(fn pool_whitelist)]
	pub type PoolContributionWhitelists<T: Config> =
		StorageMap<_, Twox64Concat, u64, Vec<T::AccountId>>;

	/// Mapping for pools that store their descriptions set by owner
	#[pallet::storage]
	#[pallet::getter(fn pool_descriptions)]
	pub type PoolDescriptions<T: Config> = StorageMap<_, Twox64Concat, u64, DescStr>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A withdrawal request is inserted to a queue
		///
		/// Affected states:
		/// - a new item is inserted to or an old item is being replaced by the new item in the
		///   withdraw queue in [`Pools`]
		WithdrawalQueued {
			pid: u64,
			user: T::AccountId,
			shares: BalanceOf<T>,
		},
		/// Some stake was withdrawn from a pool
		///
		/// The lock in [`Balances`](pallet_balances::pallet::Pallet) is updated to release the
		/// locked stake.
		///
		/// Affected states:
		/// - the stake related fields in [`Pools`]
		/// - the user staking asset account
		Withdrawal {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},
		/// A pool contribution whitelist is added
		///
		/// - lazy operated when the first staker is added to the whitelist
		PoolWhitelistCreated { pid: u64 },

		/// The pool contribution whitelist is deleted
		///
		/// - lazy operated when the last staker is removed from the whitelist
		PoolWhitelistDeleted { pid: u64 },

		/// A staker is added to the pool contribution whitelist
		PoolWhitelistStakerAdded { pid: u64, staker: T::AccountId },

		/// A staker is removed from the pool contribution whitelist
		PoolWhitelistStakerRemoved { pid: u64, staker: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// basepool's collection_id isn't founded
		MissCollectionId,
		/// The pool has already got all the stake completely slashed.
		///
		/// In this case, no more funds can be contributed to the pool until all the pending slash
		/// has been resolved.
		PoolBankrupt,
		/// CheckSub less than zero, indicate share amount is invalid
		InvalidShareToWithdraw,
		/// The withdrawal amount is too small (considered as dust)
		InvalidWithdrawalAmount,
		/// RMRK errors
		RmrkError,
		/// The Specified pid does not match to any pool
		PoolDoesNotExist,
		/// Tried to access a pool type that doesn't match the actual pool type in the storage.
		///
		/// E.g. Try to access a vault but it's actually a  stake pool.
		PoolTypeNotMatch,
		/// NftId does not match any nft
		NftIdNotFound,
		/// Occurs when pool's shares is zero
		InvalidSharePrice,
		/// Tried to get a `NftGuard` when the nft is locked. It indicates an internal error occured.
		AttrLocked,
		/// The caller is not the owner of the pool
		UnauthorizedPoolOwner,
		/// Can not add the staker to whitelist because the staker is already in whitelist.
		AlreadyInContributeWhitelist,
		/// Invalid staker to contribute because origin isn't in Pool's contribution whitelist.
		NotInContributeWhitelist,
		/// Too many stakers in contribution whitelist that exceed the limit
		ExceedWhitelistMaxLen,
		/// The pool hasn't have a whitelist created
		NoWhitelistCreated,
		/// Too long for pool description length
		ExceedMaxDescriptionLen,
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct BasePool<AccountId, Balance> {
		/// Pool ID
		pub pid: u64,
		/// The owner of the pool
		pub owner: AccountId,
		/// Total shares
		///
		/// Tracks the total amount of the shares distributed to the contributors. Guaranteed to be
		/// non-dust.
		pub total_shares: Balance,
		/// Tracks the current estimated asset value owned by the pool
		///
		/// Including:
		/// 1. Freestakes of the pool (both case)
		/// 2. Stakes bounded to the worker (stakepool case)
		/// 3. Shares' current values owned by pool_account_id (vault case)
		/// 4. Shares' current values which is in withdraw_queue (both case)
		pub total_value: Balance,
		/// The queue of withdraw requests
		pub withdraw_queue: VecDeque<WithdrawInfo<AccountId>>,
		/// The downstream pools that subscribe to this pool's value changes
		pub value_subscribers: VecDeque<u64>,
		/// The nft collection_id of the pool
		pub cid: CollectionId,
		/// The account generated for the pool and controlled by the pallet
		///
		/// Usage:
		/// 1. Maintains freestakes of the pool with its asset account
		/// 2. Contributes or withdraws from a stakepool (vault case)
		pub pool_account_id: AccountId,
	}

	/// The scope guard that holds the lock of a nft for internal data security
	///
	/// When guard runs out of the scope, it will release the lock in the storage automatically.
	#[derive(Encode, Decode, TypeInfo, PartialEq, Eq, RuntimeDebug)]
	pub struct NftGuard<T: Config> {
		/// CollectionId of the nft
		cid: CollectionId,
		/// Nft's ID
		nftid: NftId,
		/// Nft-attr instance
		pub attr: NftAttr<BalanceOf<T>>,
	}

	/// Unlocks the nft lock when the object deconstructed
	impl<T: Config> Drop for NftGuard<T> {
		fn drop(&mut self) {
			NftLocks::<T>::remove((self.cid, self.nftid));
		}
	}

	impl<T: Config> NftGuard<T> {
		/// Only way to save nft's attributes from outside the pallet
		pub fn save(self) -> DispatchResult
		where
			T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
			BalanceOf<T>:
				sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
			T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
			T: Config + pawnshop::Config + vault::Config,
		{
			Pallet::<T>::set_nft_attr(self.cid, self.nftid, &self.attr)?;
			Ok(())
		}
		/// Deconstructs the [`NftGuard`] proactively. Used in the read-only case.
		pub fn unlock(self) {}
	}

	impl<AccountId, Balance> BasePool<AccountId, Balance>
	where
		AccountId: codec::FullCodec + PartialEq + Clone + Encode + Decode + TypeInfo + Member,
		Balance: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
	{
		pub fn share_price(&self) -> Option<FixedPoint> {
			self.total_value
				.to_fixed()
				.checked_div(self.total_shares.to_fixed())
		}
		pub fn slash(&mut self, amount: Balance) {
			debug_assert!(
				is_nondust_balance(self.total_shares),
				"No share in the pool. This shouldn't happen."
			);
			debug_assert!(
				self.total_value >= amount,
				"No enough stake to slash (total = {}, slash = {})",
				self.total_value,
				amount
			);
			let amount = self.total_value.min(amount);
			// Note that once the stake reaches zero by slashing (implying the share is non-zero),
			// the pool goes bankrupt. In such case, the pool becomes frozen.
			// (TODO: maybe can be recovered by removing all the workers from the pool? How to take
			// care of PoolUsers?)
			let (new_stake, _) = extract_dust(self.total_value - amount);
			self.total_value = new_stake;
		}

		pub fn get_free_stakes<T: Config>(&self) -> Balance
		where
			T: pallet_assets::Config<AssetId = u32, Balance = Balance>,
			T: Config<AccountId = AccountId>,
			T: Config + pawnshop::Config + vault::Config,
		{
			pallet_assets::Pallet::<T>::balance(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&self.pool_account_id,
			)
		}

		// Distributes additional rewards to the current share holders.
		//
		// Additional rewards contribute to the face value of the pool shares. The value of each
		// share effectively grows by (rewards / total_shares).
		//
		// Will adjust total_values of all the value subscribers (i.e. vaults that contributed to the rewarded stakepool).
		//
		// Warning: `total_reward` mustn't be zero.
		// TODO(mingxuan): must be checked carfully before final merge.
		pub fn distribute_reward<T: Config>(&mut self, rewards: Balance)
		where
			T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
			BalanceOf<T>:
				sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
			T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
			T: Config<AccountId = AccountId>,
			T: Config + pawnshop::Config + vault::Config,
		{
			self.total_value += rewards;
			for vault_staker in &self.value_subscribers {
				// The share held by the vault
				let mut vault = ensure_vault::<T>(*vault_staker)
					.expect("vault in value_subscribers should always exist: qed.");
				let nft_id = Pallet::<T>::merge_or_init_nft_for_staker(
					self.cid,
					vault.basepool.pool_account_id.clone(),
				)
				.expect("merge nft shoule always success: qed.");

				let nft_guard = Pallet::<T>::get_nft_attr_guard(self.cid, nft_id)
					.expect("get nft attr should always success: qed.");
				let mut vault_shares = nft_guard.attr.shares.to_fixed();
				nft_guard.unlock();

				// The share in the pool's withdraw queue
				let withdraw_vec: VecDeque<_> = self
					.withdraw_queue
					.iter()
					.filter(|x| x.user == vault.basepool.pool_account_id)
					.collect();
				for withdraw_info in &withdraw_vec {
					let nft_guard = Pallet::<T>::get_nft_attr_guard(self.cid, withdraw_info.nft_id)
						.expect("get nft attr should always success: qed.");

					let withdraw_nft = &nft_guard.attr;
					vault_shares += withdraw_nft.shares.to_fixed();
				}
				let stake_ratio = match vault_shares.checked_div(self.total_shares.to_fixed()) {
					Some(ratio) => BalanceOf::<T>::from_fixed(&ratio),
					None => continue,
				};
				let settled_stake = bmul(stake_ratio, &rewards.to_fixed());
				vault.basepool.distribute_reward::<T>(settled_stake);
				Pools::<T>::insert(*vault_staker, PoolProxy::Vault(vault));
			}
		}
	}

	/// Returns the pallet-controlled account_id used to issue pool nfts
	pub fn pallet_id<T>() -> T
	where
		T: Encode + Decode,
	{
		(b"basepool")
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed")
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + vault::Config,
	{
		/// Adds a staker accountid to contribution whitelist.
		///
		/// Calling this method will forbide stakers contribute who isn't in the whitelist.
		/// The caller must be the owner of the pool.
		/// If a pool hasn't registed in the wihtelist map, any staker could contribute as what they use to do.
		/// The whitelist has a lmit len of 100 stakers.
		#[pallet::weight(0)]
		pub fn add_staker_to_whitelist(
			origin: OriginFor<T>,
			pid: u64,
			staker: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pool_info = ensure_stake_pool::<T>(pid)?;
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			if let Some(mut whitelist) = PoolContributionWhitelists::<T>::get(&pid) {
				ensure!(
					!whitelist.contains(&staker),
					Error::<T>::AlreadyInContributeWhitelist
				);
				ensure!(
					(whitelist.len() as u32) < MAX_WHITELIST_LEN,
					Error::<T>::ExceedWhitelistMaxLen
				);
				whitelist.push(staker.clone());
				PoolContributionWhitelists::<T>::insert(&pid, &whitelist);
			} else {
				let new_list = vec![staker.clone()];
				PoolContributionWhitelists::<T>::insert(&pid, &new_list);
				Self::deposit_event(Event::<T>::PoolWhitelistCreated { pid });
			}
			Self::deposit_event(Event::<T>::PoolWhitelistStakerAdded { pid, staker });

			Ok(())
		}

		/// Adds a description to the pool
		///
		/// The caller must be the owner of the pool.
		#[pallet::weight(0)]
		pub fn set_pool_description(
			origin: OriginFor<T>,
			pid: u64,
			description: DescStr,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pool_info = ensure_stake_pool::<T>(pid)?;
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			PoolDescriptions::<T>::insert(&pid, description);

			Ok(())
		}

		/// Removes a staker accountid to contribution whitelist.
		///
		/// The caller must be the owner of the pool.
		/// If the last staker in the whitelist is removed, the pool will return back to a normal pool that allow anyone to contribute.
		#[pallet::weight(0)]
		pub fn remove_staker_from_whitelist(
			origin: OriginFor<T>,
			pid: u64,
			staker: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pool_info = ensure_stake_pool::<T>(pid)?;
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			let mut whitelist =
				PoolContributionWhitelists::<T>::get(&pid).ok_or(Error::<T>::NoWhitelistCreated)?;
			ensure!(
				whitelist.contains(&staker),
				Error::<T>::NotInContributeWhitelist
			);
			whitelist.retain(|accountid| accountid != &staker);
			if whitelist.is_empty() {
				PoolContributionWhitelists::<T>::remove(&pid);
				Self::deposit_event(Event::<T>::PoolWhitelistStakerRemoved {
					pid,
					staker: staker.clone(),
				});
				Self::deposit_event(Event::<T>::PoolWhitelistDeleted { pid });
			} else {
				PoolContributionWhitelists::<T>::insert(&pid, &whitelist);
				Self::deposit_event(Event::<T>::PoolWhitelistStakerRemoved {
					pid,
					staker: staker.clone(),
				});
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + pawnshop::Config + vault::Config,
	{
		/// Returns a [`NftGuard`] object that can read or write to the nft attributes
		///
		/// Will return an error when another [`NftGuard`] object of the nft is alive
		pub fn get_nft_attr_guard(
			cid: CollectionId,
			nftid: NftId,
		) -> Result<NftGuard<T>, DispatchError> {
			let nft = Self::get_nft_attr(cid, nftid)?;
			if NftLocks::<T>::get((cid, nftid)).is_some() {
				Err(Error::<T>::AttrLocked)?;
			}
			NftLocks::<T>::insert((cid, nftid), ());
			let guard = NftGuard {
				cid,
				nftid,
				attr: nft,
			};
			Ok(guard)
		}

		/// Contributes some stake to the pool
		///
		/// Before minting a new nft to the delegator, the function will try to merge all nfts in the pool-collection into the unified nft
		///
		/// Requires:
		/// 1. The pool exists
		#[frame_support::transactional]
		pub fn contribute(
			pool: &mut BasePool<T::AccountId, BalanceOf<T>>,
			account_id: T::AccountId,
			amount: BalanceOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			ensure!(
				// There's no share, meaning the pool is empty;
				pool.total_shares == Zero::zero()
                // or there's no trivial `total_value`, meaning it's still operating normally
                || pool.total_value > Zero::zero(),
				Error::<T>::PoolBankrupt
			);
			Self::merge_or_init_nft_for_staker(pool.cid, account_id.clone())?;
			// The nft instance must be wrote to Nft storage at the end of the function
			// this nft's property shouldn't be accessed or wrote again from storage before set_nft_attr
			// is called. Or the property of the nft will be overwrote incorrectly.
			let shares = Self::add_stake_to_new_nft(pool, account_id, amount);
			Ok(shares)
		}

		/// Splits a new nft with required withdraw shares and push it to the end of the withdraw_queue
		///
		/// The nft contains withdraw shares is owned by the global pallet_account_id, and record the staker it should belongs to.
		///
		/// If there already has a withdraw request queued by the staker, the formal withdraw request
		/// would be poped out and return shares inside to the staker
		#[frame_support::transactional]
		pub fn push_withdraw_in_queue(
			pool: &mut BasePool<T::AccountId, BalanceOf<T>>,
			nft: &mut NftAttr<BalanceOf<T>>,
			account_id: T::AccountId,
			shares: BalanceOf<T>,
		) -> DispatchResult {
			if pool.share_price().is_none() {
				nft.shares = nft
					.shares
					.checked_sub(&shares)
					.ok_or(Error::<T>::InvalidShareToWithdraw)?;
				return Ok(());
			}

			// Remove the existing withdraw request in the queue if there is any.
			let (in_queue_nfts, new_withdraw_queue): (VecDeque<_>, VecDeque<_>) = pool
				.withdraw_queue
				.clone()
				.into_iter()
				.partition(|withdraw| withdraw.user == account_id);
			// only one nft withdraw request should be in the queue
			pool.withdraw_queue = new_withdraw_queue;

			for withdrawinfo in &in_queue_nfts {
				let nft_guard = Self::get_nft_attr_guard(pool.cid, withdrawinfo.nft_id)?;
				let in_queue_nft = nft_guard.attr.clone();
				nft_guard.unlock();
				nft.shares += in_queue_nft.shares;
				Self::burn_nft(&pallet_id(), pool.cid, withdrawinfo.nft_id)
					.expect("burn nft attr should always success; qed.");
			}

			let split_nft_id = Self::mint_nft(pool.cid, pallet_id(), shares)
				.expect("mint nft should always success");
			nft.shares = nft
				.shares
				.checked_sub(&shares)
				.ok_or(Error::<T>::InvalidShareToWithdraw)?;
			// Push the request
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			pool.withdraw_queue.push_back(WithdrawInfo {
				user: account_id,
				start_time: now,
				nft_id: split_nft_id,
			});

			Ok(())
		}

		/// Returns the new pid that will assigned to the creating pool
		pub fn consume_new_pid() -> u64 {
			let pid = PoolCount::<T>::get();
			PoolCount::<T>::put(pid + 1);
			pid
		}

		/// Checks if there has expired withdraw request in the withdraw queue
		///
		/// Releasing stakes (stakes in workers that is cooling down (stakepool case), or shares in withdraw_queue (vault case))
		/// should be caculated outside the function and put in
		pub fn has_expired_withdrawal(
			pool: &BasePool<T::AccountId, BalanceOf<T>>,
			now: u64,
			grace_period: u64,
			releasing_stake: BalanceOf<T>,
		) -> bool {
			// If the pool is bankrupt, or there's no share, we just skip this pool.
			let price = match pool.share_price() {
				Some(price) if price != fp!(0) => price,
				_ => return false,
			};
			let mut budget = pool.get_free_stakes::<T>() + releasing_stake;
			for request in &pool.withdraw_queue {
				let nft_guard = Self::get_nft_attr_guard(pool.cid, request.nft_id)
					.expect("get nftattr should always success; qed.");
				let withdraw_nft = nft_guard.attr.clone();
				nft_guard.unlock();
				let amount = bmul(withdraw_nft.shares, &price);
				if amount > budget {
					// Run out of budget, let's check if the request is still in the grace period
					return now - request.start_time > grace_period;
				} else {
					// Otherwise we allocate some budget to virtually fulfill the request
					budget -= amount;
				}
			}
			false
		}

		
		/// Mints a new nft in the Pool's collection and store some shares in it
		pub fn add_stake_to_new_nft(
			pool: &mut BasePool<T::AccountId, BalanceOf<T>>,
			account_id: T::AccountId,
			amount: BalanceOf<T>,
		) -> BalanceOf<T> {
			let shares = match pool.share_price() {
				Some(price) if price != fp!(0) => bdiv(amount, &price),
				_ => amount, // adding new stake (share price = 1)
			};
			Self::mint_nft(pool.cid, account_id.clone(), shares)
				.expect("mint should always success; qed.");
			pool.total_shares += shares;
			pool.total_value += amount;
			<pallet_assets::pallet::Pallet<T> as Transfer<T::AccountId>>::transfer(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&account_id,
				&pool.pool_account_id,
				amount,
				true,
			)
			.expect("transfer should not fail");
			shares
		}

		/// Removes some stakes from nft when withdraw or process_withdraw_queue called.
		pub fn remove_stake_from_nft(
			pool: &mut BasePool<T::AccountId, BalanceOf<T>>,
			shares: BalanceOf<T>,
			nft: &mut NftAttr<BalanceOf<T>>,
			userid: &T::AccountId,
		) -> Option<(BalanceOf<T>, BalanceOf<T>)> {
			let price = pool.share_price()?;
			let amount = bmul(shares, &price);

			let amount = amount.min(pool.get_free_stakes::<T>());

			let user_shares = nft.shares.checked_sub(&shares)?;
			let (user_shares, shares_dust) = extract_dust(user_shares);

			let removed_shares = shares + shares_dust;
			let total_shares = pool.total_shares.checked_sub(&removed_shares)?;

			let (total_stake, _) = extract_dust(pool.total_value - amount);

			<pallet_assets::pallet::Pallet<T> as Transfer<T::AccountId>>::transfer(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&pool.pool_account_id,
				userid,
				amount,
				false,
			)
			.expect("transfer should not fail");
			if total_stake > Zero::zero() {
				pool.total_value -= amount;
			} else {
				pool.total_value = Zero::zero();
			}
			pool.total_shares = total_shares;
			nft.shares = user_shares;

			Some((amount, removed_shares))
		}

		/// Mints a nft and wrap some shares within
		#[frame_support::transactional]
		pub fn mint_nft(
			cid: CollectionId,
			contributer: T::AccountId,
			shares: BalanceOf<T>,
		) -> Result<NftId, DispatchError> {
			pallet_rmrk_core::Collections::<T>::get(cid)
				.ok_or(pallet_rmrk_core::Error::<T>::CollectionUnknown)?;
			let nft_id = Self::get_next_nft_id(cid)?;

			pallet_rmrk_core::Pallet::<T>::mint_nft(
				Origin::<T>::Signed(pallet_id()).into(),
				Some(contributer),
				nft_id,
				cid,
				None,
				None,
				Default::default(),
				true,
				None,
			)?;

			let attr = NftAttr { shares };
			//TODO(mingxuan): an external lock is needed to protect nft_attr from dirty write.
			Self::set_nft_attr(cid, nft_id, &attr)?;
			pallet_rmrk_core::Pallet::<T>::set_lock((cid, nft_id), true);
			Ok(nft_id)
		}

		/// Simpily burn the nft
		#[frame_support::transactional]
		pub fn burn_nft(owner: &T::AccountId, cid: CollectionId, nft_id: NftId) -> DispatchResult {
			pallet_rmrk_core::Pallet::<T>::set_lock((cid, nft_id), false);
			pallet_rmrk_core::Pallet::<T>::nft_burn(owner.clone(), cid, nft_id, MAX_RECURSIONS)?;
			Ok(())
		}

		/// Merges multiple nfts belong to one user in the pool.
		pub fn merge_or_init_nft_for_staker(
			cid: CollectionId,
			staker: T::AccountId,
		) -> Result<NftId, DispatchError> {
			let mut total_shares: BalanceOf<T> = Zero::zero();
			pallet_uniques::Pallet::<T>::owned_in_collection(&cid, &staker).for_each(|nftid| {
				let nft_guard =
					Self::get_nft_attr_guard(cid, nftid).expect("get nft should not fail: qed.");
				let property = nft_guard.attr.clone();
				nft_guard.unlock();
				total_shares += property.shares;
				Self::burn_nft(&staker, cid, nftid).expect("burn nft should not fail: qed.");
			});
			Self::mint_nft(cid, staker, total_shares)
		}

		/// Gets nft attr, can only be called in the pallet
		fn get_nft_attr(
			cid: CollectionId,
			nft_id: NftId,
		) -> Result<NftAttr<BalanceOf<T>>, DispatchError> {
			let key: BoundedVec<u8, <T as pallet_uniques::Config>::KeyLimit> = NFT_PROPERTY_KEY
				.as_bytes()
				.to_vec()
				.try_into()
				.expect("str coverts to bvec should never fail; qed.");
			let raw_value: BoundedVec<u8, <T as pallet_uniques::Config>::ValueLimit> =
				pallet_rmrk_core::Pallet::<T>::properties((cid, Some(nft_id), key))
					.ok_or(Error::<T>::MissCollectionId)?;
			Ok(Decode::decode(&mut raw_value.as_slice()).expect("Decode should never fail; qed."))
		}

		/// Gets a new nftid in certain collectionid
		pub fn get_next_nft_id(collection_id: CollectionId) -> Result<NftId, Error<T>> {
			NextNftId::<T>::try_mutate(collection_id, |id| {
				let current_id = *id;
				*id = id.checked_add(1).ok_or(Error::<T>::NftIdNotFound)?;
				Ok(current_id)
			})
		}

		/// Sets nft attr, can only be called in the pallet
		#[frame_support::transactional]
		fn set_nft_attr(
			cid: CollectionId,
			nft_id: NftId,
			nft_attr: &NftAttr<BalanceOf<T>>,
		) -> DispatchResult {
			let encode_attr = nft_attr.encode();
			let key: BoundedVec<u8, <T as pallet_uniques::Config>::KeyLimit> = NFT_PROPERTY_KEY
				.as_bytes()
				.to_vec()
				.try_into()
				.expect("str coverts to bvec should never fail; qed.");
			let value: BoundedVec<u8, <T as pallet_uniques::Config>::ValueLimit> =
				encode_attr.try_into().unwrap();
			pallet_rmrk_core::Pallet::<T>::set_lock((cid, nft_id), false);
			pallet_rmrk_core::Pallet::<T>::do_set_property(cid, Some(nft_id), key, value)?;
			pallet_rmrk_core::Pallet::<T>::set_lock((cid, nft_id), true);
			Ok(())
		}

		/// Tries to withdraw a specific amount from a pool.
		///
		/// The withdraw request would be delayed if the free stake is not enough, otherwise
		/// withdraw from the free stake immediately.
		///
		/// Requires:
		/// 1. The user's pending slash is already settled.
		/// 2. The pool must has shares and stake (or it can cause division by zero error)
		pub fn try_withdraw(
			pool_info: &mut BasePool<T::AccountId, BalanceOf<T>>,
			nft: &mut NftAttr<BalanceOf<T>>,
			userid: T::AccountId,
			shares: BalanceOf<T>,
		) -> DispatchResult {
			Self::push_withdraw_in_queue(pool_info, nft, userid.clone(), shares)?;
			Self::deposit_event(Event::<T>::WithdrawalQueued {
				pid: pool_info.pid,
				user: userid,
				shares,
			});
			Self::try_process_withdraw_queue(pool_info);

			Ok(())
		}

		pub fn maybe_remove_dust(
			pool_info: &mut BasePool<T::AccountId, BalanceOf<T>>,
			nft: &NftAttr<BalanceOf<T>>,
		) -> bool {
			if is_nondust_balance(nft.shares) {
				return false;
			}
			pool_info.total_shares -= nft.shares;
			true
		}

		/// Removes withdrawing_shares from the nft
		pub fn do_withdraw_shares(
			withdrawing_shares: BalanceOf<T>,
			pool_info: &mut BasePool<T::AccountId, BalanceOf<T>>,
			nft: &mut NftAttr<BalanceOf<T>>,
			userid: T::AccountId,
		) {
			// Overflow warning: remove_stake is carefully written to avoid precision error.
			// (I hope so)
			let (reduced, withdrawn_shares) =
				Self::remove_stake_from_nft(pool_info, withdrawing_shares, nft, &userid)
					.expect("There are enough withdrawing_shares; qed.");
			Self::deposit_event(Event::<T>::Withdrawal {
				pid: pool_info.pid,
				user: userid,
				amount: reduced,
				shares: withdrawn_shares,
			});
		}

		/// Tries to fulfill the withdraw queue with the newly freed stake
		pub fn try_process_withdraw_queue(pool_info: &mut BasePool<T::AccountId, BalanceOf<T>>) {
			// The share price shouldn't change at any point in this function. So we can calculate
			// only once at the beginning.
			let price = match pool_info.share_price() {
				Some(price) => price,
				None => return,
			};

			while is_nondust_balance(pool_info.get_free_stakes::<T>()) {
				if let Some(withdraw) = pool_info.withdraw_queue.front().cloned() {
					// Must clear the pending reward before any stake change
					let mut withdraw_nft_guard =
						Self::get_nft_attr_guard(pool_info.cid, withdraw.nft_id)
							.expect("get nftattr should always success; qed.");
					let mut withdraw_nft = withdraw_nft_guard.attr.clone();
					// Try to fulfill the withdraw requests as much as possible
					let free_shares = if price == fp!(0) {
						withdraw_nft.shares // 100% slashed
					} else {
						bdiv(pool_info.get_free_stakes::<T>(), &price)
					};
					// This is the shares to withdraw immedately. It should NOT contain any dust
					// because we ensure (1) `free_shares` is not dust earlier, and (2) the shares
					// in any withdraw request mustn't be dust when inserting and updating it.
					let withdrawing_shares = free_shares.min(withdraw_nft.shares);
					debug_assert!(
						is_nondust_balance(withdrawing_shares),
						"withdrawing_shares must be positive"
					);
					// Actually remove the fulfilled withdraw request. Dust in the user shares is
					// considered but it in the request is ignored.
					Self::do_withdraw_shares(
						withdrawing_shares,
						pool_info,
						&mut withdraw_nft,
						withdraw.user.clone(),
					);
					withdraw_nft_guard.attr = withdraw_nft.clone();
					withdraw_nft_guard
						.save()
						.expect("save nft should always success");
					// Update if the withdraw is partially fulfilled, otherwise pop it out of the
					// queue
					if withdraw_nft.shares == Zero::zero()
						|| Self::maybe_remove_dust(pool_info, &withdraw_nft)
					{
						pool_info.withdraw_queue.pop_front();
						Self::burn_nft(&pallet_id(), pool_info.cid, withdraw.nft_id)
							.expect("burn nft should always success");
					} else {
						*pool_info
							.withdraw_queue
							.front_mut()
							.expect("front exists as just checked; qed.") = withdraw;
					}
				} else {
					break;
				}
			}
		}
	}

	/// Creates a pool_account_id bounded with the pool
	pub fn generate_staker_account<T>(pid: u64, owner: T) -> T
	where
		T: Encode + Decode,
	{
		let hash = crate::hashing::blake2_256(&(pid, owner).encode());
		// stake pool worker
		(b"bp/", hash)
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed")
	}
}

/// Returns true if `n` is close to zero (1000 pico, or 1e-8).
pub fn balance_close_to_zero<B: AtLeast32BitUnsigned + Copy>(n: B) -> bool {
	n <= B::from(1000u32)
}

/// Returns true if `n` is a non-trivial positive balance
pub fn is_nondust_balance<B: AtLeast32BitUnsigned + Copy>(n: B) -> bool {
	!balance_close_to_zero(n)
}
/// Normalizes `n` to zero if it's a dust balance.
///
/// Returns type  `n` itself if it's a non-trivial positive balance, otherwise zero.
pub fn extract_dust<B: AtLeast32BitUnsigned + Copy>(n: B) -> (B, B) {
	if balance_close_to_zero(n) {
		(Zero::zero(), n)
	} else {
		(n, Zero::zero())
	}
}
