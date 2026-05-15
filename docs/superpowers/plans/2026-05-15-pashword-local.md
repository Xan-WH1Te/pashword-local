# Pashword Local Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a native desktop (Windows) and Android password manager app — forked from pashword/pashword — with deterministic password generation and an AES-256-GCM encrypted local vault, using Tauri v2.

**Architecture:** Tauri v2 monorepo. React + TypeScript + Tailwind frontend communicates via Tauri IPC (`invoke`) with a Rust backend that handles Pashword hashing (SHA3-512 → scrypt → SHAKE-256), AES-256-GCM encryption, and SQLite storage. Single codebase produces both `.exe` (Windows) and `.apk` (Android ARM64).

**Tech Stack:** Tauri v2, React 18, TypeScript, Tailwind CSS, Vite, Rust (aes-gcm, sha3, scrypt, rusqlite crates)

---

### Prerequisites

- [ ] **Step 1: Install Rust toolchain**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
rustc --version
```
Expected: `rustc 1.x.x (......)` 

- [ ] **Step 2: Install Node.js via nvm**

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.bashrc
nvm install 22
nvm use 22
node --version && npm --version
```
Expected: `v22.x.x` and `10.x.x`

- [ ] **Step 3: Install Tauri CLI and Android targets**

```bash
cargo install tauri-cli --version "^2"
rustup target add aarch64-linux-android
cargo tauri --version
```
Expected: `cargo-tauri 2.x.x`

- [ ] **Step 4: Install Android SDK prerequisites (for APK builds later)**

```bash
# Install Java (required for Android SDK)
sudo apt-get update && sudo apt-get install -y openjdk-17-jdk-headless
# Install Android command-line tools
mkdir -p ~/Android/Sdk/cmdline-tools
cd ~/Android/Sdk/cmdline-tools
curl -O https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
unzip commandlinetools-linux-11076708_latest.zip
mv cmdline-tools latest
rm commandlinetools-linux-11076708_latest.zip
# Set env vars
echo 'export ANDROID_HOME="$HOME/Android/Sdk"' >> ~/.bashrc
echo 'export ANDROID_SDK_ROOT="$ANDROID_HOME"' >> ~/.bashrc
echo 'export JAVA_HOME="/usr/lib/jvm/java-17-openjdk-amd64"' >> ~/.bashrc
source ~/.bashrc
# Accept licenses and install SDK components
yes | $ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager --licenses
$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager "platforms;android-34" "build-tools;34.0.0" "ndk;27.0.12077973"
```

- [ ] **Step 5: Commit Prerequisites**

```bash
# No code changes yet — just note completion
```

---

### Phase 1: Project Scaffolding

### Task 1: Scaffold Tauri v2 + React + TypeScript project

**Files:**
- Create: `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/icons/`
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `tailwind.config.js`, `postcss.config.js`
- Create: `index.html`, `src/main.tsx`, `src/App.tsx`, `src/styles/globals.css`

- [ ] **Step 1: Scaffold with create-tauri-app**

```bash
cd /home/xanwh1te/projects/pashword-local
npm create tauri-app@latest . -- --template react-ts --manager npm
# Interactive: confirm overwrite with "yes"
```
Expected: Template files created

- [ ] **Step 2: Install frontend dependencies with Tailwind**

```bash
cd /home/xanwh1te/projects/pashword-local
npm install
npm install -D tailwindcss @tailwindcss/vite postcss
```

- [ ] **Step 3: Configure Vite with Tailwind plugin**

File: `vite.config.ts`
```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
```

- [ ] **Step 4: Configure Tailwind CSS**

File: `src/styles/globals.css`
```css
@import "tailwindcss";
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap');

@theme {
  --color-bg: #0a0a10;
  --color-card: rgba(255, 255, 255, 0.03);
  --color-card-border: rgba(255, 255, 255, 0.08);
  --color-input-bg: #0d0d18;
  --color-input-border: rgba(255, 255, 255, 0.10);
  --color-accent: #8b5cf6;
  --color-accent-end: #a855f7;
  --color-text-primary: #ffffff;
  --color-text-secondary: #a0a0b8;
  --font-sans: 'Inter', system-ui, sans-serif;
}

body {
  background-color: var(--color-bg);
  color: var(--color-text-primary);
  font-family: var(--font-sans);
  margin: 0;
  min-height: 100vh;
}
```

- [ ] **Step 5: Verify dev server starts**

```bash
npm run tauri dev
```
Expected: Window opens with default Tauri welcome screen. Close it after confirming.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri v2 + React + TypeScript + Tailwind project"
```

---

### Phase 2: Rust Backend — Pashword Algorithm

### Task 2: Port pashword-lib hashing pipeline to Rust

**Files:**
- Create: `src-tauri/src/pashword.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/tests/pashword_test.rs`

- [ ] **Step 1: Add crypto dependencies to Cargo.toml**

File: `src-tauri/Cargo.toml` — add under `[dependencies]`:
```toml
sha3 = "0.10"
scrypt = "0.11"
```

- [ ] **Step 2: Write pashword.rs**

File: `src-tauri/src/pashword.rs`
```rust
use sha3::{Sha3_512, Shake256, Digest};
use sha3::digest::{ExtendableOutput, Update};
use scrypt::{scrypt, Params};

const SCRYPT_N: u8 = 14;       // log2(16384) = 14
const SCRYPT_R: u32 = 8;
const SCRYPT_P: u32 = 1;

const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

pub fn generate_pashword(
    website: &str,
    username: &str,
    secret_key: &str,
    length: u8,
) -> Result<String, String> {
    let input = format!("{}{}{}{}", website, username, secret_key, length);

    // Step 1: SHA3-512
    let sha3_hash = {
        let mut hasher = Sha3_512::new();
        hasher.update(input.as_bytes());
        hasher.finalize()
    };

    // Step 2: scrypt (N=16384, r=8, p=1)
    let params = Params::new(SCRYPT_N, SCRYPT_R, SCRYPT_P, 64)
        .map_err(|e| format!("scrypt params error: {}", e))?;
    let salt = &sha3_hash[..32];
    let mut scrypt_output = vec![0u8; 64];
    scrypt(&sha3_hash, salt, &params, &mut scrypt_output)
        .map_err(|e| format!("scrypt error: {}", e))?;

    // Step 3: SHAKE-256 as CSPRNG
    let mut shake = Shake256::default();
    shake.update(&scrypt_output);
    let mut rng = shake.finalize_xof();

    // Step 4: Generate password with required character classes
    let mut chars: Vec<u8> = Vec::with_capacity(length as usize);
    let mut buf = [0u8; 1];

    // Ensure at least one of each required class
    let sets: &[&[u8]] = &[UPPER, LOWER, DIGITS, SYMBOLS];
    for set in sets {
        rng.read(&mut buf);
        chars.push(set[buf[0] as usize % set.len()]);
    }

    let all: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    while chars.len() < length as usize {
        rng.read(&mut buf);
        chars.push(all[buf[0] as usize % all.len()]);
    }

    // Shuffle using Fisher-Yates with CSPRNG
    for i in (1..chars.len()).rev() {
        rng.read(&mut buf);
        let j = buf[0] as usize % (i + 1);
        chars.swap(i, j);
    }

    Ok(String::from_utf8(chars).unwrap())
}

/// Derive an AES-256 key from master password using same pipeline.
/// Returns 32 bytes suitable for AES-256-GCM.
pub fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let input = format!("{}{}32", master_password, hex::encode(salt));

    let sha3_hash = {
        let mut hasher = Sha3_512::new();
        hasher.update(input.as_bytes());
        hasher.finalize()
    };

    let params = Params::new(SCRYPT_N, SCRYPT_R, SCRYPT_P, 64)
        .map_err(|e| format!("scrypt params error: {}", e))?;
    let scrypt_salt = &sha3_hash[..32];
    let mut scrypt_output = vec![0u8; 64];
    scrypt(&sha3_hash, scrypt_salt, &params, &mut scrypt_output)
        .map_err(|e| format!("scrypt error: {}", e))?;

    let mut shake = Shake256::default();
    shake.update(&scrypt_output);
    let mut rng = shake.finalize_xof();
    let mut key = [0u8; 32];
    sha3::digest::ExtendableOutput::read(&mut rng, &mut key);

    Ok(key)
}
```

- [ ] **Step 3: Add hex crate dependency**

File: `src-tauri/Cargo.toml` — add:
```toml
hex = "0.4"
```

- [ ] **Step 4: Register module in lib.rs**

File: `src-tauri/src/lib.rs`
```rust
pub mod pashword;
```

- [ ] **Step 5: Write and run test**

File: `src-tauri/tests/pashword_test.rs`
```rust
#[test]
fn test_deterministic_generation() {
    let pw1 = pashword_local_lib::pashword::generate_pashword(
        "example.com", "alice", "my-secret", 32
    ).unwrap();
    let pw2 = pashword_local_lib::pashword::generate_pashword(
        "example.com", "alice", "my-secret", 32
    ).unwrap();
    assert_eq!(pw1, pw2);
    assert_eq!(pw1.len(), 32);
}

#[test]
fn test_different_inputs_different_outputs() {
    let pw1 = pashword_local_lib::pashword::generate_pashword(
        "a.com", "bob", "secret1", 32
    ).unwrap();
    let pw2 = pashword_local_lib::pashword::generate_pashword(
        "b.com", "bob", "secret1", 32
    ).unwrap();
    assert_ne!(pw1, pw2);
}

#[test]
fn test_has_required_chars() {
    let pw = pashword_local_lib::pashword::generate_pashword(
        "test.com", "user", "key", 32
    ).unwrap();
    assert!(pw.chars().any(|c| c.is_uppercase()));
    assert!(pw.chars().any(|c| c.is_lowercase()));
    assert!(pw.chars().any(|c| c.is_ascii_digit()));
}

#[test]
fn test_derive_key_deterministic() {
    let salt = b"0123456789abcdef0123456789abcdef";
    let k1 = pashword_local_lib::pashword::derive_key("master", salt).unwrap();
    let k2 = pashword_local_lib::pashword::derive_key("master", salt).unwrap();
    assert_eq!(k1, k2);
    assert_eq!(k1.len(), 32);
}
```

Run:
```bash
cd src-tauri && cargo test
```
Expected: 4 tests pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/
git commit -m "feat: port pashword-lib hashing pipeline to Rust"
```

---

### Phase 3: Rust Backend — Crypto Module

### Task 3: Implement AES-256-GCM encryption/decryption

**Files:**
- Create: `src-tauri/src/crypto.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add crypto dependencies**

File: `src-tauri/Cargo.toml` — add:
```toml
aes-gcm = "0.10"
rand = "0.8"
```

- [ ] **Step 2: Write crypto.rs**

File: `src-tauri/src/crypto.rs`
```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, AeadCore,
};
use rand::RngCore;

/// Encrypt plaintext with AES-256-GCM.
/// Returns: nonce (12 bytes) || ciphertext || tag (16 bytes)
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("invalid key: {}", e))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("encryption failed: {}", e))?;

    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt ciphertext produced by `encrypt`.
/// Input: nonce (12 bytes) || ciphertext || tag (16 bytes)
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 28 {
        return Err("ciphertext too short".into());
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("invalid key: {}", e))?;
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("decryption failed: {}", e))
}

/// Generate 32 random bytes for salt.
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}
```

- [ ] **Step 3: Register module in lib.rs**

File: `src-tauri/src/lib.rs` — append:
```rust
pub mod crypto;
```

- [ ] **Step 4: Write tests and run**

File: `src-tauri/tests/crypto_test.rs`
```rust
#[test]
fn test_encrypt_decrypt_roundtrip() {
    use pashword_local_lib::crypto;
    let key = [42u8; 32];
    let plaintext = b"hello world this is a secret message";
    let encrypted = crypto::encrypt(&key, plaintext).unwrap();
    let decrypted = crypto::decrypt(&key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_wrong_key_fails() {
    use pashword_local_lib::crypto;
    let key1 = [1u8; 32];
    let key2 = [2u8; 32];
    let encrypted = crypto::encrypt(&key1, b"secret").unwrap();
    let result = crypto::decrypt(&key2, &encrypted);
    assert!(result.is_err());
}

#[test]
fn test_different_nonces_produce_different_ciphertexts() {
    use pashword_local_lib::crypto;
    let key = [99u8; 32];
    let plaintext = b"same message";
    let c1 = crypto::encrypt(&key, plaintext).unwrap();
    let c2 = crypto::encrypt(&key, plaintext).unwrap();
    assert_ne!(c1, c2);
}

#[test]
fn test_salt_is_random() {
    let s1 = pashword_local_lib::crypto::generate_salt();
    let s2 = pashword_local_lib::crypto::generate_salt();
    assert_ne!(s1, s2);
}
```

Run:
```bash
cd src-tauri && cargo test
```
Expected: All tests pass (8 total: 4 pashword + 4 crypto)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat: add AES-256-GCM encryption/decryption module"
```

---

### Phase 4: Rust Backend — Storage Module

### Task 4: Implement SQLite storage with encrypted fields

**Files:**
- Create: `src-tauri/src/storage.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add rusqlite dependency**

File: `src-tauri/Cargo.toml` — add:
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
```

- [ ] **Step 2: Write storage.rs**

File: `src-tauri/src/storage.rs`
```rust
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

        conn.execute("PRAGMA journal_mode=WAL", [])
            .map_err(|e| format!("wal: {}", e))?;

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
        conn.execute("PRAGMA integrity_check", [])
            .map_err(|e| format!("integrity check failed: {}", e))?;
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

    pub fn get_all_entries(&self) -> Result<Vec<VaultEntry>, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, website, username, secret_key, pashword, created_at, updated_at FROM vault ORDER BY updated_at DESC"
        ).map_err(|e| format!("prepare: {}", e))?;

        let rows = stmt.query_map([], |row| {
            Ok(VaultEntry {
                id: row.get(0)?,
                website: vec_to_string(row.get::<_, Vec<u8>>(1)?),
                username: vec_to_string(row.get::<_, Vec<u8>>(2)?),
                secret_key: vec_to_string(row.get::<_, Vec<u8>>(3)?),
                pashword: vec_to_string(row.get::<_, Vec<u8>>(4)?),
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

fn vec_to_string(v: Vec<u8>) -> String {
    String::from_utf8_lossy(&v).to_string()
}
```

- [ ] **Step 3: Register module in lib.rs**

File: `src-tauri/src/lib.rs` — append:
```rust
pub mod storage;
```

- [ ] **Step 4: Write test and run**

File: `src-tauri/tests/storage_test.rs`
```rust
#[test]
fn test_insert_and_retrieve() {
    use pashword_local_lib::storage::Vault;
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
    use pashword_local_lib::storage::Vault;
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
    use pashword_local_lib::storage::Vault;
    let dir = std::env::temp_dir();
    let db_path = dir.join("test_del_vault.db");
    let _ = std::fs::remove_file(&db_path);

    let vault = Vault::open(&db_path).unwrap();
    let id = vault.insert_entry(b"x.com", b"y", b"z", b"w").unwrap();
    vault.delete_entry(id).unwrap();
    assert_eq!(vault.get_entry_blobs().unwrap().len(), 0);

    let _ = std::fs::remove_file(&db_path);
}
```

Run:
```bash
cd src-tauri && cargo test
```
Expected: All tests pass (11 total)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat: add encrypted SQLite storage module"
```

---

### Phase 5: Rust Backend — Tauri Commands (IPC Bridge)

### Task 5: Wire up Tauri commands connecting frontend to backend

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Add serde dependency**

File: `src-tauri/Cargo.toml` — add:
```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Write commands.rs**

File: `src-tauri/src/commands.rs`
```rust
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

fn default_db_path() -> PathBuf {
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

    // Store salt
    let vault = Vault::open(&state.db_path)?;
    vault.check_integrity()?;
    vault.set_setting("salt", &salt)?;

    // Store a known plaintext for future password verification
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

    // Verify by decrypting the known token
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

    // Check duplicate
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
```

- [ ] **Step 3: Update lib.rs to wire up state**

File: `src-tauri/src/lib.rs` — replace content:
```rust
pub mod pashword;
pub mod crypto;
pub mod storage;
pub mod commands;
```

- [ ] **Step 4: Update main.rs**

File: `src-tauri/src/main.rs`
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pashword_local_lib::commands::{self, AppState};
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
```

- [ ] **Step 5: Add dirs dependency and make default_db_path public**

File: `src-tauri/Cargo.toml` — add:
```toml
dirs = "5"
```

In `src-tauri/src/commands.rs`, make `default_db_path` public:
```rust
pub fn default_db_path() -> PathBuf {
```

- [ ] **Step 6: Build and verify compilation**

```bash
cargo tauri build --debug 2>&1 | tail -20
```
Expected: Build succeeds with no errors

- [ ] **Step 7: Commit**

```bash
git add src-tauri/
git commit -m "feat: add Tauri IPC commands for vault operations"
```

---

### Phase 6: React Frontend — UI Foundation

### Task 6: Build the theme, layout shell, and Tauri hook

**Files:**
- Create: `src/hooks/useTauri.ts`
- Modify: `src/App.tsx`
- Modify: `src/styles/globals.css`

- [ ] **Step 1: Write typed Tauri invoke wrapper**

File: `src/hooks/useTauri.ts`
```typescript
import { invoke } from "@tauri-apps/api/core";

export interface VaultEntry {
  id: number;
  website: string;
  username: string;
  secret_key: string;
  pashword: string;
  created_at: string;
  updated_at: string;
}

export const commands = {
  generatePassword: (website: string, username: string, secretKey: string, length: number) =>
    invoke<string>("generate_password", { website, username, secretKey, length }),

  setupVault: (masterPassword: string) =>
    invoke<void>("setup_vault", { masterPassword }),

  unlockVault: (masterPassword: string) =>
    invoke<boolean>("unlock_vault", { masterPassword }),

  isVaultInitialized: () =>
    invoke<boolean>("is_vault_initialized"),

  saveEntry: (website: string, username: string, secretKey: string, pashword: string) =>
    invoke<number>("save_entry", { website, username, secretKey, pashword }),

  listEntries: () =>
    invoke<VaultEntry[]>("list_entries"),

  deleteEntry: (id: number) =>
    invoke<void>("delete_entry", { id }),
};
```

- [ ] **Step 2: Write App.tsx layout shell**

File: `src/App.tsx`
```tsx
import { useState, useEffect } from "react";
import { commands } from "./hooks/useTauri";
import { UnlockScreen } from "./components/UnlockScreen";
import { Generator } from "./components/Generator";
import { Vault } from "./components/Vault";

type Screen = "generator" | "vault";

function App() {
  const [unlocked, setUnlocked] = useState(false);
  const [initialized, setInitialized] = useState<boolean | null>(null);
  const [screen, setScreen] = useState<Screen>("generator");
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    commands.isVaultInitialized().then(setInitialized);
  }, []);

  if (initialized === null) return null;

  if (!unlocked) {
    return (
      <UnlockScreen
        initialized={initialized}
        onUnlocked={() => setUnlocked(true)}
      />
    );
  }

  const refreshVault = () => setRefreshKey((k) => k + 1);

  return (
    <main className="relative min-h-screen flex flex-col items-center px-4 py-8">
      {/* Background gradient */}
      <div className="fixed inset-0 bg-gradient-to-br from-[#0a0a10] via-[#0d0d1a] to-[#0a0a18] -z-10" />
      <div className="fixed inset-0 bg-[radial-gradient(ellipse_at_top,rgba(139,92,246,0.08),transparent_50%)] -z-10" />

      {/* Nav tabs */}
      <nav className="flex gap-2 mb-8 bg-[rgba(255,255,255,0.03)] border border-[rgba(255,255,255,0.08)] rounded-xl p-1 backdrop-blur-xl">
        <button
          onClick={() => setScreen("generator")}
          className={`px-5 py-2 rounded-lg text-sm font-medium transition-all ${
            screen === "generator"
              ? "bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white shadow-lg shadow-purple-500/20"
              : "text-[#a0a0b8] hover:text-white"
          }`}
        >
          Generator
        </button>
        <button
          onClick={() => setScreen("vault")}
          className={`px-5 py-2 rounded-lg text-sm font-medium transition-all ${
            screen === "vault"
              ? "bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white shadow-lg shadow-purple-500/20"
              : "text-[#a0a0b8] hover:text-white"
          }`}
        >
          Vault
        </button>
      </nav>

      {screen === "generator" ? (
        <Generator onSaved={refreshVault} />
      ) : (
        <Vault key={refreshKey} />
      )}
    </main>
  );
}

export default App;
```

- [ ] **Step 3: Verify dev server builds**

```bash
npm run tauri dev
```
Expected: Window opens with gradient background and tab nav. Close after confirming.

- [ ] **Step 4: Commit**

```bash
git add src/
git commit -m "feat: add app shell with navigation and Tauri IPC hooks"
```

---

### Phase 7: React Frontend — Unlock Screen

### Task 7: Build unlock / setup screen

**Files:**
- Create: `src/components/UnlockScreen.tsx`

- [ ] **Step 1: Write UnlockScreen.tsx**

File: `src/components/UnlockScreen.tsx`
```tsx
import { useState } from "react";
import { commands } from "../hooks/useTauri";

interface Props {
  initialized: boolean;
  onUnlocked: () => void;
}

export function UnlockScreen({ initialized, onUnlocked }: Props) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [showWarning, setShowWarning] = useState(false);

  const isSetup = !initialized;

  async function handleSubmit() {
    setError("");
    if (isSetup && password !== confirm) {
      setError("Passwords don't match");
      return;
    }
    if (password.length < 4) {
      setError("Password must be at least 4 characters");
      return;
    }

    setLoading(true);
    try {
      if (isSetup) {
        await commands.setupVault(password);
        onUnlocked();
      } else {
        const ok = await commands.unlockVault(password);
        if (ok) {
          onUnlocked();
        } else {
          setError("Incorrect master password");
        }
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="relative min-h-screen flex flex-col items-center justify-center px-4">
      <div className="fixed inset-0 bg-gradient-to-br from-[#0a0a10] via-[#0d0d1a] to-[#0a0a18] -z-10" />
      <div className="fixed inset-0 bg-[radial-gradient(ellipse_at_top,rgba(139,92,246,0.08),transparent_50%)] -z-10" />

      <div className="w-full max-w-[420px] bg-[rgba(255,255,255,0.03)] border border-[rgba(255,255,255,0.08)] rounded-2xl p-8 backdrop-blur-xl shadow-2xl">
        <h1 className="text-2xl font-bold text-white text-center mb-1">Pashword</h1>
        <p className="text-[#a0a0b8] text-sm text-center mb-6">
          {isSetup ? "Create your master password" : "Unlock your vault"}
        </p>

        <div className="space-y-4">
          <input
            type="password"
            placeholder="Master password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
            className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white placeholder-[#666] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
          />

          {isSetup && (
            <input
              type="password"
              placeholder="Confirm master password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
              className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white placeholder-[#666] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
            />
          )}

          {error && (
            <p className="text-red-400 text-xs text-center">{error}</p>
          )}

          <button
            onClick={handleSubmit}
            disabled={loading}
            className="w-full bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white font-semibold rounded-[10px] py-3 text-sm shadow-lg shadow-purple-500/20 hover:brightness-110 transition-all disabled:opacity-50"
          >
            {loading ? "..." : isSetup ? "Create Vault" : "Unlock"}
          </button>

          {isSetup && (
            <p className="text-[#a0a0b8] text-xs text-center mt-4">
              If you forget this password, your vault is <strong className="text-red-400">gone forever</strong>.
              There is no recovery.
            </p>
          )}
        </div>
      </div>
    </main>
  );
}
```

- [ ] **Step 2: Verify — run dev and test unlock/setup flow**

```bash
npm run tauri dev
```
Expected: Unlock screen appears. Create master password → Generator screen. Close and reopen → unlock with password.

- [ ] **Step 3: Commit**

```bash
git add src/components/UnlockScreen.tsx
git commit -m "feat: add unlock/setup screen with master password flow"
```

---

### Phase 8: React Frontend — Generator Screen

### Task 8: Build the Pashword generation form

**Files:**
- Create: `src/components/Generator.tsx`

- [ ] **Step 1: Write Generator.tsx**

File: `src/components/Generator.tsx`
```tsx
import { useState } from "react";
import { commands } from "../hooks/useTauri";

interface Props {
  onSaved: () => void;
}

export function Generator({ onSaved }: Props) {
  const [website, setWebsite] = useState("");
  const [username, setUsername] = useState("");
  const [secretKey, setSecretKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [length, setLength] = useState(32);
  const [generated, setGenerated] = useState("");
  const [copied, setCopied] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveMsg, setSaveMsg] = useState("");

  function copyToClipboard(text: string, label: string) {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(label);
      setTimeout(() => setCopied(null), 2000);
    });
  }

  async function handleGenerate() {
    if (!website || !username || !secretKey) return;
    try {
      const pw = await commands.generatePassword(website, username, secretKey, length);
      setGenerated(pw);
    } catch (e) {
      setGenerated("Error: " + String(e));
    }
  }

  async function handleSave() {
    if (!generated) return;
    setSaving(true);
    setSaveMsg("");
    try {
      await commands.saveEntry(website, username, secretKey, generated);
      setSaveMsg("Saved to vault!");
      onSaved();
      setTimeout(() => setSaveMsg(""), 2000);
    } catch (e) {
      setSaveMsg("Save failed: " + String(e));
    } finally {
      setSaving(false);
    }
  }

  const canGenerate = website && username && secretKey;

  return (
    <div className="w-full max-w-[500px] bg-[rgba(255,255,255,0.03)] border border-[rgba(255,255,255,0.08)] rounded-2xl p-8 backdrop-blur-xl shadow-2xl">
      <h2 className="text-xl font-bold text-white text-center mb-1">
        Pashword
      </h2>
      <p className="text-[#a0a0b8] text-sm text-center mb-6">
        Passwords done right
      </p>

      <div className="space-y-4">
        {/* Website */}
        <div>
          <label className="block text-xs text-[#a0a0b8] uppercase tracking-wider mb-1.5">
            Website
          </label>
          <input
            type="text"
            placeholder="example.com"
            value={website}
            onChange={(e) => setWebsite(e.target.value)}
            className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white placeholder-[#666] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
          />
        </div>

        {/* Username */}
        <div>
          <label className="block text-xs text-[#a0a0b8] uppercase tracking-wider mb-1.5">
            Username
          </label>
          <input
            type="text"
            placeholder="alice@example.com"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white placeholder-[#666] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
          />
        </div>

        {/* Secret Key */}
        <div>
          <label className="block text-xs text-[#a0a0b8] uppercase tracking-wider mb-1.5">
            Secret Key
          </label>
          <div className="relative">
            <input
              type={showKey ? "text" : "password"}
              placeholder="Your secret key"
              value={secretKey}
              onChange={(e) => setSecretKey(e.target.value)}
              className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 pr-12 text-white placeholder-[#666] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
            />
            <button
              onClick={() => setShowKey(!showKey)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-[#666] hover:text-[#a0a0b8] text-sm transition-colors"
              tabIndex={-1}
            >
              {showKey ? "Hide" : "Show"}
            </button>
          </div>
        </div>

        {/* Length */}
        <div>
          <label className="block text-xs text-[#a0a0b8] uppercase tracking-wider mb-1.5">
            Length
          </label>
          <select
            value={length}
            onChange={(e) => setLength(Number(e.target.value))}
            className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white text-sm focus:outline-none focus:border-[#8b5cf6] transition-all appearance-none cursor-pointer"
          >
            {[16, 20, 24, 28, 32, 40, 48, 64].map((n) => (
              <option key={n} value={n}>
                {n === 32 ? `${n} (Recommended)` : n}
              </option>
            ))}
          </select>
        </div>

        {/* Generate Button */}
        <button
          onClick={handleGenerate}
          disabled={!canGenerate}
          className="w-full bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white font-semibold rounded-[10px] py-3 text-sm shadow-lg shadow-purple-500/20 hover:brightness-110 transition-all disabled:opacity-40 disabled:cursor-not-allowed"
        >
          Get Pashword 😎
        </button>

        {/* Result */}
        {generated && (
          <div className="mt-4 p-4 bg-[#0d0d18] border border-[rgba(255,255,255,0.08)] rounded-[10px] space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs text-[#a0a0b8] uppercase tracking-wider">
                Your Pashword
              </span>
              <button
                onClick={() => copyToClipboard(generated, "password")}
                className="text-xs text-[#8b5cf6] hover:text-[#a855f7] transition-colors"
              >
                {copied === "password" ? "Copied!" : "Copy"}
              </button>
            </div>
            <p className="text-white text-sm break-all font-mono bg-[#0a0a10] rounded-lg p-3">
              {generated}
            </p>

            <button
              onClick={handleSave}
              disabled={saving}
              className="w-full bg-[rgba(139,92,246,0.15)] border border-[rgba(139,92,246,0.3)] text-[#a78bfa] font-medium rounded-[10px] py-2.5 text-sm hover:bg-[rgba(139,92,246,0.25)] transition-all disabled:opacity-50"
            >
              {saving ? "Saving..." : "Save to Vault"}
            </button>
            {saveMsg && (
              <p className={`text-xs text-center ${saveMsg.includes("failed") ? "text-red-400" : "text-green-400"}`}>
                {saveMsg}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify — run dev and test generation**

```bash
npm run tauri dev
```
Expected: Unlock → Generator. Enter test data → "Get Pashword 😎" → password appears with Copy and Save buttons.

- [ ] **Step 3: Commit**

```bash
git add src/components/Generator.tsx
git commit -m "feat: add pashword generation form with save-to-vault"
```

---

### Phase 9: React Frontend — Vault Screen

### Task 9: Build the password vault list with search, expand, copy, delete

**Files:**
- Create: `src/components/Vault.tsx`
- Create: `src/components/PasswordCard.tsx`
- Create: `src/components/SearchBar.tsx`

- [ ] **Step 1: Write SearchBar.tsx**

File: `src/components/SearchBar.tsx`
```tsx
interface Props {
  value: string;
  onChange: (v: string) => void;
  count: number;
}

export function SearchBar({ value, onChange, count }: Props) {
  return (
    <div className="flex items-center gap-3 mb-4">
      <div className="relative flex-1">
        <input
          type="text"
          placeholder="Search vault..."
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-2.5 text-white placeholder-[#666] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
        />
        {value && (
          <button
            onClick={() => onChange("")}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-[#666] hover:text-white text-sm"
          >
            Clear
          </button>
        )}
      </div>
      <span className="text-xs text-[#a0a0b8] whitespace-nowrap">
        {count} {count === 1 ? "entry" : "entries"}
      </span>
    </div>
  );
}
```

- [ ] **Step 2: Write PasswordCard.tsx**

File: `src/components/PasswordCard.tsx`
```tsx
import { useState } from "react";
import type { VaultEntry } from "../hooks/useTauri";

interface Props {
  entry: VaultEntry;
  onDelete: (id: number) => void;
}

export function PasswordCard({ entry, onDelete }: Props) {
  const [expanded, setExpanded] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);

  function copyToClipboard(text: string, label: string) {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(label);
      setTimeout(() => setCopied(null), 2000);
    });
  }

  function handleDelete() {
    if (confirm(`Delete entry for ${entry.website}?`)) {
      onDelete(entry.id);
    }
  }

  return (
    <div className="bg-[rgba(255,255,255,0.02)] border border-[rgba(255,255,255,0.06)] rounded-xl overflow-hidden transition-all hover:border-[rgba(255,255,255,0.12)]">
      {/* Collapsed row */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between px-4 py-3.5 text-left"
      >
        <div className="min-w-0">
          <p className="text-white text-sm font-medium truncate">
            {entry.website}
          </p>
          <p className="text-[#a0a0b8] text-xs truncate">{entry.username}</p>
        </div>
        <span className="text-[#666] text-xs ml-3 shrink-0">
          {expanded ? "▲" : "▼"}
        </span>
      </button>

      {/* Expanded details */}
      {expanded && (
        <div className="px-4 pb-4 space-y-3 border-t border-[rgba(255,255,255,0.06)] pt-3">
          <FieldRow
            label="Website"
            value={entry.website}
            copied={copied}
            onCopy={() => copyToClipboard(entry.website, "w-" + entry.id)}
            copyId={"w-" + entry.id}
          />
          <FieldRow
            label="Username"
            value={entry.username}
            copied={copied}
            onCopy={() => copyToClipboard(entry.username, "u-" + entry.id)}
            copyId={"u-" + entry.id}
          />
          <FieldRow
            label="Secret Key"
            value={entry.secret_key}
            copied={copied}
            onCopy={() => copyToClipboard(entry.secret_key, "sk-" + entry.id)}
            copyId={"sk-" + entry.id}
            sensitive
          />
          <FieldRow
            label="Pashword"
            value={entry.pashword}
            copied={copied}
            onCopy={() => copyToClipboard(entry.pashword, "pw-" + entry.id)}
            copyId={"pw-" + entry.id}
            mono
          />

          <button
            onClick={handleDelete}
            className="w-full text-center text-xs text-red-400/70 hover:text-red-400 py-2 transition-colors"
          >
            Delete Entry
          </button>
        </div>
      )}
    </div>
  );
}

function FieldRow({
  label,
  value,
  copied,
  copyId,
  onCopy,
  sensitive,
  mono,
}: {
  label: string;
  value: string;
  copied: string | null;
  copyId: string;
  onCopy: () => void;
  sensitive?: boolean;
  mono?: boolean;
}) {
  return (
    <div className="flex items-start justify-between gap-2">
      <div className="min-w-0">
        <p className="text-xs text-[#666] uppercase tracking-wider mb-0.5">
          {label}
        </p>
        <p
          className={`text-sm text-white break-all ${
            mono ? "font-mono text-xs" : ""
          } ${sensitive ? "blur-sm hover:blur-none transition-all select-none" : ""}`}
        >
          {value}
        </p>
      </div>
      <button
        onClick={onCopy}
        className="text-xs text-[#8b5cf6] hover:text-[#a855f7] shrink-0 transition-colors mt-1"
      >
        {copied === copyId ? "Copied!" : "Copy"}
      </button>
    </div>
  );
}
```

- [ ] **Step 3: Write Vault.tsx**

File: `src/components/Vault.tsx`
```tsx
import { useState, useEffect } from "react";
import { commands, type VaultEntry } from "../hooks/useTauri";
import { SearchBar } from "./SearchBar";
import { PasswordCard } from "./PasswordCard";

export function Vault() {
  const [entries, setEntries] = useState<VaultEntry[]>([]);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadEntries();
  }, []);

  async function loadEntries() {
    try {
      const list = await commands.listEntries();
      setEntries(list);
    } catch (e) {
      console.error("Failed to load entries:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete(id: number) {
    try {
      await commands.deleteEntry(id);
      setEntries((prev) => prev.filter((e) => e.id !== id));
    } catch (e) {
      console.error("Delete failed:", e);
    }
  }

  const filtered = entries.filter(
    (e) =>
      e.website.toLowerCase().includes(search.toLowerCase()) ||
      e.username.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="w-full max-w-[560px]">
      <SearchBar value={search} onChange={setSearch} count={filtered.length} />

      {loading ? (
        <p className="text-[#a0a0b8] text-sm text-center py-12">Loading...</p>
      ) : filtered.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-[#a0a0b8] text-sm">
            {search ? "No matching entries" : "No saved passwords"}
          </p>
          {!search && (
            <p className="text-[#666] text-xs mt-2">
              Generate and save a password to get started
            </p>
          )}
        </div>
      ) : (
        <div className="space-y-2">
          {filtered.map((entry) => (
            <PasswordCard
              key={entry.id}
              entry={entry}
              onDelete={handleDelete}
            />
          ))}
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 4: Verify full flow in dev**

```bash
npm run tauri dev
```
Expected: Unlock → Generator → generate → Save → switch to Vault → see entry → expand → copy fields → delete

- [ ] **Step 5: Commit**

```bash
git add src/components/
git commit -m "feat: add vault screen with search, expand, copy, and delete"
```

---

### Phase 10: Desktop Polish & Build

### Task 10: Configure Tauri for Windows release build

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Update tauri.conf.json for release**

File: `src-tauri/tauri.conf.json` — ensure these values are set:
```json
{
  "productName": "Pashword",
  "version": "0.1.0",
  "identifier": "com.pashword.local",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "Pashword",
        "width": 480,
        "height": 720,
        "resizable": true,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "nsis",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 2: Generate placeholder icons**

```bash
cd src-tauri
# Create minimal placeholder icon using built-in Tauri tool
cargo tauri icon 2>&1 || echo "Skipping icon generation - will use defaults"
```

- [ ] **Step 3: Build Windows release**

```bash
cargo tauri build 2>&1 | tail -20
```
Expected: Build succeeds. Output at `src-tauri/target/release/bundle/nsis/Pashword_0.1.0_x64-setup.exe`

- [ ] **Step 4: Test the .exe**

```bash
# Run the built exe and verify it works standalone
ls -lh src-tauri/target/release/bundle/nsis/
```
Expected: `.exe` installer exists

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: configure Tauri for Windows NSIS release build"
```

---

### Phase 11: Android Build Setup

### Task 11: Configure and build Android APK

**Files:**
- Create: `src-tauri/tauri.android.conf.json`

- [ ] **Step 1: Initialize Tauri Android project**

```bash
cd src-tauri
cargo tauri android init
```
Expected: Creates android project structure

- [ ] **Step 2: Set Android app metadata**

Edit `src-tauri/tauri.android.conf.json`:
```json
{
  "productName": "Pashword",
  "version": "0.1.0",
  "identifier": "com.pashword.local",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "Pashword",
        "width": 480,
        "height": 720,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": ["apk"],
    "android": {
      "minSdkVersion": 26
    }
  }
}
```

- [ ] **Step 3: Build Android APK**

```bash
cd src-tauri
cargo tauri android build --target aarch64-linux-android 2>&1 | tail -30
```
Expected: Build succeeds. Output APK at `src-tauri/gen/android/app/build/outputs/apk/release/`

- [ ] **Step 4: Verify APK size**

```bash
ls -lh src-tauri/gen/android/app/build/outputs/apk/release/*.apk
```
Expected: ~15-20 MB

- [ ] **Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat: add Android build configuration and APK target"
```

---

### Phase 12: Final Cleanup & GitHub Release

### Task 12: Add README and .gitignore

**Files:**
- Create: `README.md`
- Modify: `.gitignore`

- [ ] **Step 1: Write .gitignore**

File: `.gitignore`
```
node_modules/
dist/
src-tauri/target/
src-tauri/gen/
*.apk
*.exe
.DS_Store
```

- [ ] **Step 2: Write README.md**

File: `README.md`
```markdown
# Pashword Local

A self-hosted, offline password generator and vault. Deterministic password generation
forked from [pashword/pashword](https://github.com/pashword/pashword). Built with
Tauri v2 + React + Rust.

## Features

- **Deterministic password generation** — same inputs always produce the same password
- **Encrypted local vault** — AES-256-GCM encrypted SQLite storage
- **No server, no cloud** — everything stays on your device
- **Cross-platform** — Windows desktop + Android

## Build from source

### Prerequisites
- Rust (rustup)
- Node.js 22+
- Tauri CLI (`cargo install tauri-cli`)

### Desktop (Windows)
```bash
npm install
cargo tauri build
# Output: src-tauri/target/release/bundle/nsis/Pashword_*.exe
```

### Android
```bash
npm install
cargo tauri android init
cargo tauri android build
# Output: .apk
```

## Security

All vault data is encrypted with AES-256-GCM. The encryption key is derived from
your master password using Pashword's SHA3-512 → scrypt → SHAKE-256 pipeline.
The master password is never stored. If you forget it, the vault is unrecoverable.

## License

AGPL-3.0 (inherited from pashword/pashword)
```

- [ ] **Step 3: Commit and tag**

```bash
git add .gitignore README.md
git commit -m "docs: add README and .gitignore"
git tag v0.1.0
```

- [ ] **Step 4: Push to GitHub**

```bash
git remote add origin git@github.com:Xan-WH1Te/pashword-local.git
git push -u origin master --tags
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| **Prerequisites** | 5 steps | Install Rust, Node, Tauri CLI, Android SDK |
| **Phase 1** | Task 1 | Scaffold Tauri v2 + React + TS + Tailwind |
| **Phase 2** | Task 2 | Port pashword-lib hashing to Rust |
| **Phase 3** | Task 3 | AES-256-GCM encrypt/decrypt |
| **Phase 4** | Task 4 | SQLite storage with encrypted fields |
| **Phase 5** | Task 5 | Tauri IPC commands (bridge) |
| **Phase 6** | Task 6 | App shell, theme, navigation, hooks |
| **Phase 7** | Task 7 | Unlock / setup screen |
| **Phase 8** | Task 8 | Generator screen |
| **Phase 9** | Task 9 | Vault screen (search, copy, delete) |
| **Phase 10** | Task 10 | Windows .exe release build |
| **Phase 11** | Task 11 | Android .apk build |
| **Phase 12** | Task 12 | README, .gitignore, push |

**Estimated total:** 12 tasks, ~60 steps
