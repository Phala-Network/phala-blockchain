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
        let pubkey: pink::EcdsaPublicKey = sig::get_public_key(&privkey, SigType::Ecdsa)
            .try_into()
            .unwrap();
        let message = [1u8; 32];
        let signature = sig::ecdsa_sign_prehashed(&privkey, message);
        let pass = sig::ecdsa_verify_prehashed(signature, message, pubkey);
        let fake_message = [2u8; 32];
        assert!(pass);
        let pass = sig::ecdsa_verify_prehashed(signature, fake_message, pubkey);
        assert!(!pass);
    }
}
