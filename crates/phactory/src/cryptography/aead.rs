pub const IV_BYTES: usize = 12;
pub type IV = [u8; IV_BYTES];

fn load_key(raw: &[u8]) -> ring::aead::LessSafeKey {
    let unbound_key = ring::aead::UnboundKey::new(&ring::aead::AES_256_GCM, raw)
        .expect("Failed to load the secret key");
    ring::aead::LessSafeKey::new(unbound_key)
}

// Encrypts the data in-place and appends a 128bit auth tag
#[allow(dead_code)]
pub fn encrypt(iv: &IV, secret: &[u8], in_out: &mut Vec<u8>) {
    let nonce = ring::aead::Nonce::assume_unique_for_key(*iv);
    let key = load_key(secret);

    key.seal_in_place_append_tag(nonce, ring::aead::Aad::empty(), in_out)
        .expect("seal_in_place_separate_tag failed");
}

// Decrypts the cipher (with 128 auth tag appended) in-place and returns the message as a slice.
#[allow(dead_code)]
pub fn decrypt<'in_out>(iv: &[u8], secret: &[u8], in_out: &'in_out mut [u8]) -> &'in_out mut [u8] {
    let mut iv_arr = [0u8; IV_BYTES];
    iv_arr.copy_from_slice(&iv[..IV_BYTES]);
    let key = load_key(secret);
    let nonce = ring::aead::Nonce::assume_unique_for_key(iv_arr);

    key.open_in_place(nonce, ring::aead::Aad::empty(), in_out)
        .expect("open_in_place failed")
}

// TODO: handle error
