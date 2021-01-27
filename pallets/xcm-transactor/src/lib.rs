#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    traits::{Currency, ExistenceRequirement, Get, WithdrawReasons},
};

use sp_runtime::{
    traits::{CheckedConversion, SaturatedConversion},
    RuntimeDebug,
};
use sp_std::{
    collections::btree_map::BTreeMap,
    convert::{TryInto, TryFrom},
    marker::PhantomData,
    prelude::*,
    result,
};

use codec::{Decode, Encode};

use cumulus_primitives::ParaId;
use xcm::v0::{Error, Junction, MultiAsset, MultiLocation, Result};
use xcm_executor::traits::{
    FilterAssetLocation, LocationConversion, MatchesFungible, NativeAsset, TransactAsset,
};

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
        PHAXCurrencyId {
            chain_id,
            currency_id,
        }
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

type BalanceOf<T> =
    <<T as Config>::OwnedCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Configuration trait of this pallet.
pub trait Config: frame_system::Config {
    /// Event type used by the runtime.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Matcher: MatchesFungible<BalanceOf<Self>>;
    type AccountIdConverter: LocationConversion<Self::AccountId>;
    type XCurrencyIdConverter: XCurrencyIdConversion;
    type OwnedCurrency: Currency<Self::AccountId>;
    type ParaId: Get<ParaId>;
}

decl_storage! {
    trait Store for Module<T: Config> as PhalaXCMAdapter {}
}

decl_event! (
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// Deposit asset into current chain. [currency_id, account_id, amount, to_tee]
        DepositAsset(Vec<u8>, AccountId, Balance, bool),

        /// Withdraw asset from current chain. [currency_id, account_id, amount, to_tee]
        WithdrawAsset(Vec<u8>, AccountId, Balance, bool),
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
    }
}

impl<T> TransactAsset for Module<T>
where
    T: Config
{
    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
        debug::info!("----------------------- xcm-transactor: trying deposit -------------------------");
        debug::info!(
            ">>> asset: {:?}, location: {:?}",
            asset,
            location
        );

		if let MultiLocation::X1(Junction::AccountId32 { id, network }) = location {
            let who = T::AccountIdConverter::from_location(location).ok_or(())?;
            debug::info!("who: {:?}", who);
            let currency_id =
                T::XCurrencyIdConverter::from_asset(asset).ok_or(())?;
            debug::info!("currency_id: {:?}", currency_id);
            let amount: BalanceOf<T> = T::Matcher::matches_fungible(&asset)
                .ok_or(())?
                .saturated_into();
            debug::info!("amount: {:?}", amount);
            let balance_amount = amount.try_into().map_err(|_| ())?;
            debug::info!("balance amount: {:?}", balance_amount);
    
            let mut is_owned_currency = false;
            if let ChainId::ParaChain(paraid) = currency_id.chain_id {
                if T::ParaId::get() == paraid {
                    is_owned_currency = true;
                    let _ = T::OwnedCurrency::deposit_creating(&who, balance_amount);
                }
            }
    
            Self::deposit_event(Event::<T>::DepositAsset(
                currency_id.clone().into(),
                who,
                balance_amount,
                !is_owned_currency,
            ));
    
            debug::info!("------------------ xcm-transactor: success deposit ------------------------------");
        }
        
        Ok(())
    }

    fn withdraw_asset(
        asset: &MultiAsset,
        location: &MultiLocation,
    ) -> result::Result<MultiAsset, Error> {
        debug::info!("--------------------- xdm-adapter: trying withdraw ---------------------------");
        debug::info!(
            ">>> asset: {:?}, location: {:?}",
            asset,
            location
        );

		if let MultiLocation::X1(Junction::AccountId32 {id, network}) = location {
            let who = T::AccountIdConverter::from_location(location).ok_or(())?;
            debug::info!("who: {:?}", who);
            let currency_id =
                T::XCurrencyIdConverter::from_asset(asset).ok_or(())?;
            debug::info!("currency_id: {:?}", currency_id);
            let amount: BalanceOf<T> = T::Matcher::matches_fungible(&asset)
                .ok_or(())?
                .saturated_into();
            debug::info!("amount: {:?}", amount);
            let balance_amount = amount.try_into().map_err(|_| ())?;
            debug::info!("balance amount: {:?}", balance_amount);
    
            let mut is_owned_currency = false;
            if let ChainId::ParaChain(paraid) = currency_id.chain_id {
                if T::ParaId::get() == paraid {
                    is_owned_currency = true;
                    let _ = T::OwnedCurrency::withdraw(
                        &who,
                        balance_amount,
                        WithdrawReasons::TRANSFER,
                        ExistenceRequirement::AllowDeath,
                    )
                    .map_err(|_| Error::UnhandledEffect)?;
                }
            }
    
            Self::deposit_event(Event::<T>::WithdrawAsset(
                currency_id.clone().into(),
                who,
                balance_amount,
                !is_owned_currency,
            ));
        }

        debug::info!("--------------------- xcm-transactor: success withdraw ---------------------------");
        Ok(asset.clone())
    }
}

pub trait XCurrencyIdConversion {
    fn from_asset(
        asset: &MultiAsset
    ) -> Option<PHAXCurrencyId>;
}

pub struct XCurrencyIdConverter<NativeTokens>(PhantomData<NativeTokens>);
impl<NativeTokens: Get<BTreeMap<Vec<u8>, MultiLocation>>> XCurrencyIdConversion
    for XCurrencyIdConverter<NativeTokens>
{
    fn from_asset(
        multi_asset: &MultiAsset
    ) -> Option<PHAXCurrencyId> {
        if let MultiAsset::ConcreteFungible { ref id, .. } = multi_asset {
            if id == &MultiLocation::X1(Junction::Parent) {
                let relaychain_currency: PHAXCurrencyId = PHAXCurrencyId {
                    chain_id: ChainId::RelayChain,
                    currency_id: b"DOT".to_vec(),
                };
                return Some(relaychain_currency);
            }

            if let Some(Junction::GeneralKey(key)) = id.last() {
                if NativeTokens::get().contains_key(key) {
                    // here we can trust the currency matchs the parachain, case NativePalletAssetOr already check this
                    if let MultiLocation::X2(Junction::Parent, Junction::Parachain { id: paraid }) =
                        NativeTokens::get().get(key).unwrap()
                    {
                        let parachain_currency: PHAXCurrencyId = PHAXCurrencyId {
                            chain_id: ChainId::ParaChain((*paraid).into()),
                            currency_id: key.clone(),
                        };
                        return Some(parachain_currency);
                    }

                    if let MultiLocation::X1(Junction::Parachain { id: paraid }) =
                        NativeTokens::get().get(key).unwrap()
                    {
                        let parachain_currency: PHAXCurrencyId = PHAXCurrencyId {
                            chain_id: ChainId::ParaChain((*paraid).into()),
                            currency_id: key.clone(),
                        };
                        return Some(parachain_currency);
                    }
                }
            }
        }
        None
    }
}

pub struct ConcreteMatcher<T>(PhantomData<T>);
impl<T: Get<MultiLocation>, B: TryFrom<u128>> MatchesFungible<B> for ConcreteMatcher<T> {
	fn matches_fungible(a: &MultiAsset) -> Option<B> {
        if let MultiAsset::ConcreteFungible { id, amount } = a {
            if id == &T::get() {
                return CheckedConversion::checked_from(*amount);
            } else if let Some(Junction::GeneralKey(key)) = id.last() {
                return CheckedConversion::checked_from(*amount);
            } else {
                return None;
            }
        }
        None
	}
}

pub struct AssetLocationFilter<NativeTokens>(PhantomData<NativeTokens>);
impl<NativeTokens: Get<BTreeMap<Vec<u8>, MultiLocation>>> FilterAssetLocation
    for AssetLocationFilter<NativeTokens>
{
    fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {

        debug::info!("filter_asset_location, asset: {:?}, origin: {:?}", asset, origin);

        if NativeAsset::filter_asset_location(asset, origin) {
            return true;
        }

        // native asset identified by a general key
        if let MultiAsset::ConcreteFungible { ref id, .. } = asset {
            if let Some(Junction::GeneralKey(key)) = id.last() {
                if NativeTokens::get().contains_key(key) {
                    return (*origin) == *(NativeTokens::get().get(key).unwrap());
                }
            }
        }

        false
    }
}
