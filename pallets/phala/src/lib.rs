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
use sp_runtime::{traits::AccountIdConversion, ModuleId};
use frame_support::{decl_module, decl_event, decl_storage, decl_error, ensure, dispatch};
use frame_system::{self as system, ensure_signed};
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

// TODO: Enable root certificate check
//
// type SignatureAlgorithms = &'static [&'static webpki::SignatureAlgorithm];
// static SUPPORTED_SIG_ALGS: SignatureAlgorithms = &[
// 	// &webpki::ECDSA_P256_SHA256,
// 	// &webpki::ECDSA_P256_SHA384,
// 	// &webpki::ECDSA_P384_SHA256,
// 	// &webpki::ECDSA_P384_SHA384,
// 	&webpki::RSA_PKCS1_2048_8192_SHA256,
// 	&webpki::RSA_PKCS1_2048_8192_SHA384,
// 	&webpki::RSA_PKCS1_2048_8192_SHA512,
// 	&webpki::RSA_PKCS1_3072_8192_SHA384,
// ];

// pub static IAS_SERVER_ROOTS: webpki::TLSServerTrustAnchors = webpki::TLSServerTrustAnchors(&[
// 	/*
// 	 * -----BEGIN CERTIFICATE-----
// 	 * MIIFSzCCA7OgAwIBAgIJANEHdl0yo7CUMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNV
// 	 * BAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNV
// 	 * BAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0
// 	 * YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwIBcNMTYxMTE0MTUzNzMxWhgPMjA0OTEy
// 	 * MzEyMzU5NTlaMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwL
// 	 * U2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQD
// 	 * DCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwggGiMA0G
// 	 * CSqGSIb3DQEBAQUAA4IBjwAwggGKAoIBgQCfPGR+tXc8u1EtJzLA10Feu1Wg+p7e
// 	 * LmSRmeaCHbkQ1TF3Nwl3RmpqXkeGzNLd69QUnWovYyVSndEMyYc3sHecGgfinEeh
// 	 * rgBJSEdsSJ9FpaFdesjsxqzGRa20PYdnnfWcCTvFoulpbFR4VBuXnnVLVzkUvlXT
// 	 * L/TAnd8nIZk0zZkFJ7P5LtePvykkar7LcSQO85wtcQe0R1Raf/sQ6wYKaKmFgCGe
// 	 * NpEJUmg4ktal4qgIAxk+QHUxQE42sxViN5mqglB0QJdUot/o9a/V/mMeH8KvOAiQ
// 	 * byinkNndn+Bgk5sSV5DFgF0DffVqmVMblt5p3jPtImzBIH0QQrXJq39AT8cRwP5H
// 	 * afuVeLHcDsRp6hol4P+ZFIhu8mmbI1u0hH3W/0C2BuYXB5PC+5izFFh/nP0lc2Lf
// 	 * 6rELO9LZdnOhpL1ExFOq9H/B8tPQ84T3Sgb4nAifDabNt/zu6MmCGo5U8lwEFtGM
// 	 * RoOaX4AS+909x00lYnmtwsDVWv9vBiJCXRsCAwEAAaOByTCBxjBgBgNVHR8EWTBX
// 	 * MFWgU6BRhk9odHRwOi8vdHJ1c3RlZHNlcnZpY2VzLmludGVsLmNvbS9jb250ZW50
// 	 * L0NSTC9TR1gvQXR0ZXN0YXRpb25SZXBvcnRTaWduaW5nQ0EuY3JsMB0GA1UdDgQW
// 	 * BBR4Q3t2pn680K9+QjfrNXw7hwFRPDAfBgNVHSMEGDAWgBR4Q3t2pn680K9+Qjfr
// 	 * NXw7hwFRPDAOBgNVHQ8BAf8EBAMCAQYwEgYDVR0TAQH/BAgwBgEB/wIBADANBgkq
// 	 * hkiG9w0BAQsFAAOCAYEAeF8tYMXICvQqeXYQITkV2oLJsp6J4JAqJabHWxYJHGir
// 	 * IEqucRiJSSx+HjIJEUVaj8E0QjEud6Y5lNmXlcjqRXaCPOqK0eGRz6hi+ripMtPZ
// 	 * sFNaBwLQVV905SDjAzDzNIDnrcnXyB4gcDFCvwDFKKgLRjOB/WAqgscDUoGq5ZVi
// 	 * zLUzTqiQPmULAQaB9c6Oti6snEFJiCQ67JLyW/E83/frzCmO5Ru6WjU4tmsmy8Ra
// 	 * Ud4APK0wZTGtfPXU7w+IBdG5Ez0kE1qzxGQaL4gINJ1zMyleDnbuS8UicjJijvqA
// 	 * 152Sq049ESDz+1rRGc2NVEqh1KaGXmtXvqxXcTB+Ljy5Bw2ke0v8iGngFBPqCTVB
// 	 * 3op5KBG3RjbF6RRSzwzuWfL7QErNC8WEy5yDVARzTA5+xmBc388v9Dm21HGfcC8O
// 	 * DD+gT9sSpssq0ascmvH49MOgjt1yoysLtdCtJW/9FZpoOypaHx0R+mJTLwPXVMrv
// 	 * DaVzWh5aiEx+idkSGMnX
// 	 * -----END CERTIFICATE-----
// 	 */
// 	webpki::TrustAnchor {
// 		subject: b"1\x0b0\t\x06\x03U\x04\x06\x13\x02US1\x0b0\t\x06\x03U\x04\x08\x0c\x02CA1\x140\x12\x06\x03U\x04\x07\x0c\x0bSanta Clara1\x1a0\x18\x06\x03U\x04\n\x0c\x11Intel Corporation100.\x06\x03U\x04\x03\x0c\'Intel SGX Attestation Report Signing CA",
// 		spki: b"0\r\x06\t*\x86H\x86\xf7\r\x01\x01\x01\x05\x00\x03\x82\x01\x8f\x000\x82\x01\x8a\x02\x82\x01\x81\x00\x9f<d~\xb5w<\xbbQ-\'2\xc0\xd7A^\xbbU\xa0\xfa\x9e\xde.d\x91\x99\xe6\x82\x1d\xb9\x10\xd51w7\twFjj^G\x86\xcc\xd2\xdd\xeb\xd4\x14\x9dj/c%R\x9d\xd1\x0c\xc9\x877\xb0w\x9c\x1a\x07\xe2\x9cG\xa1\xae\x00IHGlH\x9fE\xa5\xa1]z\xc8\xec\xc6\xac\xc6E\xad\xb4=\x87g\x9d\xf5\x9c\t;\xc5\xa2\xe9ilTxT\x1b\x97\x9euKW9\x14\xbeU\xd3/\xf4\xc0\x9d\xdf\'!\x994\xcd\x99\x05\'\xb3\xf9.\xd7\x8f\xbf)$j\xbe\xcbq$\x0e\xf3\x9c-q\x07\xb4GTZ\x7f\xfb\x10\xeb\x06\nh\xa9\x85\x80!\x9e6\x91\tRh8\x92\xd6\xa5\xe2\xa8\x08\x03\x19>@u1@N6\xb3\x15b7\x99\xaa\x82Pt@\x97T\xa2\xdf\xe8\xf5\xaf\xd5\xfec\x1e\x1f\xc2\xaf8\x08\x90o(\xa7\x90\xd9\xdd\x9f\xe0`\x93\x9b\x12W\x90\xc5\x80]\x03}\xf5j\x99S\x1b\x96\xdei\xde3\xed\"l\xc1 }\x10B\xb5\xc9\xab\x7f@O\xc7\x11\xc0\xfeGi\xfb\x95x\xb1\xdc\x0e\xc4i\xea\x1a%\xe0\xff\x99\x14\x88n\xf2i\x9b#[\xb4\x84}\xd6\xff@\xb6\x06\xe6\x17\x07\x93\xc2\xfb\x98\xb3\x14X\x7f\x9c\xfd%sb\xdf\xea\xb1\x0b;\xd2\xd9vs\xa1\xa4\xbdD\xc4S\xaa\xf4\x7f\xc1\xf2\xd3\xd0\xf3\x84\xf7J\x06\xf8\x9c\x08\x9f\r\xa6\xcd\xb7\xfc\xee\xe8\xc9\x82\x1a\x8eT\xf2\\\x04\x16\xd1\x8cF\x83\x9a_\x80\x12\xfb\xdd=\xc7M%by\xad\xc2\xc0\xd5Z\xffo\x06\"B]\x1b\x02\x03\x01\x00\x01",
// 		name_constraints: None
// 	},
// ]);

type BalanceOf<T> = <<T as Trait>::TEECurrency as Currency<<T as system::Trait>::AccountId>>::Balance;
type SequenceType = u32;
const PALLET_ID: ModuleId = ModuleId(*b"Phala!!!");

#[derive(Encode, Decode)]
pub struct Transfer<AccountId, Balance> {
	pub dest: AccountId,
	pub amount: Balance,
	pub sequence: SequenceType,
}
#[derive(Encode, Decode)]
pub struct TransferData<AccountId, Balance> {
	pub data: Transfer<AccountId, Balance>,
	pub signature: Vec<u8>,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type TEECurrency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as PhalaModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		CommandNumber get(fn command_number): Option<u32>;

		// Store a map of Machine and account, map Vec<u8> => T::AccountId
		MachineOwner get(fn owners): map hasher(blake2_128_concat) Vec<u8> => T::AccountId;

		// Store a map of Machine and account, map Vec<u8> => (pub_key, score)
		Machine get(fn machines): map hasher(blake2_128_concat) Vec<u8> => (Vec<u8>, u8);

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

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, Balance = BalanceOf<T> {
		CommandPushed(AccountId, u32, Vec<u8>, u32),
		LogString(Vec<u8>),
		TransferToTee(Vec<u8>, Balance),
		TransferToChain(Vec<u8>, Balance, SequenceType),
		WorkerRegistered(AccountId, Vec<u8>),
		WorkerUnregistered(AccountId, Vec<u8>),
		SimpleEvent(u32),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
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
type Score = u8;
#[derive(Encode, Decode)]
struct TEERuntimeInfo {
	machine_id: MachineId,
	pub_key: PublicKey,
	score: Score
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
			Machine::insert(machine_id.clone(), (runtime_info.pub_key.to_vec(), runtime_info.score));
			if <MachineOwner<T>>::contains_key(&machine_id) {
				let last_owner =  <MachineOwner<T>>::get(machine_id.clone());
				<Miner<T>>::remove(&last_owner);
				Self::deposit_event(RawEvent::WorkerUnregistered(last_owner, machine_id.clone()));
			}
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

			let data = Decode::decode(&mut &data[..]);
			ensure!(data.is_ok(), "Bad transaction data");
			let transfer_data: TransferData<<T as system::Trait>::AccountId, BalanceOf<T>> = data.unwrap();

			let sequence = Sequence::get();
			ensure!(transfer_data.data.sequence == sequence + 1, "Bad sequence");

			ensure!(<Miner<T>>::contains_key(&who), Error::<T>::MinerNotFound);

			let machine_id = <Miner<T>>::get(who);
			ensure!(Machine::contains_key(&machine_id), Error::<T>::BadMachineId);

			let serialized_pk = Machine::get(machine_id).0;
			Self::verify_signature(serialized_pk, &transfer_data)?;

			T::TEECurrency::transfer(&Self::account_id(), &transfer_data.dest, transfer_data.amount, AllowDeath)
				.map_err(|_| dispatch::DispatchError::Other("Can't transfer to chain"))?;

			Sequence::set(sequence + 1);

			Self::deposit_event(RawEvent::TransferToChain(transfer_data.dest.encode(), transfer_data.amount, sequence + 1));

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

	pub fn verify_signature(serialized_pk: Vec<u8>, transfer_data: &TransferData<<T as system::Trait>::AccountId, BalanceOf<T>>) -> dispatch::DispatchResult {
		let mut pk = [0u8; 33];
		pk.copy_from_slice(&serialized_pk);
		let pub_key = secp256k1::PublicKey::parse_compressed(&pk);
		ensure!(pub_key.is_ok(), Error::<T>::InvalidPubKey);

		let signature = secp256k1::Signature::parse_slice(&transfer_data.signature);
		ensure!(signature.is_ok(), Error::<T>::InvalidSignature);

		let msg_hash = hashing::blake2_256(&Encode::encode(&transfer_data.data));
		let mut buffer = [0u8; 32];
		buffer.copy_from_slice(&msg_hash);
		let message = secp256k1::Message::parse(&buffer);

		let verified = secp256k1::verify(&message, &signature.unwrap(), &pub_key.unwrap());
		ensure!(verified, Error::<T>::FailedToVerify);

		Ok(())
	}
}
