// Copyright 2020 Parity Technologies (UK) Ltd.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error, decl_event, decl_module,
	dispatch::DispatchResult,
	traits::{Get, Currency, ExistenceRequirement, WithdrawReason},
	Parameter
};

use frame_system::ensure_signed;
use sp_runtime::traits::{AtLeast32Bit, CheckedSub, Convert, MaybeSerializeDeserialize, Member};

use codec::{Decode, Encode};

use sp_std::{
	prelude::*,
};

use cumulus_primitives::{
	relay_chain::{Balance as RelayChainBalance, DownwardMessage},
	xcmp::{XCMPMessageHandler, XCMPMessageSender},
	DownwardMessageHandler, ParaId, UpwardMessageOrigin, UpwardMessageSender,
};
use cumulus_upward_message::BalancesMessage;
use polkadot_parachain::primitives::AccountIdConversion;

#[derive(Encode, Decode)]
pub enum XCMPMessage<AccountId, Balance> {
	/// Acala xtoken based to transfer tokens to the given account from the Parachain account.
	Transfer(Vec<u8>, Balance, AccountId),
}

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait {
	/// Event type used by the runtime.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The balance type
	type Balance: Parameter + Member + AtLeast32Bit + Default + Copy + MaybeSerializeDeserialize;

	/// The sender of upward messages.
	type UpwardMessageSender: UpwardMessageSender<Self::UpwardMessage>;

	/// The upward message type used by the Parachain runtime.
	type UpwardMessage: codec::Codec + BalancesMessage<Self::AccountId, RelayChainBalance>;

	/// Currency of the runtime.
	type Currency: Currency<Self::AccountId>;

	/// The sender of XCMP messages.
	type XCMPMessageSender: XCMPMessageSender<XCMPMessage<Self::AccountId, Self::Balance>>;
}

decl_event! {
	pub enum Event<T> where
	<T as frame_system::Trait>::AccountId,
	<T as Trait>::Balance
	{
		/// Transferred tokens to the account on the relay chain.
		TransferToRelayChain(AccountId, Balance),
		/// Acala xtoken based transfer some assets to another parachain. [para_id, asset_id, dest, amount]
		TransferToParachain(ParaId, Vec<u8>, AccountId, Balance),
		/// Received relay chain token from relay chain. [dest, amount]
		ReceivedTransferFromRelayChain(AccountId, Balance),
		/// Acala xtoken based received soem assets from another parachain. [para_id, asset_id, dest, amount]
		ReceivedTransferFromParachain(ParaId, Vec<u8>, AccountId, Balance),
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		/// Transfer `amount` of tokens on the relay chain from the Parachain account to
		/// the given `dest` account.
		#[weight = 10]
		fn transfer_to_relay_chain(origin, dest: T::AccountId, amount: T::Balance) {
			Self::deposit_event(Event::<T>::TransferToRelayChain(dest, amount));
		}
	}
}

/// This is a hack to convert from one generic type to another where we are sure that both are the
/// same type/use the same encoding.
fn convert_hack<O: Decode>(input: &impl Encode) -> O {
	input.using_encoded(|e| Decode::decode(&mut &e[..]).expect("Must be compatible; qed"))
}

impl<T: Trait> DownwardMessageHandler for Module<T> {
	fn handle_downward_message(msg: &DownwardMessage) {
		match msg {
			DownwardMessage::TransferInto(dest, amount, _) => {

				// let dest = convert_hack(&dest);
				// let amount = T::FromRelayChainBalance::convert(*amount);

				// TODO: TEE catch this event and finish the rest of the work
				// Self::deposit_event(Event::<T>::ReceivedTransferFromRelayChain(dest, amount));
			}
			_ => {}
		}
	}
}

impl<T: Trait> XCMPMessageHandler<XCMPMessage<T::AccountId, T::Balance>> for Module<T> {
	fn handle_xcmp_message(src: ParaId, msg: &XCMPMessage<T::AccountId, T::Balance>) {
		match msg {
			XCMPMessage::Transfer(asset_id, amount, dest) => {
				// TODO: TEE catch this event and finish the rest of the work
				Self::deposit_event(Event::<T>::ReceivedTransferFromParachain(
					src,
					asset_id.clone(),
					dest.clone(),
					*amount,
				));
			}
		}
	}
}
