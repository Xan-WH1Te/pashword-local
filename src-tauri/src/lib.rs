pub mod pashword;
pub mod storage;
pub mod commands;

use commands::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = commands::default_db_path();
    tauri::Builder::default()
        .manage(AppState {
            vault: Mutex::new(None),
            db_path,
        })
        .invoke_handler(tauri::generate_handler![
            commands::generate_password,
            commands::save_entry,
            commands::list_entries,
            commands::delete_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
