#![cfg_attr(not(feature = "std"), no_std)]

use codec::FullCodec;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	traits::{Get, Currency, ExistenceRequirement, WithdrawReason},
	Parameter,
	debug
};

use sp_runtime::{
	traits::{CheckedConversion, Convert, SaturatedConversion, Member, AtLeast32Bit, MaybeSerializeDeserialize},
	DispatchResult, RuntimeDebug,
};
use sp_std::{
	collections::btree_set::BTreeSet,
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	prelude::*,
	result,
	fmt::Debug,
};

use codec::{Decode, Encode};

use xcm::v0::{Error, Junction, MultiAsset, MultiLocation, Result};
use xcm_executor::traits::{FilterAssetLocation, LocationConversion, MatchesFungible, NativeAsset, TransactAsset};
use cumulus_primitives::ParaId;

#[derive(Encode, Decode, Eq, PartialEq, Clone, Copy, RuntimeDebug)]
/// Identity of chain.
pub enum ChainId {
	/// The relay chain.
	RelayChain,
	/// A parachain.
	ParaChain(ParaId),
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
/// Identity of cross chain currency.
pub struct PHAXCurrencyId {
	/// The reserve chain of the currency. For instance, the reserve chain of
	/// DOT is Polkadot.
	pub chain_id: ChainId,
	/// The identity of the currency.
	pub currency_id: Vec<u8>,
}

impl PHAXCurrencyId {
	pub fn new(chain_id: ChainId, currency_id: Vec<u8>) -> Self {
		PHAXCurrencyId { chain_id, currency_id }
	}
}

impl Into<MultiLocation> for PHAXCurrencyId {
	fn into(self) -> MultiLocation {
		MultiLocation::X1(Junction::GeneralKey(self.currency_id))
	}
}

impl Into<Vec<u8>> for PHAXCurrencyId {
	fn into(self) -> Vec<u8> {
		[ChainId::encode(&self.chain_id), self.currency_id].concat()
	}
}

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait {
	/// Event type used by the runtime.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Balance: Parameter + Member + AtLeast32Bit + Default + Copy + MaybeSerializeDeserialize + Into<u128>;
	type Matcher: MatchesFungible<Self::Balance>;
	type AccountIdConverter: LocationConversion<Self::AccountId>;
	type EncodedXCurrencyId: FullCodec + Eq + PartialEq + MaybeSerializeDeserialize + Debug;
	type XCurrencyIdConverter: XCurrencyIdConversion<Self::EncodedXCurrencyId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as PhalaXCMAdapter {}
}

decl_event! (
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		<T as Trait>::EncodedXCurrencyId,
	{
		/// Deposit asset into current chain. [currency_id, account_id, amount, to_tee]
		DepositAsset(EncodedXCurrencyId, AccountId, Balance, bool),

		/// Withdraw asset from current chain. [currency_id, account_id, amount]
		WithdrawAsset(EncodedXCurrencyId, AccountId, Balance),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

    }
}

impl<T> TransactAsset for Module<T> where 
    T: Trait,
{
    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
		debug::info!("------------------------------------------------");
		debug::info!(">>> trying deposit. asset: {:?}, location: {:?}", asset, location);

		let who = T::AccountIdConverter::from_location(location).ok_or(())?;
		debug::info!("who: {:?}", who);
		let currency_id = T::XCurrencyIdConverter::from_asset_and_location(asset, location).ok_or(())?;
		debug::info!("currency_id: {:?}", currency_id);
		let amount = T::Matcher::matches_fungible(&asset).ok_or(())?.saturated_into();
		debug::info!("amount: {:?}", amount);
		let balance_amount = amount.try_into().map_err(|_| ())?;
		debug::info!("balance amount: {:?}", balance_amount);
        
        Self::deposit_event(
            RawEvent::DepositAsset(currency_id, who, balance_amount, true)
        );

		debug::info!(">>> success deposit.");
		debug::info!("------------------------------------------------");
		Ok(())
    }
    
    fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> result::Result<MultiAsset, Error> {
		debug::info!("------------------------------------------------");
		debug::info!(">>> trying withdraw. asset: {:?}, location: {:?}", asset, location);
		
		let who = T::AccountIdConverter::from_location(location).ok_or(())?;
		debug::info!("who: {:?}", who);
		let currency_id = T::XCurrencyIdConverter::from_asset_and_location(asset, location).ok_or(())?;
		debug::info!("currency_id: {:?}", currency_id);
		let amount = T::Matcher::matches_fungible(&asset).ok_or(())?.saturated_into();
		debug::info!("amount: {:?}", amount);
		let balance_amount = amount.try_into().map_err(|_| ())?;
		debug::info!("balance amount: {:?}", balance_amount);

        Self::deposit_event(
            Event::<T>::WithdrawAsset(currency_id, who, balance_amount),
        );

		debug::info!(">>> success withdraw.");
		debug::info!("------------------------------------------------");
		Ok(asset.clone())	
	}
}

pub struct IsConcreteWithGeneralKey<CurrencyId, FromRelayChainBalance>(
	PhantomData<(CurrencyId, FromRelayChainBalance)>,
);
impl<CurrencyId, B, FromRelayChainBalance> MatchesFungible<B>
	for IsConcreteWithGeneralKey<CurrencyId, FromRelayChainBalance>
where
	CurrencyId: TryFrom<Vec<u8>>,
	B: TryFrom<u128>,
	FromRelayChainBalance: Convert<u128, u128>,
{
	fn matches_fungible(a: &MultiAsset) -> Option<B> {
		if let MultiAsset::ConcreteFungible { id, amount } = a {
			if id == &MultiLocation::X1(Junction::Parent) {
				// Convert relay chain decimals to local chain
				let local_amount = FromRelayChainBalance::convert(*amount);
				return CheckedConversion::checked_from(local_amount);
			}
			if let Some(Junction::GeneralKey(key)) = id.last() {
				if TryInto::<CurrencyId>::try_into(key.clone()).is_ok() {
					return CheckedConversion::checked_from(*amount);
				}
			}
		}
		None
	}
}

pub trait CurrencyIdConversion<CurrencyId> {
	fn from_asset(asset: &MultiAsset) -> Option<CurrencyId>;
}

pub struct CurrencyIdConverter<CurrencyId>(
	PhantomData<CurrencyId>,
);

impl<CurrencyId> CurrencyIdConversion<CurrencyId>
	for CurrencyIdConverter<CurrencyId>
where
	CurrencyId: TryFrom<Vec<u8>>,
{
	fn from_asset(asset: &MultiAsset) -> Option<CurrencyId> {
		if let MultiAsset::ConcreteFungible { id: location, .. } = asset {
			if location == &MultiLocation::X1(Junction::Parent) {
				let relaychaincurrency  = PHAXCurrencyId {
					chain_id: ChainId::RelayChain,
					currency_id: b"DOT".to_vec(),
				};
				return CurrencyId::try_from(relaychaincurrency.into()).ok();
			}
			if let Some(Junction::GeneralKey(key)) = location.last() {
				return CurrencyId::try_from(key.clone()).ok();
			}
		}
		None
	}
}

pub trait XCurrencyIdConversion<EncodedXCurrencyId> {
	fn from_asset_and_location(asset: &MultiAsset, location: &MultiLocation) -> Option<EncodedXCurrencyId>;
}

pub struct XCurrencyIdConverter<EncodedXCurrencyId>(
	PhantomData<EncodedXCurrencyId>,
);

impl<EncodedXCurrencyId> XCurrencyIdConversion<EncodedXCurrencyId>
	for XCurrencyIdConverter<EncodedXCurrencyId>
where
	EncodedXCurrencyId: TryFrom<Vec<u8>>,
{
	fn from_asset_and_location(multi_asset: &MultiAsset, multi_location: &MultiLocation) -> Option<EncodedXCurrencyId> {
		if let MultiAsset::ConcreteFungible { id: location, .. } = multi_asset {
			if location == &MultiLocation::X1(Junction::Parent) {
				let relaychaincurrency  = PHAXCurrencyId {
					chain_id: ChainId::RelayChain,
					currency_id: b"DOT".to_vec(),
				};
				return EncodedXCurrencyId::try_from(relaychaincurrency.into()).ok();
			}

			if let MultiLocation::X2(Junction::Parent, Junction::Parachain { id: para_id }) = multi_location {
				if let Some(Junction::GeneralKey(key)) = location.last() {
					return EncodedXCurrencyId::try_from(PHAXCurrencyId::new(ChainId::ParaChain(para_id.clone().into()), key.clone()).into()).ok();
				}
			}
		}
		None
	}
}

pub trait XcmHandler {
	type Origin;
	type Xcm;
	fn execute(origin: Self::Origin, xcm: Self::Xcm) -> DispatchResult;
}
