#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use sp_std::{prelude::*, fmt::Debug};
use sp_io::{hashing::keccak_256, crypto::secp256k1_ecdsa_recover};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, dispatch, traits::Get, RuntimeDebug};
use frame_system::{self as system, ensure_signed, ensure_root};
use codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{self, Serialize, Deserialize, Serializer, Deserializer};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Trait> as ClaimModule {
	    EndHeight get(fn end_height): u64;
       	BurnedTransactions get(fn destroyed_transaction): map hasher(blake2_128_concat) Vec<u8> => (Vec<u8>, u64);
		ClaimState get(fn claim_state): map hasher(blake2_128_concat) Vec<u8> => bool;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event emitted when a transaction has been stored.
		TransactionStored(Vec<u8>, Vec<u8>, u64),
		/// Event emitted when a transaction has been claimed.
		TransactionClaimed(AccountId, Vec<u8>),

	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// The transaction signature is invalid.
		TransactionSignatureInvalid,
		/// The transaction infomation is invalid.
		TransactionInfoInvalid,
		/// The transaction doesn't exist
		TransactionNotFound,
		/// The transaction already exit
		TransactionAlreadyExist,
		/// The transaction has been claimed
		TransactionAlreadyClaimed,
		/// The transaction height less than end height
		LessThanEndHeight,
	}
}

/// An Ethereum address (i.e. 20 bytes, used to represent an Ethereum account).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug)]
pub struct EthereumAddress([u8; 20]);

#[cfg(feature = "std")]
impl Serialize for EthereumAddress {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
		serializer.serialize_str(&format!("0x{}", hex))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for EthereumAddress {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		let base_string = String::deserialize(deserializer)?;
		let offset = if base_string.starts_with("0x") { 2 } else { 0 };
		let s = &base_string[offset..];
		if s.len() != 40 {
			Err(serde::de::Error::custom("Bad length of Ethereum address (should be 42 including '0x')"))?;
		}
		let raw: Vec<u8> = rustc_hex::FromHex::from_hex(s)
			.map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
		let mut r = Self::default();
		r.0.copy_from_slice(&raw);
		Ok(r)
	}
}

#[derive(Encode, Decode, Clone)]
pub struct EcdsaSignature(pub [u8; 65]);

impl PartialEq for EcdsaSignature {
	fn eq(&self, other: &Self) -> bool {
		&self.0[..] == &other.0[..]
	}
}

impl sp_std::fmt::Debug for EcdsaSignature {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		write!(f, "EcdsaSignature({:?})", &self.0[..])
	}
}



// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 0]
		pub fn sync_eth_burn_transaction(origin, height: u64, txHash: Vec<u8>, fromAddress: Vec<u8>, amount:u64) -> dispatch::DispatchResult {
			// ensure_root(origin)?;
			ensure!(!BurnedTransactions::contains_key(&txHash), Error::<T>::TransactionAlreadyExist);
			let endHeight = EndHeight::get();
			ensure!(!(height < endHeight), Error::<T>::LessThanEndHeight);
			if (height > endHeight) {
				EndHeight::put(height);
			}
			BurnedTransactions::insert(&txHash, (fromAddress.clone(), amount.clone()));
			Self::deposit_event(RawEvent::TransactionStored(txHash, fromAddress, amount));
			Ok(())
		}

		// #[weight = 0]
		// pub fn claim(origin, txHash: Vec<u8>, sig: Vec<u8>) -> dispatch::DispatchResult {
		// 	ensure!(!ClaimState::get(&txHash), Error::<T>::TransactionAlreadyClaimed);
		// 	let who = ensure_signed(origin)?;
		// 	let data = who.using_encoded(to_ascii_hex);
		// 	let signer = Self::eth_recover(&sig, &data, &txHash)
		// 		.ok_or(Error::<T>::TransactionSignatureInvalid)?;
		// 	let tx = BurnedTransactions::get(&txHash);
		//
		// 	// TODO
		// 	ensure!(!(), Error::<T>::TransactionInfoInvalid);
		// 	ClaimState::insert(&txHash, true);
		// 	Self::deposit_event(RawEvent::TransactionClaimed(who, txHash));
		// 	Ok(())
		// }
	}
}

/// Converts the given binary data into ASCII-encoded hex. It will be twice the length.
fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
	let mut r = Vec::with_capacity(data.len() * 2);
	let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
	for &b in data.iter() {
		push_nibble(b / 16);
		push_nibble(b % 16);
	}
	r
}

impl<T: Trait> Module<T> {
	// Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign` would sign.
	fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
		let mut l =  what.len() + extra.len();
		let mut rev = Vec::new();
		while l > 0 {
			rev.push(b'0' + (l % 10) as u8);
			l /= 10;
		}
		let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
		v.extend(rev.into_iter().rev());
		v.extend_from_slice(what);
		v.extend_from_slice(extra);
		v
	}

	// Attempts to recover the Ethereum address from a message signature signed by using
	// the Ethereum RPC's `personal_sign` and `eth_sign`.
	fn eth_recover(s: &EcdsaSignature, what: &[u8], extra: &[u8]) -> Option<EthereumAddress> {
		let msg = keccak_256(&Self::ethereum_signable_message(what, extra));
		let mut res = EthereumAddress::default();
		res.0.copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
		Some(res)
	}
}
