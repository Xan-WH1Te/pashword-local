use tauri_app_lib::pashword;

#[test]
fn test_deterministic_generation() {
    let pw1 = pashword::generate_pashword(
        "example.com", "alice", "my-secret", 32
    ).unwrap();
    let pw2 = pashword::generate_pashword(
        "example.com", "alice", "my-secret", 32
    ).unwrap();
    assert_eq!(pw1, pw2);
    assert_eq!(pw1.len(), 32);
}

#[test]
fn test_different_inputs_different_outputs() {
    let pw1 = pashword::generate_pashword(
        "a.com", "bob", "secret1", 32
    ).unwrap();
    let pw2 = pashword::generate_pashword(
        "b.com", "bob", "secret1", 32
    ).unwrap();
    assert_ne!(pw1, pw2);
}

#[test]
fn test_has_required_chars() {
    let pw = pashword::generate_pashword(
        "test.com", "user", "key", 32
    ).unwrap();
    assert!(pw.chars().any(|c| c.is_uppercase()));
    assert!(pw.chars().any(|c| c.is_lowercase()));
    assert!(pw.chars().any(|c| c.is_ascii_digit()));
}

#[test]
fn test_derive_key_deterministic() {
    let salt = b"0123456789abcdef0123456789abcdef";
    let k1 = pashword::derive_key("master", salt).unwrap();
    let k2 = pashword::derive_key("master", salt).unwrap();
    assert_eq!(k1, k2);
    assert_eq!(k1.len(), 32);
}
