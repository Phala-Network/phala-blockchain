#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
use frame_support::{ensure, traits::Currency, RuntimeDebug};
use frame_system::{ensure_none, ensure_root, ensure_signed};
#[cfg(feature = "std")]
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_runtime::transaction_validity::{
	InvalidTransaction, TransactionLongevity, TransactionSource, TransactionValidity,
	ValidTransaction,
};

use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug)]
pub struct EthereumAddress([u8; 20]);

#[cfg(feature = "std")]
impl Serialize for EthereumAddress {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
		serializer.serialize_str(&format!("0x{}", hex))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for EthereumAddress {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let base_string = String::deserialize(deserializer)?;
		let offset = if base_string.starts_with("0x") { 2 } else { 0 };
		let s = &base_string[offset..];
		if s.len() != 40 {
			Err(serde::de::Error::custom(
				"Bad length of Ethereum address (should be 42 including '0x')",
			))?;
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
pub struct EthereumTxHash([u8; 32]);

#[cfg(feature = "std")]
impl Serialize for EthereumTxHash {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
		serializer.serialize_str(&format!("0x{}", hex))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for EthereumTxHash {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let base_string = String::deserialize(deserializer)?;
		let offset = if base_string.starts_with("0x") { 2 } else { 0 };
		let s = &base_string[offset..];
		if s.len() != 64 {
			Err(serde::de::Error::custom(
				"Bad length of Ethereum tx hash (should be 66 including '0x')",
			))?;
		}
		let raw: Vec<u8> = rustc_hex::FromHex::from_hex(s)
			.map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
		let mut r = Self::default();
		r.0.copy_from_slice(&raw);
		Ok(r)
	}
}

/// The balance type of this module.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: From<Call<Self>>;
		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn end_height)]
	pub type EndHeight<T: Config> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn destroyed_transaction)]
	pub type BurnedTransactions<T: Config> =
		StorageMap<_, Blake2_128Concat, EthereumTxHash, (EthereumAddress, BalanceOf<T>)>;

	#[pallet::storage]
	#[pallet::getter(fn claim_state)]
	pub type ClaimState<T: Config> = StorageMap<_, Blake2_128Concat, EthereumTxHash, bool>;

	#[pallet::storage]
	#[pallet::getter(fn relayer)]
	pub type Relayer<T: Config> = StorageValue<_, Option<T::AccountId>>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a transaction has been stored.
		ERC20TransactionStored(T::AccountId, EthereumTxHash, EthereumAddress, BalanceOf<T>),
		/// Event emitted when a transaction has been claimed.
		ERC20TokenClaimed(T::AccountId, EthereumTxHash, BalanceOf<T>),
		/// Event emitted when the relayer has been changed.
		RelayerChanged(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The transaction signature is invalid.
		InvalidSignature,
		/// The signer is not claim transaction sender.
		InvalidSigner,
		/// The transaction hash doesn't exist
		TxHashNotFound,
		/// The transaction hash already exist
		TxHashAlreadyExist,
		/// The transaction has been claimed
		TxAlreadyClaimed,
		/// The transaction height less than end height
		LessThanEndHeight,
		/// The relayer has not been changed
		RelayerNotChanged,
		/// The caller is not relayer
		CallerNotRelayer,
		/// The module doesn't set relyer
		NoRelayer,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0 + T::DbWeight::get().reads_writes(1,1))]
		pub fn change_relayer(
			origin: OriginFor<T>,
			new_relayer: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let old_relayer = Relayer::<T>::get();
			if old_relayer.is_some() {
				ensure!(
					Some(&new_relayer) != old_relayer.flatten().as_ref(),
					Error::<T>::RelayerNotChanged
				);
			}

			Relayer::<T>::put(Some(&new_relayer));
			Self::deposit_event(Event::RelayerChanged(new_relayer));
			Ok(().into())
		}

		#[pallet::weight(0 + T::DbWeight::get().reads_writes(1,1))]
		pub fn store_erc20_burned_transactions(
			origin: OriginFor<T>,
			height: u64,
			claims: Vec<(EthereumTxHash, EthereumAddress, BalanceOf<T>)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let relayer = Relayer::<T>::get();
			ensure!(relayer.is_some(), Error::<T>::NoRelayer);
			ensure!(
				Some(&who) == relayer.flatten().as_ref(),
				Error::<T>::CallerNotRelayer
			);
			// check first
			for (eth_tx_hash, _, _) in claims.iter() {
				ensure!(
					!BurnedTransactions::<T>::contains_key(&eth_tx_hash),
					Error::<T>::TxHashAlreadyExist
				);
			}
			for (eth_tx_hash, eth_address, erc20_amount) in claims.iter() {
				BurnedTransactions::<T>::insert(
					&eth_tx_hash,
					(eth_address.clone(), erc20_amount.clone()),
				);
				ClaimState::<T>::insert(&eth_tx_hash, false);
				Self::deposit_event(Event::ERC20TransactionStored(
					who.clone(),
					*eth_tx_hash,
					*eth_address,
					*erc20_amount,
				));
			}
			let end_height = EndHeight::<T>::get().unwrap_or_default();
			if height > end_height {
				EndHeight::<T>::put(height);
			}

			Ok(().into())
		}

		#[pallet::weight(0 + T::DbWeight::get().reads_writes(1,1))]
		pub fn claim_erc20_token(
			origin: OriginFor<T>,
			account: T::AccountId,
			eth_tx_hash: EthereumTxHash,
			eth_signature: EcdsaSignature,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_none(origin)?;
			ensure!(
				BurnedTransactions::<T>::contains_key(&eth_tx_hash),
				Error::<T>::TxHashNotFound
			);
			ensure!(
				ClaimState::<T>::get(&eth_tx_hash).is_some(),
				Error::<T>::TxAlreadyClaimed
			);
			let address = Encode::encode(&account);
			let signer = Self::eth_recover(&eth_signature, &address, &eth_tx_hash.0)
				.ok_or(Error::<T>::InvalidSignature)?;
			let tx = BurnedTransactions::<T>::get(&eth_tx_hash).unwrap_or_default();
			ensure!(signer == tx.0, Error::<T>::InvalidSigner);
			ClaimState::<T>::insert(&eth_tx_hash, true);
			// mint coins
			let imbalance = T::Currency::deposit_creating(&account, tx.1);
			drop(imbalance);
			Self::deposit_event(Event::ERC20TokenClaimed(account, eth_tx_hash, tx.1));

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
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
			res.0
				.copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
			Some(res)
		}
	}

	impl<T: Config> frame_support::unsigned::ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			const PRIORITY: u64 = 100;

			let (maybe_signer, tx_hash) = match call {
				Call::claim_erc20_token(account, eth_tx_hash, eth_signature) => {
					let address = Encode::encode(&account);
					(
						Self::eth_recover(&eth_signature, &address, &eth_tx_hash.0),
						eth_tx_hash,
					)
				}
				_ => return Err(InvalidTransaction::Call.into()),
			};

			let e = InvalidTransaction::Custom(ValidityError::TxHashNotFound.into());
			ensure!(BurnedTransactions::<T>::contains_key(&tx_hash), e);

			let signer = maybe_signer.ok_or(InvalidTransaction::BadProof)?;

			let e = InvalidTransaction::Custom(ValidityError::InvalidSigner.into());
			let stored_tx = BurnedTransactions::<T>::get(&tx_hash).unwrap_or_default();
			let stored_signer = stored_tx.0;
			ensure!(signer == stored_signer, e);

			let e = InvalidTransaction::Custom(ValidityError::TxAlreadyClaimed.into());
			ensure!(!ClaimState::<T>::get(&tx_hash).unwrap_or_default(), e);

			Ok(ValidTransaction {
				priority: PRIORITY,
				requires: vec![],
				provides: vec![("claims", signer).encode()],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			})
		}
	}
}

#[repr(u8)]
pub enum ValidityError {
	/// The transaction signature is invalid.
	InvalidSignature = 0,
	/// The transaction hash doesn't exist
	TxHashNotFound = 1,
	/// The transaction has been claimed
	TxAlreadyClaimed = 2,
	/// The signer is not claim transaction sender.
	InvalidSigner = 3,
}

impl From<ValidityError> for u8 {
	fn from(err: ValidityError) -> Self {
		err as u8
	}
}
