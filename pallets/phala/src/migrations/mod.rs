#[allow(unused_imports)]
use frame_support::{
	traits::{Currency, Get, StorageVersion},
	weights::Weight,
};
#[allow(unused_imports)]
use log;

use sp_std::prelude::*;
use crate::*;

#[allow(dead_code)]
type MiningBalanceOf<T> =
	<<T as mining::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Alias for the runtime that implements all Phala Pallets
pub trait PhalaPallets:
	fat::Config + mining::Config + mq::Config + registry::Config + stakepool::Config + fat_tokenomic::Config
{}
impl<T> PhalaPallets for T where
	T: fat::Config + mining::Config + mq::Config + registry::Config + stakepool::Config + fat_tokenomic::Config
{}

type Versions = (
	StorageVersion,
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
		StorageVersion::get::<mining::Pallet<T>>(),
		StorageVersion::get::<mq::Pallet<T>>(),
		StorageVersion::get::<registry::Pallet<T>>(),
		StorageVersion::get::<stakepool::Pallet<T>>(),
		StorageVersion::get::<fat_tokenomic::Pallet<T>>(),
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
		StorageVersion::new(version),
	)
}

#[allow(dead_code)]
fn set_unified_version<T: PhalaPallets>(version: u16) {
	StorageVersion::new(version).put::<fat::Pallet<T>>();
	StorageVersion::new(version).put::<mining::Pallet<T>>();
	StorageVersion::new(version).put::<mq::Pallet<T>>();
	StorageVersion::new(version).put::<registry::Pallet<T>>();
	StorageVersion::new(version).put::<stakepool::Pallet<T>>();
	StorageVersion::new(version).put::<fat_tokenomic::Pallet<T>>();
}

pub mod registry_v6_migration {
	use codec::{Decode, Encode};
	use scale_info::TypeInfo;

	use phala_types::{WorkerPublicKey, EcdhPublicKey, AttestationProvider};
	use crate::registry::WorkerInfo;
	use super::*;

	#[derive(Encode, Decode, TypeInfo, Debug, Clone)]
	pub struct OldWorkerInfo<AccountId> {
		/// The identity public key of the worker
		pub pubkey: WorkerPublicKey,
		/// The public key for ECDH communication
		pub ecdh_pubkey: EcdhPublicKey,
		/// The pruntime version of the worker upon registering
		pub runtime_version: u32,
		/// The unix timestamp of the last updated time
		pub last_updated: u64,
		/// The stake pool owner that can control this worker
		///
		/// When initializing pruntime, the user can specify an _operator account_. Then this field
		/// will be updated when the worker is being registered on the blockchain. Once it's set,
		/// the worker can only be added to a stake pool if the pool owner is the same as the
		/// operator. It ensures only the trusted person can control the worker.
		pub operator: Option<AccountId>,
		/// The [confidence level](https://wiki.phala.network/en-us/mine/solo/1-2-confidential-level-evaluation/#confidence-level-of-a-miner)
		/// of the worker
		pub confidence_level: u8,
		/// The performance score by benchmark
		///
		/// When a worker is registered, this field is set to `None`, indicating the worker is
		/// requested to run a benchmark. The benchmark lasts [`BenchmarkDuration`] blocks, and
		/// this field will be updated with the score once it finishes.
		///
		/// The `initial_score` is used as the baseline for mining performance test.
		pub initial_score: Option<u32>,
		/// Deprecated
		pub features: Vec<u32>,
	}

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
			T: PhalaPallets + frame_system::Config
	{
		if get_versions::<T>() != unified_versions::<T>(5) {
			log::info!("No need to run the migration");
			return T::DbWeight::get().reads(1)
		}

		let mut count = 0;
		registry::pallet::Workers::<T>::translate(
			|
				_pubkey: WorkerPublicKey,
				old: OldWorkerInfo<<T as frame_system::Config>::AccountId>
			| {
				count += 1;

				Some(
					WorkerInfo {
						pubkey: old.pubkey,
						ecdh_pubkey: old.ecdh_pubkey,
						runtime_version: old.runtime_version,
						last_updated: old.last_updated,
						operator: old.operator,
						attestation_provider: Some(AttestationProvider::Ias),
						confidence_level: old.confidence_level,
						initial_score: old.initial_score,
						features: old.features
					}
				)
			}
		);

		set_unified_version::<T>(6);

		T::DbWeight::get().reads_writes(count, count)
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
