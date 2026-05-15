# Pashword Local — Design Spec

A self-hosted, native password generator and vault app forked from [pashword/pashword](https://github.com/pashword/pashword) (AGPL-3.0). Runs entirely offline with no server dependencies.

## Target Platforms

| Platform | Format | Arch |
|----------|--------|------|
| Windows 11 | `.exe` (NSIS installer) | x86_64 |
| Android | `.apk` | ARM64 (aarch64) — vivo X200 Pro (Dimensity 9400) |

## Architecture

```
React Frontend (TypeScript + Tailwind CSS)
        │  Tauri IPC (invoke)
        ▼
Rust Backend
  ├── Crypto Module    (AES-256-GCM, scrypt, SHA3-512, SHAKE-256)
  ├── Storage Module   (SQLite via rusqlite, encrypted at rest)
  └── Pashword Gen     (ported from pashword-lib)
```

Single codebase, two build targets via Tauri v2.

## Tech Stack

| Layer | Choice |
|-------|--------|
| Desktop framework | Tauri v2 (Rust backend, WebView2 frontend) |
| Mobile framework | Tauri v2 mobile (Android WebView) |
| Frontend | React 18 + TypeScript + Tailwind CSS |
| Build tool | Vite |
| Crypto (Rust) | `aes-gcm`, `sha3`, `scrypt` crates |
| Database | `rusqlite` (bundled SQLite) |
| Android | Tauri Android plugin, NDK, ARM64 target |

## UI Design

Exact replica of [pashword.app](https://pashword.app/) dark theme:

- **Background:** `#0a0a10` with subtle purple/blue radial gradient
- **Cards:** `rgba(255,255,255,0.03)` + `1px rgba(255,255,255,0.08)` border, `border-radius: 16px`, glass blur
- **Accent:** Purple-magenta gradient (`#8b5cf6 → #a855f7`)
- **Inputs:** `#0d0d18` bg, `1px rgba(255,255,255,0.10)` border, `border-radius: 10px`, purple glow on focus
- **Buttons:** Full-width purple→magenta gradient, white bold text, `border-radius: 10px`
- **Font:** Inter
- **Max content width:** ~560px for forms, centered

### Two Screens

**Generator** — the original Pashword form: Website, Username, Secret Key (eye toggle), password length selector, "Get Pashword 😎" gradient button. Generated password with Copy button. "Save to Vault" action appears after generation.

**Vault** — list of saved entries with search bar. Each entry shows Website + Username. Tap to expand: all 4 fields with individual Copy buttons. Long-press to delete.

Navigation: top tab toggle "Generator" | "Vault", plus vault entry count.

## Data Model

```sql
CREATE TABLE vault (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    website     BLOB NOT NULL,   -- AES-256-GCM encrypted
    username    BLOB NOT NULL,   -- AES-256-GCM encrypted
    secret_key  BLOB NOT NULL,   -- AES-256-GCM encrypted
    pashword    BLOB NOT NULL,   -- AES-256-GCM encrypted
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

All fields except `id` and timestamps are AES-256-GCM encrypted.

## Security Model

### Key Derivation
Uses Pashword's own hashing pipeline (identical algorithm to password generation):

```
Master Password + Salt (32 random bytes)
    → SHA3-512 → scrypt (N=16384, r=8, p=1) → SHAKE-256
    → First 256 bits = AES-256-GCM key
```

Salt is stored in a `settings` table. Master password is never stored. No recovery possible if forgotten — user warned on setup.

### Encryption
- Per-field AES-256-GCM: `ciphertext || nonce(12 bytes) || tag(16 bytes)` stored as BLOB
- Fresh nonce per write, not per field
- On vault open: decrypt one known field to validate master password
- Clipboard auto-clear after 30 seconds (desktop), toast-only on Android

### Threat Model
- SQLite file is meaningless without master password
- Plaintext only exists in memory during active use
- Android: DB in Tauri private app directory, inaccessible to other apps

## Feature Scope

### Included
- Deterministic Pashword generation (original algorithm, ported to Rust)
- AES-256-GCM encrypted local vault
- Store Website, Username, Secret Key, generated Pashword
- Copy button for each field
- Search/filter vault entries
- Delete entries (long-press)
- Duplicate detection (same website+username)
- Integrity check on DB open

### Excluded (v1)
- Cloud sync
- Browser extension
- iOS
- OS-level autofill
- Import/export
- Biometric unlock

## Project Structure

```
pashword-local/
├── src/                      # React frontend
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── Generator.tsx      # Pashword generation form
│   │   ├── Vault.tsx           # Saved entries list
│   │   ├── PasswordCard.tsx    # Single vault entry (expandable)
│   │   ├── SearchBar.tsx       # Vault filter
│   │   └── UnlockScreen.tsx    # Master password prompt
│   ├── hooks/
│   │   └── useTauri.ts        # Typed invoke wrappers
│   └── styles/
│       └── globals.css         # Tailwind + custom theme
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── crypto.rs          # AES-GCM + Pashword key derivation
│   │   ├── pashword.rs        # Port of pashword-lib algorithm
│   │   ├── storage.rs         # SQLite CRUD
│   │   └── commands.rs        # Tauri command handlers
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
├── package.json
├── vite.config.ts
├── tsconfig.json
└── tailwind.config.js
```

## Build Commands

```bash
# Development
npm install
cargo tauri dev              # Desktop dev server with hot reload

# Desktop release
cargo tauri build            # → .exe (NSIS installer)

# Android release
cargo tauri android init
cargo tauri android build    # → .apk
```
