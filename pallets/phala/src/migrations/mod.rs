#[allow(unused_imports)]
use frame_support::{
	traits::{Get, StorageVersion, ExistenceRequirement::{AllowDeath, KeepAlive}, tokens::fungibles::{Inspect, Mutate}, Currency,},
	weights::Weight,
	Twox64Concat,
	BoundedVec,
};
#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;
use crate::balance_convert::mul;
use sp_std::{collections::vec_deque::VecDeque, vec};
use frame_system::Origin;
use sp_runtime::traits::Zero;
#[allow(unused_imports)]
use log;

use rmrk_traits::primitives::{CollectionId, NftId};

use crate::compute::{basepool, computation, stakepoolv2, vault};
use crate::fat;
use crate::mq;
use crate::registry;
use crate::utils::balance_convert;
use crate::{BalanceOf, PhalaConfig};
use crate::stakepool;

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

	fn migrate_stake_pools<T : PhalaPallets>()
	where
		T: PhalaPallets + stakepool::pallet::Config + PhalaConfig,
		BalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,	
	{
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
			).expect("create collection should not fail; qed.");
			let account_id =
				basepool::pallet::create_staker_account::<T::AccountId>(pid, pool_info.owner.clone());
			let (owner_reward_account, lock_account) =
				stakepoolv2::pallet::create_owner_and_lock_account::<T::AccountId>(pid, pool_info.owner.clone());
			let mut new_pool_info = poolproxy::StakePool {
					basepool: basepool::BasePool {
						pid,
						owner: pool_info.owner.clone(),
						total_shares: pool_info.total_shares.clone(),
						total_value: pool_info.total_stake.clone(),
						withdraw_queue: VecDeque::new(),
						value_subscribers: VecDeque::new(),
						cid: collection_id,
						pool_account_id: account_id,
					},
					payout_commission: pool_info.payout_commission.clone(),
					cap: pool_info.cap.clone(),
					workers: VecDeque::new(),
					cd_workers: VecDeque::new(),
					lock_account,
					owner_reward_account,
				};
			pool_info.workers.into_iter().for_each(|pubkey| {
				new_pool_info.workers.push_back(pubkey);
				let session: T::AccountId = stakepoolv2::pool_sub_account(pid, &pubkey);
				let worker_info = computation::pallet::Sessions::<T>::get(&session).expect("session data should exist; qed.");
				if worker_info.state == computation::pallet::WorkerState::WorkerCoolingDown {
					new_pool_info.cd_workers.push_back(pubkey);
				}
				let computing_stake = computation::Stakes::<T>::get(&session).unwrap_or_default();
				pawnshop::Pallet::<T>::mint_into(&new_pool_info.owner_reward_account, computing_stake)
				.expect("mint into should be success");		
			});
			pool_info.withdraw_queue.into_iter().for_each(|withdraw_info| {
				let nft_id = basepool::Pallet::<T>::mint_nft(collection_id, basepool::pallet::pallet_id(), withdraw_info.shares)
					.expect("mint nft should always success");	
				new_pool_info.basepool.withdraw_queue.push_back(basepool::WithdrawInfo {
					user: withdraw_info.user,
					start_time: withdraw_info.start_time,
					nft_id,
				});
			});
			let _ = computation::Pallet::<T>::withdraw_subsidy_pool(
				&<T as pawnshop::Config>::PawnShopAccountId::get(),
				pool_info.owner_reward,
			);
			pawnshop::Pallet::<T>::mint_into(&new_pool_info.owner_reward_account, pool_info.owner_reward)
				.expect("mint into should be success");		
			pawnshop::Pallet::<T>::mint_into(&new_pool_info.basepool.pool_account_id, pool_info.free_stake)
				.expect("mint into should be success");		
			basepool::pallet::Pools::<T>::insert(
				pid,
				poolproxy::PoolProxy::StakePool(new_pool_info),
			);
		});
		basepool::PoolCount::<T>::put(stakepool::PoolCount::<T>::get());
	}

	fn migrate_pool_stakers<T : PhalaPallets>()
	where
		T: PhalaPallets + stakepool::pallet::Config,
		BalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,	
	{
		stakepool::pallet::PoolStakers::<T>::drain().for_each(|((pid, user_id), staker_info)| {
			let mut account_status = match pawnshop::StakerAccounts::<T>::get(&user_id) {
				Some(account_status) => account_status,
				None => pawnshop::FinanceAccount::<BalanceOf<T>> {
					invest_pools: vec![],
					locked: Zero::zero(),
				},
			};
			let deprecated_pool = stakepool::pallet::StakePools::<T>::get(pid).expect("get deprecated pool should success; qed.");
			let pending_reward = mul(staker_info.shares, &deprecated_pool.reward_acc.into()) - staker_info.reward_debt;
			let user_reward = pending_reward + staker_info.available_rewards;
			let _ = computation::Pallet::<T>::withdraw_subsidy_pool(
				&<T as pawnshop::Config>::PawnShopAccountId::get(),
				user_reward,
			);
			let pool_info = poolproxy::ensure_stake_pool::<T>(pid).expect("stakepool should exist; qed.");
			if !account_status.invest_pools.contains(&(pid, pool_info.basepool.cid)) {
				account_status.invest_pools.push((pid, pool_info.basepool.cid));
			}
			pawnshop::StakerAccounts::<T>::insert(user_id.clone(), account_status);
			pawnshop::Pallet::<T>::mint_into(&pool_info.basepool.pool_account_id, user_reward)
				.expect("mint into should be success");	
			basepool::Pallet::<T>::mint_nft(pool_info.basepool.cid, user_id, staker_info.shares)
				.expect("mint nft should always success");	
		});
	}

	fn migrate_stake_ledger<T : PhalaPallets>()
	where
		T: PhalaPallets + stakepool::pallet::Config + PhalaConfig,
		BalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
		T: pallet_assets::Config<AssetId = u32>,
		T: pallet_assets::Config<Balance = BalanceOf<T>>,	
	{
		stakepool::pallet::StakeLedger::<T>::drain().for_each(|(user_id, balance)| {
			<T as PhalaConfig>::Currency::transfer(
				&user_id,
				&<T as pawnshop::Config>::PawnShopAccountId::get(),
				balance,
				KeepAlive,
			).expect("transfer should not fail; qed.");
		});
	}

	fn migrate_unchanged_stuffs<T : PhalaPallets>()
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
			stakepoolv2::pallet::PoolContributionWhitelists::<T>::insert(k, v);
		});
		stakepool::pallet::PoolDescriptions::<T>::drain().for_each(|(k, v)| {
			stakepoolv2::pallet::PoolDescriptions::<T>::insert(k, v);
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
