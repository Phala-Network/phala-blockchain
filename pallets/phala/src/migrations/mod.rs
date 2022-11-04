use crate::balance_convert::mul;
#[cfg(not(feature = "std"))]
use alloc::format;
#[allow(unused_imports)]
use frame_support::{
	traits::{
		tokens::fungibles::{Inspect, Mutate},
		Currency,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, LockIdentifier, LockableCurrency, StorageVersion,
	},
	weights::Weight,
	BoundedVec, Twox64Concat,
};
use frame_system::Origin;
#[allow(unused_imports)]
use log;
use sp_runtime::traits::Zero;
use sp_std::{collections::vec_deque::VecDeque, vec};
#[cfg(feature = "std")]
use std::format;

use rmrk_traits::primitives::{CollectionId, NftId};

use crate::compute::{basepool, computation, pawnshop, poolproxy, stakepoolv2, vault};
use crate::fat;
use crate::mq;
use crate::registry;
use crate::stakepool;
use crate::utils::balance_convert;
use crate::{BalanceOf, PhalaConfig};

/// Alias for the runtime that implements all Phala Pallets
pub trait PhalaPallets:
	fat::Config
	+ frame_system::Config
	+ computation::Config
	+ mq::Config
	+ registry::Config
	+ stakepoolv2::Config
	+ basepool::Config
	+ vault::Config
	+ crate::PhalaConfig
{
}
impl<T> PhalaPallets for T where
	T: fat::Config
		+ frame_system::Config
		+ computation::Config
		+ mq::Config
		+ registry::Config
		+ stakepoolv2::Config
		+ basepool::Config
		+ vault::Config
		+ pawnshop::Config
		+ crate::PhalaConfig
{
}

type Versions = (
	StorageVersion,
	StorageVersion,
	StorageVersion,
	StorageVersion,
	StorageVersion,
);

#[allow(dead_code)]
fn get_versions<T: PhalaPallets>() -> Versions {
	(
		StorageVersion::get::<fat::Pallet<T>>(),
		StorageVersion::get::<computation::Pallet<T>>(),
		StorageVersion::get::<mq::Pallet<T>>(),
		StorageVersion::get::<registry::Pallet<T>>(),
		StorageVersion::get::<stakepoolv2::Pallet<T>>(),
	)
}

#[allow(dead_code)]
fn unified_versions<T: PhalaPallets>(version: u16) -> Versions {
	(
		StorageVersion::new(version),
		StorageVersion::new(version),
		StorageVersion::new(version),
		StorageVersion::new(version),
		StorageVersion::new(version),
	)
}

#[allow(dead_code)]
fn set_unified_version<T: PhalaPallets>(version: u16) {
	StorageVersion::new(version).put::<fat::Pallet<T>>();
	StorageVersion::new(version).put::<computation::Pallet<T>>();
	StorageVersion::new(version).put::<mq::Pallet<T>>();
	StorageVersion::new(version).put::<registry::Pallet<T>>();
	StorageVersion::new(version).put::<stakepoolv2::Pallet<T>>();
}

pub mod v6 {
	use super::*;

	#[cfg(feature = "try-runtime")]
	pub fn pre_migrate<T: PhalaPallets>() -> Result<(), &'static str> {
		frame_support::ensure!(
			get_versions::<T>() == unified_versions::<T>(5),
			"incorrect pallet versions"
		);
		Ok(())
	}

	pub fn migrate<T>() -> Weight
	where
		T: PhalaPallets,
		BalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		if get_versions::<T>() == unified_versions::<T>(5) {
			let mut weight: Weight = Weight::zero();
			log::info!("Ᵽ migrating phala-pallets to v6");
			weight += stakepoolv2::Pallet::<T>::migration_remove_assignments();
			log::info!("Ᵽ pallets migrated to v6");

			set_unified_version::<T>(6);
			weight += T::DbWeight::get().reads_writes(5, 5);
			weight
		} else {
			T::DbWeight::get().reads(5)
		}
	}

	#[cfg(feature = "try-runtime")]
	pub fn post_migrate<T: PhalaPallets>() -> Result<(), &'static str> {
		frame_support::ensure!(
			get_versions::<T>() == unified_versions::<T>(6),
			"incorrect pallet versions postmigrate"
		);
		log::info!("Ᵽ phala pallet migration passes POST migrate checks ✅",);
		Ok(())
	}
}

pub mod stakepoolv2_migration {
	use super::*;

	const STAKING_ID: LockIdentifier = *b"phala/sp";
	const VESTING_ID: LockIdentifier = *b"vesting ";
	const DEMOCRAC_ID: LockIdentifier = *b"democrac";
	const PHRELECT_ID: LockIdentifier = *b"phrelect";
	fn migrate_stake_pools<T: PhalaPallets>()
	where
		T: PhalaPallets + stakepool::pallet::Config + PhalaConfig,
		BalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		let pawnshop_accountid = <T as pawnshop::Config>::PawnShopAccountId::get();
		stakepool::pallet::StakePools::<T>::iter().for_each(|(pid, pool_info)| {
			let collection_id: CollectionId = pallet_rmrk_core::Pallet::<T>::collection_index();
			let symbol: BoundedVec<u8, <T as pallet_rmrk_core::Config>::CollectionSymbolLimit> =
				format!("STAKEPOOL-{}", pid)
					.as_bytes()
					.to_vec()
					.try_into()
					.expect("create a bvec from string should never fail; qed.");
			pallet_rmrk_core::Pallet::<T>::create_collection(
				Origin::<T>::Signed(basepool::pallet_id::<T::AccountId>()).into(),
				Default::default(),
				None,
				symbol,
			)
			.expect("create collection should not fail; qed.");
			let account_id = basepool::pallet::generate_staker_account::<T::AccountId>(
				pid,
				pool_info.owner.clone(),
			);
			let (owner_reward_account, lock_account) =
				stakepoolv2::pallet::generate_owner_and_lock_account::<T::AccountId>(
					pid,
					pool_info.owner.clone(),
				);
			let mut new_pool_info = poolproxy::StakePool {
				basepool: basepool::BasePool {
					pid,
					owner: pool_info.owner,
					total_shares: pool_info.total_shares,
					total_value: pool_info.total_stake,
					withdraw_queue: VecDeque::new(),
					value_subscribers: VecDeque::new(),
					cid: collection_id,
					pool_account_id: account_id,
				},
				payout_commission: pool_info.payout_commission,
				cap: pool_info.cap,
				workers: VecDeque::new(),
				cd_workers: VecDeque::new(),
				lock_account,
				owner_reward_account,
			};
			// TODO(mingxuan): Will include removed workers after we measure how to get their pool-belonging.
			pool_info.workers.into_iter().for_each(|pubkey| {
				new_pool_info.workers.push_back(pubkey);
				let session: T::AccountId = stakepoolv2::pool_sub_account(pid, &pubkey);
				let worker_info = computation::pallet::Sessions::<T>::get(&session)
					.expect("session data should exist; qed.");
				if worker_info.state == computation::pallet::WorkerState::WorkerCoolingDown {
					new_pool_info.cd_workers.push_back(pubkey);
				}
				let computing_stake = computation::Stakes::<T>::get(&session).unwrap_or_default();
				pawnshop::Pallet::<T>::mint_into(
					&new_pool_info.owner_reward_account,
					computing_stake,
				)
				.expect("mint into should be success");
			});
			pool_info
				.withdraw_queue
				.into_iter()
				.for_each(|withdraw_info| {
					let nft_id = basepool::Pallet::<T>::mint_nft(
						collection_id,
						basepool::pallet::pallet_id(),
						withdraw_info.shares,
					)
					.expect("mint nft should always success");
					new_pool_info
						.basepool
						.withdraw_queue
						.push_back(basepool::WithdrawInfo {
							user: withdraw_info.user,
							start_time: withdraw_info.start_time,
							nft_id,
						});
				});
			// If the balance is too low to mint, we can just drop it.
			if !pawnshop::Pallet::<T>::mint_into(
				&new_pool_info.owner_reward_account,
				pool_info.owner_reward,
			).is_err() {
				let _ = computation::Pallet::<T>::withdraw_subsidy_pool(
					&pawnshop_accountid,
					pool_info.owner_reward,
				);
			};
			pawnshop::Pallet::<T>::mint_into(
				&new_pool_info.basepool.pool_account_id,
				pool_info.free_stake,
			);
			basepool::pallet::Pools::<T>::insert(
				pid,
				poolproxy::PoolProxy::StakePool(new_pool_info),
			);
		});
		basepool::PoolCount::<T>::put(stakepool::PoolCount::<T>::get());
	}

	fn migrate_pool_stakers<T: PhalaPallets>()
	where
		T: PhalaPallets + stakepool::pallet::Config,
		BalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		let pawnshop_accountid = <T as pawnshop::Config>::PawnShopAccountId::get();
		stakepool::pallet::PoolStakers::<T>::drain().for_each(|((pid, user_id), staker_info)| {
			let mut account_status = pawnshop::StakerAccounts::<T>::get(&user_id).unwrap_or_default();
			let deprecated_pool = stakepool::pallet::StakePools::<T>::get(pid)
				.expect("get deprecated pool should success; qed.");
			let pending_reward = mul(staker_info.shares, &deprecated_pool.reward_acc.into())
				- staker_info.reward_debt;
			let user_reward = pending_reward + staker_info.available_rewards;
			let _ = computation::Pallet::<T>::withdraw_subsidy_pool(
				&pawnshop_accountid,
				user_reward,
			);
			let pool_info =
				poolproxy::ensure_stake_pool::<T>(pid).expect("stakepool should exist; qed.");
			// If the balance is too low to mint, we can just drop it.
			// Even if user_reward is a dust, we should still create user's shares
			pawnshop::Pallet::<T>::mint_into(&pool_info.basepool.pool_account_id, user_reward);
			basepool::Pallet::<T>::mint_nft(pool_info.basepool.cid, user_id.clone(), staker_info.shares);
			if !account_status
				.invest_pools
				.contains(&(pid, pool_info.basepool.cid))
			{
				account_status
					.invest_pools
					.push((pid, pool_info.basepool.cid));
			}
			pawnshop::StakerAccounts::<T>::insert(user_id, account_status);

		});
		stakepool::pallet::StakePools::<T>::drain().for_each(|(pid, pool_info)| {});
	}

	fn migrate_stake_ledger<T: PhalaPallets>()
	where
		T: PhalaPallets + stakepool::pallet::Config + PhalaConfig,
		BalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		let pawnshop_accountid = <T as pawnshop::Config>::PawnShopAccountId::get();
		stakepool::pallet::StakeLedger::<T>::drain().for_each(|(user_id, balance)| {
			<T as PhalaConfig>::Currency::remove_lock(STAKING_ID, &user_id);
			<T as PhalaConfig>::Currency::remove_lock(VESTING_ID, &user_id);
			<T as PhalaConfig>::Currency::remove_lock(DEMOCRAC_ID, &user_id);
			<T as PhalaConfig>::Currency::remove_lock(PHRELECT_ID, &user_id);
			if <T as PhalaConfig>::Currency::transfer(
				&user_id,
				&pawnshop_accountid,
				balance,
				KeepAlive,
			)
			.is_err()
			{
				// TODO(mingxuan): switch the currency sending account to a identical account before final push.
				<T as PhalaConfig>::Currency::transfer(
					&computation::Pallet::<T>::account_id(),
					&user_id,
					<T as PhalaConfig>::Currency::minimum_balance(),
					KeepAlive,
				);
				<T as PhalaConfig>::Currency::transfer(
					&user_id,
					&pawnshop_accountid,
					balance,
					KeepAlive,
				)
				.expect("transfer should not fail; qed.");
			}
		});
	}

	fn migrate_unchanged_stuffs<T: PhalaPallets>()
	where
		T: PhalaPallets + stakepool::pallet::Config + PhalaConfig,
		BalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		stakepool::pallet::WorkerAssignments::<T>::drain().for_each(|(k, v)| {
			stakepoolv2::pallet::WorkerAssignments::<T>::insert(k, v);
		});
		stakepool::pallet::SubAccountPreimages::<T>::drain().for_each(|(k, v)| {
			stakepoolv2::pallet::SubAccountPreimages::<T>::insert(k, v);
		});
		stakepool::pallet::PoolContributionWhitelists::<T>::drain().for_each(|(k, v)| {
			basepool::pallet::PoolContributionWhitelists::<T>::insert(k, v);
		});
		stakepool::pallet::PoolDescriptions::<T>::drain().for_each(|(k, v)| {
			basepool::pallet::PoolDescriptions::<T>::insert(k, v);
		});
	}

	#[cfg(feature = "try-runtime")]
	pub fn pre_migrate<T: PhalaPallets>() -> Result<(), &'static str> {
		frame_support::ensure!(
			get_versions::<T>() == unified_versions::<T>(6),
			"incorrect pallet versions"
		);
		Ok(())
	}

	pub fn migrate<T>() -> Weight
	where
		T: PhalaPallets + stakepool::pallet::Config + PhalaConfig,
		BalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,
	{
		if get_versions::<T>() == unified_versions::<T>(6) {
			let mut weight: Weight = Weight::zero();
			log::info!("Ᵽ migrating phala-pallets to v7");
			weight += stakepoolv2::Pallet::<T>::migration_remove_assignments();
			log::info!("Ᵽ pallets migrated to v7");

			migrate_stake_pools::<T>();
			migrate_pool_stakers::<T>();
			migrate_stake_ledger::<T>();
			migrate_unchanged_stuffs::<T>();
			set_unified_version::<T>(7);
			// TODO(mingxuan): Do benchmarking before final push.
			weight += T::DbWeight::get().reads_writes(5, 5);
			weight
		} else {
			T::DbWeight::get().reads(5)
		}
	}

	#[cfg(feature = "try-runtime")]
	pub fn post_migrate<T: PhalaPallets>() -> Result<(), &'static str> {
		frame_support::ensure!(
			get_versions::<T>() == unified_versions::<T>(7),
			"incorrect pallet versions postmigrate"
		);
		log::info!("Ᵽ phala pallet migration passes POST migrate checks ✅",);
		Ok(())
	}
}
