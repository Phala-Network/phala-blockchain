#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests {
    use pink_extension as pink;

    #[test]
    fn getrandom_works() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        let bytes = pink::ext().getrandom(3);
        assert_eq!(bytes.len(), 3);
        assert!(bytes != [0; 3]);
    }

    #[test]
    fn test_signing() {
        use pink::chain_extension::signing as sig;
        use pink::chain_extension::SigType;

        pink_extension_runtime::mock_ext::mock_all_ext();

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

        pink_extension_runtime::mock_ext::mock_all_ext();

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
        pink_extension_runtime::mock_ext::mock_all_ext();

        assert!(pink::ext().cache_set(b"foo", b"bar-0").is_ok());
        let value = pink::ext().cache_get(b"foo");
        assert_eq!(value, Some(b"bar-0".to_vec()));
    }

    #[test]
    fn local_cache_works_thread1() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        assert!(pink::ext().cache_set(b"foo", b"bar-1").is_ok());
        let value = pink::ext().cache_get(b"foo");
        assert_eq!(value, Some(b"bar-1".to_vec()));
    }

    #[test]
    fn local_cache_works_thread2() {
        pink_extension_runtime::mock_ext::mock_all_ext();

        assert!(pink::ext().cache_set(b"foo", b"bar-2").is_ok());
        let value = pink::ext().cache_get(b"foo");
        assert_eq!(value, Some(b"bar-2".to_vec()));
    }
}
