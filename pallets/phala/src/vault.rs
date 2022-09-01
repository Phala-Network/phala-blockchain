//! Pool for collaboratively mining staking

pub use self::pallet::*;
use crate::mining;
use frame_support::traits::Currency;

type BalanceOf<T> =
	<<T as mining::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
	#[cfg(not(feature = "std"))]
	use alloc::format;
	#[cfg(feature = "std")]
	use std::format;

	use crate::balance_convert::{div as bdiv, mul as bmul, FixedPointConvert};
	use crate::basepool;
	use crate::mining;
	use crate::pawnshop;
	use crate::poolproxy::{ensure_stake_pool, ensure_vault, PoolProxy, Vault};
	use crate::registry;
	use crate::stakepoolv2;

	use super::BalanceOf;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{LockableCurrency, StorageVersion, UnixTime},
	};
	use frame_system::{pallet_prelude::*, Origin};

	use sp_runtime::{traits::Zero, Permill, SaturatedConversion};
	use sp_std::{collections::vec_deque::VecDeque, fmt::Display, prelude::*};

	pub use rmrk_traits::primitives::{CollectionId, NftId};

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ registry::Config
		+ mining::Config
		+ pallet_rmrk_core::Config
		+ basepool::Config
		+ pallet_assets::Config
		+ pallet_democracy::Config
		+ pawnshop::Config
		+ stakepoolv2::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(5);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type VaultAccountAssignments<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u64>;

	#[pallet::storage]
	pub type VaultLocks<T: Config> = StorageMap<_, Twox64Concat, u64, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated {
			owner: T::AccountId,
			pid: u64,
		},

		VaultCommissionSet {
			pid: u64,
			commission: u32,
		},

		OwnerSharesStartWithdraw {
			pid: u64,
			user: T::AccountId,
			shares: BalanceOf<T>,
		},

		OwnerSharesGained {
			pid: u64,
			shares: BalanceOf<T>,
		},

		Contribution {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		UnauthorizedPoolOwner,

		InvaildWithdrawSharesAmount,

		NoRewardToClaim,

		AssetAccountNotExist,

		InvalidWithdrawalAmount,

		InsufficientBalance,

		InsufficientContribution,

		VaultPriceIsZero,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
	{
		/// Creates a new stake pool
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn create_vault(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pid = basepool::Pallet::<T>::consume_new_pid();
			// TODO(mingxuan): create_collection should return cid
			let collection_id: CollectionId = pallet_rmrk_core::Pallet::<T>::collection_index();
			// Create a NFT collection related to the new stake pool
			let symbol: BoundedVec<u8, <T as pallet_rmrk_core::Config>::CollectionSymbolLimit> =
				format!("VAULT-{}", pid)
					.as_bytes()
					.to_vec()
					.try_into()
					.expect("create a bvec from string should never fail; qed.");
			pallet_rmrk_core::Pallet::<T>::create_collection(
				Origin::<T>::Signed(basepool::pallet_id::<T::AccountId>()).into(),
				Default::default(),
				None,
				symbol,
			)?;
			let account_id =
				basepool::pallet::create_staker_account::<T::AccountId>(pid, owner.clone());
			basepool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::Vault(Vault {
					basepool: basepool::BasePool {
						pid,
						owner: owner.clone(),
						total_shares: Zero::zero(),
						total_value: Zero::zero(),
						withdraw_queue: VecDeque::new(),
						value_subscribers: VecDeque::new(),
						cid: collection_id,
						pool_account_id: account_id.clone(),
					},
					commission: None,
					owner_shares: Zero::zero(),
					last_share_price_checkpoint: Zero::zero(),
					invest_pools: VecDeque::new(),
				}),
			);
			VaultAccountAssignments::<T>::insert(account_id, pid);
			Self::deposit_event(Event::<T>::PoolCreated { owner, pid });

			Ok(())
		}

		/// Change the pool commission rate
		///
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::weight(0)]
		pub fn set_vault_payout_pref(
			origin: OriginFor<T>,
			pid: u64,
			payout_commission: Option<Permill>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut pool_info = ensure_vault::<T>(pid)?;
			// origin must be owner of pool
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);

			pool_info.commission = payout_commission;
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::Vault(pool_info));

			let mut commission: u32 = 0;
			if let Some(ratio) = payout_commission {
				commission = ratio.deconstruct();
			}
			Self::deposit_event(Event::<T>::VaultCommissionSet { pid, commission });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn claim_owner_shares(
			origin: OriginFor<T>,
			vault_pid: u64,
			target: T::AccountId,
			shares: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let mut pool_info = ensure_vault::<T>(vault_pid)?;
			// Add pool owner's reward if applicable
			ensure!(
				who == pool_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			ensure!(
				pool_info.owner_shares >= shares,
				Error::<T>::InvaildWithdrawSharesAmount
			);
			ensure!(shares > Zero::zero(), Error::<T>::NoRewardToClaim);
			let nft_id =
				basepool::Pallet::<T>::mint_nft(pool_info.basepool.cid, target, shares.clone())?;
			pool_info.owner_shares -= shares;
			basepool::pallet::Pools::<T>::insert(vault_pid, PoolProxy::Vault(pool_info));
			Self::deposit_event(Event::<T>::OwnerSharesStartWithdraw {
				pid: vault_pid,
				user: who,
				shares,
			});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn maybe_gain_owner_shares(origin: OriginFor<T>, vault_pid: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut pool_info = ensure_vault::<T>(vault_pid)?;
			// Add pool owner's reward if applicable
			ensure!(
				who == pool_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			let current_price = match pool_info.basepool.share_price() {
				Some(price) => BalanceOf::<T>::from_fixed(&price),
				None => return Ok(()),
			};
			if pool_info.last_share_price_checkpoint == Zero::zero() {
				pool_info.last_share_price_checkpoint = current_price;
				basepool::pallet::Pools::<T>::insert(vault_pid, PoolProxy::Vault(pool_info));
				return Ok(());
			}
			if current_price <= pool_info.last_share_price_checkpoint {
				return Ok(());
			}
			let delta_price = pool_info.commission.unwrap_or_default()
				* (current_price - pool_info.last_share_price_checkpoint);
			let new_price = current_price - delta_price;
			let adjust_shares = bdiv(pool_info.basepool.total_value, &new_price.to_fixed())
				- pool_info.basepool.total_shares;
			pool_info.basepool.total_shares += adjust_shares;
			pool_info.owner_shares += adjust_shares;
			pool_info.last_share_price_checkpoint = current_price;

			basepool::pallet::Pools::<T>::insert(vault_pid, PoolProxy::Vault(pool_info));
			Self::deposit_event(Event::<T>::OwnerSharesGained {
				pid: vault_pid,
				shares: adjust_shares,
			});

			Ok(())
		}

		/// Let any user to launch a stakepool withdraw. Then check if the pool need to be forced shutdown.
		///
		/// If the shutdown condition is met, all workers in the pool will be forced shutdown.
		/// Note: This function doesn't guarantee no-op when there's error.
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn vault_check_and_maybe_force_withdraw(
			origin: OriginFor<T>,
			vault_pid: u64,
		) -> DispatchResult {
			ensure_signed(origin.clone())?;
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			let mut vault = ensure_vault::<T>(vault_pid)?;
			basepool::Pallet::<T>::try_process_withdraw_queue(&mut vault.basepool);
			let grace_period = T::GracePeriod::get();
			let mut releasing_stake = Zero::zero();
			for pid in vault.invest_pools.iter() {
				let stake_pool = ensure_stake_pool::<T>(*pid)?;
				let withdraw_vec: VecDeque<_> = stake_pool
					.basepool
					.withdraw_queue
					.iter()
					.filter(|x| x.user == vault.basepool.pool_account_id)
					.collect();
				// the length of vec should be 1
				for withdraw in withdraw_vec {
					let nft_guard = basepool::Pallet::<T>::get_nft_attr_guard(
						stake_pool.basepool.cid,
						withdraw.nft_id,
					)?;
					let price = stake_pool
						.basepool
						.share_price()
						.ok_or(Error::<T>::VaultPriceIsZero)?;
					releasing_stake += bmul(nft_guard.attr.shares.clone(), &price);
				}
			}
			if vault.basepool.withdraw_queue.len() <= 0 {
				VaultLocks::<T>::remove(vault_pid);
			}
			basepool::pallet::Pools::<T>::insert(vault_pid, PoolProxy::Vault(vault.clone()));
			if basepool::Pallet::<T>::has_expired_withdrawal(
				&vault.basepool,
				now,
				grace_period,
				releasing_stake,
			) {
				VaultLocks::<T>::insert(vault_pid, ());
				for pid in vault.invest_pools.iter() {
					let stake_pool = ensure_stake_pool::<T>(*pid)?;
					let withdraw_vec: VecDeque<_> = stake_pool
						.basepool
						.withdraw_queue
						.iter()
						.filter(|x| x.user == vault.basepool.pool_account_id)
						.collect();
					// the length of vec should be 1
					for withdraw in withdraw_vec {
						let nft_guard = basepool::Pallet::<T>::get_nft_attr_guard(
							stake_pool.basepool.cid,
							withdraw.nft_id,
						)?;
						stakepoolv2::Pallet::<T>::withdraw(
							Origin::<T>::Signed(vault.basepool.owner.clone()).into(),
							stake_pool.basepool.pid,
							nft_guard.attr.shares,
							Some(vault_pid),
						)?;
					}
				}
			}

			Ok(())
		}

		/// Contributes some stake to a vault
		///
		/// Requires:
		/// 1. The pool exists
		/// 2. After the deposit, the pool doesn't reach the cap
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn contribute(origin: OriginFor<T>, pid: u64, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut pool_info = ensure_vault::<T>(pid)?;
			let a = amount; // Alias to reduce confusion in the code below

			ensure!(
				a >= T::MinContribution::get(),
				Error::<T>::InsufficientContribution
			);
			let free = pallet_assets::Pallet::<T>::maybe_balance(
				<T as pawnshop::Config>::PPhaAssetId::get(),
				&who,
			)
			.ok_or(Error::<T>::AssetAccountNotExist)?;
			ensure!(free >= a, Error::<T>::InsufficientBalance);

			let shares =
				basepool::Pallet::<T>::contribute(&mut pool_info.basepool, who.clone(), amount)?;

			// We have new free stake now, try to handle the waiting withdraw queue

			basepool::Pallet::<T>::try_process_withdraw_queue(&mut pool_info.basepool);

			// Persist
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::Vault(pool_info.clone()));
			basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;

			pawnshop::Pallet::<T>::maybe_update_account_status(&who, pid, pool_info.basepool.cid)?;

			Self::deposit_event(Event::<T>::Contribution {
				pid,
				user: who,
				amount: a,
				shares,
			});
			Ok(())
		}

		/// Demands the return of some stake from a pool.
		///
		/// Note: there are two scenarios people may meet
		///
		/// - if the pool has free stake and the amount of the free stake is greater than or equal
		///     to the withdrawal amount (e.g. pool.free_stake >= amount), the withdrawal would
		///     take effect immediately.
		/// - else the withdrawal would be queued and delayed until there is enough free stake.
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn withdraw(origin: OriginFor<T>, pid: u64, shares: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut pool_info = ensure_vault::<T>(pid)?;
			let collection_id = pool_info.basepool.cid;
			let nft_id = basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			// The nft instance must be wrote to Nft storage at the end of the function
			// this nft's property shouldn't be accessed or wrote again from storage before set_nft_attr
			// is called. Or the property of the nft will be overwrote incorrectly.
			let mut nft_guard =
				basepool::Pallet::<T>::get_nft_attr_guard(pool_info.basepool.cid, nft_id)?;
			let mut nft = &mut nft_guard.attr;
			let in_queue_shares = match pool_info
				.basepool
				.withdraw_queue
				.iter()
				.find(|&withdraw| withdraw.user == who)
			{
				Some(withdraw) => {
					let withdraw_nft_guard = basepool::Pallet::<T>::get_nft_attr_guard(
						pool_info.basepool.cid,
						withdraw.nft_id,
					)
					.expect("get nftattr should always success; qed.");
					withdraw_nft_guard.attr.shares
				}
				None => Zero::zero(),
			};
			ensure!(
				basepool::is_nondust_balance(shares) && (shares <= nft.shares + in_queue_shares),
				Error::<T>::InvalidWithdrawalAmount
			);
			basepool::Pallet::<T>::try_withdraw(
				&mut pool_info.basepool,
				&mut nft,
				who.clone(),
				shares,
			)?;

			nft_guard.save()?;
			let nft_id = basepool::Pallet::<T>::merge_or_init_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
			)?;
			basepool::pallet::Pools::<T>::insert(pid, PoolProxy::Vault(pool_info));

			Ok(())
		}
	}
}
