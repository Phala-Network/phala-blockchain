use ink::env::chain_extension::FromStatusCode;
use pink::{
    chain_extension::{
        CodableError, ErrorCode, HttpRequest, HttpRequestError, HttpResponse, JsCode, JsValue,
        StorageQuotaExceeded,
    },
    HookPoint, PinkEvent, ResultExt, SidevmOperation,
};
use pink_extension_runtime::{local_cache::apply_quotas, mock_ext::mock_all_ext};
use rusty_fork::rusty_fork_test;
use scale::{Decode, Encode};
use std::convert::TryInto;

#[test]
fn getrandom_works() {
    mock_all_ext();

    let bytes = pink::ext().getrandom(3);
    assert_eq!(bytes.len(), 3);
    assert!(bytes != [0; 3]);
}

#[test]
fn test_signing() {
    use pink::chain_extension::signing as sig;
    use pink::chain_extension::SigType;

    mock_all_ext();

    let privkey = sig::derive_sr25519_key(b"a spoon of salt");
    let pubkey = sig::get_public_key(&privkey, SigType::Sr25519);
    let message = b"hello world";
    let signature = sig::sign(message, &privkey, SigType::Sr25519);
    let pass = sig::verify(message, &pubkey, &signature, SigType::Sr25519);
    assert!(pass);
    let pass = sig::verify(b"Fake", &pubkey, &signature, SigType::Sr25519);
    assert!(!pass);
}

#[test]
fn test_ecdsa_signing() {
    use pink::chain_extension::signing as sig;
    use pink::chain_extension::SigType;

    mock_all_ext();

    let privkey = sig::derive_sr25519_key(b"salt");
    let privkey = &privkey[..32];
    let pubkey: pink::EcdsaPublicKey = sig::get_public_key(privkey, SigType::Ecdsa)
        .try_into()
        .unwrap();
    let message = [1u8; 32];
    let signature = sig::ecdsa_sign_prehashed(privkey, message);
    let pass = sig::ecdsa_verify_prehashed(signature, message, pubkey);
    let fake_message = [2u8; 32];
    assert!(pass);
    let pass = sig::ecdsa_verify_prehashed(signature, fake_message, pubkey);
    assert!(!pass);
}

#[test]
fn local_cache_works_thread0() {
    mock_all_ext();

    assert!(pink::ext().cache_set(b"foo", b"bar-0").is_ok());
    let value = pink::ext().cache_get(b"foo");
    assert_eq!(value, Some(b"bar-0".to_vec()));
    pink::ext().cache_remove(b"foo");
    assert_eq!(pink::ext().cache_get(b"foo"), None);
}

#[test]
fn local_cache_works_thread1() {
    mock_all_ext();

    assert!(pink::ext().cache_set(b"foo", b"bar-1").is_ok());
    let value = pink::ext().cache_get(b"foo");
    assert_eq!(value, Some(b"bar-1".to_vec()));
    pink::ext().cache_set_expiration(b"foo", 0);
    assert_eq!(pink::ext().cache_get(b"foo"), None);
}

#[test]
fn local_cache_works_thread2() {
    mock_all_ext();

    assert!(pink::ext().cache_set(b"foo", b"bar-2").is_ok());
    let value = pink::ext().cache_get(b"foo");
    assert_eq!(value, Some(b"bar-2".to_vec()));
}

#[test]
fn test_systime() {
    use std::time::SystemTime;
    mock_all_ext();

    let ms = pink::ext().untrusted_millis_since_unix_epoch();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    assert!((now as i128 - ms as i128).abs() < 100);
}

#[cfg(coverage)]
#[test]
fn test_http_req() {
    mock_all_ext();

    let resp = pink::http_get!(
        "https://httpbin.org/get",
        vec![("X-Foo".into(), "Bar".into())]
    );
    assert_eq!(resp.status_code, 200);

    let resp = pink::http_post!(
        "https://httpbin.org/post",
        *b"hello world",
        vec![("X-Foo".into(), "Bar".into())]
    );
    assert_eq!(resp.status_code, 200);
}

#[test]
fn test_http_invalid_url() {
    mock_all_ext();
    let request = HttpRequest::new(
        "sip://httpbin.org/get",
        "GET",
        vec![("X-Foo".into(), "Bar".into())],
        b"".to_vec(),
    );
    let resp = pink::ext().http_request(request);
    assert_eq!(resp.status_code, 523);
}

#[test]
fn test_chores() {
    mock_all_ext();

    assert!(HttpResponse::not_found().encode() != HttpResponse::ok(vec![]).encode());
    use HttpRequestError::*;

    for err in [
        InvalidUrl,
        InvalidMethod,
        InvalidHeaderName,
        InvalidHeaderValue,
        FailedToCreateClient,
        Timeout,
        NotAllowed,
        TooManyRequests,
        NetworkError,
        ResponseTooLarge,
    ] {
        let encoded = CodableError::encode(&err);
        let decoded = HttpRequestError::from(ErrorCode::from_status_code(encoded).unwrap_err());
        assert_eq!(err, decoded);
        assert!(!decoded.display().is_empty());
        assert!(!format!("{:?}", decoded).is_empty());

        let scl = Encode::encode(&decoded.clone());
        let scl_decoded: HttpRequestError = Decode::decode(&mut &scl[..]).unwrap();
        assert_eq!(err, scl_decoded);
    }

    assert!(!pink::ext().is_in_transaction());
    assert_eq!(pink::ext().worker_pubkey(), [0u8; 32]);
    assert!(!pink::ext().code_exists([1; 32], false));
    assert_eq!(pink::ext().runtime_version(), (1, 0));
    assert_eq!(pink::ext().current_event_chain_head().0, 0);
    let rv = pink::ext().js_eval(vec![], vec![]);
    assert_eq!(rv, JsValue::Exception("No Js Runtime".to_string()));
}

#[test]
#[should_panic]
fn test_get_system_contract_id() {
    mock_all_ext();
    let _ = pink::ext().system_contract_id();
}

#[test]
fn dump_type_info() {
    use type_info_stringify::type_info_stringify;

    #[derive(scale_info::TypeInfo)]
    struct Root {
        _http_request: HttpRequest,
        _http_response: HttpResponse,
        _http_error: HttpRequestError,
        _quota_error: StorageQuotaExceeded,
        _error_code: ErrorCode,
        _js_code: JsCode,
        _js_value: JsValue,
        _pink_event: PinkEvent,
        _system_err: pink::system::Error,
        _driver_err: pink::system::DriverError,
        _code_type: pink::system::CodeType,
    }

    insta::assert_snapshot!(type_info_stringify::<Root>())
}

#[test]
fn test_codecs() {
    use scale::{Decode, Encode};
    let encoded = Encode::encode(&StorageQuotaExceeded);
    assert!(encoded.is_empty());
    let _decoded = <StorageQuotaExceeded as Decode>::decode(&mut &encoded[..]).unwrap();
    assert_eq!(
        format!("{:?}", StorageQuotaExceeded),
        "StorageQuotaExceeded"
    );
    assert!(<StorageQuotaExceeded as CodableError>::decode(0).is_none());
}

rusty_fork_test! {
    #[test]
    fn cache_quotas_works() {
        tracing_subscriber::fmt::init();
        mock_all_ext();
        apply_quotas(vec![(&b""[..], 10)]);
        assert!(pink::ext().cache_set(b"foo", b"bar-0").is_ok());
        assert!(pink::ext().cache_get(b"foo").is_some());
        assert!(pink::ext().cache_set(b"bar", b"baaaaaaaaaaaaaaaar").is_err());
        assert!(pink::ext().cache_get(b"bar").is_none());
    }

    #[test]
    fn trigger_logs() {
        tracing_subscriber::fmt::init();
        mock_all_ext();

        pink::error!("pink error");
        pink::warn!("pink warn");
        pink::info!("pink info");
        pink::debug!("pink debug");
        pink::trace!("pink trace");

        _ = Result::<(), &str>::Ok(()).log_err("Hello");
        _ = Result::<(), &str>::Err("Error message").log_err("Hello");
        _ = Result::<(), &str>::Ok(())
            .log_err_with_level(pink::logger::Level::Info, "Hello");
        _ = Result::<(), &str>::Err("Error message")
            .log_err_with_level(pink::logger::Level::Info, "Hello");
    }
}

#[test]
fn cov_emit_events() {
    mock_all_ext();

    pink::deploy_sidevm_to([0u8; 32].into(), [0; 32]);
    pink::force_stop_sidevm();
    pink::push_sidevm_message(b"foo".to_vec());
    pink::set_contract_weight([0u8; 32].into(), 0);
    pink::set_hook(HookPoint::OnBlockEnd, [0u8; 32].into(), 0, 0);
    pink::set_js_runtime([0u8; 32]);
    pink::set_log_handler([0u8; 32].into());
    pink::stop_sidevm_at([0u8; 32].into());
    pink::upgrade_runtime((0, 1));

    let result = pink::query_local_sidevm([0u8; 32].into(), vec![]);
    assert!(result.is_err());
}

#[test]
fn cov_pink_events_display() {
    fn test_all(events: &[(bool, bool, &str, PinkEvent)]) {
        for (is_private, allowed_in_query, name, event) in events {
            assert_eq!(event.name(), *name);
            assert_eq!(event.is_private(), *is_private);
            assert_eq!(event.allowed_in_query(), *allowed_in_query);
        }
    }

    test_all(&[
        // Test for PinkEvent::SetHook variant
        (
            false,
            false,
            "SetHook",
            PinkEvent::SetHook {
                hook: HookPoint::OnBlockEnd,
                contract: [0u8; 32].into(),
                selector: 0,
                gas_limit: 0,
            },
        ),
        // Test for PinkEvent::DeploySidevmTo variant
        (
            false,
            true,
            "DeploySidevmTo",
            PinkEvent::DeploySidevmTo {
                contract: [0u8; 32].into(),
                code_hash: [0u8; 32],
            },
        ),
        // Test for PinkEvent::SidevmMessage variant
        (
            true,
            true,
            "SidevmMessage",
            PinkEvent::SidevmMessage(vec![]),
        ),
        // Test for PinkEvent::CacheOp variant
        (
            true,
            true,
            "CacheOp",
            PinkEvent::CacheOp(pink::CacheOp::Set {
                key: vec![],
                value: vec![],
            }),
        ),
        // Test for PinkEvent::StopSidevm variant
        (false, true, "StopSidevm", PinkEvent::StopSidevm),
        // Test for PinkEvent::ForceStopSidevm variant
        (
            false,
            true,
            "ForceStopSidevm",
            PinkEvent::ForceStopSidevm {
                contract: [0u8; 32].into(),
            },
        ),
        // Test for PinkEvent::SetLogHandler variant
        (
            false,
            false,
            "SetLogHandler",
            PinkEvent::SetLogHandler([0u8; 32].into()),
        ),
        // Test for PinkEvent::SetContractWeight variant
        (
            false,
            false,
            "SetContractWeight",
            PinkEvent::SetContractWeight {
                contract: [0u8; 32].into(),
                weight: 123,
            },
        ),
        // Test for PinkEvent::UpgradeRuntimeTo variant
        (
            false,
            false,
            "UpgradeRuntimeTo",
            PinkEvent::UpgradeRuntimeTo { version: (1, 0) },
        ),
        // Test for PinkEvent::SidevmOperation - Start variant
        (
            false,
            true,
            "SidevmOperation",
            PinkEvent::SidevmOperation(SidevmOperation::Start {
                contract: [0u8; 32].into(),
                code_hash: [0u8; 32],
                workers: pink::Workers::List(vec![]),
                config: pink::SidevmConfig::default(),
            }),
        ),
        // Test for PinkEvent::SetJsRuntime variant
        (
            false,
            false,
            "SetJsRuntime",
            PinkEvent::SetJsRuntime([0u8; 32]),
        ),
    ])
}

#[test]
fn cov_env() {
    ink::env::test::set_block_timestamp::<pink::PinkEnvironment>(1);
    assert_eq!(pink::env().block_timestamp(), 1);
}

#[test]
fn cov_vrf() {
    mock_all_ext();

    let vrf = pink::vrf(&[]);
    assert_eq!(vrf.len(), 64);
}

#[test]
#[should_panic]
fn cov_start_sidevm() {
    mock_all_ext();

    let result = pink::start_sidevm([0u8; 32]);
    assert!(result.is_err());
}

#[test]
fn cov_system_error() {
    use pink::system::{CodeType, DriverError, Error};

    assert!(matches!(
        DriverError::from(Error::CodeNotFound),
        DriverError::SystemError(_)
    ));

    assert!(CodeType::Ink.is_ink());
    assert!(!CodeType::Ink.is_sidevm());
    assert!(CodeType::Sidevm.is_sidevm());
    assert!(!CodeType::Sidevm.is_ink());
}
