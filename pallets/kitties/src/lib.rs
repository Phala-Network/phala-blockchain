#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult as Result, ensure,
	traits::Randomness, PalletId, StorageMap, StorageValue,
};
use frame_system::{self as system, ensure_signed};
use pallet_balances as balances;
use secp256k1;
use sp_runtime::traits::{AccountIdConversion, Hash, Zero};
use sp_std::prelude::*;

mod hashing;

const PALLET_ID: PalletId = PalletId(*b"Kitty!!!");
#[derive(Encode, Decode, Default, Debug, Clone, PartialEq)]
pub struct Kitty<Hash, Balance> {
	id: Hash,
	dna: Hash,
	price: Balance,
	gen: u64,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct KittyTransfer<AccountId> {
	dest: AccountId,
	kitty_id: Vec<u8>,
	sequence: u64,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct KittyTransferData<AccountId> {
	data: KittyTransfer<AccountId>,
	signature: Vec<u8>,
}

pub trait SignedDataType<T> {
	fn raw_data(&self) -> Vec<u8>;
	fn signature(&self) -> T;
}

impl<AccountId: Encode> SignedDataType<Vec<u8>> for KittyTransferData<AccountId> {
	fn raw_data(&self) -> Vec<u8> {
		Encode::encode(&self.data)
	}

	fn signature(&self) -> Vec<u8> {
		self.signature.clone()
	}
}

pub trait Config: balances::Config {
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
}

decl_event!(
	pub enum Event<T>
	where
		<T as system::Config>::AccountId,
		<T as system::Config>::Hash
	{
		Created(AccountId, Hash),
		Transferred(AccountId, AccountId, Hash),
		TransferToChain(AccountId, Hash, u64),
		NewLottery(u32, u32),
		Open(u32, Hash, Hash),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		InvalidPubKey,
		InvalidSignature,
		FailedToVerify,
		/// Wrong sequence number of a message
		BadMessageSequence,
		/// Bad input parameter
		InvalidInput,
		/// Invalid contract
		InvalidContract,
		InvalidOwner,
		InvalidKitty,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as KittyStorage {
		pub Kitties get(fn kitty): map hasher(blake2_128_concat) T::Hash => Kitty<T::Hash, T::Balance>;
		pub KittyOwner get(fn owner_of): map hasher(blake2_128_concat) T::Hash => Option<T::AccountId>;
		pub AllKittiesArray get(fn kitty_by_index): map hasher(blake2_128_concat) u64 => T::Hash;
		pub AllKittiesCount get(fn all_kitties_count): u64;
		pub AllKittiesIndex: map hasher(blake2_128_concat) T::Hash => u64;

		pub OwnedKittiesArray get(fn kitty_of_owner_by_index): map hasher(blake2_128_concat) (T::AccountId, u64) => T::Hash;
		pub OwnedKittiesCount get(fn owned_kitties_count): map hasher(blake2_128_concat) T::AccountId => u64;
		pub OwnedKittiesIndex: map hasher(blake2_128_concat) T::Hash => u64;

		ContractKey get(fn contract_key): map hasher(twox_64_concat) u32 => Vec<u8>;
		IngressSequence get(fn ingress_sequence): map hasher(twox_64_concat) u32 => u64;

		pub Nonce: u64;
	}
	add_extra_genesis {
		config(contract_keys): Vec<Vec<u8>>;
		build(|config: &GenesisConfig| {
			// Insert the default contract key here
			for (i, key) in config.contract_keys.iter().enumerate() {
				ContractKey::insert(i as u32, key);
			}
		});
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		fn deposit_event()= default;

		#[weight = 0]
		fn create_kitty(origin) -> Result {
			ensure_signed(origin)?;
			let sender = Self::account_id();

			let owned_kitties_count = Self::owned_kitties_count(&sender);
			let new_owned_kitties_count = owned_kitties_count.checked_add(1).ok_or("overflow")?;

			let all_kitties_count = Self::all_kitties_count();
			let new_all_kitties_count = all_kitties_count.checked_add(1).ok_or("Overflow adding a new kitty to total supply")?;
			let nonce = <Nonce>::get();
			let random_hash = (<pallet_randomness_collective_flip::Module<T>>::random_seed(), &sender, nonce)
				.using_encoded(<T as system::Config>::Hashing::hash);

			ensure!(!<KittyOwner<T>>::contains_key(random_hash), "Kitty already exists");

			let new_kitty = Kitty {
				id: random_hash,
				dna: random_hash,
				price: Zero::zero(),
				gen: 0,
			};

			<Kitties<T>>::insert(random_hash, &new_kitty);
			<KittyOwner<T>>::insert(random_hash, &sender);
			<AllKittiesArray<T>>::insert(all_kitties_count, random_hash);
			<AllKittiesCount>::put(new_all_kitties_count);
			<AllKittiesIndex<T>>::insert(random_hash, all_kitties_count);
			<OwnedKittiesArray<T>>::insert((sender.clone(), owned_kitties_count), random_hash);
			<OwnedKittiesCount<T>>::insert(&sender, new_owned_kitties_count);
			<OwnedKittiesIndex<T>>::insert(random_hash, owned_kitties_count);

			<Nonce>::mutate(|n| *n += 1);

			Self::deposit_event(RawEvent::Created(sender, random_hash));

			Ok(())
		}
		#[weight = 0]
		fn transfer(_origin, to: T::AccountId, kitty_id: T::Hash) -> Result {
			let sender = Self::account_id();

			let _owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
			let owned_kitty_count_from = Self::owned_kitties_count(&sender);
			let owned_kitty_count_to = Self::owned_kitties_count(&to);
			let new_owned_kitty_count_to = owned_kitty_count_to.checked_add(1)
				.ok_or("Transfer causes overflow of 'to' kitty balance")?;

			let new_owned_kitty_count_from = owned_kitty_count_from.checked_sub(1)
				.ok_or("Transfer causes underflow of 'from' kitty balance")?;

			let kitty_index = <OwnedKittiesIndex<T>>::get(kitty_id);
			if kitty_index != new_owned_kitty_count_from {
				let last_kitty_id = <OwnedKittiesArray<T>>::get((sender.clone(), new_owned_kitty_count_from));
				<OwnedKittiesArray<T>>::insert((sender.clone(), kitty_index), last_kitty_id);
				<OwnedKittiesIndex<T>>::insert(last_kitty_id, kitty_index);
			}
			<KittyOwner<T>>::insert(&kitty_id, &to);
			<OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count_to);

			<OwnedKittiesArray<T>>::remove((sender.clone(), new_owned_kitty_count_from));
			<OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);

			<OwnedKittiesCount<T>>::insert(&sender, new_owned_kitty_count_from);
			<OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);

			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));

			Ok(())
		}
		#[weight = 0]
		pub fn create_kitties(origin) -> Result {
			ensure_signed(origin.clone())?;
			let number = 10;
			for _i in 0..number{
				Self::create_kitty(origin.clone())?;
			}
			Ok(())
		}

		#[weight = 0]
		pub fn transfer_to_chain(origin, data: Vec<u8>) -> Result {
			const CONTRACT_ID: u32 = 6;
			let transfer_data: KittyTransferData<<T as system::Config>::AccountId> = Decode::decode(&mut &data[..]).map_err(|_| Error::<T>::InvalidInput)?;
			// Check sequence
			let sequence = IngressSequence::get(CONTRACT_ID);
			ensure!(transfer_data.data.sequence == sequence + 1, Error::<T>::BadMessageSequence);
			// Contract key
			ensure!(ContractKey::contains_key(CONTRACT_ID), Error::<T>::InvalidContract);
			let pubkey = ContractKey::get(CONTRACT_ID);

			let new_owner = &transfer_data.data.dest;
			let new_owner_kitty_id = &transfer_data.data.kitty_id;
			// Validate TEE signature
			Self::verify_signature(&pubkey, &transfer_data)?;
			// Announce the successful execution
			let kitty_id: T::Hash = Decode::decode(&mut &new_owner_kitty_id[..]).map_err(|_| Error::<T>::InvalidKitty)?;
			Self::transfer(origin, new_owner.clone(), kitty_id.clone())?;
			IngressSequence::insert(CONTRACT_ID, sequence + 1);

			Self::deposit_event(RawEvent::TransferToChain(transfer_data.data.dest, kitty_id, sequence + 1));
			Ok(())
		}
	}
}
impl<T: Config> Module<T> {
	pub fn account_id() -> T::AccountId {
		PALLET_ID.into_account()
	}
	pub fn verify_signature(
		serialized_pk: &Vec<u8>,
		data: &impl SignedDataType<Vec<u8>>,
	) -> Result {
		let pub_key = Self::try_parse_ecdsa_key(serialized_pk)?;
		let signature = secp256k1::Signature::parse_slice(&data.signature())
			.map_err(|_| Error::<T>::InvalidSignature)?;
		let msg_hash = hashing::blake2_256(&data.raw_data());
		let mut buffer = [0u8; 32];
		buffer.copy_from_slice(&msg_hash);
		let message = secp256k1::Message::parse(&buffer);
		let verified = secp256k1::verify(&message, &signature, &pub_key);
		ensure!(verified, Error::<T>::FailedToVerify);
		Ok(())
	}

	fn try_parse_ecdsa_key(
		serialized_pk: &Vec<u8>,
	) -> frame_support::dispatch::result::Result<secp256k1::PublicKey, Error<T>> {
		let mut pk = [0u8; 33];
		if serialized_pk.len() != 33 {
			return Err(Error::<T>::InvalidPubKey);
		}
		pk.copy_from_slice(&serialized_pk);
		secp256k1::PublicKey::parse_compressed(&pk).map_err(|_| Error::<T>::InvalidPubKey)
	}
}
