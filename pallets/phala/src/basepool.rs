pub use self::pallet::*;
use crate::mining;

use frame_support::traits::Currency;
use sp_runtime::traits::Zero;

pub type BalanceOf<T> =
	<<T as mining::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use crate::mining;
	use crate::poolproxy::*;
	use crate::registry;

	pub use rmrk_traits::{primitives::{CollectionId, NftId}, Property, Nft};

	use super::{extract_dust, is_nondust_balance, BalanceOf};

	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::nonfungibles::InspectEnumerable, LockableCurrency, StorageVersion, UnixTime,
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
		+ pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct NftAttr<Balance> {
		/// Shares that the Nft contains
		pub shares: Balance,
		/// the stakes of Shares at the moment Nft created or transfered
		stakes: Balance,
	}

	impl<Balance> NftAttr<Balance>
	where
		Balance: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
	{
		pub fn remove_stake(&mut self, checked_shares: Balance) -> Balance {
			let nft_price = self
				.stakes
				.to_fixed()
				.checked_div(self.shares.to_fixed())
				.expect("shares will not be zero: qed.");
			self.stakes = bmul(checked_shares, &nft_price);
			let (_, user_dust) = extract_dust(self.stakes.clone());
			user_dust
		}
		pub fn get_stake(&self) -> Balance {
			self.stakes
		}
		pub fn set_stake(&mut self, amount: Balance) {
			self.stakes = amount;
		}
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

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
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct BasePool<AccountId, Balance> {
		pub pid: u64,

		pub owner: AccountId,

		pub total_shares: Balance,

		pub total_value: Balance,

		pub free_stake: Balance,

		pub withdraw_queue: VecDeque<WithdrawInfo<AccountId>>,

		pub value_subscribers: VecDeque<u64>,

		pub cid: CollectionId,
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

		/// Settles the pending slash for a pool user.
		///
		/// The slash is
		///
		/// Returns the slashed amount if succeeded, otherwise None.
		pub fn settle_nft_slash(&self, nft: &mut NftAttr<Balance>) -> Option<Balance> {
			let price = self.share_price()?;
			let locked = nft.stakes;
			let new_locked = bmul(nft.shares, &price);
			// Double check the new_locked won't exceed the original locked
			let new_locked = new_locked.min(locked);
			// When only dust remaining in the pool, we include the dust in the slash amount
			let (new_locked, _) = extract_dust(new_locked);
			nft.stakes = new_locked;
			// The actual slashed amount. Usually slash will only cause the share price decreasing.
			// However in some edge case (i.e. the pool got slashed to 0 and then new contribution
			// added), the locked amount may even become larger
			Some(locked - new_locked)
		}

		// Distributes additional rewards to the current share holders.
		//
		// Additional rewards contribute to the face value of the pool shares. The value of each
		// share effectively grows by (rewards / total_shares).
		//
		// Warning: `total_reward` mustn't be zero.
		// TODO(mingxuan): must be checked carfully before final merge.
		pub fn distribute_reward<T: Config>(
			&mut self,
			rewards: Balance,
			need_update_free_stake: bool,
		) where
			T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
			BalanceOf<T>:
				sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
			T: Config<AccountId = AccountId>,
		{
			self.total_value += rewards;
			if need_update_free_stake {
				self.free_stake += rewards;
			}

			for vault_staker in &self.value_subscribers {
				let mut vault = ensure_vault::<T>(*vault_staker)
					.expect("vault in value_subscribers should always exist: qed.");
				let nft_id = Pallet::<T>::merge_or_init_nft_for_staker(
					self.cid,
					vault.pool_account_id.clone(),
				)
				.expect("merge nft shoule always success: qed.");
				let mut nft = Pallet::<T>::get_nft_attr(self.cid, nft_id)
					.expect("get nft attr should always success: qed.");
				let mut vault_shares = nft.shares.to_fixed();
				let withdraw_vec: VecDeque<_> = self
					.withdraw_queue
					.iter()
					.filter(|x| x.user == vault.pool_account_id)
					.collect();
				for withdraw_info in &withdraw_vec {
					let withdraw_nft = Pallet::<T>::get_nft_attr(self.cid, withdraw_info.nft_id)
						.expect("get nft attr should always success: qed.");
					vault_shares += withdraw_nft.shares.to_fixed();
				}
				let price = match self.share_price() {
					Some(price) => price,
					None => return,
				};
				nft.stakes = bmul(nft.shares, &price);
				Pallet::<T>::set_nft_attr(self.cid, nft_id, &nft)
					.expect("set attr should not fail");
				let stake_ratio = match vault_shares.checked_div(self.total_shares.to_fixed()) {
					Some(ratio) => BalanceOf::<T>::from_fixed(&ratio),
					None => continue,
				};
				let settled_stake = bmul(stake_ratio, &rewards.to_fixed());
				vault.basepool.distribute_reward::<T>(settled_stake, false);
				Pools::<T>::insert(
					*vault_staker,
					PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(vault.clone()),
				);
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
	{
		#[frame_support::transactional]
		pub fn contribute<F>(
			pool: &mut BasePool<T::AccountId, BalanceOf<T>>,
			account_id: T::AccountId,
			amount: BalanceOf<T>,
			settle_callback: Option<F>,
		) -> Result<BalanceOf<T>, DispatchError>
		where
			F: FnMut(
				&BasePool<T::AccountId, BalanceOf<T>>,
				&mut NftAttr<BalanceOf<T>>,
				T::AccountId,
			),
		{
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
			let mut nft = Self::get_nft_attr(pool.cid, nft_id)?;
			// NFT should always settled befroe adding/ removing.

			if let Some(mut settle_method) = settle_callback {
				settle_method(pool, &mut nft, account_id.clone());
			}

			let shares = Self::add_stake_to_new_nft(pool, account_id, amount);
			// Lock the funds

			Self::set_nft_attr(pool.cid, nft_id, &nft)
				.expect("set nft attr should always success; qed.");

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
			maybe_vault_pid: Option<u64>,
		) -> DispatchResult {
			// Remove the existing withdraw request in the queue if there is any.
			let (in_queue_nfts, new_withdraw_queue): (VecDeque<_>, VecDeque<_>) = pool
				.withdraw_queue
				.clone()
				.into_iter()
				.partition(|withdraw| withdraw.user == account_id);
			// only one nft withdraw request should be in the queue
			pool.withdraw_queue = new_withdraw_queue;
			let price = pool
				.share_price()
				.expect("In withdraw case, price should always exists;");

			match maybe_vault_pid {
				Some(pid) => {
					let mut vault = ensure_vault::<T>(pid)?;
					for withdrawinfo in &in_queue_nfts {
						let in_queue_nft = Self::get_nft_attr(pool.cid, withdrawinfo.nft_id)
							.expect("get nft attr should always success; qed.");
						let withdraw_amount = bmul(in_queue_nft.shares.clone(), &price);
						nft.stakes += in_queue_nft.stakes.min(withdraw_amount);
						nft.shares += in_queue_nft.shares;
						Self::burn_nft(pool.cid, withdrawinfo.nft_id)
							.expect("burn nft attr should always success; qed.");
						vault.basepool.free_stake += withdraw_amount;
					}
					Pools::<T>::insert(
						pid,
						PoolProxy::<T::AccountId, BalanceOf<T>>::Vault(vault.clone()),
					);
				}
				None => {
					for withdrawinfo in &in_queue_nfts {
						let in_queue_nft = Self::get_nft_attr(pool.cid, withdrawinfo.nft_id)
							.expect("get nft attr should always success; qed.");
						let withdraw_amount = bmul(in_queue_nft.shares.clone(), &price);
						nft.stakes += in_queue_nft.stakes.min(withdraw_amount);
						nft.shares += in_queue_nft.shares;
						Self::burn_nft(pool.cid, withdrawinfo.nft_id)
							.expect("burn nft attr should always success; qed.");
					}
				}
			};

			let amount = bmul(shares, &price);
			let split_nft_id = Self::mint_nft(pool.cid, pallet_id(), shares, amount)
				.expect("mint nft should always success");
			nft.shares = nft
				.shares
				.checked_sub(&shares)
				.ok_or(Error::<T>::InvalidShareToWithdraw)?;
			nft.stakes = nft
				.stakes
				.checked_sub(&amount)
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

		pub fn has_expired_withdrawal(
			pool: &BasePool<T::AccountId, BalanceOf<T>>,
			now: u64,
			grace_period: u64,
			releasing_stake: BalanceOf<T>,
		) -> bool {
			debug_assert!(
				pool.free_stake == Zero::zero(),
				"We really don't want to have free stake and withdraw requests at the same time"
			);

			// If the pool is bankrupt, or there's no share, we just skip this pool.
			let price = match pool.share_price() {
				Some(price) if price != fp!(0) => price,
				_ => return false,
			};
			let mut budget = pool.free_stake + releasing_stake;
			for request in &pool.withdraw_queue {
				let withdraw_nft = Self::get_nft_attr(pool.cid, request.nft_id)
					.expect("get nftattr should always success; qed.");
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
			Self::mint_nft(pool.cid, account_id, shares, amount)
				.expect("mint should always success; qed.");
			pool.total_shares += shares;
			pool.total_value += amount;
			pool.free_stake += amount;
			shares
		}

		/// Remove some stakes from nft when withdraw or process_withdraw_queue called.
		pub fn remove_stake_from_nft(
			pool: &mut BasePool<T::AccountId, BalanceOf<T>>,
			shares: BalanceOf<T>,
			nft: &mut NftAttr<BalanceOf<T>>,
		) -> Option<(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>)> {
			let price = pool.share_price()?;
			let amount = bmul(shares, &price);

			let amount = amount.min(pool.free_stake);

			let user_shares = nft.shares.checked_sub(&shares)?;
			let (user_shares, shares_dust) = extract_dust(user_shares);

			let removed_shares = shares + shares_dust;
			let total_shares = pool.total_shares.checked_sub(&removed_shares)?;

			let (total_stake, _) = extract_dust(pool.total_value - amount);
			if total_stake > Zero::zero() {
				pool.free_stake -= amount;
				pool.total_value -= amount;
			} else {
				pool.free_stake = Zero::zero();
				pool.total_value = Zero::zero();
			}
			pool.total_shares = total_shares;
			let user_dust = nft.remove_stake(user_shares);
			nft.shares = user_shares;

			Some((amount, user_dust, removed_shares))
		}

		#[frame_support::transactional]
		pub fn mint_nft(
			cid: CollectionId,
			contributer: T::AccountId,
			shares: BalanceOf<T>,
			stakes: BalanceOf<T>,
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

			let attr = NftAttr { shares, stakes };
			//TODO(mingxuan): an external lock is needed to protect nft_attr from dirty write.
			Self::set_nft_attr(cid, nft_id, &attr)?;
			pallet_rmrk_core::Pallet::<T>::set_lock((cid, nft_id), true);
			Ok(nft_id)
		}

		#[frame_support::transactional]
		pub fn burn_nft(cid: CollectionId, nft_id: NftId) -> DispatchResult {
			pallet_rmrk_core::Pallet::<T>::set_lock((cid, nft_id), false);
			pallet_rmrk_core::Pallet::<T>::nft_burn(
				cid,
				nft_id,
				MAX_RECURSIONS,
			)?;

			Ok(())
		}

		/// Merge multiple nfts belong to one user in the pool.
		pub fn merge_or_init_nft_for_staker(
			cid: CollectionId,
			staker: T::AccountId,
		) -> Result<NftId, DispatchError> {
			let mut total_stakes: BalanceOf<T> = Zero::zero();
			let mut total_shares: BalanceOf<T> = Zero::zero();
			pallet_uniques::Pallet::<T>::owned_in_collection(&cid, &staker).for_each(|nftid| {
				let property =
					Self::get_nft_attr(cid, nftid).expect("get nft should not fail: qed.");
				total_stakes += property.stakes;
				total_shares += property.shares;
				Self::burn_nft(cid, nftid).expect("burn nft should not fail: qed.");
			});

			Self::mint_nft(cid, staker, total_shares, total_stakes)
		}

		pub fn get_nft_attr(
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
		pub fn set_nft_attr(
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
			pallet_rmrk_core::Pallet::<T>::do_set_property(
				cid,
				Some(nft_id),
				key,
				value,
			)?;
			pallet_rmrk_core::Pallet::<T>::set_lock((cid, nft_id), true);
			Ok(())
		}
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
