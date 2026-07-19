# Plan: Personal — Local-First Personal Finance App

> **Tagline:** Personal Finance should be private.

## Overview

A local-first, crash-proof desktop application built with pure Rust and `iced` for robust double-entry accounting. Targets non-US/EU users by prioritising fault-tolerant Excel/CSV import over API scraping. All financial data is encrypted at rest; the master key never touches the disk.

**Stack (locked):**

| Layer             | Choice                                             | Reason                                             |
| ----------------- | -------------------------------------------------- | -------------------------------------------------- |
| Runtime           | Native Desktop                                     | High performance native execution                  |
| Frontend          | iced                                               | Retained-mode GUI, The Elm Architecture, single binary footprint |
| Styling           | iced themes                                        | Built-in theming, fast rendering                   |
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

### Test-Driven & API-Boundary First Architecture

The system is strictly layered. UI is decoupled from the core business logic. We define clear boundaries (Traits/Contracts) for the Storage, Parser, and Ledger layers, and test them rigorously before writing any UI code.

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

## Architecture (Layered Boundaries)

```
+------------------------------------------------------------+
|                     User's Machine                         |
|                                                            |
|  +------------------------------------------------------+  |
|  |              Rust Application Core                   |  |
|  |                                                      |  |
|  |  [ domain::Parser ]  -> Reads CSV/XLSX safely        |  |
|  |  [ domain::Ledger ]  -> Enforces double-entry rules  |  |
|  |  [ domain::Storage ] -> Abstract DB operations       |  |
|  |                                                      |  |
|  +------------------------------------------------------+  |
|                             |                              |
|  +------------------------------------------------------+  |
|  |              iced Desktop Frontend                   |  |
|  |                                                      |  |
|  |  Consumes domain interfaces for interaction          |  |
|  +------------------------------------------------------+  |
+------------------------------------------------------------+
```

---

## Database Schema (Storage Layer)

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

### Phase 1 — API Boundaries & Contracts (Current)

**Goal:** Define the interface traits and data structures (models) for `Storage`, `Parser`, and `Ledger`.

### Phase 2 — Storage Implementation & Testing

**Goal:** Implement the encrypted SQLite storage backend and verify it heavily with automated tests.

### Phase 3 — Parser Implementation & Testing

**Goal:** Implement the zero-trust data pipeline and test it with mock CSV/Excel inputs.

### Phase 4 — Core Ledger Implementation

**Goal:** Wire the storage and parser together, enforcing all business logic and double-entry rules.

### Phase 5 — UI Construction

**Goal:** Hook the heavily tested backend to the `iced` frontend. Ensure premium aesthetic (no generic grey/black themes) and performant retained-mode GUI rendering using The Elm Architecture.
