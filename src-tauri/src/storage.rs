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

        // Enable WAL mode (best-effort, not all filesystems support it)
        let _ = conn.execute("PRAGMA journal_mode=WAL", []);

        // Check if old BLOB-schema table exists from previous encrypted build
        let needs_migration = match conn.query_row(
            "SELECT type FROM pragma_table_info('vault') WHERE name='website'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            Ok(col_type) => col_type == "BLOB",
            Err(_) => false, // table doesn't exist yet
        };

        if needs_migration {
            conn.execute("DROP TABLE IF EXISTS vault", [])
                .map_err(|e| format!("drop old table: {}", e))?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vault (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                website    TEXT NOT NULL,
                username   TEXT NOT NULL,
                secret_key TEXT NOT NULL,
                pashword   TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        ).map_err(|e| format!("create table: {}", e))?;

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

    pub fn insert_entry(&self, website: &str, username: &str, secret_key: &str, pashword: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        conn.execute(
            "INSERT INTO vault (website, username, secret_key, pashword) VALUES (?1, ?2, ?3, ?4)",
            params![website, username, secret_key, pashword],
        ).map_err(|e| format!("insert: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_entries(&self) -> Result<Vec<VaultEntry>, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, website, username, secret_key, pashword, created_at, updated_at FROM vault ORDER BY updated_at DESC"
        ).map_err(|e| format!("prepare: {}", e))?;

        let rows = stmt.query_map([], |row| {
            Ok(VaultEntry {
                id: row.get(0)?,
                website: row.get(1)?,
                username: row.get(2)?,
                secret_key: row.get(3)?,
                pashword: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
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

    pub fn update_entry(&self, id: i64, website: &str, username: &str, secret_key: &str, pashword: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        conn.execute(
            "UPDATE vault SET website=?1, username=?2, secret_key=?3, pashword=?4, updated_at=datetime('now') WHERE id=?5",
            params![website, username, secret_key, pashword, id],
        ).map_err(|e| format!("update: {}", e))?;
        Ok(())
    }

    pub fn check_duplicate(&self, website: &str, username: &str) -> Result<Option<i64>, String> {
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
}
