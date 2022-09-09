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

fn set_unified_version<T: PhalaPallets>(version: u16) {
	StorageVersion::new(version).put::<fat::Pallet<T>>();
	StorageVersion::new(version).put::<mining::Pallet<T>>();
	StorageVersion::new(version).put::<mq::Pallet<T>>();
	StorageVersion::new(version).put::<registry::Pallet<T>>();
	StorageVersion::new(version).put::<stakepool::Pallet<T>>();
}
