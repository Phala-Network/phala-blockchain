pub use self::pallet::*;
use crate::mining;

use frame_support::traits::Currency;
use sp_runtime::traits::Zero;

pub type BalanceOf<T> =
	<<T as mining::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use crate::mining;
	use crate::pawnshop;
	use crate::poolproxy::*;
	use crate::registry;
	use crate::vault;

	pub use rmrk_traits::{
		primitives::{CollectionId, NftId},
		Nft, Property,
	};

	use super::{extract_dust, is_nondust_balance, BalanceOf};

	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::fungibles::Transfer, tokens::nonfungibles::InspectEnumerable, LockableCurrency,
			StorageVersion, UnixTime,
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

	use frame_system::Origin;

	use scale_info::TypeInfo;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(5);

	const NFT_PROPERTY_KEY: &str = "stake-info";

	const MAX_RECURSIONS: u32 = 100;

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
		+ registry::Config
		+ pallet_rmrk_core::Config
		+ mining::Config
		+ Encode
		+ Decode
		+ pallet_assets::Config
		+ pallet_democracy::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
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

	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub type NextNftId<T: Config> = StorageMap<_, Twox64Concat, CollectionId, NftId, ValueQuery>;
	/// The number of total pools
	#[pallet::storage]
	#[pallet::getter(fn pool_count)]
	pub type PoolCount<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_collection)]
	pub type Pools<T: Config> =
		StorageMap<_, Twox64Concat, u64, PoolProxy<T::AccountId, BalanceOf<T>>>;

	#[pallet::storage]
	pub(super) type NftLocks<T: Config> = StorageMap<_, Twox64Concat, (CollectionId, NftId), ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A withdrawal request is inserted to a queue
		///
		/// Affected states:
		/// - a new item is inserted to or an old item is being replaced by the new item in the
		///   withdraw queue in [`StakePools`]
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
		/// - the stake related fields in [`StakePools`]
		/// - the user staking account at [`PoolStakers`]
		/// - the locking ledger of the contributor at [`StakeLedger`]
		Withdrawal {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},
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

		PoolDoesNotExist,

		PoolTypeNotMatch,

		NoAvailableNftId,

		InvalidSharePrice,

		AttrLocked,
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct BasePool<AccountId, Balance> {
		pub pid: u64,

		pub owner: AccountId,

		pub total_shares: Balance,

		pub total_value: Balance,

		pub withdraw_queue: VecDeque<WithdrawInfo<AccountId>>,

		pub value_subscribers: VecDeque<u64>,

		pub cid: CollectionId,

		pub pool_account_id: AccountId,
	}

	#[derive(Encode, Decode, TypeInfo, PartialEq, Eq, RuntimeDebug)]
	pub struct NftGuard<T: Config> {
		cid: CollectionId,
		nftid: NftId,
		pub attr: NftAttr<BalanceOf<T>>,
	}

	impl<T: Config> Drop for NftGuard<T> {
		fn drop(&mut self) {
			NftLocks::<T>::remove((self.cid, self.nftid));
		}
	}

	impl<T: Config> NftGuard<T> {
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
			// (TODO: maybe can be recovered by removing all the miners from the pool? How to take
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
			}
		}
	}

	pub fn pallet_id<T>() -> T
	where
		T: Encode + Decode,
	{
		(b"basepool")
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed")
	}

	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
		T: Config + pawnshop::Config + vault::Config,
	{
		pub fn get_nft_attr_guard(
			cid: CollectionId,
			nftid: NftId,
		) -> Result<NftGuard<T>, DispatchError> {
			let nft = Self::get_nft_attr(cid, nftid)?;
			if let Some(_) = NftLocks::<T>::get((cid, nftid)) {
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
			let nft_id = Self::merge_or_init_nft_for_staker(pool.cid, account_id.clone())?;
			// The nft instance must be wrote to Nft storage at the end of the function
			// this nft's property shouldn't be accessed or wrote again from storage before set_nft_attr
			// is called. Or the property of the nft will be overwrote incorrectly.
			let mut nft_guard = Self::get_nft_attr_guard(pool.cid, nft_id)?;
			let shares = Self::add_stake_to_new_nft(pool, account_id, amount);
			nft_guard.attr.shares += shares;
			nft_guard.save()?;

			Ok(shares)
		}

		/// Tries to withdraw a specific amount from a pool.
		///
		/// The withdraw request would be delayed if the free stake is not enough, otherwise
		/// withdraw from the free stake immediately.
		///
		/// The updates are made in `pool_info` and `user_info`. It's up to the caller to persist
		/// the data.
		///
		/// Requires:
		/// 1. The user's pending slash is already settled.
		/// 2. The pool must has shares and stake (or it can cause division by zero error)
		#[frame_support::transactional]
		pub fn push_withdraw_in_queue(
			pool: &mut BasePool<T::AccountId, BalanceOf<T>>,
			nft: &mut NftAttr<BalanceOf<T>>,
			account_id: T::AccountId,
			shares: BalanceOf<T>,
		) -> DispatchResult {
			if let None = pool.share_price() {
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
				Self::burn_nft(pool.cid, withdrawinfo.nft_id)
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

		pub fn consume_new_pid() -> u64 {
			let pid = PoolCount::<T>::get();
			PoolCount::<T>::put(pid + 1);
			pid
		}

		pub fn has_expired_withdrawal(
			pool: &BasePool<T::AccountId, BalanceOf<T>>,
			now: u64,
			grace_period: u64,
			releasing_stake: BalanceOf<T>,
		) -> bool {
			debug_assert!(
				pool.get_free_stakes::<T>() == Zero::zero(),
				"We really don't want to have free stake and withdraw requests at the same time"
			);

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

		/// Mint a new nft in the Pool's collection and store some shares in it
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

		/// Remove some stakes from nft when withdraw or process_withdraw_queue called.
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
				&userid,
				amount,
				true,
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

		#[frame_support::transactional]
		pub fn burn_nft(cid: CollectionId, nft_id: NftId) -> DispatchResult {
			pallet_rmrk_core::Pallet::<T>::set_lock((cid, nft_id), false);
			pallet_rmrk_core::Pallet::<T>::nft_burn(cid, nft_id, MAX_RECURSIONS)?;

			Ok(())
		}

		/// Merge multiple nfts belong to one user in the pool.
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
				Self::burn_nft(cid, nftid).expect("burn nft should not fail: qed.");
			});

			Self::mint_nft(cid, staker, total_shares)
		}

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

		pub fn get_next_nft_id(collection_id: CollectionId) -> Result<NftId, Error<T>> {
			NextNftId::<T>::try_mutate(collection_id, |id| {
				let current_id = *id;
				*id = id.checked_add(1).ok_or(Error::<T>::NoAvailableNftId)?;
				Ok(current_id)
			})
		}

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
		/// The updates are made in `pool_info` and `user_info`. It's up to the caller to persist
		/// the data.
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
				shares: shares,
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

		///should be very carful to avoid the vault_info got mutable ref outside the function and saved after this function called
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
						Self::burn_nft(pool_info.cid, withdraw.nft_id)
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
	pub fn create_staker_account<T>(pid: u64, owner: T) -> T
	where
		T: Encode + Decode,
	{
		let hash = crate::hashing::blake2_256(&(pid, owner).encode());
		// stake pool miner
		(b"basepool/", hash)
			.using_encoded(|b| T::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed")
	}
}

use sp_runtime::traits::AtLeast32BitUnsigned;

/// Returns true if `n` is close to zero (1000 pico, or 1e-8).
pub fn balance_close_to_zero<B: AtLeast32BitUnsigned + Copy>(n: B) -> bool {
	n <= B::from(1000u32)
}

/// Returns true if `a` and `b` are close enough (1000 pico, or 1e-8)
pub fn balances_nearly_equal<B: AtLeast32BitUnsigned + Copy>(a: B, b: B) -> bool {
	if a > b {
		balance_close_to_zero(a - b)
	} else {
		balance_close_to_zero(b - a)
	}
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
