#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

extern crate alloc;

extern crate untrusted;
extern crate base64;
extern crate itertools;
extern crate hex;

extern crate webpki;

use alloc::vec::Vec;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as PhalaModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		CommandNumber get(fn command_number): Option<u32>;

		// Store a map of TEE key and TEE register info, map Vec<u8> => Vec<u8>
		TEERegisterInfo get(fn tee_register_info): map hasher(blake2_128_concat) Vec<u8> => Vec<u8>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		CommandPushed(AccountId, u32, Vec<u8>, u32),
		LogString(Vec<u8>),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		#[weight = 0]
		pub fn push_command(origin, contract_id: u32, payload: Vec<u8>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let num = Self::command_number().unwrap_or(0);
			CommandNumber::put(num + 1);
			Self::deposit_event(RawEvent::CommandPushed(who, contract_id, payload, num));
			Ok(())
		}

		#[weight = 0]
		pub fn register_worker(origin, pubkey: Vec<u8>, info: Vec<u8>) -> dispatch::DispatchResult {
			ensure_signed(origin)?;

			// if pubkey exists, the info will be updated
			TEERegisterInfo::insert(pubkey, info);

			Ok(())
		}
	}
}
