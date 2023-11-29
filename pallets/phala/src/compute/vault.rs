pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use alloc::format;

	use crate::balance_convert::{div as bdiv, mul as bmul, FixedPointConvert};
	use crate::base_pool;
	use crate::computation;
	use crate::pool_proxy::{ensure_stake_pool, ensure_vault, PoolProxy, Vault};
	use crate::registry;
	use crate::stake_pool_v2;
	use crate::wrapped_balances;

	use crate::BalanceOf;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{tokens::nonfungibles::InspectEnumerable, StorageVersion, UnixTime},
	};
	use frame_system::{pallet_prelude::*, Origin};

	use sp_runtime::{traits::Zero, Permill, SaturatedConversion};
	use sp_std::{collections::vec_deque::VecDeque, fmt::Display, prelude::*, vec};

	pub use rmrk_traits::primitives::{CollectionId, NftId};

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ crate::PhalaConfig
		+ registry::Config
		+ computation::Config
		+ pallet_rmrk_core::Config
		+ base_pool::Config
		+ pallet_assets::Config
		+ pallet_democracy::Config
		+ wrapped_balances::Config
		+ stake_pool_v2::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type InitialPriceCheckPoint: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type VaultQueuePeriod: Get<u64>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Mapping from the vault pid to its owner authority locking status
	///
	/// Using to forbid vault's owner to trigger an withdraw for the vault and override the withdraw request issued by `force shutdown`.
	#[pallet::storage]
	pub type VaultLocks<T: Config> = StorageMap<_, Twox64Concat, u64, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A vault is created by `owner`
		///
		/// Affected states:
		/// - a new entry in [`Pools`] with the pid
		PoolCreated {
			owner: T::AccountId,
			pid: u64,
			cid: CollectionId,
			pool_account_id: T::AccountId,
		},

		/// The commission of a vault is updated
		///
		/// The commission ratio is represented by an integer. The real value is
		/// `commission / 1_000_000u32`.
		///
		/// Affected states:
		/// - the `commission` field in [`Pools`] is updated
		VaultCommissionSet { pid: u64, commission: u32 },

		/// Owner shares is claimed by pool owner
		/// Affected states:
		/// - the shares related fields in [`Pools`]
		/// - the nft related storages in rmrk and pallet unique
		OwnerSharesClaimed {
			pid: u64,
			user: T::AccountId,
			shares: BalanceOf<T>,
		},

		/// Additional owner shares are mint into the pool
		///
		/// Affected states:
		/// - the shares related fields in [`Pools`]
		/// - last_share_price_checkpoint in [`Pools`]
		OwnerSharesGained {
			pid: u64,
			shares: BalanceOf<T>,
			checkout_price: BalanceOf<T>,
		},

		/// Someone contributed to a vault
		///
		/// Affected states:
		/// - the stake related fields in [`Pools`]
		/// - the user W-PHA balance reduced
		/// - the user recive ad share NFT once contribution succeeded
		/// - when there was any request in the withdraw queue, the action may trigger withdrawals
		///   ([`Withdrawal`](#variant.Withdrawal) event)
		Contribution {
			pid: u64,
			user: T::AccountId,
			amount: BalanceOf<T>,
			shares: BalanceOf<T>,
		},
		ForceShutdown {
			pid: u64,
			reason: ForceShutdownReason,
		},
	}

	#[derive(PartialEq, Eq, Clone, Debug, Encode, Decode, scale_info::TypeInfo)]
	pub enum ForceShutdownReason {
		NoEnoughReleasingStake,
		Waiting3xGracePeriod,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The caller is not the owner of the pool
		UnauthorizedPoolOwner,
		/// The withdrawal amount is too small or too large
		NoEnoughShareToClaim,
		/// The vault have no owner shares to claim
		NoRewardToClaim,
		/// The asset account hasn't been created. It indicates an internal error.
		AssetAccountNotExist,
		/// Trying to contribute more than the available balance
		InsufficientBalance,
		/// The contributed stake is smaller than the minimum threshold
		InsufficientContribution,
		/// The Vault was bankrupt; cannot interact with it unless all the shares are withdrawn.
		VaultBankrupt,
		/// The caller has no nft to withdraw
		NoNftToWithdraw,
		/// The commission is not changed
		CommissionNotChanged,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_rmrk_core::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
	{
		/// Creates a new vault
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		#[frame_support::transactional]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let pid = base_pool::Pallet::<T>::consume_new_pid();
			let collection_id: CollectionId = base_pool::Pallet::<T>::consume_new_cid();
			// Create a NFT collection related to the new stake pool
			let symbol: BoundedVec<u8, <T as pallet_rmrk_core::Config>::CollectionSymbolLimit> =
				format!("VAULT-{pid}")
					.as_bytes()
					.to_vec()
					.try_into()
					.expect("create a bvec from string should never fail; qed.");
			pallet_rmrk_core::Pallet::<T>::create_collection(
				Origin::<T>::Signed(base_pool::pallet_id::<T::AccountId>()).into(),
				collection_id,
				Default::default(),
				None,
				symbol,
			)?;
			let account_id =
				base_pool::pallet::generate_staker_account::<T::AccountId>(pid, owner.clone());
			base_pool::pallet::Pools::<T>::insert(
				pid,
				PoolProxy::Vault(Vault {
					basepool: base_pool::BasePool {
						pid,
						owner: owner.clone(),
						total_shares: Zero::zero(),
						total_value: Zero::zero(),
						withdraw_queue: VecDeque::new(),
						value_subscribers: vec![],
						cid: collection_id,
						pool_account_id: account_id.clone(),
					},
					commission: None,
					owner_shares: Zero::zero(),
					last_share_price_checkpoint: T::InitialPriceCheckPoint::get(),
					invest_pools: vec![],
				}),
			);
			base_pool::pallet::PoolCollections::<T>::insert(collection_id, pid);
			Self::deposit_event(Event::<T>::PoolCreated {
				owner,
				pid,
				cid: collection_id,
				pool_account_id: account_id,
			});

			Ok(())
		}

		/// Changes the vault commission rate
		///
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		pub fn set_payout_pref(
			origin: OriginFor<T>,
			pid: u64,
			payout_commission: Option<Permill>,
		) -> DispatchResult {
			let owner = ensure_signed(origin.clone())?;
			let pool_info = ensure_vault::<T>(pid)?;
			// origin must be owner of pool
			ensure!(
				pool_info.basepool.owner == owner,
				Error::<T>::UnauthorizedPoolOwner
			);

			if pool_info.commission == payout_commission {
				return Err(Error::<T>::CommissionNotChanged.into());
			}
			// Settle the shares anyway to ensure all the old commission is paid out
			Self::maybe_gain_owner_shares(origin, pid)?;
			// Reload the latest pool info after the settlement.
			let mut pool_info = ensure_vault::<T>(pid).expect("Pool is a known vault; qed.");
			pool_info.commission = payout_commission;
			base_pool::pallet::Pools::<T>::insert(pid, PoolProxy::Vault(pool_info));

			let mut commission: u32 = 0;
			if let Some(ratio) = payout_commission {
				commission = ratio.deconstruct();
			}
			Self::deposit_event(Event::<T>::VaultCommissionSet { pid, commission });

			Ok(())
		}

		/// Transfers some owner shares wrapped in a nft to the assigned account
		///
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		pub fn claim_owner_shares(
			origin: OriginFor<T>,
			vault_pid: u64,
			target: T::AccountId,
			shares: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let mut pool_info = ensure_vault::<T>(vault_pid)?;
			ensure!(
				who == pool_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			ensure!(
				pool_info.owner_shares >= shares,
				Error::<T>::NoEnoughShareToClaim
			);
			ensure!(shares > Zero::zero(), Error::<T>::NoRewardToClaim);
			let _nft_id = base_pool::Pallet::<T>::mint_nft(
				pool_info.basepool.cid,
				target.clone(),
				shares,
				vault_pid,
			)?;
			let _ = base_pool::Pallet::<T>::merge_nft_for_staker(
				pool_info.basepool.cid,
				target,
				pool_info.basepool.pid,
			)?;
			pool_info.owner_shares -= shares;
			wrapped_balances::Pallet::<T>::maybe_subscribe_to_pool(
				&who,
				vault_pid,
				pool_info.basepool.cid,
			)?;
			base_pool::pallet::Pools::<T>::insert(vault_pid, PoolProxy::Vault(pool_info));
			Self::deposit_event(Event::<T>::OwnerSharesClaimed {
				pid: vault_pid,
				user: who,
				shares,
			});

			Ok(())
		}

		/// Tries to settle owner shares if the vault profits
		///
		/// The mechanism of issuing shares to distribute owner reward is metioned in comments of struct `Vault` in poolproxy.rs
		///
		/// Requires:
		/// 1. The sender is the owner
		#[pallet::call_index(3)]
		#[pallet::weight({0})]
		pub fn maybe_gain_owner_shares(origin: OriginFor<T>, vault_pid: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let pool_info = ensure_vault::<T>(vault_pid)?;
			ensure!(
				who == pool_info.basepool.owner,
				Error::<T>::UnauthorizedPoolOwner
			);
			Self::do_gain_owner_share(vault_pid)
		}

		/// Let any user to launch a vault withdraw. Then check if the vault need to be forced withdraw all its contributions.
		///
		/// If the shutdown condition is met, all shares owned by the vault will be forced withdraw.
		/// Note: This function doesn't guarantee no-op when there's error.
		#[pallet::call_index(4)]
		#[pallet::weight({0})]
		#[frame_support::transactional]
		pub fn check_and_maybe_force_withdraw(
			origin: OriginFor<T>,
			vault_pid: u64,
		) -> DispatchResult {
			ensure_signed(origin.clone())?;
			let mut vault = ensure_vault::<T>(vault_pid)?;
			// Try to process withdraw queue with the free token.
			// Unlock the vault if the withdrawals are clear.
			base_pool::Pallet::<T>::try_process_withdraw_queue(&mut vault.basepool);
			if vault.basepool.withdraw_queue.is_empty() {
				VaultLocks::<T>::remove(vault_pid);
			}
			base_pool::pallet::Pools::<T>::insert(vault_pid, PoolProxy::Vault(vault.clone()));

			// Trigger force withdrawal and lock if there's any expired withdrawal.
			// If already locked, we don't need to trigger it again.
			if VaultLocks::<T>::contains_key(vault_pid) {
				return Ok(());
			}
			// Case 1: There's a request over 3x GracePeriod old, the pool must be shutdown.
			let mut shutdown_reason: Option<ForceShutdownReason> = None;
			let now = <T as registry::Config>::UnixTime::now()
				.as_secs()
				.saturated_into::<u64>();
			let grace_period = T::GracePeriod::get();
			if let Some(withdraw) = vault.basepool.withdraw_queue.front() {
				if withdraw.start_time + 3 * grace_period < now {
					shutdown_reason = Some(ForceShutdownReason::Waiting3xGracePeriod);
				}
			}
			// Case 2: There's no enough releasing stake to address the expired withdrawals.
			// Sum up the releasing stake
			if shutdown_reason.is_none() {
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
						let nft_guard = base_pool::Pallet::<T>::get_nft_attr_guard(
							stake_pool.basepool.cid,
							withdraw.nft_id,
						)?;
						let price = stake_pool
							.basepool
							.share_price()
							.ok_or(Error::<T>::VaultBankrupt)?;
						releasing_stake += bmul(nft_guard.attr.shares, &price);
					}
				}
				// Check if the releasing stake can cover the expired withdrawals
				if base_pool::Pallet::<T>::has_expired_withdrawal(
					&vault.basepool,
					now,
					grace_period,
					Some(T::VaultQueuePeriod::get()),
					releasing_stake,
				) {
					shutdown_reason = Some(ForceShutdownReason::NoEnoughReleasingStake);
				}
			}
			let Some(shutdown_reason) = shutdown_reason else {
				// No need to shutdown.
				return Ok(());
			};
			// Try to withdraw from the upstream stake pools
			for pid in vault.invest_pools.iter() {
				let stake_pool = ensure_stake_pool::<T>(*pid)?;
				let mut total_shares = Zero::zero();
				let withdraw_vec: VecDeque<_> = stake_pool
					.basepool
					.withdraw_queue
					.iter()
					.filter(|x| x.user == vault.basepool.pool_account_id)
					.collect();
				// the length of vec should be 1
				for withdraw in withdraw_vec {
					let nft_guard = base_pool::Pallet::<T>::get_nft_attr_guard(
						stake_pool.basepool.cid,
						withdraw.nft_id,
					)?;
					total_shares += nft_guard.attr.shares;
				}
				pallet_uniques::Pallet::<T>::owned_in_collection(
					&stake_pool.basepool.cid,
					&vault.basepool.pool_account_id,
				)
				.for_each(|nftid| {
					let property_guard =
						base_pool::Pallet::<T>::get_nft_attr_guard(stake_pool.basepool.cid, nftid)
							.expect("get nft should not fail: qed.");
					let property = &property_guard.attr;
					if !base_pool::is_nondust_balance(property.shares) {
						let _ = base_pool::Pallet::<T>::burn_nft(
							&base_pool::pallet_id::<T::AccountId>(),
							stake_pool.basepool.cid,
							nftid,
						);
						return;
					}
					total_shares += property.shares;
				});
				if !base_pool::is_nondust_balance(total_shares) {
					continue;
				}
				stake_pool_v2::Pallet::<T>::withdraw(
					Origin::<T>::Signed(vault.basepool.owner.clone()).into(),
					stake_pool.basepool.pid,
					total_shares,
					Some(vault_pid),
				)?;
			}
			VaultLocks::<T>::insert(vault_pid, ());
			Self::deposit_event(Event::<T>::ForceShutdown {
				pid: vault_pid,
				reason: shutdown_reason,
			});
			Ok(())
		}

		/// Contributes some stake to a vault
		///
		/// Requires:
		/// 1. The pool exists
		/// 2. After the deposit, the pool doesn't reach the cap
		#[pallet::call_index(5)]
		#[pallet::weight({0})]
		#[frame_support::transactional]
		pub fn contribute(origin: OriginFor<T>, pid: u64, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let a = amount; // Alias to reduce confusion in the code below
			ensure!(
				a >= T::MinContribution::get(),
				Error::<T>::InsufficientContribution
			);
			let free = pallet_assets::Pallet::<T>::maybe_balance(
				<T as wrapped_balances::Config>::WPhaAssetId::get(),
				&who,
			)
			.ok_or(Error::<T>::AssetAccountNotExist)?;
			ensure!(free >= a, Error::<T>::InsufficientBalance);

			// Trigger owner reward share distribution before contribution to ensure no harm to the
			// contributor.
			Self::do_gain_owner_share(pid)?;
			let mut pool_info = ensure_vault::<T>(pid)?;

			let shares =
				base_pool::Pallet::<T>::contribute(&mut pool_info.basepool, who.clone(), amount)?;

			// We have new free stake now, try to handle the waiting withdraw queue
			base_pool::Pallet::<T>::try_process_withdraw_queue(&mut pool_info.basepool);

			// Persist
			base_pool::pallet::Pools::<T>::insert(pid, PoolProxy::Vault(pool_info.clone()));
			base_pool::Pallet::<T>::merge_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
				pool_info.basepool.pid,
			)?;

			wrapped_balances::Pallet::<T>::maybe_subscribe_to_pool(
				&who,
				pid,
				pool_info.basepool.cid,
			)?;

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
		/// Once a withdraw request is proceeded successfully, The withdrawal would be queued and waiting to be dealed.
		/// Afer the withdrawal is queued, The withdraw queue will be automaticly consumed util there are not enough free stakes to fullfill withdrawals.
		/// Everytime the free stakes in the pools increases, the withdraw queue will be consumed as it describes above.
		#[pallet::call_index(6)]
		#[pallet::weight({0})]
		#[frame_support::transactional]
		pub fn withdraw(origin: OriginFor<T>, pid: u64, shares: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Trigger owner reward share distribution before withdrawal to ensure no harm to the
			// pool owner.
			Self::do_gain_owner_share(pid)?;
			let mut pool_info = ensure_vault::<T>(pid)?;

			let maybe_nft_id = base_pool::Pallet::<T>::merge_nft_for_staker(
				pool_info.basepool.cid,
				who.clone(),
				pool_info.basepool.pid,
			)?;
			let nft_id = maybe_nft_id.ok_or(Error::<T>::NoNftToWithdraw)?;
			// The nft instance must be wrote to Nft storage at the end of the function
			// this nft's property shouldn't be accessed or wrote again from storage before set_nft_attr
			// is called. Or the property of the nft will be overwrote incorrectly.
			let mut nft_guard =
				base_pool::Pallet::<T>::get_nft_attr_guard(pool_info.basepool.cid, nft_id)?;
			let nft = &mut nft_guard.attr;
			let in_queue_shares = match pool_info
				.basepool
				.withdraw_queue
				.iter()
				.find(|&withdraw| withdraw.user == who)
			{
				Some(withdraw) => {
					let withdraw_nft_guard = base_pool::Pallet::<T>::get_nft_attr_guard(
						pool_info.basepool.cid,
						withdraw.nft_id,
					)
					.expect("get nftattr should always success; qed.");
					withdraw_nft_guard.attr.shares
				}
				None => Zero::zero(),
			};
			ensure!(
				base_pool::is_nondust_balance(shares) && (shares <= nft.shares + in_queue_shares),
				Error::<T>::NoEnoughShareToClaim
			);
			base_pool::Pallet::<T>::try_withdraw(
				&mut pool_info.basepool,
				nft,
				who.clone(),
				shares,
				nft_id,
				None,
			)?;

			nft_guard.save()?;
			let _nft_id = base_pool::Pallet::<T>::merge_nft_for_staker(
				pool_info.basepool.cid,
				who,
				pool_info.basepool.pid,
			)?;
			base_pool::pallet::Pools::<T>::insert(pid, PoolProxy::Vault(pool_info));

			Ok(())
		}

		// Reserved: #[pallet::call_index(7)]
	}

	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert + Display,
		T: pallet_rmrk_core::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32, Balance = BalanceOf<T>>,
	{
		/// Triggers owner reward share distribution
		///
		/// Note 1: This function does mutate the pool info. After calling this function, the caller
		/// must read the pool info again if it's accessed.
		///
		/// Note 2: This function guarantees no-op when it returns error.
		fn do_gain_owner_share(vault_pid: u64) -> DispatchResult {
			let mut pool_info = ensure_vault::<T>(vault_pid)?;
			let current_price = match pool_info.basepool.share_price() {
				Some(price) => BalanceOf::<T>::from_fixed(&price),
				None => return Ok(()),
			};
			if pool_info.last_share_price_checkpoint == Zero::zero() {
				pool_info.last_share_price_checkpoint = current_price;
				base_pool::pallet::Pools::<T>::insert(vault_pid, PoolProxy::Vault(pool_info));
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
			pool_info.last_share_price_checkpoint = new_price;

			base_pool::pallet::Pools::<T>::insert(vault_pid, PoolProxy::Vault(pool_info));
			Self::deposit_event(Event::<T>::OwnerSharesGained {
				pid: vault_pid,
				shares: adjust_shares,
				checkout_price: new_price,
			});

			Ok(())
		}
	}
}
