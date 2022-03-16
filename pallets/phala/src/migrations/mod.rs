/// Migrations to v4
///
/// From v4, all the storage version will become 4 at once. Before v4, the latest storage versions
/// are:
/// - fat: 1
/// - mining: 3
/// - mq: 0
/// - ott: 1
/// - registry: 1
/// - stakepool: 1
pub mod v4 {
	use crate::*;
	use frame_support::{
		traits::{Get, StorageVersion},
		weights::Weight,
	};
	use log;

	type Versions = (
		StorageVersion,
		StorageVersion,
		StorageVersion,
		StorageVersion,
		StorageVersion,
	);

	const EXPECTED_STORAGE_VERSION: Versions = (
		StorageVersion::new(1),
		StorageVersion::new(3),
		StorageVersion::new(0),
		StorageVersion::new(1),
		StorageVersion::new(1),
	);

	const FINAL_STORAGE_VERSION: Versions = (
		StorageVersion::new(4),
		StorageVersion::new(4),
		StorageVersion::new(4),
		StorageVersion::new(4),
		StorageVersion::new(4),
	);

	fn get_versions<T>() -> Versions
	where
		T: fat::Config + mining::Config + mq::Config + registry::Config + stakepool::Config,
	{
		(
			StorageVersion::get::<fat::Pallet<T>>(),
			StorageVersion::get::<mining::Pallet<T>>(),
			StorageVersion::get::<mq::Pallet<T>>(),
			StorageVersion::get::<registry::Pallet<T>>(),
			StorageVersion::get::<stakepool::Pallet<T>>(),
		)
	}

	#[cfg(feature = "try-runtime")]
	pub fn pre_migrate<T>() -> Result<(), &'static str>
	where
		T: fat::Config + mining::Config + mq::Config + registry::Config + stakepool::Config,
	{
		frame_support::ensure!(
			get_versions::<T>() == EXPECTED_STORAGE_VERSION,
			"incorrect pallet versions"
		);
		Ok(())
	}

	pub fn migrate<T>() -> Weight
	where
		T: fat::Config + mining::Config + mq::Config + registry::Config + stakepool::Config,
	{
		if get_versions::<T>() == EXPECTED_STORAGE_VERSION {
			let mut weight: Weight = 0;
			log::info!("Ᵽ migrating phala-pallets to v4");
			weight += mining::migrations::fix_676::<T>();
			log::info!("Ᵽ pallets migrated to v4");

			StorageVersion::new(4).put::<fat::Pallet<T>>();
			StorageVersion::new(4).put::<mining::Pallet<T>>();
			StorageVersion::new(4).put::<mq::Pallet<T>>();
			StorageVersion::new(4).put::<registry::Pallet<T>>();
			StorageVersion::new(4).put::<stakepool::Pallet<T>>();
			weight += T::DbWeight::get().writes(5);
			weight
		} else {
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	pub fn post_migrate<T>() -> Result<(), &'static str>
	where
		T: fat::Config + mining::Config + mq::Config + registry::Config + stakepool::Config,
	{
		frame_support::ensure!(
			get_versions::<T>() == FINAL_STORAGE_VERSION,
			"incorrect pallet versions postmigrate"
		);
		log::info!("Ᵽ phala pallet migration passes POST migrate checks ✅",);
		Ok(())
	}
}
