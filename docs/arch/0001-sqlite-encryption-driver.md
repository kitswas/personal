# ADR 0001 — SQLite Encryption Driver

- **Date:** 2026-07-08
- **Status:** Accepted

## Context

The blueprint mandates full-database encryption at rest (AES-256). SQLite does
not support encryption natively. Three realistic options were evaluated:

| Option                              | Mechanism                                                  | Build complexity on Windows              |
| ----------------------------------- | ---------------------------------------------------------- | ---------------------------------------- |
| `sqlx` + sqlite feature             | No encryption                                              | — (rejected outright)                    |
| `sqlcipher` (system lib)            | Full-page AES-256 via SQLCipher fork                       | High — requires external OpenSSL headers |
| `rusqlite` `bundled-sqlcipher`      | Full-page AES-256; SQLCipher compiled from source by Cargo | Medium — requires system OpenSSL         |
| FrankenSQLite                       | Page-level XChaCha20-Poly1305, pure Rust                   | Requires nightly; experimental           |
| Column-level application encryption | Selective field encryption                                 | Metadata leakage; error-prone            |

Column-level encryption was rejected explicitly: it leaks metadata (row counts,
timestamps, payee name lengths) and is difficult to implement without subtle
cryptographic errors. Full-database encryption at the page level is the only
approach that satisfies the zero-trust threat model.

## Decision

Use `rusqlite` with the `bundled-sqlcipher` feature.

```toml
rusqlite = { version = "0.32", features = ["bundled-sqlcipher"] }
```

This compiles SQLCipher from source during `cargo build`, relying on the system's installation of OpenSSL (or LibreSSL) for cryptographic primitives. The encryption key is supplied immediately after opening the connection via `PRAGMA key = '...'` and is retrieved from the OS keyring — never from disk.

### Rejected Alternative: `bundled-sqlcipher-vendored-openssl`

The `bundled-sqlcipher-vendored-openssl` feature allows using `bundled-sqlcipher` with a vendored version of OpenSSL (via the `openssl-sys` crate) as the crypto provider. This uses the `openssl-src` crate to compile and statically link to a copy of OpenSSL.

However, the build process requires a C compiler, Perl (and perl-core), and make. To avoid this heavy and platform-specific toolchain, we rejected the vendored OpenSSL approach.

## Consequences

- **Good:** Works without requiring a complex Perl/MSVC toolchain setup.
- **Good:** Industry-standard AES-256 page-level encryption; same algorithm used by Signal, WhatsApp, and 1Password for local SQLite stores.
- **Good:** Compatible with WAL mode for crash-safe writes.
- **Trade-off:** Requires developers and CI environments to have OpenSSL/LibreSSL pre-installed (e.g. via `vcpkg`, `choco`, `brew`, or `apt`) to provide `libcrypto`.
- **Trade-off:** `rusqlite` is synchronous. Async commands that touch the DB are wrapped in `tokio::task::spawn_blocking` to avoid blocking the async executor. See ADR 0005 for the connection-handle strategy.
