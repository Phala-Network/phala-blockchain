//! Phala pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use codec::Encode;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use sp_runtime::traits::Bounded;
use crate::Module as PhalaModule;
use crate::types::{
	Transfer, TransferData, RoundStats, WorkerStateEnum,
	WorkerMessagePayload, WorkerMessage, SignedWorkerMessage,
};
const CONTRACT_ID: u32 = 123;
const SEED: u32 = 0;
const DOLLARS: u128 = 1_000_000_000_000;

type SignatureAlgorithms = &'static [&'static webpki::SignatureAlgorithm];
static SUPPORTED_SIG_ALGS: SignatureAlgorithms = &[
	// &webpki::ECDSA_P256_SHA256,
	// &webpki::ECDSA_P256_SHA384,
	// &webpki::ECDSA_P384_SHA256,
	// &webpki::ECDSA_P384_SHA384,
	&webpki::RSA_PKCS1_2048_8192_SHA256,
	&webpki::RSA_PKCS1_2048_8192_SHA384,
	&webpki::RSA_PKCS1_2048_8192_SHA512,
	&webpki::RSA_PKCS1_3072_8192_SHA384,
];

pub static IAS_SERVER_ROOTS: webpki::TLSServerTrustAnchors = webpki::TLSServerTrustAnchors(&[
	/*
	 * -----BEGIN CERTIFICATE-----
	 * MIIFSzCCA7OgAwIBAgIJANEHdl0yo7CUMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNV
	 * BAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNV
	 * BAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0
	 * YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwIBcNMTYxMTE0MTUzNzMxWhgPMjA0OTEy
	 * MzEyMzU5NTlaMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwL
	 * U2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQD
	 * DCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwggGiMA0G
	 * CSqGSIb3DQEBAQUAA4IBjwAwggGKAoIBgQCfPGR+tXc8u1EtJzLA10Feu1Wg+p7e
	 * LmSRmeaCHbkQ1TF3Nwl3RmpqXkeGzNLd69QUnWovYyVSndEMyYc3sHecGgfinEeh
	 * rgBJSEdsSJ9FpaFdesjsxqzGRa20PYdnnfWcCTvFoulpbFR4VBuXnnVLVzkUvlXT
	 * L/TAnd8nIZk0zZkFJ7P5LtePvykkar7LcSQO85wtcQe0R1Raf/sQ6wYKaKmFgCGe
	 * NpEJUmg4ktal4qgIAxk+QHUxQE42sxViN5mqglB0QJdUot/o9a/V/mMeH8KvOAiQ
	 * byinkNndn+Bgk5sSV5DFgF0DffVqmVMblt5p3jPtImzBIH0QQrXJq39AT8cRwP5H
	 * afuVeLHcDsRp6hol4P+ZFIhu8mmbI1u0hH3W/0C2BuYXB5PC+5izFFh/nP0lc2Lf
	 * 6rELO9LZdnOhpL1ExFOq9H/B8tPQ84T3Sgb4nAifDabNt/zu6MmCGo5U8lwEFtGM
	 * RoOaX4AS+909x00lYnmtwsDVWv9vBiJCXRsCAwEAAaOByTCBxjBgBgNVHR8EWTBX
	 * MFWgU6BRhk9odHRwOi8vdHJ1c3RlZHNlcnZpY2VzLmludGVsLmNvbS9jb250ZW50
	 * L0NSTC9TR1gvQXR0ZXN0YXRpb25SZXBvcnRTaWduaW5nQ0EuY3JsMB0GA1UdDgQW
	 * BBR4Q3t2pn680K9+QjfrNXw7hwFRPDAfBgNVHSMEGDAWgBR4Q3t2pn680K9+Qjfr
	 * NXw7hwFRPDAOBgNVHQ8BAf8EBAMCAQYwEgYDVR0TAQH/BAgwBgEB/wIBADANBgkq
	 * hkiG9w0BAQsFAAOCAYEAeF8tYMXICvQqeXYQITkV2oLJsp6J4JAqJabHWxYJHGir
	 * IEqucRiJSSx+HjIJEUVaj8E0QjEud6Y5lNmXlcjqRXaCPOqK0eGRz6hi+ripMtPZ
	 * sFNaBwLQVV905SDjAzDzNIDnrcnXyB4gcDFCvwDFKKgLRjOB/WAqgscDUoGq5ZVi
	 * zLUzTqiQPmULAQaB9c6Oti6snEFJiCQ67JLyW/E83/frzCmO5Ru6WjU4tmsmy8Ra
	 * Ud4APK0wZTGtfPXU7w+IBdG5Ez0kE1qzxGQaL4gINJ1zMyleDnbuS8UicjJijvqA
	 * 152Sq049ESDz+1rRGc2NVEqh1KaGXmtXvqxXcTB+Ljy5Bw2ke0v8iGngFBPqCTVB
	 * 3op5KBG3RjbF6RRSzwzuWfL7QErNC8WEy5yDVARzTA5+xmBc388v9Dm21HGfcC8O
	 * DD+gT9sSpssq0ascmvH49MOgjt1yoysLtdCtJW/9FZpoOypaHx0R+mJTLwPXVMrv
	 * DaVzWh5aiEx+idkSGMnX
	 * -----END CERTIFICATE-----
	 */
	webpki::TrustAnchor {
		subject: b"1\x0b0\t\x06\x03U\x04\x06\x13\x02US1\x0b0\t\x06\x03U\x04\x08\x0c\x02CA1\x140\x12\x06\x03U\x04\x07\x0c\x0bSanta Clara1\x1a0\x18\x06\x03U\x04\n\x0c\x11Intel Corporation100.\x06\x03U\x04\x03\x0c\'Intel SGX Attestation Report Signing CA",
		spki: b"0\r\x06\t*\x86H\x86\xf7\r\x01\x01\x01\x05\x00\x03\x82\x01\x8f\x000\x82\x01\x8a\x02\x82\x01\x81\x00\x9f<d~\xb5w<\xbbQ-\'2\xc0\xd7A^\xbbU\xa0\xfa\x9e\xde.d\x91\x99\xe6\x82\x1d\xb9\x10\xd51w7\twFjj^G\x86\xcc\xd2\xdd\xeb\xd4\x14\x9dj/c%R\x9d\xd1\x0c\xc9\x877\xb0w\x9c\x1a\x07\xe2\x9cG\xa1\xae\x00IHGlH\x9fE\xa5\xa1]z\xc8\xec\xc6\xac\xc6E\xad\xb4=\x87g\x9d\xf5\x9c\t;\xc5\xa2\xe9ilTxT\x1b\x97\x9euKW9\x14\xbeU\xd3/\xf4\xc0\x9d\xdf\'!\x994\xcd\x99\x05\'\xb3\xf9.\xd7\x8f\xbf)$j\xbe\xcbq$\x0e\xf3\x9c-q\x07\xb4GTZ\x7f\xfb\x10\xeb\x06\nh\xa9\x85\x80!\x9e6\x91\tRh8\x92\xd6\xa5\xe2\xa8\x08\x03\x19>@u1@N6\xb3\x15b7\x99\xaa\x82Pt@\x97T\xa2\xdf\xe8\xf5\xaf\xd5\xfec\x1e\x1f\xc2\xaf8\x08\x90o(\xa7\x90\xd9\xdd\x9f\xe0`\x93\x9b\x12W\x90\xc5\x80]\x03}\xf5j\x99S\x1b\x96\xdei\xde3\xed\"l\xc1 }\x10B\xb5\xc9\xab\x7f@O\xc7\x11\xc0\xfeGi\xfb\x95x\xb1\xdc\x0e\xc4i\xea\x1a%\xe0\xff\x99\x14\x88n\xf2i\x9b#[\xb4\x84}\xd6\xff@\xb6\x06\xe6\x17\x07\x93\xc2\xfb\x98\xb3\x14X\x7f\x9c\xfd%sb\xdf\xea\xb1\x0b;\xd2\xd9vs\xa1\xa4\xbdD\xc4S\xaa\xf4\x7f\xc1\xf2\xd3\xd0\xf3\x84\xf7J\x06\xf8\x9c\x08\x9f\r\xa6\xcd\xb7\xfc\xee\xe8\xc9\x82\x1a\x8eT\xf2\\\x04\x16\xd1\x8cF\x83\x9a_\x80\x12\xfb\xdd=\xc7M%by\xad\xc2\xc0\xd5Z\xffo\x06\"B]\x1b\x02\x03\x01\x00\x01",
		name_constraints: None
	},
]);

pub const IAS_REPORT_SAMPLE: &[u8] = include_bytes!("../sample/report");
pub const IAS_REPORT_SIGNATURE: &[u8] = include_bytes!("../sample/report_signature");
pub const IAS_REPORT_SIGNING_CERTIFICATE: &[u8] = include_bytes!("../sample/report_signing_certificate");
pub const ENCODED_RUNTIME_INFO: &[u8] =  &[1, 122, 238, 139, 126, 110, 55, 54, 207, 3, 19, 185, 137, 120, 238, 90, 71, 2, 28, 239, 90, 188, 129, 213, 193, 164, 64, 149, 82, 38, 229, 204, 150, 142, 110, 10, 182, 8, 122, 212, 50, 211, 194, 12, 193, 229, 219, 235, 185, 232, 8, 4, 0, 0, 0, 1, 0, 0, 0];
pub const MR_ENCLAVE: &[u8] = &[197, 133, 134, 94, 240, 217, 241, 198, 183, 30, 13, 63, 33, 137, 194, 220, 173, 192, 217, 60, 149, 183, 155, 167, 154, 211, 78, 127, 110, 181, 249, 174];
pub const MR_SIGNER: &[u8] = &[131, 215, 25, 231, 125, 234, 202, 20, 112, 246, 186, 246, 42, 77, 119, 67, 3, 200, 153, 219, 105, 2, 15, 156, 112, 238, 29, 252, 8, 199, 206, 158];
pub const ISV_PROD_ID: &[u8] = &[0, 0];
pub const ISV_SVN: &[u8] = &[0, 0];
pub const TRANSFERDATASIG: &[u8] = &[183, 44, 217, 136, 114, 98, 81, 55, 139, 1, 17, 128, 125, 144, 62, 253, 168, 121, 42, 165, 3, 167, 211, 202, 133, 18, 248, 42, 189, 204, 69, 75, 68, 12, 46, 11, 158, 139, 159, 198, 37, 100, 52, 130, 225, 156, 204, 89, 158, 25, 86, 246, 182, 241, 193, 182, 200, 224, 155, 139, 192, 232, 181, 212, 0];
pub const WORKMESSAGESIG: &[u8] = &[77, 39, 136, 77, 181, 131, 61, 148, 132, 59, 159, 217, 196, 162, 190, 219, 179, 121, 60, 89, 35, 21, 101, 185, 217, 143, 154, 196, 56, 49, 153, 13, 37, 157, 76, 131, 244, 5, 217, 70, 162, 20, 13, 246, 218, 146, 27, 249, 180, 68, 150, 252, 166, 123, 167, 66, 114, 102, 31, 138, 237, 221, 220, 55, 1];

benchmarks! {
	_ { }

	push_command {
		let caller = whitelisted_caller();
		let payload = b"hello world".to_vec();
	}: {
		PhalaModule::<T>::push_command(RawOrigin::Signed(caller).into(), CONTRACT_ID, payload)?;
	}
	verify {
		// CommandNumber = 0; CommandNumber++;
		assert_eq!(PhalaModule::<T>::command_number().unwrap(), 1);
	}

	// To create the worst scenario, we set a controller as coller first,
	// then set_stash would remove it before set new controller.
	set_stash {
		let caller: T::AccountId = whitelisted_caller();
		let new_controller: T::AccountId = account("newcontroller", 0, SEED);
		let stash_state = StashInfo {
			controller: caller.clone(),
			payout_prefs: PayoutPrefs {
				commission: 0,
				target: caller.clone(),
			}
		};

		StashState::<T>::insert(&caller, stash_state);
		Stash::<T>::insert(&caller, caller.clone());
	}: {
		PhalaModule::<T>::set_stash(RawOrigin::Signed(caller.clone()).into(), new_controller.clone())?;
	}
	verify {
		// new controller should be set
		assert_eq!(PhalaModule::<T>::stash(&new_controller), caller.clone());
	}

	set_payout_prefs {
		let caller: T::AccountId = whitelisted_caller();
		let payout_commission: u32 = 99;
		let payout_target: T::AccountId = account("payouttarget", 0, SEED);

		PhalaModule::<T>::set_stash(RawOrigin::Signed(caller.clone()).into(), caller.clone())?;
	}: {
		PhalaModule::<T>::set_payout_prefs(RawOrigin::Signed(caller.clone()).into(), payout_commission.clone().into(), payout_target.clone().into())?;
	}
	verify {
		let mut stash_info = StashState::<T>::get(&caller);
		assert_eq!(stash_info.payout_prefs.commission, payout_commission);
		assert_eq!(stash_info.payout_prefs.target, payout_target);
	}

	register_worker {
		let caller: T::AccountId = whitelisted_caller();
		let sig: Vec<u8> = match base64::decode(&IAS_REPORT_SIGNATURE) {
			Ok(x) => x,
			Err(_) => panic!("decode sig failed")
		};
		let sig_cert_dec: Vec<u8> = match base64::decode_config(&IAS_REPORT_SIGNING_CERTIFICATE, base64::STANDARD) {
			Ok(x) => x,
			Err(_) => panic!("decode cert failed")
		};

		PhalaModule::<T>::add_mrenclave(RawOrigin::Root.into(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec())?;
		PhalaModule::<T>::set_stash(RawOrigin::Signed(caller.clone()).into(), caller.clone())?;
	}: {
		PhalaModule::<T>::register_worker(RawOrigin::Signed(caller).into(), ENCODED_RUNTIME_INFO.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone())?;
	}

	force_register_worker {
		let caller: T::AccountId = whitelisted_caller();

		PhalaModule::<T>::set_stash(RawOrigin::Signed(caller.clone()).into(), caller.clone())?;
	}: {
		PhalaModule::<T>::force_register_worker(RawOrigin::Root.into(), caller, vec![0], vec![1])?;
	}

	force_set_contract_key {

	}: {
		PhalaModule::<T>::force_set_contract_key(RawOrigin::Root.into(), 0, vec![0])?;
	}

	start_mining_intention {
		let caller: T::AccountId = whitelisted_caller();
		let sig: Vec<u8> = match base64::decode(&IAS_REPORT_SIGNATURE) {
			Ok(x) => x,
			Err(_) => panic!("decode sig failed")
		};
		let sig_cert_dec: Vec<u8> = match base64::decode_config(&IAS_REPORT_SIGNING_CERTIFICATE, base64::STANDARD) {
			Ok(x) => x,
			Err(_) => panic!("decode cert failed")
		};

		frame_system::Module::<T>::set_block_number(1u32.into());
		PhalaModule::<T>::add_mrenclave(RawOrigin::Root.into(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec())?;
		PhalaModule::<T>::set_stash(RawOrigin::Signed(caller.clone()).into(), caller.clone())?;
		PhalaModule::<T>::register_worker(RawOrigin::Signed(caller.clone()).into(), ENCODED_RUNTIME_INFO.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone())?;
	}: {
		PhalaModule::<T>::start_mining_intention(RawOrigin::Signed(caller.clone()).into())?;
	}
	verify {
		let stash = Stash::<T>::get(caller);
		let mut worker_info = WorkerState::<T>::get(&stash);
		assert_eq!(worker_info.state, WorkerStateEnum::MiningPending);
	}

	stop_mining_intention {
		let caller: T::AccountId = whitelisted_caller();
		let sig: Vec<u8> = match base64::decode(&IAS_REPORT_SIGNATURE) {
			Ok(x) => x,
			Err(_) => panic!("decode sig failed")
		};
		let sig_cert_dec: Vec<u8> = match base64::decode_config(&IAS_REPORT_SIGNING_CERTIFICATE, base64::STANDARD) {
			Ok(x) => x,
			Err(_) => panic!("decode cert failed")
		};

		frame_system::Module::<T>::set_block_number(1u32.into());
		PhalaModule::<T>::add_mrenclave(RawOrigin::Root.into(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec())?;
		PhalaModule::<T>::set_stash(RawOrigin::Signed(caller.clone()).into(), caller.clone())?;
		PhalaModule::<T>::register_worker(RawOrigin::Signed(caller.clone()).into(), ENCODED_RUNTIME_INFO.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone())?;
		PhalaModule::<T>::start_mining_intention(RawOrigin::Signed(caller.clone()).into())?;
	}: {
		PhalaModule::<T>::stop_mining_intention(RawOrigin::Signed(caller.clone()).into())?;
	}
	verify {
		let stash = Stash::<T>::get(caller);
		let mut worker_info = WorkerState::<T>::get(&stash);
		assert_eq!(worker_info.state, WorkerStateEnum::Free);
	}


	transfer_to_tee {
		let caller: T::AccountId = whitelisted_caller();
		// set contract key
		let pubkey = hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798").unwrap();
		PhalaModule::<T>::force_set_contract_key(RawOrigin::Root.into(), 2, pubkey)?;
		let imbalance: BalanceOf<T> = 100u32.into();
		T::TEECurrency::deposit_creating(&caller, imbalance);
		assert_eq!(imbalance, T::TEECurrency::free_balance(&caller));
	}: {
		PhalaModule::<T>::transfer_to_tee(RawOrigin::Signed(caller.clone()).into(), 50u32.into())?;
	}
	verify {
		let free_balance: BalanceOf<T> = 50u32.into();
		assert_eq!(free_balance, T::TEECurrency::free_balance(&caller));
	}

	transfer_to_chain {
		let caller: T::AccountId = whitelisted_caller();
		let receiver: T::AccountId = account("receiver", 0, SEED);

		// let raw_sk = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		let pubkey = hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798").unwrap();
		// let sk = ecdsa_load_sk(&raw_sk);
		PhalaModule::<T>::force_set_contract_key(RawOrigin::Root.into(), 2, pubkey)?;
		T::TEECurrency::deposit_creating(&caller, 200u32.into());
		PhalaModule::<T>::transfer_to_tee(RawOrigin::Signed(caller.clone()).into(), 100u32.into())?;
		let transfer = Transfer::<T::AccountId, BalanceOf<T>> {
			dest: receiver.clone(),
			amount: 50u32.into(),
			sequence: 1,
		};
		// let signature = ecdsa_sign(&sk, &transfer);

		// use prepared signature of transfer, see fn test_mockdata_transfer_to_chain() in tests.rs
		let data = TransferData {
			data: transfer,
			signature: TRANSFERDATASIG.to_vec(),
		};
	}: {
		PhalaModule::<T>::transfer_to_chain(RawOrigin::Signed(caller.clone()).into(), data.encode())?;
	}
	verify {
		let free_balance: BalanceOf<T> = 50u32.into();
		assert_eq!(free_balance, T::TEECurrency::free_balance(&receiver));
	}

	sync_worker_message {
		let caller: T::AccountId = whitelisted_caller();
		// let raw_sk = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		// let sk = ecdsa_load_sk(&raw_sk);
		let pubkey = hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798").unwrap();

		let sig: Vec<u8> = match base64::decode(&IAS_REPORT_SIGNATURE) {
			Ok(x) => x,
			Err(_) => panic!("decode sig failed")
		};
		let sig_cert_dec: Vec<u8> = match base64::decode_config(&IAS_REPORT_SIGNING_CERTIFICATE, base64::STANDARD) {
			Ok(x) => x,
			Err(_) => panic!("decode cert failed")
		};
		PhalaModule::<T>::add_mrenclave(RawOrigin::Root.into(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec())?;
		PhalaModule::<T>::set_stash(RawOrigin::Signed(caller.clone()).into(), caller.clone())?;
		PhalaModule::<T>::register_worker(RawOrigin::Signed(caller.clone()).into(), ENCODED_RUNTIME_INFO.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone())?;
		PhalaModule::<T>::start_mining_intention(RawOrigin::Signed(caller.clone()).into())?;

		let stash = Stash::<T>::get(&caller);
		let mut worker_info = WorkerState::<T>::get(&stash);
		worker_info.pubkey = pubkey.clone();
		WorkerState::<T>::insert(stash, worker_info);

		let work_message = WorkerMessage {
			payload: WorkerMessagePayload::Heartbeat{
				block_num: 123u32,
				claim_online: true,
				claim_compute: true,
			},
			sequence: 0,
		};
		// let signature = ecdsa_sign(&sk, &work_message);

		// use prepared WorkMessage signature, see fn test_mockdata_sync_worker_message() in tests.rs
		let signed_workmessage = SignedWorkerMessage {
			data: work_message,
			signature: WORKMESSAGESIG.to_vec(),
		};
	}: {
		PhalaModule::<T>::sync_worker_message(RawOrigin::Signed(caller.clone()).into(), signed_workmessage.encode())?;
	}

	force_next_round {
	}: {
		PhalaModule::<T>::force_next_round(RawOrigin::Root.into())?;
	}

	force_add_fire {
		let fire0: T::AccountId = account("fire0", 0, SEED);
		let fire1: T::AccountId = account("fire1", 0, SEED);
		let fire2: T::AccountId = account("fire2", 0, SEED);

	}: {
		PhalaModule::<T>::force_add_fire(
			RawOrigin::Root.into(),
			vec![fire1.clone(), fire2.clone()],
			vec![100u32.into(), 200u32.into()],
		)?;
	}
	verify {
		assert_eq!(PhalaModule::<T>::fire(fire0.clone()), 0u32.into());
		assert_eq!(PhalaModule::<T>::fire(fire1.clone()), 100u32.into());
		assert_eq!(PhalaModule::<T>::fire(fire2.clone()), 200u32.into());
	}

	add_mrenclave {
		frame_system::Module::<T>::set_block_number(1u32.into());
	}: {
		PhalaModule::<T>::add_mrenclave(RawOrigin::Root.into(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec())?;
	}

	remove_mrenclave_by_raw_data {
		frame_system::Module::<T>::set_block_number(1u32.into());
		PhalaModule::<T>::add_mrenclave(RawOrigin::Root.into(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec())?;
	}: {
		PhalaModule::<T>::remove_mrenclave_by_raw_data(RawOrigin::Root.into(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec())?;
	}

	remove_mrenclave_by_index {
		frame_system::Module::<T>::set_block_number(1u32.into());
		PhalaModule::<T>::add_mrenclave(RawOrigin::Root.into(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec())?;
	}: {
		PhalaModule::<T>::remove_mrenclave_by_index(RawOrigin::Root.into(), 0)?;
	}
}