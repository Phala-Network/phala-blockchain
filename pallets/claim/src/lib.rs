#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_io::{hashing::keccak_256, crypto::secp256k1_ecdsa_recover};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, dispatch, RuntimeDebug};
use frame_system::{ensure_signed, ensure_root};
use codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{self, Serialize, Deserialize, Serializer, Deserializer};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug)]
pub struct EthereumTxHash(pub [u8; 32]);

#[cfg(feature = "std")]
impl Serialize for EthereumTxHash {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
		serializer.serialize_str(&format!("0x{}", hex))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for EthereumTxHash {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		let base_string = String::deserialize(deserializer)?;
		let offset = if base_string.starts_with("0x") { 2 } else { 0 };
		let s = &base_string[offset..];
		if s.len() != 64 {
			Err(serde::de::Error::custom("Bad length of Ethereum tx hash (should be 66 including '0x')"))?;
		}
		let raw: Vec<u8> = rustc_hex::FromHex::from_hex(s)
			.map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
		let mut r = Self::default();
		r.0.copy_from_slice(&raw);
		Ok(r)
	}
}


/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as ClaimModule {
    	AdminId get(fn admin_id): T::AccountId;
    	EndHeight get(fn end_height): u64;
    	BurnedTransactions get(fn destroyed_transaction): map hasher(blake2_128_concat) EthereumTxHash => (EthereumAddress, u64);
    	ClaimState get(fn claim_state): map hasher(blake2_128_concat) EthereumTxHash => bool;
    }
}

decl_event!(
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        /// Event emitted when a transaction has been stored.
        ERC20TransactionStored(AccountId, EthereumTxHash, EthereumAddress, u64),
        /// Event emitted when a transaction has been claimed.
        ERC20TokenClaimed(AccountId, EthereumTxHash, u64),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Not administrator
        NotAdministrator,
        /// The transaction signature is invalid.
        InvalidSignature,
        /// The signer is not transaction sender.
        NotTransactionSender,
        /// The transaction hash doesn't exist
        TxHashNotFound,
        /// The transaction hash already exit
        TxHashAlreadyExist,
        /// The transaction has been claimed
        TxAlreadyClaimed,
        /// The transaction height less than end height
        LessThanEndHeight,
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
      	pub fn set_admin(origin, admin: T::AccountId) -> dispatch::DispatchResult {
      		ensure_root(origin)?;
      		Admin_Id::<T>::put(admin);
            Ok(())
        }


        #[weight = 0]
        pub fn store_erc20_burned_transaction(origin, height: u64, eth_tx_hash: EthereumTxHash, eth_address: EthereumAddress, amount:u64) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            // let admin = Admin_Id::<T>::get();
            // ensure!(who == admin, Error::<T>::NotAdministrator)
            ensure!(!BurnedTransactions::contains_key(&eth_tx_hash), Error::<T>::TxHashAlreadyExist);
            let end_height = EndHeight::get();
            if height > end_height {
                EndHeight::put(height);
            }
            BurnedTransactions::insert(&eth_tx_hash, (eth_address.clone(), amount.clone()));
            ClaimState::insert(&eth_tx_hash, false);
            Self::deposit_event(RawEvent::ERC20TransactionStored(who, eth_tx_hash, eth_address, amount));
            Ok(())
        }


        #[weight = 0]
        pub fn claim_erc20_token(origin, eth_tx_hash: EthereumTxHash, eth_signature: EcdsaSignature) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(BurnedTransactions::contains_key(&eth_tx_hash), Error::<T>::TxHashNotFound);
            ensure!(!ClaimState::get(&eth_tx_hash), Error::<T>::TxAlreadyClaimed);
            let address = Encode::encode(&who);
            let signer = Self::eth_recover(&eth_signature, &address, &eth_tx_hash.0)
                .ok_or(Error::<T>::InvalidSignature)?;
            let tx = BurnedTransactions::get(&eth_tx_hash);
            ensure!(signer == tx.0, Error::<T>::NotTransactionSender);
            ClaimState::insert(&eth_tx_hash, true);
            Self::deposit_event(RawEvent::ERC20TokenClaimed(who, eth_tx_hash, tx.1));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
	// Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign` would sign.
	fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
		let mut l = what.len() + extra.len();
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

