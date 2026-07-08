# Plan: Personal — Local-First Personal Finance App

> **Tagline:** Personal Finance should be private.

## Overview

A local-first, crash-proof desktop application built with Tauri v2 for robust
double-entry accounting. Targets non-US/EU users by prioritising fault-tolerant
Excel/CSV import over API scraping. All financial data is encrypted at rest;
the master key never touches the disk.

**Stack (locked):**

| Layer | Choice | Reason |
|---|---|---|
| Runtime | Tauri v2 | Tiny footprint, future mobile targets |
| Frontend | Svelte 5 + TypeScript | Path A from blueprint; already scaffolded |
| Styling | Oat.ink + plain CSS (fluid units) | Blueprint requirement; no Tailwind |
| Charts | SveltePlot | Grammar-of-graphics for analytics |
| Backend | Rust | Memory-safe systems language |
| DB | SQLite encrypted via SQLCipher | Full-page AES-256 encryption |
| Encryption driver | `rusqlite` with `bundled-sqlcipher-vendored-openssl` | Self-contained build on all platforms; no external OpenSSL install required |
| Key storage | `keyring` crate (OS Keychain / Credential Manager) | Key never written to disk |
| Excel parsing | `calamine` | Handles .xls / .xlsx without COM |
| CSV parsing | `csv` crate | Zero-allocation reader |
| Template format | TOML | Literal strings eliminate regex-escape hell |

---

## Architecture

```
+------------------------------------------------------------+
|                     User''s Machine                         |
|                                                            |
|  +------------------------------------------------------+  |
|  |              Tauri Application Process               |  |
|  |                                                      |  |
|  |  +-----------------------+  +---------------------+  |  |
|  |  |   Svelte 5 Frontend   |  |    Rust Backend     |  |  |
|  |  |                       |  |                     |  |  |
|  |  |  Oat.ink UI           |<-|  IPC Command        |  |  |
|  |  |  SveltePlot           |  |  Handler            |  |  |
|  |  |  Triage Grid          |->|                     |  |  |
|  |  |  pretext layout       |  |  calamine / csv     |  |  |
|  |  |                       |  |  TOML Templates     |  |  |
|  |  |                       |  |  Naive Bayes        |  |  |
|  |  |                       |  |  rusqlite+cipher    |  |  |
|  |  +-----------------------+  +----------+----------+  |  |
|  |                                        |              |  |
|  +----------------------------------------+--------------+  |
|                                           |                  |
|  +----------------------+  +--------------v--------------+  |
|  | OS Keystore          |  | data.db (SQLCipher AES-256) |  |
|  | (Master Key)         |->| $APP_DATA/Personal/         |  |
|  +----------------------+  +-----------------------------+  |
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
    id        TEXT PRIMARY KEY,          -- UUIDv4
    name      TEXT NOT NULL,
    type      TEXT NOT NULL CHECK(type IN ('asset','liability','equity','revenue','expense')),
    commodity TEXT NOT NULL              -- e.g. 'INR', 'USD'
);

CREATE TABLE transactions (
    id    TEXT PRIMARY KEY,
    date  TEXT NOT NULL,                 -- ISO 8601
    payee TEXT NOT NULL,
    notes TEXT
);

CREATE TABLE postings (
    id             TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id     TEXT NOT NULL REFERENCES accounts(id),
    amount         INTEGER NOT NULL,     -- smallest currency unit (paise)
    commodity      TEXT NOT NULL
);

CREATE INDEX idx_postings_txn     ON postings(transaction_id);
CREATE INDEX idx_postings_account ON postings(account_id);
CREATE INDEX idx_txn_date         ON transactions(date);
```

**Invariant (enforced in Rust before every commit):**

> For every transaction t: SUM(postings.amount WHERE transaction_id = t.id) = 0

---

## Phases

### Phase 1 — Dependencies & DB Foundation

**Goal:** Compilable Tauri app that opens an encrypted database on launch.

#### `src-tauri/Cargo.toml` — crates to add

```toml
[dependencies]
tauri                 = { version = "2", features = ["dialog"] }
tauri-plugin-opener   = "2"
tauri-plugin-dialog   = "2"
serde                 = { version = "1", features = ["derive"] }
serde_json            = "1"

# Encrypted SQLite — bundles SQLCipher + OpenSSL; no external install needed
rusqlite = { version = "0.32", features = ["bundled-sqlcipher-vendored-openssl"] }

# Async runtime
tokio    = { version = "1", features = ["full"] }

# Parsing
calamine = "0.26"
csv      = "1"
toml     = "0.8"
regex    = "1"

# Utilities
keyring   = { version = "3", features = ["windows-native"] }
uuid      = { version = "1", features = ["v4"] }
chrono    = { version = "0.4", features = ["serde"] }
anyhow    = "1"
thiserror = "1"
```

#### New files

| Path | Purpose |
|---|---|
| `src-tauri/migrations/0001_initial.sql` | Full schema |
| `src-tauri/src/db.rs` | Open encrypted DB, run migrations |
| `src-tauri/src/error.rs` | `AppError` typed error enum |

#### `src-tauri/src/lib.rs` changes

- Open `rusqlite::Connection` immediately wrapped in `Arc<Mutex<Connection>>`
- Store as `tauri::State`
- Execute `PRAGMA key = '...'` immediately after open, before any query
- Run migration SQL if tables do not exist
- Enable `PRAGMA journal_mode = WAL` for crash-safe writes

---

### Phase 2 — Rust Backend: IPC Commands

**Goal:** All backend logic exposed as typed Tauri commands.

#### File layout

```
src-tauri/src/
├── commands/
│   ├── mod.rs
│   ├── accounts.rs
│   ├── transactions.rs
│   ├── import.rs
│   └── security.rs
├── parser/
│   ├── mod.rs
│   ├── excel.rs
│   └── csv_parser.rs
├── template/
│   ├── mod.rs
│   └── types.rs
├── classifier.rs       <- Naive Bayes over SQLite history
├── models.rs
├── db.rs
├── error.rs
└── lib.rs
```

#### `models.rs` — the ParsedRow data contract

```rust
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum ParsedRow {
    Valid {
        row_idx:           usize,
        date:              String,
        payee:             String,
        amount:            i64,
        target_account_id: String,
    },
    Invalid {
        row_idx:      usize,
        raw_data:     String,
        error_reason: String,
    },
}
```

#### `commands/accounts.rs`

| Command | Signature |
|---|---|
| `list_accounts` | `() -> Vec<Account>` |
| `create_account` | `(name, type, commodity) -> Account` |
| `delete_account` | `(id) -> ()` |
| `get_account_balance` | `(account_id) -> i64` |

#### `commands/transactions.rs`

| Command | Signature |
|---|---|
| `list_transactions` | `(limit, offset) -> Vec<TransactionWithPostings>` |
| `commit_transaction` | `(date, payee, notes, postings) -> Transaction` — rejects if SUM != 0 |
| `get_running_balances` | `(account_id) -> Vec<BalanceEntry>` |
| `delete_transaction` | `(id) -> ()` |

#### `commands/import.rs`

| Command | Signature |
|---|---|
| `list_templates` | `() -> Vec<TemplateMeta>` |
| `parse_statement` | `(file_path, template_name) -> Vec<ParsedRow>` — never panics |
| `commit_import_batch` | `(Vec<ValidRow>) -> BatchResult` |

#### `commands/security.rs`

| Command | Signature |
|---|---|
| `is_first_run` | `() -> bool` |
| `setup_master_password` | `(password, default_currency) -> ()` |
| `unlock` | `(password) -> bool` |
| `change_master_password` | `(old, new) -> ()` |

#### Auto-categorisation (`classifier.rs`)

1. **Exact rule match** — TOML-defined regex patterns tried first
2. **Naive Bayes fallback** — trains on `payee` text of historical transactions
3. Low-confidence predictions surface as `Invalid` for human review

---

### Phase 3 — Frontend: Design System & Layout Shell

**Goal:** Working shell with sidebar nav, dark-mode-first design, fluid layout.

#### `src/app.css` — design tokens (no hardcoded px for layout)

```css
:root {
  --color-bg:       hsl(220, 13%,  9%);
  --color-surface:  hsl(220, 13%, 13%);
  --color-border:   hsl(220, 13%, 20%);
  --color-text:     hsl(220, 14%, 90%);
  --color-muted:    hsl(220, 10%, 55%);
  --color-accent:   hsl(252, 80%, 68%);
  --color-positive: hsl(150, 65%, 48%);
  --color-negative: hsl(  0, 72%, 58%);
  --color-warning:  hsl( 38, 92%, 58%);

  --sidebar-w: 14%;   /* fluid; collapses on narrow viewports */
}
```

#### `src/routes/+layout.svelte`

- Two-panel CSS grid: `[sidebar] [main]`
- Sidebar collapses to bottom tabs at `< 640px`
- Navigation: Dashboard / Accounts / Transactions / Import / Analytics / Settings
- "Personal" app name + tagline in sidebar header

---

### Phase 4 — Onboarding Flow

First-run flow — shown before the main UI when `is_first_run()` is true.

```
Screen 1: Welcome
  "Personal Finance should be private."
  [Get Started]

Screen 2: Set Master Password
  Password + Confirm fields
  Strength indicator
  [Next]

Screen 3: Default Currency
  Searchable currency picker (INR default)
  [Next]

Screen 4: Seed Accounts (optional)
  "Add default accounts: Checking, Savings, Cash, Credit Card"
  [Finish Setup]
```

`src/routes/onboarding/+page.svelte` — multi-step wizard.
`+layout.svelte` redirects to `/onboarding` if `is_first_run()` returns true.

---

### Phase 5 — Core Pages

#### Dashboard (`src/routes/+page.svelte`)
- Net worth card (sum of all asset accounts)
- 30-day income vs expense bar
- Last 10 transactions

#### Accounts (`src/routes/accounts/+page.svelte`)
- Accounts grouped by type with running balances
- Add account modal (name, type, commodity)
- Delete with confirmation if account has postings

#### Transactions (`src/routes/transactions/+page.svelte`)
- Paginated ledger: date / payee / amount columns
- Expand row to see postings
- Manual entry form: client-side SUM=0 check + server-side enforcement

---

### Phase 6 — Import / Triage Grid

`src/routes/import/+page.svelte` — the most complex page.

```
+--------------------------------------------------+
|  1. Drop zone (.xls / .xlsx / .csv)              |
|  2. Template picker                              |
|  3. [Parse File] button                          |
+--------------------------------------------------+
|  Triage Table (oat-table):                       |
|  | row | date | payee | amount | account        ||
|  | [OK]  ...     ...     ...    dropdown        ||
|  | [!!]  raw_data / error_reason / inline edit  ||
|                                                  |
|  [Commit N valid rows]                           |
+--------------------------------------------------+
```

- Valid rows: green highlight, ready to commit
- Invalid rows: red highlight, inline-editable fields, re-validate on change
- Batch commit calls `commit_import_batch()` — Rust validates each posting set

---

### Phase 7 — Analytics

`src/routes/analytics/+page.svelte`

- Monthly income vs expense (SveltePlot bar chart)
- Running balance per account over time (SveltePlot line chart)
- Top spending categories (SveltePlot donut / area chart)

---

### Phase 8 — Import Templates

Pre-bundled TOML templates shipped with the binary via `tauri::path::resource_dir`:

| File | Covers |
|---|---|
| `templates/form_26as_tds.toml` | Form 26AS TDS (see blueprint §4.1) |
| `templates/hdfc_savings.toml` | HDFC Bank savings CSV |
| `templates/axis_bank.toml` | Axis Bank statement CSV |
| `templates/sbi_statement.toml` | SBI bank statement CSV |
| `templates/generic_csv.toml` | Fallback: auto-detect date/amount columns |

---

### Phase 9 — Security Hardening

#### `tauri.conf.json`
- `productName` and `title`: `"Personal"`
- `csp`: `"default-src 'self'; style-src 'self' 'unsafe-inline'"`

#### `capabilities/default.json`
- Allow: `dialog:open`, `dialog:save` (file picker only)
- Frontend never touches the filesystem directly

#### DB lifecycle
- Connection opened only after `unlock()` succeeds (key retrieved from keyring)
- `PRAGMA rekey` executed on password change
- `PRAGMA journal_mode = WAL` for crash-safe writes

---

## Testing Strategy (mandatory)

### Unit Tests (Rust)

Every module contains `#[cfg(test)]` tests.

| Module | Coverage |
|---|---|
| `template/` | TOML parse, regex extraction, multi-leg postings |
| `parser/excel.rs` | Valid row -> Valid; corrupt cell -> Invalid (never panic) |
| `parser/csv_parser.rs` | BOM, varying delimiters, empty rows, bad encoding |
| `classifier.rs` | 0/1/N training samples; prediction stability |
| `commands/transactions.rs` | SUM != 0 -> Err; SUM = 0 -> Ok |
| `db.rs` | Migration idempotency (run twice -> no error) |

```bash
cd src-tauri && cargo test
```

### Fuzz Testing (cargo-fuzz — recommended)

> `cargo-fuzz` requires nightly Rust and runs on Linux/macOS.
> Use WSL2 on Windows or a Linux GitHub Actions runner for CI.
> Alternative: `tauri-fuzz` (CrabNebula) for Tauri-aware IPC boundary testing.

Fuzz targets:

| Target | Input | Risk |
|---|---|---|
| `fuzz_parse_excel` | Arbitrary bytes as .xlsx | Panic / OOM in calamine |
| `fuzz_parse_csv` | Arbitrary bytes as .csv | Malformed UTF-8, delimiter confusion |
| `fuzz_apply_template` | Random TOML + row data | Regex catastrophic backtracking |
| `fuzz_commit_transaction` | Random postings Vec | Balance invariant bypass |

Setup:

```bash
rustup install nightly
cargo install cargo-fuzz

cd src-tauri
cargo fuzz init
cargo fuzz add fuzz_parse_excel
cargo fuzz add fuzz_parse_csv
cargo fuzz add fuzz_apply_template
cargo fuzz add fuzz_commit_transaction

# Run in WSL2 or Linux CI
cargo +nightly fuzz run fuzz_parse_excel -- -max_total_time=300
```

Corpus: place at least one valid sample file per target in
`src-tauri/fuzz/corpus/<target>/`.

### Frontend Tests (Vitest)

```bash
pnpm add -D vitest @testing-library/svelte
```

| Test | Asserts |
|---|---|
| Triage grid: Valid row | Green highlight, no error text |
| Triage grid: Invalid row | Red highlight, `error_reason` displayed |
| Manual entry form | Submit disabled when SUM != 0 |
| Balance card | Correct number formatting for INR (paise -> rupees) |

```bash
pnpm test
```

### Manual Verification Checklist

- [ ] First launch -> onboarding screens; main UI blocked until complete
- [ ] Wrong master password -> `unlock()` returns false, DB stays locked
- [ ] Create accounts of all 5 types; verify balances start at 0
- [ ] Balanced manual transaction -> saves
- [ ] Unbalanced manual transaction -> Rust rejects with clear error
- [ ] Import happy path: valid HDFC CSV + template -> all rows green -> commit -> ledger updated
- [ ] Import unhappy path: corrupt file -> red rows -> edit inline -> commit
- [ ] `data.db` is unreadable as plaintext (hex editor shows ciphertext)
- [ ] Window < 640px -> sidebar collapses to bottom tabs

---

## Complete File Map

```
personal/
├── Plan.md                          <- this file
├── blueprint.md
├── package.json                     [MODIFY] add vitest, svelteplot
├── pnpm-workspace.yaml
├── svelte.config.js
├── vite.config.js
│
├── src/
│   ├── app.css                      [NEW] design tokens
│   ├── app.html
│   └── routes/
│       ├── +layout.svelte           [MODIFY] sidebar shell
│       ├── +layout.ts
│       ├── +page.svelte             [MODIFY] dashboard
│       ├── onboarding/
│       │   └── +page.svelte         [NEW]
│       ├── accounts/
│       │   └── +page.svelte         [NEW]
│       ├── transactions/
│       │   └── +page.svelte         [NEW]
│       ├── import/
│       │   └── +page.svelte         [NEW] triage grid
│       ├── analytics/
│       │   └── +page.svelte         [NEW]
│       └── settings/
│           └── +page.svelte         [NEW]
│
└── src-tauri/
    ├── Cargo.toml                   [MODIFY] add all crates
    ├── tauri.conf.json              [MODIFY] rename, harden CSP
    ├── capabilities/
    │   └── default.json             [MODIFY] restrict to dialog only
    ├── migrations/
    │   └── 0001_initial.sql         [NEW]
    ├── templates/                   [NEW dir]
    │   ├── form_26as_tds.toml
    │   ├── hdfc_savings.toml
    │   ├── axis_bank.toml
    │   ├── sbi_statement.toml
    │   └── generic_csv.toml
    ├── fuzz/                        [NEW dir]
    │   ├── Cargo.toml
    │   └── fuzz_targets/
    │       ├── fuzz_parse_excel.rs
    │       ├── fuzz_parse_csv.rs
    │       ├── fuzz_apply_template.rs
    │       └── fuzz_commit_transaction.rs
    └── src/
        ├── lib.rs                   [MODIFY] bootstrap DB + state
        ├── main.rs
        ├── error.rs                 [NEW]
        ├── models.rs                [NEW]
        ├── db.rs                    [NEW]
        ├── classifier.rs            [NEW]
        ├── commands/
        │   ├── mod.rs               [NEW]
        │   ├── accounts.rs          [NEW]
        │   ├── transactions.rs      [NEW]
        │   ├── import.rs            [NEW]
        │   └── security.rs          [NEW]
        ├── parser/
        │   ├── mod.rs               [NEW]
        │   ├── excel.rs             [NEW]
        │   └── csv_parser.rs        [NEW]
        └── template/
            ├── mod.rs               [NEW]
            └── types.rs             [NEW]
```

---

## Implementation Order

| # | Phase | Deliverable | Effort |
|---|---|---|---|
| 1 | Dependencies & DB | Encrypted DB opens on launch | Medium |
| 2 | Rust IPC commands | All backend commands wired | High |
| 3 | Design system & layout | Nav shell, tokens, dark mode | Medium |
| 4 | Onboarding | First-run wizard | Low |
| 5 | Core pages | Accounts, Transactions, Dashboard | High |
| 6 | Import / Triage Grid | Full parse -> review -> commit | High |
| 7 | Analytics charts | SveltePlot visualisations | Medium |
| 8 | Import templates | 5 pre-bundled TOML templates | Low |
| 9 | Security hardening | CSP, capabilities, WAL | Medium |
| 10 | Tests | Unit + fuzz + E2E coverage | High |

---

## Rejected Alternatives

| Technology | Reason |
|---|---|
| Tailwind CSS | Use plain CSS + fluid units + oat.ink |
| npm / yarn / bun | Only pnpm permitted |
| TigerBeetle / hledger | Dual-DB sync causes race conditions |
| Account Aggregator APIs | Cloud dependency; binary bloat |
| Column-level encryption | Metadata leakage; full-DB encryption mandated |
| `sqlx` with sqlite feature | Cannot do full-page encryption without custom build |
| FrankenSQLite | Requires nightly Rust; experimental in 2026 |
