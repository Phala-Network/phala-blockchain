//! Migrate to version [`3.0.0`]
//!
//! Referece PR: <>

use codec::FullCodec;
use frame_support::{
	Blake2_128Concat,
	debug,
	storage::types::{StorageMap, StorageValue},
	traits::{GetPalletVersion, PalletVersion},
	weights::Weight,
};

use types::{
	WorkerStateEnum,
	WorkerInfo,
	MinerStatsDelta,
};

/// Trait to implement to give information about types used for migration
pub trait V2ToV3 {
	type Module: GetPalletVersion;
	type AccountId: 'static + FullCodec;
	type BlockNumber: 'static + FullCodec;
}

// Pallet: PhalaModule
struct __PhalaPallet;
struct __PhalaPalletInfo;
impl frame_support::traits::PalletInfo for __PhalaPalletInfo {
	fn index<P: 'static>() -> Option<usize> { Some(0) }
	fn name<P: 'static>() -> Option<&'static str> { Some("PhalaModule") }
}

// State: WorkerState
struct __WorkerState;
impl frame_support::traits::StorageInstance for __WorkerState {
	type Pallet = __PhalaPallet;
	type PalletInfo = __PhalaPalletInfo;
	const STORAGE_PREFIX: &'static str = "WorkerState";
}
#[allow(type_alias_bounds)]
type WorkerState<T: V2ToV3> = StorageMap<
	__WorkerState, Blake2_128Concat,
	T::AccountId, WorkerInfo<T::BlockNumber>
>;

// State: OnlineWorkers
struct __OnlineWorkers;
impl frame_support::traits::StorageInstance for __OnlineWorkers {
	type Pallet = __PhalaPallet;
	type PalletInfo = __PhalaPalletInfo;
	const STORAGE_PREFIX: &'static str = "OnlineWorkers";
}
#[allow(type_alias_bounds)]
type OnlineWorkers = StorageValue<__OnlineWorkers, u32>;

// State: TotalPower
struct __TotalPower;
impl frame_support::traits::StorageInstance for __TotalPower {
	type Pallet = __PhalaPallet;
	type PalletInfo = __PhalaPalletInfo;
	const STORAGE_PREFIX: &'static str = "TotalPower";
}
#[allow(type_alias_bounds)]
type TotalPower = StorageValue<__TotalPower, u32>;

// State: PendingExitingDelta
struct __PendingExitingDelta;
impl frame_support::traits::StorageInstance for __PendingExitingDelta {
	type Pallet = __PhalaPallet;
	type PalletInfo = __PhalaPalletInfo;
	const STORAGE_PREFIX: &'static str = "PendingExitingDelta";
}
#[allow(type_alias_bounds)]
type PendingExitingDelta = StorageValue<__PendingExitingDelta, MinerStatsDelta>;

fn reconcile_online_workers<T: V2ToV3>() {
	debug::info!("Entered `reconcile_online_workers`");
	// Remove all corrupted worker entries
	WorkerState::<T>::translate(
		|_k, v: WorkerInfo<T::BlockNumber>| -> Option<WorkerInfo<T::BlockNumber>>
	{
		if v.machine_id.len() == 0 { None }
		else { Some(v) }
	});
	// Rescan all workers for the stats
	let mut total_power = 0u32;
	let online_workers = WorkerState::<T>::iter().filter(|(_k, v)|
		v.score.is_some() && match v.state {
			WorkerStateEnum::Mining(_) | WorkerStateEnum::MiningStopping => true,
			_ => false
		}
	).map(|(_k, v)| {
		let score = v.score.as_ref().unwrap().overall_score;
		total_power += score;
		()
	}).count();
	// Clean PendingExitingDelta at the same time
	let exit_delta = PendingExitingDelta::take().unwrap_or_default();
	debug::info!("  - exit_delta.num_worker = {}", exit_delta.num_worker);
	debug::info!("  - exit_delta.num_power = {}", exit_delta.num_power);
	debug::info!("  - online_workers = {}", online_workers);
	debug::info!("  - total_power = {}", total_power);
	OnlineWorkers::put((online_workers as i32 + exit_delta.num_worker) as u32);
	TotalPower::put((total_power as i32 + exit_delta.num_power) as u32);
	debug::info!("End `reconcile_online_workers`");
}

pub fn apply<T: V2ToV3>() -> Weight {
	debug::RuntimeLogger::init();

	let maybe_storage_version = <T::Module as GetPalletVersion>::storage_version();
	match maybe_storage_version {
		Some(version) if version == PalletVersion::new(2, 0, 0) => {
			reconcile_online_workers::<T>();
			Weight::max_value()
		}
		_ => 0,
	}
}
