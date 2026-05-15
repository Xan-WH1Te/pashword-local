use crate::crypto;
use crate::pashword;
use crate::storage::Vault;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub vault: Mutex<Option<Vault>>,
    pub encryption_key: Mutex<Option<[u8; 32]>>,
    pub db_path: PathBuf,
}

#[derive(Serialize)]
pub struct VaultEntryDto {
    pub id: i64,
    pub website: String,
    pub username: String,
    pub secret_key: String,
    pub pashword: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn default_db_path() -> PathBuf {
    let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    path.push("pashword_vault.db");
    path
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    { Some(PathBuf::from("/data/data/com.pashword.local/files")) }
    #[cfg(not(target_os = "android"))]
    { dirs::data_dir() }
}

#[tauri::command]
pub fn generate_password(
    website: String,
    username: String,
    secret_key: String,
    length: u8,
) -> Result<String, String> {
    pashword::generate_pashword(&website, &username, &secret_key, length)
}

#[tauri::command]
pub fn setup_vault(
    state: State<AppState>,
    master_password: String,
) -> Result<(), String> {
    let salt = crypto::generate_salt();
    let key = pashword::derive_key(&master_password, &salt)?;

    let vault = Vault::open(&state.db_path)?;
    vault.check_integrity()?;
    vault.set_setting("salt", &salt)?;

    let known = b"pashword_vault_initialized";
    let encrypted_known = crypto::encrypt(&key, known)?;
    vault.set_setting("verify_token", &encrypted_known)?;

    *state.encryption_key.lock().unwrap() = Some(key);
    *state.vault.lock().unwrap() = Some(vault);

    Ok(())
}

#[tauri::command]
pub fn unlock_vault(
    state: State<AppState>,
    master_password: String,
) -> Result<bool, String> {
    let vault = Vault::open(&state.db_path)?;
    vault.check_integrity()?;
    let salt = vault.get_setting("salt")?
        .ok_or("vault not initialized")?;
    let salt: [u8; 32] = salt.try_into()
        .map_err(|_| "invalid salt".to_string())?;

    let key = pashword::derive_key(&master_password, &salt)?;

    let encrypted_token = vault.get_setting("verify_token")?
        .ok_or("vault not initialized")?;

    match crypto::decrypt(&key, &encrypted_token) {
        Ok(decrypted) if decrypted == b"pashword_vault_initialized" => {
            *state.encryption_key.lock().unwrap() = Some(key);
            *state.vault.lock().unwrap() = Some(vault);
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[tauri::command]
pub fn is_vault_initialized(state: State<AppState>) -> Result<bool, String> {
    match Vault::open(&state.db_path) {
        Ok(vault) => Ok(vault.get_setting("salt").unwrap_or(None).is_some()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub fn save_entry(
    state: State<AppState>,
    website: String,
    username: String,
    secret_key: String,
    pashword: String,
) -> Result<i64, String> {
    let key = state.encryption_key.lock().unwrap()
        .ok_or("vault not unlocked")?;
    let vault = state.vault.lock().unwrap();
    let vault = vault.as_ref().ok_or("vault not unlocked")?;

    let enc_website = crypto::encrypt(&key, website.as_bytes())?;
    let enc_username = crypto::encrypt(&key, username.as_bytes())?;
    let enc_secret_key = crypto::encrypt(&key, secret_key.as_bytes())?;
    let enc_pashword = crypto::encrypt(&key, pashword.as_bytes())?;

    if let Some(existing_id) = vault.check_duplicate(&enc_website, &enc_username)? {
        vault.update_entry(existing_id, &enc_website, &enc_username, &enc_secret_key, &enc_pashword)?;
        return Ok(existing_id);
    }

    vault.insert_entry(&enc_website, &enc_username, &enc_secret_key, &enc_pashword)
}

#[tauri::command]
pub fn list_entries(state: State<AppState>) -> Result<Vec<VaultEntryDto>, String> {
    let key = state.encryption_key.lock().unwrap()
        .ok_or("vault not unlocked")?;
    let vault = state.vault.lock().unwrap();
    let vault = vault.as_ref().ok_or("vault not unlocked")?;

    let blobs = vault.get_entry_blobs()?;
    let mut entries = Vec::new();
    for (id, w, u, sk, pw, ca, ua) in blobs {
        entries.push(VaultEntryDto {
            id,
            website: String::from_utf8_lossy(&crypto::decrypt(&key, &w)?).to_string(),
            username: String::from_utf8_lossy(&crypto::decrypt(&key, &u)?).to_string(),
            secret_key: String::from_utf8_lossy(&crypto::decrypt(&key, &sk)?).to_string(),
            pashword: String::from_utf8_lossy(&crypto::decrypt(&key, &pw)?).to_string(),
            created_at: ca,
            updated_at: ua,
        });
    }
    Ok(entries)
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, id: i64) -> Result<(), String> {
    let vault = state.vault.lock().unwrap();
    let vault = vault.as_ref().ok_or("vault not unlocked")?;
    vault.delete_entry(id)
}
