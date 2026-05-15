#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_app_lib::commands::{self, AppState};
use std::sync::Mutex;

fn main() {
    let db_path = commands::default_db_path();
    tauri::Builder::default()
        .manage(AppState {
            vault: Mutex::new(None),
            encryption_key: Mutex::new(None),
            db_path,
        })
        .invoke_handler(tauri::generate_handler![
            commands::generate_password,
            commands::setup_vault,
            commands::unlock_vault,
            commands::is_vault_initialized,
            commands::save_entry,
            commands::list_entries,
            commands::delete_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
