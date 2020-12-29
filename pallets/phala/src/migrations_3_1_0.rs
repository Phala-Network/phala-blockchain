//! Migrate to version [`3.0.0`]
//!
//! Referece PR: <https://github.com/paritytech/substrate/pull/7040>

use alloc::vec::Vec;
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
pub trait V30ToV31 {
	type Module: GetPalletVersion;
	type AccountId: 'static + FullCodec + Clone;
	type BlockNumber: 'static + FullCodec;
}

// Pallet: PhalaModule
struct __PhalaPallet;
struct __PhalaPalletInfo;
impl frame_support::traits::PalletInfo for __PhalaPalletInfo {
	fn index<P: 'static>() -> Option<usize> { Some(0) }	// Not used; will remove in the future
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
type WorkerState<T: V30ToV31> = StorageMap<
	__WorkerState, Blake2_128Concat,
	T::AccountId, WorkerInfo<T::BlockNumber>
>;

// State: MachineOwner
struct __MachineOwner;
impl frame_support::traits::StorageInstance for __MachineOwner {
	type Pallet = __PhalaPallet;
	type PalletInfo = __PhalaPalletInfo;
	const STORAGE_PREFIX: &'static str = "MachineOwner";
}
#[allow(type_alias_bounds)]
type MachineOwner<T: V30ToV31> = StorageMap<
	__MachineOwner, Blake2_128Concat,
	Vec<u8>, T::AccountId
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

// State: PendingUpdate
struct __PendingUpdate;
impl frame_support::traits::StorageInstance for __PendingUpdate {
	type Pallet = __PhalaPallet;
	type PalletInfo = __PhalaPalletInfo;
	const STORAGE_PREFIX: &'static str = "PendingUpdate";
}
#[allow(type_alias_bounds)]
type PendingUpdate<T: V30ToV31> = StorageValue<__PendingUpdate, Vec<T::AccountId>>;

fn reconcile_worker_state<T: V30ToV31>() {
	debug::info!("Entered `reconcile_worker_state`");
	// Remove all corrupted worker entries
	WorkerState::<T>::translate(
		|_k, v: WorkerInfo<T::BlockNumber>| -> Option<WorkerInfo<T::BlockNumber>>
	{
		if v.machine_id.len() == 0 { None }
		else { Some(v) }
	});
	// Rebuild index: MachineOwner
	MachineOwner::<T>::drain().for_each(drop);
	let mut to_remove = Vec::<T::AccountId>::new();
	for (k, v) in WorkerState::<T>::iter() {
		if v.machine_id.is_empty() {
			to_remove.push(k);
		} else if MachineOwner::<T>::contains_key(&v.machine_id) {
			to_remove.push(k);
		} else {
			MachineOwner::<T>::insert(&v.machine_id, &k);
		}
	}
	for stash in to_remove.iter() {
		WorkerState::<T>::remove(&stash);
	}
	// Rebuild index: cleanup PendingUpdate
	let updates = PendingUpdate::<T>::get();
	if let Some(updates) = updates {
		let filtered: Vec<T::AccountId> = updates
			.iter()
			.filter(|stash| WorkerState::<T>::contains_key(stash))
			.cloned()
			.collect();
		debug::info!(
			"  - PendingUpdate filtered: before = {}, after = {}",
			updates.len(), filtered.len());
		PendingUpdate::<T>::put(filtered);
	}
	// Rebuild index: OnlineWorkers, TotalPower
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
	// Rebuild index: clean PendingExitingDelta
	let exit_delta = PendingExitingDelta::take().unwrap_or_default();
	debug::info!("  - exit_delta.num_worker = {}", exit_delta.num_worker);
	debug::info!("  - exit_delta.num_power = {}", exit_delta.num_power);
	debug::info!("  - online_workers = {}", online_workers);
	debug::info!("  - total_power = {}", total_power);
	OnlineWorkers::put((online_workers as i32 + exit_delta.num_worker) as u32);
	TotalPower::put((total_power as i32 + exit_delta.num_power) as u32);
	debug::info!("End `reconcile_worker_state`");
}

pub fn apply<T: V30ToV31>() -> Weight {
	let maybe_storage_version = <T::Module as GetPalletVersion>::storage_version();
	match maybe_storage_version {
		Some(version) if version == PalletVersion::new(3, 0, 0) => {
			debug::RuntimeLogger::init();
			reconcile_worker_state::<T>();
			Weight::max_value()
		}
		_ => 0,
	}
}
