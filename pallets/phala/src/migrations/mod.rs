use crate::*;
use frame_support::{
	traits::{Currency, Get, StorageVersion},
	weights::Weight,
};
use log;

type MiningBalanceOf<T> =
	<<T as mining::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Alias for the runtime that implements all Phala Pallets
pub trait PhalaPallets:
	fat::Config + mining::Config + mq::Config + registry::Config + stakepool::Config
{
}
impl<T> PhalaPallets for T where
	T: fat::Config + mining::Config + mq::Config + registry::Config + stakepool::Config
{
}

type Versions = (
	StorageVersion,
	StorageVersion,
	StorageVersion,
	StorageVersion,
	StorageVersion,
);

fn get_versions<T: PhalaPallets>() -> Versions {
	(
		StorageVersion::get::<fat::Pallet<T>>(),
		StorageVersion::get::<mining::Pallet<T>>(),
		StorageVersion::get::<mq::Pallet<T>>(),
		StorageVersion::get::<registry::Pallet<T>>(),
		StorageVersion::get::<stakepool::Pallet<T>>(),
	)
}

fn unified_versions<T: PhalaPallets>(version: u16) -> Versions {
	(
		StorageVersion::new(version),
		StorageVersion::new(version),
		StorageVersion::new(version),
		StorageVersion::new(version),
		StorageVersion::new(version),
	)
}

fn set_unified_versoin<T: PhalaPallets>(version: u16) {
	StorageVersion::new(version).put::<fat::Pallet<T>>();
	StorageVersion::new(version).put::<mining::Pallet<T>>();
	StorageVersion::new(version).put::<mq::Pallet<T>>();
	StorageVersion::new(version).put::<registry::Pallet<T>>();
	StorageVersion::new(version).put::<stakepool::Pallet<T>>();
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
		MiningBalanceOf<T>: balance_convert::FixedPointConvert + sp_std::fmt::Display,
		T: mining::pallet::Config<Currency = <T as stakepool::pallet::Config>::Currency>,
	{
		if get_versions::<T>() == unified_versions::<T>(5) {
			let mut weight: Weight = 0;
			log::info!("Ᵽ migrating phala-pallets to v6");
			weight += stakepool::Pallet::<T>::migration_remove_assignments();
			log::info!("Ᵽ pallets migrated to v6");

			set_unified_versoin::<T>(6);
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
