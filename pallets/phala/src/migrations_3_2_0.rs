//! Migrate to version [`3.2.0`]
//!
//! Referece PR: <https://github.com/paritytech/substrate/pull/7040>

use codec::{Decode, FullCodec};
use frame_support::{
    debug,
    storage::types::StorageMap,
    traits::{GetPalletVersion, PalletVersion},
    weights::Weight,
    Twox64Concat,
};

use types::RoundStats;

#[derive(Decode)]
pub struct RoundStatsV31 {
    pub round: u32,
    pub online_workers: u32,
    pub compute_workers: u32,
    /// The targeted online reward counts in fraction (base: 100_000)
    pub frac_target_online_reward: u32,
    pub total_power: u32,
}

/// Trait to implement to give information about types used for migration
pub trait V31ToV32 {
    type Module: GetPalletVersion;
    type AccountId: 'static + FullCodec + Clone;
    type BlockNumber: 'static + FullCodec;
}

// Pallet: PhalaModule
struct __PhalaPallet;
struct __PhalaPalletInfo;
impl frame_support::traits::PalletInfo for __PhalaPalletInfo {
    fn index<P: 'static>() -> Option<usize> {
        Some(0)
    } // Not used; will remove in the future
    fn name<P: 'static>() -> Option<&'static str> {
        Some("PhalaModule")
    }
}

// State: MachineOwner
struct __RoundStatsHistory;
impl frame_support::traits::StorageInstance for __RoundStatsHistory {
    type Pallet = __PhalaPallet;
    type PalletInfo = __PhalaPalletInfo;
    const STORAGE_PREFIX: &'static str = "RoundStatsHistory";
}

type RoundStatsHistory = StorageMap<__RoundStatsHistory, Twox64Concat, u32, RoundStats>;

fn reconcile_round_stats<T: V31ToV32>() {
    debug::info!("Entered `reconcile_round_stats`");
    // Remove all corrupted worker entries
    RoundStatsHistory::translate(|_k, v: RoundStatsV31| -> Option<RoundStats> {
        Some(RoundStats {
            round: v.round,
            online_workers: v.online_workers,
            compute_workers: v.compute_workers,
            frac_target_online_reward: v.frac_target_online_reward,
            total_power: v.total_power,
            frac_target_compute_reward: 0,
        })
    });
    debug::info!("End `reconcile_round_stats`");
}

pub fn apply<T: V31ToV32>() -> Weight {
    let maybe_storage_version = <T::Module as GetPalletVersion>::storage_version();
    match maybe_storage_version {
        Some(version) if version == PalletVersion::new(3, 1, 0) => {
            debug::RuntimeLogger::init();
            reconcile_round_stats::<T>();
            Weight::max_value()
        }
        _ => 0,
    }
}
