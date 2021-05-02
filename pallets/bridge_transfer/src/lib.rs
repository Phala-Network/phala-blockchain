// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, EnsureOrigin, ExistenceRequirement::AllowDeath, Get};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
use pallet_bridge as bridge;
use sp_arithmetic::traits::SaturatedConversion;
use sp_core::U256;
use sp_std::prelude::*;

mod mock;
mod tests;

type ResourceId = bridge::ResourceId;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub trait Config: system::Config + bridge::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	/// Specifies the origin check provided by the bridge for calls that can only be called by the bridge pallet
	type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

	/// The currency mechanism.
	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Config> as BridgeTransfer {
		BridgeTokenId get(fn bridge_tokenid): ResourceId;
		BridgeHashId get(fn bridge_hashid): ResourceId;
	}

	add_extra_genesis {
		config(bridge_tokenid): ResourceId;
		config(bridge_hashid): ResourceId;
		build(|config: &GenesisConfig| {
			BridgeTokenId::put(config.bridge_tokenid);
			BridgeHashId::put(config.bridge_hashid);
		});
	}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Config>::Hash,
	{
		Remark(Hash),
	}
}

decl_error! {
	pub enum Error for Module<T: Config>{
		InvalidTransfer,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		//
		// Initiation calls. These start a bridge transfer.
		//

		/// Transfers an arbitrary hash to a (whitelisted) destination chain.
		#[weight = 195_000_000]
		pub fn transfer_hash(origin, hash: T::Hash, dest_id: bridge::ChainId) -> DispatchResult {
			ensure_signed(origin)?;

			let resource_id = Self::bridge_hashid();
			let metadata: Vec<u8> = hash.as_ref().to_vec();
			<bridge::Module<T>>::transfer_generic(dest_id, resource_id, metadata)
		}

		/// Transfers some amount of the native token to some recipient on a (whitelisted) destination chain.
		#[weight = 195_000_000]
		pub fn transfer_native(origin, amount: BalanceOf<T>, recipient: Vec<u8>, dest_id: bridge::ChainId) -> DispatchResult {
			let source = ensure_signed(origin)?;
			ensure!(<bridge::Module<T>>::chain_whitelisted(dest_id), Error::<T>::InvalidTransfer);
			let bridge_id = <bridge::Module<T>>::account_id();
			T::Currency::transfer(&source, &bridge_id, amount.into(), AllowDeath)?;

			let resource_id = Self::bridge_tokenid();

			<bridge::Module<T>>::transfer_fungible(dest_id, resource_id, recipient, U256::from(amount.saturated_into::<u128>()))
		}

		//
		// Executable calls. These can be triggered by a bridge transfer initiated on another chain
		//

		/// Executes a simple currency transfer using the bridge account as the source
		#[weight = 195_000_000]
		pub fn transfer(origin, to: T::AccountId, amount: BalanceOf<T>, r_id: ResourceId) -> DispatchResult {
			let source = T::BridgeOrigin::ensure_origin(origin)?;
			<T as Config>::Currency::transfer(&source, &to, amount.into(), AllowDeath)?;
			Ok(())
		}

		/// This can be called by the bridge to demonstrate an arbitrary call from a proposal.
		#[weight = 195_000_000]
		pub fn remark(origin, hash: T::Hash, r_id: ResourceId) -> DispatchResult {
			T::BridgeOrigin::ensure_origin(origin)?;
			Self::deposit_event(RawEvent::Remark(hash));
			Ok(())
		}
	}
}
