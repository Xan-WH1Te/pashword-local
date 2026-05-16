use crate::pashword;
use crate::storage::Vault;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub vault: Mutex<Option<Vault>>,
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

fn get_vault(state: &State<AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.lock().map_err(|e| format!("lock: {}", e))?;
    if vault_lock.is_none() {
        *vault_lock = Some(Vault::open(&state.db_path)?);
    }
    Ok(())
}

fn with_vault<T>(state: &State<AppState>, f: impl FnOnce(&Vault) -> Result<T, String>) -> Result<T, String> {
    get_vault(state)?;
    let vault_lock = state.vault.lock().map_err(|e| format!("lock: {}", e))?;
    let vault = vault_lock.as_ref().unwrap();
    f(vault)
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
pub fn save_entry(
    state: State<AppState>,
    website: String,
    username: String,
    secret_key: String,
    pashword: String,
) -> Result<i64, String> {
    with_vault(&state, |vault| {
        if let Some(existing_id) = vault.check_duplicate(&website, &username)? {
            vault.update_entry(existing_id, &website, &username, &secret_key, &pashword)?;
            return Ok(existing_id);
        }
        vault.insert_entry(&website, &username, &secret_key, &pashword)
    })
}

#[tauri::command]
pub fn list_entries(state: State<AppState>) -> Result<Vec<VaultEntryDto>, String> {
    with_vault(&state, |vault| {
        let entries = vault.get_all_entries()?;
        Ok(entries.into_iter().map(|e| VaultEntryDto {
            id: e.id,
            website: e.website,
            username: e.username,
            secret_key: e.secret_key,
            pashword: e.pashword,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }).collect())
    })
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, id: i64) -> Result<(), String> {
    with_vault(&state, |vault| vault.delete_entry(id))
}
