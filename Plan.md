# Plan: Personal — Local-First Personal Finance App

> **Tagline:** Personal Finance should be private.

## Overview

A local-first, crash-proof desktop application built with pure Rust and `egui` for robust double-entry accounting. Targets non-US/EU users by prioritising fault-tolerant Excel/CSV import over API scraping. All financial data is encrypted at rest; the master key never touches the disk.

**Stack (locked):**

| Layer             | Choice                                             | Reason                                             |
| ----------------- | -------------------------------------------------- | -------------------------------------------------- |
| Runtime           | Native Desktop (eframe)                            | High performance native execution                  |
| Frontend          | egui                                               | Immediate mode GUI, single binary footprint        |
| Styling           | egui themes                                        | Built-in theming, fast rendering                   |
| Backend           | Rust                                               | Memory-safe systems language                       |
| DB                | SQLite encrypted via SQLCipher                     | Full-page AES-256 encryption at rest               |
| Encryption driver | `rusqlite` with `bundled-sqlcipher`                | Requires external OpenSSL/LibreSSL to be installed |
| Key storage       | `keyring` crate (OS Keychain / Credential Manager) | Key never written to disk                          |
| Excel parsing     | `calamine`                                         | Handles .xls / .xlsx without COM                   |
| CSV parsing       | `csv` crate                                        | Zero-allocation reader                             |
| Template format   | TOML                                               | Literal strings eliminate regex-escape hell        |

---

## Correctness Principles

This codebase enforces **total program correctness**: every branch must terminate and every reachable state must satisfy the program invariants.

### Functional Programming Style

- **Rust backend:** Pure functions preferred; side effects (DB, keyring, FS) are isolated to Tokio background tasks.
- **State Machine:** UI projection is entirely driven by an immutable reference to the `AppState`. All interactions yield `Message` variants that cleanly transition the state.
- **No panics in library code.** `unwrap()` and `expect()` are banned outside of `main` bootstrap. All fallible paths return `Result<T, AppError>`.
- **No `unsafe` blocks** except where required by the SQLCipher FFI (which is encapsulated inside `rusqlite` itself).

### Total Correctness Guarantees

| Property                    | Mechanism                                                                                                                                                                              |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Termination**             | All recursive functions are bounded (no unbounded loops in parser; regex engine has a timeout via `regex::RegexBuilder::size_limit`). Naive Bayes uses a fixed feature vocabulary.     |
| **Partial correctness**     | State transitions gracefully handle all error variants.                                                            |
| **No unhandled exceptions** | Rust compiler guarantees.                                                                           |
| **No data races**           | `std::sync::mpsc` or `crossbeam-channel` queues safely hand off Tokio async results to the main rendering thread. |
| **No undefined behaviour**  | No `unsafe` in application code. Clippy `#![deny(unsafe_code)]` on library crates.                                                                                                     |
| **All branches covered**    | Rust enums (e.g., `Message`, `Command`) are exhaustively matched in `apply_message`.                                                                    |

---

## Architecture

```
+------------------------------------------------------------+
|                     User''s Machine                        |
|                                                            |
|  +------------------------------------------------------+  |
|  |              Rust eframe Application                 |  |
|  |                                                      |  |
|  |  +-----------------------+  +---------------------+  |  |
|  |  |   egui UI (app.rs)    |  |    State Machine    |  |  |
|  |  |                       |  |    (state.rs)       |  |  |
|  |  |  Read-only State Ref  |<-|  AppState           |  |  |
|  |  |                       |  |                     |  |  |
|  |  |  Events -> Message    |->|  apply_message()    |  |  |
|  |  +-----------------------+  +----------+----------+  |  |
|  |                                        | Command     |  |
|  |                             +----------v----------+  |  |
|  |                             |   Tokio Tasks       |  |  |
|  |                             |   (DB, File I/O)    |  |  |
|  |                             +----------+----------+  |  |
|  |                                        |             |  |
|  +----------------------------------------+-------------+  |
|                                           |                |
|  +----------------------+  +--------------v--------------+ |
|  | OS Keystore          |  | data.db (SQLCipher AES-256) | |
|  | (Master Key)         |->| $APP_DATA/Personal/         | |
|  +----------------------+  +-----------------------------+ |
+------------------------------------------------------------+
```

---

## Database Schema

```sql
-- Currency stored as integers (paise for INR). Floating-point forbidden.

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE accounts (
    id        TEXT PRIMARY KEY,
    name      TEXT NOT NULL,
    type      TEXT NOT NULL CHECK(type IN ('asset','liability','equity','revenue','expense')),
    commodity TEXT NOT NULL
);

CREATE TABLE transactions (
    id    TEXT PRIMARY KEY,
    date  TEXT NOT NULL,   -- ISO 8601
    payee TEXT NOT NULL,
    notes TEXT
);

CREATE TABLE postings (
    id             TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id     TEXT NOT NULL REFERENCES accounts(id),
    amount         INTEGER NOT NULL,   -- smallest currency unit (paise)
    commodity      TEXT NOT NULL
);

CREATE INDEX idx_postings_txn     ON postings(transaction_id);
CREATE INDEX idx_postings_account ON postings(account_id);
CREATE INDEX idx_txn_date         ON transactions(date);
```

**Invariant (Rust-enforced before every commit):**

> For every transaction t: SUM(postings.amount WHERE transaction_id = t.id) = 0

---

## Phases

### Phase 1 — Foundation

**Goal:** Compilable eframe app.

#### `Cargo.toml` — crates to add

```toml
[dependencies]
eframe = "0.35"
egui = "0.35"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
crossbeam-channel = "0.5"
```
