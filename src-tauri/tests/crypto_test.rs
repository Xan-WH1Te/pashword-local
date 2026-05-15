use tauri_app_lib::crypto;

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let key = [42u8; 32];
    let plaintext = b"hello world this is a secret message";
    let encrypted = crypto::encrypt(&key, plaintext).unwrap();
    let decrypted = crypto::decrypt(&key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_wrong_key_fails() {
    let key1 = [1u8; 32];
    let key2 = [2u8; 32];
    let encrypted = crypto::encrypt(&key1, b"secret").unwrap();
    let result = crypto::decrypt(&key2, &encrypted);
    assert!(result.is_err());
}

#[test]
fn test_different_nonces_produce_different_ciphertexts() {
    let key = [99u8; 32];
    let plaintext = b"same message";
    let c1 = crypto::encrypt(&key, plaintext).unwrap();
    let c2 = crypto::encrypt(&key, plaintext).unwrap();
    assert_ne!(c1, c2);
}

#[test]
fn test_salt_is_random() {
    let s1 = crypto::generate_salt();
    let s2 = crypto::generate_salt();
    assert_ne!(s1, s2);
}
