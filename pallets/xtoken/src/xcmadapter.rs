#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
	traits::{CheckedConversion, Convert, SaturatedConversion, Member, AtLeast32Bit, MaybeSerializeDeserialize},
	DispatchResult,
};
use sp_std::{
	collections::btree_set::BTreeSet,
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	prelude::*,
	result,
};

use xcm::v0::{Error, Junction, MultiAsset, MultiLocation, Result};
use xcm_executor::traits::{FilterAssetLocation, LocationConversion, MatchesFungible, NativeAsset, TransactAsset};

use frame_support::{debug, traits::Get,Parameter};

pub struct XCMAdapter (

);

impl TransactAsset for XCMAdapter
{
    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
		debug::info!("===========================================");
		debug::info!(">>> success deposit.");
		debug::info!("===========================================");
		Ok(())
    }
    
    fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> result::Result<MultiAsset, Error> {
		debug::info!("===========================================");
		debug::info!(">>> success withdraw.");
		debug::info!("===========================================");
		Ok(asset.clone())
	}
}

pub trait XcmHandler {
	type Origin;
	type Xcm;
	fn execute(origin: Self::Origin, xcm: Self::Xcm) -> DispatchResult;
}

