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
#[allow(unused_imports)]
use log;

use rmrk_traits::primitives::{CollectionId, NftId};

use crate::compute::{base_pool, computation, wrapped_balances, stake_pool_v2, vault};
use crate::fat;
use crate::mq;
use crate::registry;
use crate::utils::balance_convert;
use crate::BalanceOf;

/// Alias for the runtime that implements all Phala Pallets
pub trait PhalaPallets:
	fat::Config
	+ frame_system::Config
	+ computation::Config
	+ mq::Config
	+ registry::Config
	+ stake_pool_v2::Config
	+ base_pool::Config
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
		+ stake_pool_v2::Config
		+ base_pool::Config
		+ vault::Config
		+ wrapped_balances::Config
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
		StorageVersion::get::<stake_pool_v2::Pallet<T>>(),
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
	StorageVersion::new(version).put::<stake_pool_v2::Pallet<T>>();
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
			weight += stake_pool_v2::Pallet::<T>::migration_remove_assignments();
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
