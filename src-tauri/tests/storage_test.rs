#[test]
fn test_insert_and_retrieve() {
    use tauri_app_lib::storage::Vault;
    let dir = std::env::temp_dir();
    let db_path = dir.join("test_pashword_vault.db");
    let _ = std::fs::remove_file(&db_path);

    let vault = Vault::open(&db_path).unwrap();
    vault.check_integrity().unwrap();

    let id = vault.insert_entry(b"example.com", b"alice", b"secret123", b"generatedpw").unwrap();
    assert!(id > 0);

    let entries = vault.get_entry_blobs().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1, b"example.com");

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_duplicate_detection() {
    use tauri_app_lib::storage::Vault;
    let dir = std::env::temp_dir();
    let db_path = dir.join("test_dup_vault.db");
    let _ = std::fs::remove_file(&db_path);

    let vault = Vault::open(&db_path).unwrap();
    vault.insert_entry(b"site.com", b"user", b"sk", b"pw").unwrap();
    let dup = vault.check_duplicate(b"site.com", b"user").unwrap();
    assert!(dup.is_some());
    let nodup = vault.check_duplicate(b"other.com", b"user").unwrap();
    assert!(nodup.is_none());

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_delete_entry() {
    use tauri_app_lib::storage::Vault;
    let dir = std::env::temp_dir();
    let db_path = dir.join("test_del_vault.db");
    let _ = std::fs::remove_file(&db_path);

    let vault = Vault::open(&db_path).unwrap();
    let id = vault.insert_entry(b"x.com", b"y", b"z", b"w").unwrap();
    vault.delete_entry(id).unwrap();
    assert_eq!(vault.get_entry_blobs().unwrap().len(), 0);

    let _ = std::fs::remove_file(&db_path);
}
