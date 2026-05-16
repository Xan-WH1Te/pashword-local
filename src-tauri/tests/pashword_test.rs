use tauri_app_lib::pashword;

#[test]
fn test_deterministic_generation() {
    let pw1 = pashword::generate_pashword(
        "example.com", "alice", "my-secret", 20
    ).unwrap();
    let pw2 = pashword::generate_pashword(
        "example.com", "alice", "my-secret", 20
    ).unwrap();
    assert_eq!(pw1, pw2);
    assert_eq!(pw1.len(), 20);
}

#[test]
fn test_different_inputs_different_outputs() {
    let pw1 = pashword::generate_pashword(
        "a.com", "bob", "secret1", 20
    ).unwrap();
    let pw2 = pashword::generate_pashword(
        "b.com", "bob", "secret1", 20
    ).unwrap();
    assert_ne!(pw1, pw2);
}

#[test]
fn test_has_required_chars() {
    let pw = pashword::generate_pashword(
        "test.com", "user", "key", 20
    ).unwrap();
    assert!(pw.chars().any(|c| c.is_uppercase()));
    assert!(pw.chars().any(|c| c.is_lowercase()));
    assert!(pw.chars().any(|c| c.is_ascii_digit()));
}

#[test]
fn test_matches_original_pashword() {
    // Verified against pashword-lib test vectors
    let pw = pashword::generate_pashword(
        "reddit.com", "asda1111", "asdasd1", 20
    ).unwrap();
    assert_eq!(pw, "&a3SZ5e$99m%ZK9rN*_T");

    let pw2 = pashword::generate_pashword(
        "reddit.com", "asdasda", "asasdasd", 11
    ).unwrap();
    assert_eq!(pw2, "fAp.&5Ri82#");
}
