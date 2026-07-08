# ADR 0001 — SQLite Encryption Driver

- **Date:** 2026-07-08
- **Status:** Accepted

## Context

The blueprint mandates full-database encryption at rest (AES-256). SQLite does
not support encryption natively. Three realistic options were evaluated:

| Option | Mechanism | Build complexity on Windows |
|---|---|---|
| `sqlx` + sqlite feature | No encryption | — (rejected outright) |
| `sqlcipher` (system lib) | Full-page AES-256 via SQLCipher fork | High — requires external OpenSSL headers |
| `rusqlite` `bundled-sqlcipher-vendored-openssl` | Full-page AES-256; SQLCipher + OpenSSL compiled from source by Cargo | Low — self-contained `cargo build` |
| FrankenSQLite | Page-level XChaCha20-Poly1305, pure Rust | Requires nightly; experimental |
| Column-level application encryption | Selective field encryption | Metadata leakage; error-prone |

Column-level encryption was rejected explicitly: it leaks metadata (row counts,
timestamps, payee name lengths) and is difficult to implement without subtle
cryptographic errors. Full-database encryption at the page level is the only
approach that satisfies the zero-trust threat model.

## Decision

Use `rusqlite` with the `bundled-sqlcipher-vendored-openssl` feature.

```toml
rusqlite = { version = "0.32", features = ["bundled-sqlcipher-vendored-openssl"] }
```

This compiles SQLCipher and OpenSSL from vendored source during `cargo build`,
producing a self-contained binary with no external runtime dependencies. The
encryption key is supplied immediately after opening the connection via
`PRAGMA key = '...'` and is retrieved from the OS keyring — never from disk.

## Consequences

- **Good:** Works on Windows, macOS, and Linux with a plain `cargo build`.
  No OpenSSL install required on developer machines or CI.
- **Good:** Industry-standard AES-256 page-level encryption; same algorithm
  used by Signal, WhatsApp, and 1Password for local SQLite stores.
- **Good:** Compatible with WAL mode for crash-safe writes.
- **Trade-off:** Build times increase because OpenSSL is compiled from source.
  This is a one-time cost per clean build; incremental builds are unaffected.
- **Trade-off:** `rusqlite` is synchronous. Tauri commands that touch the DB
  are wrapped in `tokio::task::spawn_blocking` to avoid blocking the async
  executor. See ADR 0005 for the connection-handle strategy.
- **Watch:** If a linking conflict arises between vendored OpenSSL and another
  crate bundling BoringSSL (e.g., a future WebRTC plugin), the dependency must
  be resolved before merging.
