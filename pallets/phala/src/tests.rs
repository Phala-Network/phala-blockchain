// Tests to be written here
use crate::{Error, mock::*};
use crate::RawEvent;
use frame_support::{assert_ok, assert_noop};
use frame_system::RawOrigin;
use sp_runtime::traits::BadOrigin;

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

pub const IAS_REPORT_SAMPLE : &[u8] = include_bytes!("../sample/report");
pub const IAS_REPORT_SIGNATURE : &[u8] = include_bytes!("../sample/report_signature");
pub const IAS_REPORT_SIGNING_CERTIFICATE : &[u8] = include_bytes!("../sample/report_signing_certificate");
pub const TEE_REPORT_SAMPLE : [u8; 84] = [134, 222, 118, 78, 114, 234, 214, 152, 216, 8, 81, 21, 74, 185, 3, 20, 94, 86, 50, 217, 38, 6, 207, 126, 193, 213, 116, 158, 165, 3, 240, 52, 152, 244, 211, 227, 176, 63, 91, 64, 75, 255, 58, 155, 45, 184, 227, 42, 136, 168, 214, 4, 170, 88, 87, 19, 140, 173, 116, 12, 194, 233, 181, 179, 174, 91, 242, 172, 70, 33, 88, 178, 202, 201, 201, 211, 39, 66, 56, 86, 40, 0, 0, 0];
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
fn it_works_for_test() {
	new_test_ext().execute_with(|| {
		let sig: Vec<u8> = match base64::decode(&IAS_REPORT_SIGNATURE) {
			Ok(x) => x,
			Err(_) => panic!("decode sig failed")
		};

		let sig_cert_dec: Vec<u8> = match base64::decode_config(&IAS_REPORT_SIGNING_CERTIFICATE, base64::STANDARD) {
			Ok(x) => x,
			Err(_) => panic!("decode cert failed")
		};

		assert_ok!(PhalaModule::register_worker(Origin::signed(1), TEE_REPORT_SAMPLE.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone()));
		assert_ok!(PhalaModule::register_worker(Origin::signed(1), TEE_REPORT_SAMPLE.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone()));
	});
}

#[test]
fn test_whitelist_works() {
	new_test_ext().execute_with(|| {
		// Set block number to 1 to test the events
		System::set_block_number(1);

		let sig: Vec<u8> = base64::decode(&IAS_REPORT_SIGNATURE).expect("decode sig failed");
		let sig_cert_dec: Vec<u8> = base64::decode_config(&IAS_REPORT_SIGNING_CERTIFICATE, base64::STANDARD).expect("decode cert failed");

		// TODO: Handle RA report replay attack
		assert_ok!(PhalaModule::register_worker(Origin::signed(1), TEE_REPORT_SAMPLE.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone()));
		let machine_id = PhalaModule::miner(1);
		assert_eq!(true, machine_id.len() > 0);
		assert_ok!(PhalaModule::register_worker(Origin::signed(2), TEE_REPORT_SAMPLE.to_vec(), IAS_REPORT_SAMPLE.to_vec(), sig.clone(), sig_cert_dec.clone()));
		let machine_id2 = PhalaModule::miner(2);
		assert_eq!(true, machine_id2.len() > 0);
		let machine_id1 = PhalaModule::miner(1);
		assert_eq!(true, machine_id1.len() == 0);
		// Check emitted events
		assert_eq!(
			true,
			match events().as_slice() {[
					TestEvent::phala(RawEvent::WorkerRegistered(1, _)),
					TestEvent::phala(RawEvent::WorkerUnregistered(1, _)),
					TestEvent::phala(RawEvent::WorkerRegistered(2, _))
				] => true,
				_ => false
			}
		);
	});
}

#[test]
fn test_verify_signature() {
	use secp256k1;
	use crate::hashing;
	use codec::Encode;
	use rand;

	new_test_ext().execute_with(|| {
		let data = super::Transfer {
			dest: 1u64,
			amount: 2u128,
			sequence: 3u32,
		};

		let mut prng = rand::rngs::OsRng::default();
		let sk = secp256k1::SecretKey::random(&mut prng);
		let pk = secp256k1::PublicKey::from_secret_key(&sk);
		let serialized_pk = pk.serialize_compressed();

		let msg_hash = hashing::blake2_256(&Encode::encode(&data));
		let mut buffer = [0u8; 32];
		buffer.copy_from_slice(&msg_hash);
		let message = secp256k1::Message::parse(&buffer);
		let sig = secp256k1::sign(&message, &sk);
		let transfer_data = super::TransferData {
			data,
			signature: sig.0.serialize().to_vec(),
		};

		let actual = PhalaModule::verify_signature(serialized_pk.to_vec(), &transfer_data);
		assert_eq!(true, actual.is_ok());
	});
}

#[test]
fn test_force_register_worker() {
	new_test_ext().execute_with(|| {
		assert_ok!(PhalaModule::force_register_worker(RawOrigin::Root.into(), 1, vec![0], vec![1]));
		assert_noop!(PhalaModule::force_register_worker(Origin::signed(1), 1, vec![0], vec![1]), BadOrigin);
	});
}
