use codec::Encode;
use frame_support::{assert_ok, assert_noop, traits::{Currency, OnFinalize}};
use frame_system::RawOrigin;
use hex_literal::hex;
use secp256k1;
use sp_runtime::traits::BadOrigin;

use crate::{Error, mock::*};
use crate::{RawEvent, types::{Transfer, TransferData, RoundStats, WorkerStateEnum}};

fn events() -> Vec<TestEvent> {
	let evt = System::events().into_iter().map(|evt| evt.event).collect::<Vec<_>>();
	System::reset_events();
	evt
}

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
const PANIC: bool = false;

#[test]
fn test_validate_cert() {
	let sig: Vec<u8> = match base64::decode(&IAS_REPORT_SIGNATURE) {
		Ok(x) => x,
		Err(_) => panic!("decode sig failed")
	};

	let sig_cert_dec: Vec<u8> = match base64::decode_config(&IAS_REPORT_SIGNING_CERTIFICATE, base64::STANDARD) {
		Ok(x) => x,
		Err(_) => panic!("decode cert failed")
	};
	let sig_cert: webpki::EndEntityCert = match webpki::EndEntityCert::from(&sig_cert_dec) {
		Ok(x) => x,
		Err(_) => panic!("parse sig failed")
	};

	let chain: Vec<&[u8]> = Vec::new();
	let now_func = webpki::Time::from_seconds_since_unix_epoch(1573419050);

	match sig_cert.verify_is_valid_tls_server_cert(
		SUPPORTED_SIG_ALGS,
		&IAS_SERVER_ROOTS,
		&chain,
		now_func
	) {
		Ok(()) => (),
		Err(_) => panic!("verify cert failed")
	};

	let verify_result = sig_cert.verify_signature(
		&webpki::RSA_PKCS1_2048_8192_SHA256,
		&IAS_REPORT_SAMPLE,
		&sig
	);

	if PANIC {
		verify_result.unwrap();
	} else {
		match verify_result {
			Ok(()) => (),
			Err(_) => panic!("verify sig failed")
		};
	}
}

#[test]
fn test_register_worker() {
	new_test_ext().execute_with(|| {
		let sig: Vec<u8> = match base64::decode(&IAS_REPORT_SIGNATURE) {
			Ok(x) => x,
			Err(_) => panic!("decode sig failed")
		};

		let sig_cert_dec: Vec<u8> = match base64::decode_config(&IAS_REPORT_SIGNING_CERTIFICATE, base64::STANDARD) {
			Ok(x) => x,
			Err(_) => panic!("decode cert failed")
		};
		assert_ok!(PhalaModule::add_mrenclave(Origin::root(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec()));
		assert_ok!(PhalaModule::set_stash(Origin::signed(1), 1));
		assert_ok!(PhalaModule::register_worker(Origin::signed(1), ENCODED_RUNTIME_INFO.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone()));
		assert_ok!(PhalaModule::register_worker(Origin::signed(1), ENCODED_RUNTIME_INFO.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone()));
	});
}

#[test]
fn test_whitelist_works() {
	new_test_ext().execute_with(|| {
		// Set block number to 1 to test the events
		System::set_block_number(1);

		let sig: Vec<u8> = base64::decode(&IAS_REPORT_SIGNATURE).expect("decode sig failed");
		let sig_cert_dec: Vec<u8> = base64::decode_config(&IAS_REPORT_SIGNING_CERTIFICATE, base64::STANDARD).expect("decode cert failed");

		assert_ok!(PhalaModule::set_stash(Origin::signed(1), 1));
		assert_ok!(PhalaModule::set_stash(Origin::signed(2), 2));

		// TODO: Handle RA report replay attack
		assert_ok!(PhalaModule::add_mrenclave(Origin::root(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec()));
		assert_ok!(PhalaModule::register_worker(Origin::signed(1), ENCODED_RUNTIME_INFO.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone()));
		let machine_id = &PhalaModule::worker_state(1).machine_id;
		assert_eq!(true, machine_id.len() > 0);
		assert_ok!(PhalaModule::register_worker(Origin::signed(2), ENCODED_RUNTIME_INFO.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone()));
		let machine_id2 = &PhalaModule::worker_state(2).machine_id;
		assert_eq!(true, machine_id2.len() > 0);
		let machine_id1 = &PhalaModule::worker_state(1).machine_id;
		assert_eq!(true, machine_id1.len() == 0);
		// Check emitted events
		assert_eq!(
			true,
			match events().as_slice() {[
					TestEvent::phala(RawEvent::WhitelistAdded(_)),
					TestEvent::phala(RawEvent::WorkerRegistered(1, _, _)),
					TestEvent::phala(RawEvent::WorkerUnregistered(1, _)),
					TestEvent::phala(RawEvent::WorkerRegistered(2, _, _))
				] => true,
				_ => false
			}
		);
	});
}

#[test]
fn test_remove_mrenclave_works() {
	new_test_ext().execute_with(|| {
		// Set block number to 1 to test the events
		System::set_block_number(1);
		assert_ok!(PhalaModule::add_mrenclave(Origin::root(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec()));
		assert_ok!(PhalaModule::remove_mrenclave_by_raw_data(Origin::root(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec()));
		assert_ok!(PhalaModule::add_mrenclave(Origin::root(), MR_ENCLAVE.to_vec(), MR_SIGNER.to_vec(), ISV_PROD_ID.to_vec(), ISV_SVN.to_vec()));
		assert_ok!(PhalaModule::remove_mrenclave_by_index(Origin::root(), 0));

		// Check emitted events
		assert_eq!(
			true,
			match events().as_slice() {[
			TestEvent::phala(RawEvent::WhitelistAdded(_)),
			TestEvent::phala(RawEvent::WhitelistRemoved(_)),
			TestEvent::phala(RawEvent::WhitelistAdded(_)),
			TestEvent::phala(RawEvent::WhitelistRemoved(_)),
			] => true,
				_ => false
			}
		);
	});
}

#[test]
fn test_verify_signature() {
	use rand;

	new_test_ext().execute_with(|| {
		let data = Transfer {
			dest: 1u64,
			amount: 2u128,
			sequence: 3u64,
		};

		let mut prng = rand::rngs::OsRng::default();
		let sk = secp256k1::SecretKey::random(&mut prng);
		let pk = secp256k1::PublicKey::from_secret_key(&sk);
		let serialized_pk = pk.serialize_compressed().to_vec();
		let signature = ecdsa_sign(&sk, &data);
		let transfer_data = super::TransferData {
			data, signature,
		};

		let actual = PhalaModule::verify_signature(&serialized_pk, &transfer_data);
		assert_eq!(true, actual.is_ok());
	});
}

#[test]
fn test_force_register_worker() {
	new_test_ext().execute_with(|| {
		assert_ok!(PhalaModule::set_stash(Origin::signed(1), 1));
		assert_ok!(PhalaModule::force_register_worker(RawOrigin::Root.into(), 1, vec![0], vec![1]));
		assert_noop!(PhalaModule::force_register_worker(Origin::signed(1), 1, vec![0], vec![1]), BadOrigin);
	});
}

#[test]
fn test_mine() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		// Invalid actions
		assert_noop!(PhalaModule::start_mining_intention(Origin::signed(1)), Error::<Test>::ControllerNotFound);
		assert_noop!(PhalaModule::stop_mining_intention(Origin::signed(1)), Error::<Test>::ControllerNotFound);
		assert_ok!(PhalaModule::set_stash(Origin::signed(1), 1));
		assert_ok!(PhalaModule::force_register_worker(RawOrigin::Root.into(), 1, vec![0], vec![1]));
		// Free <-> MiningPending
		assert_ok!(PhalaModule::start_mining_intention(Origin::signed(1)));
		assert_eq!(PhalaModule::worker_state(1).state, WorkerStateEnum::MiningPending);
		assert_ok!(PhalaModule::stop_mining_intention(Origin::signed(1)));
		assert_eq!(PhalaModule::worker_state(1).state, WorkerStateEnum::Free);
		// MiningPending -> Mining
		assert_ok!(PhalaModule::start_mining_intention(Origin::signed(1)));
		assert_ok!(PhalaModule::force_next_round(RawOrigin::Root.into()));
		PhalaModule::on_finalize(1);
		System::finalize();
		assert_matches!(PhalaModule::worker_state(1).state, WorkerStateEnum::Mining(_));
		assert_eq!(PhalaModule::online_workers(), 1);  // Miner stats increased
		// Mining -> MiningStopping
		System::set_block_number(2);
		assert_ok!(PhalaModule::stop_mining_intention(Origin::signed(1)));
		assert_eq!(PhalaModule::worker_state(1).state, WorkerStateEnum::MiningStopping);
		assert_eq!(PhalaModule::online_workers(), 1);
		// MiningStoping -> Free
		assert_ok!(PhalaModule::force_next_round(RawOrigin::Root.into()));
		PhalaModule::on_finalize(2);
		System::finalize();
		assert_eq!(PhalaModule::worker_state(1).state, WorkerStateEnum::Free);
		assert_eq!(PhalaModule::online_workers(), 0);  // Miner stats reduced
	});
}

#[test]
fn test_transfer() {
	new_test_ext().execute_with(|| {
		// set contract key
		let raw_sk = hex!["0000000000000000000000000000000000000000000000000000000000000001"];
		let pubkey = hex!["0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"].to_vec();
		let sk = ecdsa_load_sk(&raw_sk);
		assert_ok!(PhalaModule::force_set_contract_key(RawOrigin::Root.into(), 2, pubkey));
		// Get some coins
		let imbalance = Balances::deposit_creating(&1, 100);
		drop(imbalance);
		// tranfer_to_tee(some coin)
		assert_ok!(PhalaModule::transfer_to_tee(Origin::signed(1), 50));
		assert_eq!(50, Balances::free_balance(1));
		// transfer_to_chain
		let transfer = Transfer::<u64, Balance> {
			dest: 2u64,
			amount: 10,
			sequence: 1,
		};
		let signature = ecdsa_sign(&sk, &transfer);
		let data = TransferData {
			data: transfer,
			signature,
		};
		assert_ok!(PhalaModule::transfer_to_chain(Origin::signed(1), data.encode()));
		// check balance
		assert_eq!(10, Balances::free_balance(2));
	});
}

#[test]
fn test_randomness() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		PhalaModule::on_finalize(1);
		System::finalize();

		assert_eq!(events().as_slice(), [
			TestEvent::phala(RawEvent::RewardSeed(Default::default()))
		]);
	});
}

// Token economics

#[test]
fn test_clipped_target_number() {
	// The configuration is at most 2 tx per hour for each worker
	// Lower bound
	assert_eq!(PhalaModule::clipped_target_number(20, 1), 333);  // 0.00333 tx/block
	assert_eq!(PhalaModule::clipped_target_number(20, 20), 6666);  // 0.06666 tx/block
	// Normal
	assert_eq!(PhalaModule::clipped_target_number(20, 1000), 3_33333);  // 3.33 tx/block
	assert_eq!(PhalaModule::clipped_target_number(20, 100000), 20_00000);  // 20 tx/block
}

#[test]
fn test_round_mining_reward_at() {
	// 129600000 PHA / (180 days / 1 hour) = 30000 PHA
	assert_eq!(PhalaModule::round_mining_reward_at(0), 30000 * DOLLARS);
}

#[test]
fn test_round_stats() {
	new_test_ext().execute_with(|| {
		// Block 1
		System::set_block_number(1);
		assert_ok!(PhalaModule::force_next_round(RawOrigin::Root.into()));
		PhalaModule::on_finalize(1);
		// Check round 1
		assert_eq!(PhalaModule::round_stats_history(1), RoundStats {
			round: 1,
			online_workers: 0,
			compute_workers: 0,
			frac_target_online_reward: 0,
			total_power: 0
		});
		assert_matches!(events().as_slice(), [
			TestEvent::phala(RawEvent::RewardSeed(_)),
			TestEvent::phala(RawEvent::NewMiningRound(1))
		]);
		// Block 2
		System::set_block_number(2);
		assert_eq!(PhalaModule::round_stats_at(2), PhalaModule::round_stats_history(1));
		assert_ok!(PhalaModule::force_next_round(RawOrigin::Root.into()));
		PhalaModule::on_finalize(2);
		// Check round 2
		assert_eq!(PhalaModule::round_stats_history(2), RoundStats {
			round: 2,
			online_workers: 0,
			compute_workers: 0,
			frac_target_online_reward: 0,
			total_power: 0
		});
		// Block 3
		System::set_block_number(3);
		assert_eq!(PhalaModule::round_stats_at(2), PhalaModule::round_stats_history(1));
		assert_eq!(PhalaModule::round_stats_at(3), PhalaModule::round_stats_history(2));
	});
}

#[test]
fn test_pretax_online_reward() {
	// [Scenario 1]
	//   - 30000 PHA in this round
	//   - 100% hashpower
	//   - X ~ B(600, 0.00333)
	//   - Only one worker
	// Result: ~ 30000 PHA / 2 tx * 37.5% = 5625 ~= 5630
	assert_eq!(
		PhalaModule::pretax_online_reward(
			30000 * DOLLARS,
			10000, 10000,
			PhalaModule::clipped_target_number(20, 1),  // target 20 tx but only one online worker
			1
		),
		5630_630630630631);

	// [Scenario 2]
	//   - 30000 PHA in this round
	//   - 10% hashpower
	//   - X ~ B(600, 0.00333)
	//   - 1000 workers
	// Result: ~ 30000 PHA * 10% * / 2 tx * 37.5% = 562.5
	assert_eq!(
		PhalaModule::pretax_online_reward(
			30000 * DOLLARS,
			1000, 10000,
			PhalaModule::clipped_target_number(20, 1000),  // target 20 tx, ~3.33 tx/block
			1000
		),
		562_500562500562);

	// [Scenario 3]
	//   - 30000 PHA in this round
	//   - 0.002% hashpower
	//   - X ~ B(600, 0.00333)
	//   - 100000 workers
	// Result: ~ 30000 PHA * 0.002% * / 0.12 tx * 37.5% = 1.875 PHA
	assert_eq!(
		PhalaModule::pretax_online_reward(
			30000 * DOLLARS,
			2, 100000,
			PhalaModule::clipped_target_number(20, 100000),  // target 20 tx, 20 tx/block
			100000
		),
		1_875000000000);
}

#[test]
fn test_payout() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		PhalaModule::payout(100 * DOLLARS, &1);
		assert_eq!(
			events().as_slice(),
			[TestEvent::phala(RawEvent::Payout(1, 80 * DOLLARS, 20 * DOLLARS))]
		);
	});
}

#[test]
fn test_payout_and_missed() {
	new_test_ext().execute_with(|| {
		use frame_support::storage::{StorageValue, StorageMap};
		// Set states
		crate::WorkerState::<Test>::insert(1, phala_types::WorkerInfo::<BlockNumber> {
			machine_id: Vec::new(),
			pubkey: Vec::new(),
			last_updated: 1,
			state: phala_types::WorkerStateEnum::Mining(1),
			score: None,
		});
		crate::Round::<Test>::put(phala_types::RoundInfo::<BlockNumber> {
			round: 1,
			start_block: 1,
		});
		crate::RoundStatsHistory::insert(1, phala_types::RoundStats {
			round: 1,
			online_workers: 1,
			compute_workers: 0,
			frac_target_online_reward: 333,
			total_power: 100,
		});
		// Check missed reward (window + 1)
		let window = PhalaModule::reward_window();
		System::set_block_number(1 + window + 1);
		PhalaModule::handle_claim_reward(&1, &2, true, false, 100, 1);
		assert_eq!(events().as_slice(), [TestEvent::phala(RawEvent::PayoutMissed(1, 2))]);
		// Check some reward (right within the window)
		System::set_block_number(1 + window);
		PhalaModule::handle_claim_reward(&1, &2, true, false, 100, 1);
		assert_eq!(events().as_slice(), [
			TestEvent::phala(RawEvent::Payout(2, 4504_504504504504, 1126_126126126127))]);
	});
}

#[test]
fn test_force_add_fire() {
	new_test_ext().execute_with(|| {
		assert_ok!(PhalaModule::force_add_fire(
			Origin::root(),
			vec![1, 2],
			vec![100, 200],
		));
		assert_eq!(PhalaModule::fire2(0), 0);
		assert_eq!(PhalaModule::fire2(1), 100);
		assert_eq!(PhalaModule::fire2(2), 200);
	});
}

#[test]
fn test_mining_lifecycle_force_reregister() {
	new_test_ext().execute_with(|| {
		let machine_id = vec![0];
		let pubkey = vec![1];

		// Block 1: register a worker at stash1 and start mining
		System::set_block_number(1);
		assert_ok!(PhalaModule::set_stash(Origin::signed(1), 1));
		assert_ok!(PhalaModule::force_register_worker(
			RawOrigin::Root.into(), 1, machine_id.clone(), pubkey.clone()));
		assert_ok!(PhalaModule::start_mining_intention(Origin::signed(1)));
		assert_eq!(PhalaModule::worker_state(1).state, WorkerStateEnum::MiningPending);
		assert_ok!(PhalaModule::force_next_round(RawOrigin::Root.into()));
		PhalaModule::on_finalize(1);
		System::finalize();
		assert_matches!(events().as_slice(), [
			TestEvent::phala(RawEvent::WorkerRegistered(1, x, y)),
			TestEvent::phala(RawEvent::WorkerStateUpdated(1)),
			TestEvent::phala(RawEvent::RewardSeed(_)),
			TestEvent::phala(RawEvent::MinerStarted(1, 1)),
			TestEvent::phala(RawEvent::WorkerStateUpdated(1)),
			TestEvent::phala(RawEvent::NewMiningRound(1))
		] if x == &pubkey && y == &machine_id);
		assert_matches!(PhalaModule::worker_state(1).state,
			WorkerStateEnum::<BlockNumber>::Mining(_)
		);
		// We have 1 worker with 100 power
		assert_eq!(PhalaModule::online_workers(), 1);
		assert_eq!(PhalaModule::total_power(), 100);

		// Block 2: force reregister to stash2
		System::set_block_number(2);
		assert_ok!(PhalaModule::set_stash(Origin::signed(2), 2));
		assert_ok!(PhalaModule::force_register_worker(
			RawOrigin::Root.into(), 2, machine_id.clone(), pubkey.clone()));
		assert_ok!(PhalaModule::force_next_round(RawOrigin::Root.into()));
		PhalaModule::on_finalize(2);
		System::finalize();
		assert_matches!(events().as_slice(), [
			TestEvent::phala(RawEvent::WorkerUnregistered(1, x)),
			TestEvent::phala(RawEvent::WorkerRegistered(2, y, z)),
			TestEvent::phala(RawEvent::RewardSeed(_)),
			TestEvent::phala(RawEvent::NewMiningRound(2))
		] if x == &machine_id && y == &pubkey && z == &machine_id);
		// WorkerState for stash1 is gone
		assert_eq!(PhalaModule::worker_state(1).state, WorkerStateEnum::Empty);
		// Stash2 now have one worker registered
		assert_eq!(PhalaModule::worker_state(2).state, WorkerStateEnum::Free);
		// We should have zero worker with 0 power
		assert_eq!(PhalaModule::online_workers(), 0);
		assert_eq!(PhalaModule::total_power(), 0);
	});
}

#[test]
fn test_mining_lifecycle_renew() {
	new_test_ext().execute_with(|| {
		let machine_id = vec![0];
		let pubkey = vec![1];

		// Block 1: register a worker at stash1 and start mining
		System::set_block_number(1);
		assert_ok!(PhalaModule::set_stash(Origin::signed(1), 1));
		assert_ok!(PhalaModule::force_register_worker(
			RawOrigin::Root.into(), 1, machine_id.clone(), pubkey.clone()));
		assert_ok!(PhalaModule::start_mining_intention(Origin::signed(1)));
		assert_eq!(PhalaModule::worker_state(1).state, WorkerStateEnum::MiningPending);
		assert_ok!(PhalaModule::force_next_round(RawOrigin::Root.into()));
		PhalaModule::on_finalize(1);
		System::finalize();
		assert_matches!(events().as_slice(), [
			TestEvent::phala(RawEvent::WorkerRegistered(1, x, y)),
			TestEvent::phala(RawEvent::WorkerStateUpdated(1)),
			TestEvent::phala(RawEvent::RewardSeed(_)),
			TestEvent::phala(RawEvent::MinerStarted(1, 1)),
			TestEvent::phala(RawEvent::WorkerStateUpdated(1)),
			TestEvent::phala(RawEvent::NewMiningRound(1))
		] if x == &pubkey && y == &machine_id);
		assert_matches!(PhalaModule::worker_state(1).state,
			WorkerStateEnum::<BlockNumber>::Mining(_)
		);
		// We have 1 worker with 100 power
		assert_eq!(PhalaModule::online_workers(), 1);
		assert_eq!(PhalaModule::total_power(), 100);

		// Block 2: force reregister to stash2
		System::set_block_number(2);
		assert_ok!(PhalaModule::force_register_worker(
			RawOrigin::Root.into(), 1, machine_id.clone(), pubkey.clone()));
		assert_ok!(PhalaModule::force_next_round(RawOrigin::Root.into()));
		PhalaModule::on_finalize(2);
		System::finalize();
		assert_matches!(events().as_slice(), [
			TestEvent::phala(RawEvent::WorkerRenewed(1, x)),
			TestEvent::phala(RawEvent::RewardSeed(_)),
			TestEvent::phala(RawEvent::NewMiningRound(2))
		] if x == &machine_id);
		assert_matches!(PhalaModule::worker_state(1).state,
			WorkerStateEnum::<BlockNumber>::Mining(_)
		);
		// Same as the last block
		assert_eq!(PhalaModule::online_workers(), 1);
		assert_eq!(PhalaModule::total_power(), 100);
	});
}

#[test]
fn test_bug_119() {
	new_test_ext().execute_with(|| {
		use frame_support::storage::StorageMap;

		let machine_id1 = vec![1];
		let machine_id2 = vec![2];
		let pubkey1 = vec![11];
		let pubkey2 = vec![12];

		// Block 1: register worker1 at account1 and start mining
		System::set_block_number(1);
		println!("---- block 1");
		assert_ok!(PhalaModule::set_stash(Origin::signed(1), 1));
		assert_ok!(PhalaModule::set_stash(Origin::signed(2), 2));
		assert_ok!(PhalaModule::force_register_worker(
			RawOrigin::Root.into(), 1, machine_id1.clone(), pubkey1.clone()));
		assert_ok!(PhalaModule::start_mining_intention(Origin::signed(1)));
		assert_ok!(PhalaModule::force_next_round(RawOrigin::Root.into()));
		// Check machine_owner is set correctly
		assert_eq!(PhalaModule::machine_owner(machine_id1.clone()), 1);
		PhalaModule::on_finalize(1);
		System::finalize();

		// Block 2: register worker2 at account 1
		System::set_block_number(2);
		println!("---- block 2");
		assert_matches!(PhalaModule::worker_state(1).state, WorkerStateEnum::Mining(_));
		assert_ok!(PhalaModule::force_register_worker(
			RawOrigin::Root.into(), 1, machine_id2.clone(), pubkey2.clone()));
		assert_eq!(PhalaModule::machine_owner(machine_id2.clone()), 1);
		assert_eq!(
			crate::MachineOwner::<Test>::contains_key(&machine_id1), false,
			"Machine1 unlinked because account1 is force linked to machine2");
		PhalaModule::on_finalize(2);
		System::finalize();

		let delta = PhalaModule::pending_exiting();
		assert_eq!(delta.num_worker, -1);
		assert_eq!(delta.num_power, -100);
	});
}

fn ecdsa_load_sk(raw_key: &[u8]) -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse_slice(raw_key).expect("can't parse private key")
}

fn ecdsa_sign(sk: &secp256k1::SecretKey, data: &impl Encode) -> Vec<u8> {
	let msg_hash = sp_core::hashing::blake2_256(&Encode::encode(&data));
	let mut buffer = [0u8; 32];
	buffer.copy_from_slice(&msg_hash);

	let message = secp256k1::Message::parse(&buffer);
	let sig = secp256k1::sign(&message, &sk);
	let raw_sig: sp_core::ecdsa::Signature = sig.into();
	raw_sig.0.to_vec()
}
