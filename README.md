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
