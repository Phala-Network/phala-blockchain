#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

extern crate alloc;

extern crate untrusted;
extern crate base64;
extern crate itertools;
extern crate hex;

extern crate webpki;

use frame_support::{ensure, decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get};
use frame_system::{self as system, ensure_signed, ensure_root};

use alloc::vec::Vec;
use sp_runtime::{traits::AccountIdConversion, ModuleId};
use frame_support::{
	traits::{Currency, ExistenceRequirement::AllowDeath},
};
use codec::{Encode, Decode};
use sp_std::prelude::*;
use secp256k1;

mod hashing;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::TEECurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type SequenceType = u32;
const PALLET_ID: ModuleId = ModuleId(*b"Phala!!!");

#[derive(Encode, Decode)]
pub struct Transfer<AccountId, Balance> {
	pub dest: AccountId,
	pub amount: Balance,
	pub sequence: SequenceType,
}

pub trait SignedDataType<T> {
	fn raw_data(&self) -> Vec<u8>;
	fn signature(&self) -> T;
}

#[derive(Encode, Decode)]
pub struct TransferData<AccountId, Balance> {
	pub data: Transfer<AccountId, Balance>,
	pub signature: Vec<u8>,
}

impl<AccountId: Encode, Balance: Encode> SignedDataType<Vec<u8>> for TransferData<AccountId, Balance> {
	fn raw_data(&self) -> Vec<u8> {
		Encode::encode(&self.data)
	}

	fn signature(&self) -> Vec<u8> {
		self.signature.clone()
	}
}

#[derive(Encode, Decode)]
pub struct HeartbeatData {
	pub data: Vec<u8>,
	pub signature: Vec<u8>,
}

impl SignedDataType<Vec<u8>> for HeartbeatData {
	fn raw_data(&self) -> Vec<u8> {
		self.data.clone()
	}

	fn signature(&self) -> Vec<u8> {
		self.signature.clone()
	}
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	type TEECurrency: Currency<Self::AccountId>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Trait> as PhalaModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		CommandNumber get(fn command_number): Option<u32>;

		// Store a map of Machine and account, map Vec<u8> => T::AccountId
		MachineOwner get(fn owners): map hasher(blake2_128_concat) Vec<u8> => T::AccountId;

		// Store a map of Machine and account, map Vec<u8> => (pub_key, score)
		Machine get(fn machines): map hasher(blake2_128_concat) Vec<u8> => (Vec<u8>, u32);

		// Store a map of Account and Machine, map T::AccountId => Vec<u8>
		pub Miner get(fn miner): map hasher(blake2_128_concat) T::AccountId => Vec<u8>;

		pub Sequence get(fn sequence): SequenceType;
	}

	add_extra_genesis {
		config(stakers): Vec<T::AccountId>;
		build(|config: &GenesisConfig<T>| {
			for controller in &config.stakers {
				<Miner<T>>::insert(controller.clone(), "Unknown machine id".as_bytes().to_vec());
			}
		});
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId, Balance = BalanceOf<T> {
		CommandPushed(AccountId, u32, Vec<u8>, u32),
		LogString(Vec<u8>),
		TransferToTee(Vec<u8>, Balance),
		TransferToChain(Vec<u8>, Balance, SequenceType),
		WorkerRegistered(AccountId, Vec<u8>),
		WorkerUnregistered(AccountId, Vec<u8>),
		SimpleEvent(u32),
		Heartbeat(),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		BadTransactionData,
		InvalidIASSigningCert,
		InvalidIASReportSignature,
		InvalidQuoteStatus,
		InvalidRuntimeInfo,
		MinerNotFound,
		BadMachineId,
		InvalidPubKey,
		InvalidSignature,
		FailedToVerify,
	}
}

type MachineId = [u8; 16];
type PublicKey = [u8; 33];
type Score = u32;

#[derive(Encode, Decode)]
struct TEERuntimeInfo {
	machine_id: MachineId,
	pub_key: PublicKey,
	score: Score
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
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
		pub fn register_worker(origin, encoded_runtime_info: Vec<u8>, report: Vec<u8>, signature: Vec<u8>, raw_signing_cert: Vec<u8>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// Validate report
			let sig_cert = webpki::EndEntityCert::from(&raw_signing_cert);
			ensure!(sig_cert.is_ok(), Error::<T>::InvalidIASSigningCert);
			let sig_cert = sig_cert.unwrap();
			let verify_result = sig_cert.verify_signature(
				&webpki::RSA_PKCS1_2048_8192_SHA256,
				&report,
				&signature
			);
			ensure!(verify_result.is_ok(), Error::<T>::InvalidIASSigningCert);
			// TODO: Validate certificate
			// let chain: Vec<&[u8]> = Vec::new();
			// let now_func = webpki::Time::from_seconds_since_unix_epoch(1573419050);
			// match sig_cert.verify_is_valid_tls_server_cert(
			// 	SUPPORTED_SIG_ALGS,
			// 	&IAS_SERVER_ROOTS,
			// 	&chain,
			// 	now_func
			// ) {
			// 	Ok(()) => (),
			// 	Err(_) => panic!("verify cert failed")
			// };

			// Validate related fields
			let parsed_report: serde_json_no_std::Value = serde_json_no_std::from_slice(&report).unwrap();
			ensure!(
				&parsed_report["isvEnclaveQuoteStatus"] == "OK" || &parsed_report["isvEnclaveQuoteStatus"] == "CONFIGURATION_NEEDED" || &parsed_report["isvEnclaveQuoteStatus"] == "GROUP_OUT_OF_DATE",
				Error::<T>::InvalidQuoteStatus
			);
			// Extract quote fields
			let raw_quote_body = parsed_report["isvEnclaveQuoteBody"].as_str().unwrap();
			let quote_body = base64::decode(&raw_quote_body).unwrap();
			// TODO: check the following fields
			// let mr_enclave = &quote_body[112..143];
			// let isv_prod_id = &quote_body[304..305];
			// let isv_svn = &quote_body[306..307];
			let report_data = &quote_body[368..432];
			// Validate report data
			let runtime_info_hash = hashing::blake2_512(&encoded_runtime_info);
			ensure!(runtime_info_hash.to_vec() == report_data, Error::<T>::InvalidRuntimeInfo);
			let runtime_info = TEERuntimeInfo::decode(&mut &encoded_runtime_info[..]);
			ensure!(runtime_info.is_ok(), Error::<T>::InvalidRuntimeInfo);
			let runtime_info = runtime_info.unwrap();
			// Associate account with machine id
			let machine_id = runtime_info.machine_id.to_vec();
			Self::remove_machine_if_present(&machine_id);
			Machine::insert(machine_id.clone(), (runtime_info.pub_key.to_vec(), runtime_info.score));
			<MachineOwner<T>>::insert(machine_id.clone(), who.clone());
			<Miner<T>>::insert(who.clone(), machine_id.clone());
			Self::deposit_event(RawEvent::WorkerRegistered(who, machine_id));

			Ok(())
		}

		#[weight = 0]
		fn transfer_to_tee(origin, #[compact] amount: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			T::TEECurrency::transfer(&who, &Self::account_id(), amount, AllowDeath)
				.map_err(|_| dispatch::DispatchError::Other("Can't transfer to tee"))?;

			Self::deposit_event(RawEvent::TransferToTee(who.encode(), amount));

			Ok(())
		}

		#[weight = 0]
		fn transfer_to_chain(origin, data: Vec<u8>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// Decode payload
			let data = Decode::decode(&mut &data[..]);
			ensure!(data.is_ok(), "Bad transaction data");

			let transfer_data: TransferData<<T as frame_system::Trait>::AccountId, BalanceOf<T>> = data.unwrap();
			// Check sequence
			let sequence = Sequence::get();
			ensure!(transfer_data.data.sequence == sequence + 1, "Bad sequence");
			// Get Identity key from account
			ensure!(<Miner<T>>::contains_key(&who), Error::<T>::MinerNotFound);
			let machine_id = <Miner<T>>::get(who);
			ensure!(Machine::contains_key(&machine_id), Error::<T>::BadMachineId);
			let serialized_pk = Machine::get(machine_id).0;
			// Validate TEE signature
			Self::verify_signature(serialized_pk, &transfer_data)?;

			// Release funds
			T::TEECurrency::transfer(&Self::account_id(), &transfer_data.data.dest, transfer_data.data.amount, AllowDeath)
				.map_err(|_| dispatch::DispatchError::Other("Can't transfer to chain"))?;
			// Advance sequence
			Sequence::set(sequence + 1);
			// Emit event
			Self::deposit_event(RawEvent::TransferToChain(transfer_data.data.dest.encode(), transfer_data.data.amount, sequence + 1));

			Ok(())
		}

		#[weight = 0]
		fn force_register_worker(origin, controller: T::AccountId, machine_id: Vec<u8>, pub_key: Vec<u8>) -> dispatch::DispatchResult {
			ensure_root(origin)?;

			// // Store a map of Machine and account, map Vec<u8> => T::AccountId
			// MachineOwner get(fn owners): map hasher(blake2_128_concat) Vec<u8> => T::AccountId;

			// // Store a map of Machine and account, map Vec<u8> => (pub_key, score)
			// Machine get(fn machines): map hasher(blake2_128_concat) Vec<u8> => (Vec<u8>, u8);

			// // Store a map of Account and Machine, map T::AccountId => Vec<u8>
			// pub Miner get(fn miner): map hasher(blake2_128_concat) T::AccountId => Vec<u8>;

			Self::remove_machine_if_present(&machine_id);
			Machine::insert(machine_id.clone(), (pub_key, 0));
			<MachineOwner<T>>::insert(machine_id.clone(), controller.clone());
			<Miner<T>>::insert(controller, machine_id);

			Ok(())
		}

		#[weight = 0]
		fn heartbeat(origin, data: Vec<u8>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// Decode payload
			let data = Decode::decode(&mut &data[..]);
			ensure!(data.is_ok(), "Bad transaction data");

			let heartbeat_data: HeartbeatData = data.unwrap();
			// Get Identity key from account
			ensure!(<Miner<T>>::contains_key(&who), Error::<T>::MinerNotFound);
			let machine_id = <Miner<T>>::get(who);
			ensure!(Machine::contains_key(&machine_id), Error::<T>::BadMachineId);
			let serialized_pk = Machine::get(machine_id).0;
			// Validate TEE signature
			Self::verify_signature(serialized_pk, &heartbeat_data)?;

			// Emit event
			Self::deposit_event(RawEvent::Heartbeat());

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn account_id() -> T::AccountId {
		PALLET_ID.into_account()
	}

	pub fn is_miner(who: T::AccountId) -> bool {
		<Miner<T>>::contains_key(&who)
	}

	pub fn verify_signature(serialized_pk: Vec<u8>, data: &impl SignedDataType<Vec<u8>>) -> dispatch::DispatchResult {
		let mut pk = [0u8; 33];
		pk.copy_from_slice(&serialized_pk);
		let pub_key = secp256k1::PublicKey::parse_compressed(&pk);
		ensure!(pub_key.is_ok(), Error::<T>::InvalidPubKey);

		let signature = secp256k1::Signature::parse_slice(&data.signature());
		ensure!(signature.is_ok(), Error::<T>::InvalidSignature);

		let msg_hash = hashing::blake2_256(&data.raw_data());
		let mut buffer = [0u8; 32];
		buffer.copy_from_slice(&msg_hash);
		let message = secp256k1::Message::parse(&buffer);

		let verified = secp256k1::verify(&message, &signature.unwrap(), &pub_key.unwrap());
		ensure!(verified, Error::<T>::FailedToVerify);

		Ok(())
	}

	fn remove_machine_if_present(machine_id: &Vec<u8>) {
		if <MachineOwner<T>>::contains_key(machine_id) {
			let last_owner = <MachineOwner<T>>::get(&machine_id);
			<Miner<T>>::remove(&last_owner);
			Self::deposit_event(RawEvent::WorkerUnregistered(last_owner, machine_id.clone()));
		}
	}
}
