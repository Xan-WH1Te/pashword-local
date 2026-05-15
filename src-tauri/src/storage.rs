use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct VaultEntry {
    pub id: i64,
    pub website: String,
    pub username: String,
    pub secret_key: String,
    pub pashword: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct Vault {
    conn: Mutex<Connection>,
}

impl Vault {
    pub fn open(db_path: &PathBuf) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("failed to open database: {}", e))?;

        let wal_mode: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(|e| format!("wal: {}", e))?;
        if wal_mode != "wal" {
            return Err(format!("expected WAL journal mode, got {}", wal_mode));
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vault (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                website    BLOB NOT NULL,
                username   BLOB NOT NULL,
                secret_key BLOB NOT NULL,
                pashword   BLOB NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        ).map_err(|e| format!("create table: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value BLOB NOT NULL
            )",
            [],
        ).map_err(|e| format!("create settings table: {}", e))?;

        Ok(Vault { conn: Mutex::new(conn) })
    }

    pub fn check_integrity(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| format!("integrity check failed: {}", e))?;
        if result != "ok" {
            return Err(format!("integrity check: {}", result));
        }
        Ok(())
    }

    pub fn insert_entry(&self, website: &[u8], username: &[u8], secret_key: &[u8], pashword: &[u8]) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        conn.execute(
            "INSERT INTO vault (website, username, secret_key, pashword) VALUES (?1, ?2, ?3, ?4)",
            params![website, username, secret_key, pashword],
        ).map_err(|e| format!("insert: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_entry_blobs(&self) -> Result<Vec<(i64, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, website, username, secret_key, pashword, created_at, updated_at FROM vault ORDER BY updated_at DESC"
        ).map_err(|e| format!("prepare: {}", e))?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        }).map_err(|e| format!("query: {}", e))?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| format!("row: {}", e))?);
        }
        Ok(entries)
    }

    pub fn delete_entry(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        conn.execute("DELETE FROM vault WHERE id = ?1", params![id])
            .map_err(|e| format!("delete: {}", e))?;
        Ok(())
    }

    pub fn update_entry(&self, id: i64, website: &[u8], username: &[u8], secret_key: &[u8], pashword: &[u8]) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        conn.execute(
            "UPDATE vault SET website=?1, username=?2, secret_key=?3, pashword=?4, updated_at=datetime('now') WHERE id=?5",
            params![website, username, secret_key, pashword, id],
        ).map_err(|e| format!("update: {}", e))?;
        Ok(())
    }

    pub fn check_duplicate(&self, website: &[u8], username: &[u8]) -> Result<Option<i64>, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id FROM vault WHERE website=?1 AND username=?2"
        ).map_err(|e| format!("prepare: {}", e))?;
        let result = stmt.query_row(params![website, username], |row| row.get(0));
        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("duplicate check: {}", e)),
        }
    }

    pub fn set_setting(&self, key: &str, value: &[u8]) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        ).map_err(|e| format!("set_setting: {}", e))?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key=?1")
            .map_err(|e| format!("prepare: {}", e))?;
        let result = stmt.query_row(params![key], |row| row.get(0));
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("get_setting: {}", e)),
        }
    }
}
