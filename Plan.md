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
| DB | SQLite encrypted via SQLCipher | Full-page AES-256 encryption at rest |
| Encryption driver | `rusqlite` with `bundled-sqlcipher-vendored-openssl` | Self-contained build; no external OpenSSL install on Windows |
| Key storage | `keyring` crate (OS Keychain / Credential Manager) | Key never written to disk |
| Excel parsing | `calamine` | Handles .xls / .xlsx without COM |
| CSV parsing | `csv` crate | Zero-allocation reader |
| Template format | TOML | Literal strings eliminate regex-escape hell |
| E2E testing | Playwright (via `tauri-driver` + WebDriver) | True desktop automation against native webview |

---

## Correctness Principles

This codebase enforces **total program correctness**: every branch must
terminate and every reachable state must satisfy the program invariants.

### Functional Programming Style

- **Rust backend:** Pure functions preferred; side effects (DB, keyring, FS)
  are isolated to the command handlers and `db.rs`. No global mutable state
  outside of `tauri::State`. All mutation goes through a single `Mutex`-guarded
  connection handle.
- **Svelte frontend:** Derived state only via `$derived` / `$derived.by`.
  No implicit side-effect chains. Event handlers are pure transformations
  returning new state.
- **No panics in library code.** `unwrap()` and `expect()` are banned outside
  of `main`/`run` bootstrap. All fallible paths return `Result<T, AppError>`.
- **No `unsafe` blocks** except where required by the SQLCipher FFI (which is
  encapsulated inside `rusqlite` itself and not written by us).

### Total Correctness Guarantees

| Property | Mechanism |
|---|---|
| **Termination** | All recursive functions are bounded (no unbounded loops in parser; regex engine has a timeout via `regex::RegexBuilder::size_limit`). Naive Bayes uses a fixed feature vocabulary. |
| **Partial correctness** | Every IPC command returns `Result<T, AppError>`; the frontend maps `Err` variants to visible error UI — no silent failures. |
| **No unhandled exceptions** | TypeScript `strict` mode; `noUncheckedIndexedAccess`; all `Promise` chains end in `.catch()` or `try/catch`. |
| **No data races** | Single `Arc<Mutex<Connection>>` for the DB; `Mutex` is held only for the duration of each SQL statement, then released. All Tauri commands are `async` and run on Tokio's thread pool. |
| **No undefined behaviour** | No `unsafe` in application code. Clippy `#![deny(unsafe_code)]` on library crates. |
| **All branches covered** | Rust enums (e.g., `ParsedRow`) are exhaustively matched. TypeScript discriminated unions are exhaustively narrowed. |

### Onboarding Guard — `is_onboarding_done()`

> **Design note:** We use `is_onboarding_done()` rather than `is_first_run()`.
> `is_first_run()` is unidirectional — once it returns false, the guard is
> gone even if the user quit mid-onboarding. `is_onboarding_done()` reads the
> `settings` table for an explicit `onboarding_complete = true` key that is
> only written at the very last step of the wizard. This guarantees that
> partially-completed onboarding is always resumed correctly.

```
App launch
   |
   v
is_onboarding_done()? ──No──> /onboarding (multi-step wizard)
   |                               |
   Yes                      [Finish Setup]
   |                               |
   v                               v
/dashboard <──────────────────────'
```

---

## Architecture

```
+------------------------------------------------------------+
|                     User''s Machine                        |
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
    -- 'onboarding_complete' = 'true' written ONLY on wizard completion
    -- 'default_commodity'   = e.g. 'INR'
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

### Phase 1 — Dependencies & DB Foundation

**Goal:** Compilable Tauri app that opens an encrypted database on launch.

#### `src-tauri/Cargo.toml` — crates to add

```toml
[dependencies]
tauri               = { version = "2", features = ["dialog"] }
tauri-plugin-opener = "2"
tauri-plugin-dialog = "2"
serde               = { version = "1", features = ["derive"] }
serde_json          = "1"

# Full-page AES-256 encrypted SQLite — bundles SQLCipher + vendored OpenSSL
rusqlite = { version = "0.32", features = ["bundled-sqlcipher-vendored-openssl"] }

# Async runtime (Tauri 2 uses Tokio internally)
tokio = { version = "1", features = ["full"] }

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

[dev-dependencies]
tempfile = "3"   # isolated DB per test
```

#### New files

| Path | Purpose |
|---|---|
| `src-tauri/migrations/0001_initial.sql` | Full schema |
| `src-tauri/src/db.rs` | Open encrypted DB, run migrations, expose connection pool |
| `src-tauri/src/error.rs` | `AppError` typed error enum; `impl From` for all crate errors |

#### `src-tauri/src/lib.rs` changes

- Open `rusqlite::Connection` wrapped in `Arc<Mutex<Connection>>`
- Store as `tauri::State<DbConn>` where `DbConn = Arc<Mutex<Connection>>`
- Execute `PRAGMA key = '...'` immediately after open, before any query
- Run migration SQL idempotently (`CREATE TABLE IF NOT EXISTS`)
- Enable `PRAGMA journal_mode = WAL` for crash-safe writes

---

### Phase 2 — Rust Backend: IPC Commands

**Goal:** All backend logic exposed as typed, total Tauri commands.
Every command returns `Result<T, String>` (serialised `AppError`).
No command may panic. All resource acquisition is guarded by `?`.

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

All matches on `ParsedRow` are exhaustive; the compiler enforces this.

#### `commands/security.rs`

| Command | Signature | Notes |
|---|---|---|
| `is_onboarding_done` | `() -> Result<bool, String>` | Reads `settings.onboarding_complete`; returns `false` if key absent OR value != `"true"` |
| `setup_master_password` | `(password, default_currency) -> Result<(), String>` | Sets keyring key, writes `onboarding_complete=true` atomically with `default_commodity` in a single DB transaction |
| `unlock` | `(password) -> Result<bool, String>` | Retrieves key from keyring, validates against DB via test query |
| `change_master_password` | `(old, new) -> Result<(), String>` | Validates old, calls `PRAGMA rekey`, updates keyring |

> **Onboarding atomicity:** `setup_master_password` writes
> `onboarding_complete = true` in the SAME SQLite transaction as the
> seed accounts and default commodity. If the app crashes between
> writing the password and completing setup, `onboarding_complete`
> remains absent and onboarding resumes from the beginning.

#### `commands/accounts.rs`

| Command | Signature |
|---|---|
| `list_accounts` | `() -> Result<Vec<Account>, String>` |
| `create_account` | `(name, type, commodity) -> Result<Account, String>` |
| `delete_account` | `(id) -> Result<(), String>` |
| `get_account_balance` | `(account_id) -> Result<i64, String>` |

#### `commands/transactions.rs`

| Command | Signature |
|---|---|
| `list_transactions` | `(limit, offset) -> Result<Vec<TransactionWithPostings>, String>` |
| `commit_transaction` | `(date, payee, notes, postings) -> Result<Transaction, String>` — Err if SUM != 0 |
| `get_running_balances` | `(account_id) -> Result<Vec<BalanceEntry>, String>` |
| `delete_transaction` | `(id) -> Result<(), String>` |

#### `commands/import.rs`

| Command | Signature |
|---|---|
| `list_templates` | `() -> Result<Vec<TemplateMeta>, String>` |
| `parse_statement` | `(file_path, template_name) -> Result<Vec<ParsedRow>, String>` — never panics; all row errors become `ParsedRow::Invalid` |
| `commit_import_batch` | `(rows: Vec<ValidRow>) -> Result<BatchResult, String>` |

#### Auto-categorisation (`classifier.rs`)

1. **Exact rule match** — TOML-defined regex patterns tried first (with `size_limit` to prevent catastrophic backtracking)
2. **Naive Bayes fallback** — trains on `payee` text of committed transactions; fixed vocabulary cap
3. Confidence below threshold -> row surfaces as `Invalid` for human review
4. All operations are pure functions over immutable inputs

---

### Phase 3 — Frontend: Design System & Layout Shell

**Goal:** Working shell with 3-pane layout, collapsible side panes, theme switcher, fully fluid.

#### Layout specification

```
Wide (> 1024px) — both panes open         Nav collapsed        Detail collapsed
┌──────┬──────────┬──────┐  ┌──┬──────────────┬──────┐  ┌──────┬───────────────┬─┐
│ Nav  │  List    │Detail│  │≤ │ List (wider) │Detail│  │ Nav  │  List (wider) │>│
│ [>]  │          │  [<] │  │  │              │  [<] │  │ [>]  │               │ │
│  30% │   40%    │  30% │  │  │      ~65%    │  35% │  │  30% │      ~70%     │ │
└──────┴──────────┴──────┘  └──┴──────────────┴──────┘  └──────┴───────────────┴─┘

     Both collapsed: List takes full width (100%)
     ┌──┬─────────────────────────┬──┐
     │≤ │         List (full)     │> │
     └──┴─────────────────────────┴──┘
```

**Pane responsibilities:**

| Pane | Width (wide) | Content |
|---|---|---|
| **Left — Nav** | `~30%` | App name, tagline, nav links, account tree with live balances; collapse toggle `[>]` on its right edge |
| **Center — List** | `~40%` (expands to fill collapsed pane space) | Primary content: transaction list, import triage grid, account ledger |
| **Right — Detail** | `~30%` | Context panel: selected transaction postings, account summary card, quick-add form; collapse toggle `[<]` on its left edge |

**Pane collapse rules:**

- Each side pane collapses independently via its edge toggle button
- Collapsed panes render as a narrow icon-strip (`~2.5rem`) — not zero-width — so
  the toggle button remains reachable and the grid never fully breaks
- The center list column takes the freed `fr` share automatically (CSS Grid)
- Collapse state is stored in `localStorage` as `{ navCollapsed: bool, detailCollapsed: bool }`
- On narrow viewports (`< 640px`) collapse toggles are hidden; the layout is
  always single-pane with a bottom tab bar

#### Theme system

Three modes: **System** (default) / **Light** / **Dark**.

- Preference stored in `localStorage` as `theme: 'system' | 'light' | 'dark'`
- `localStorage` is used (not the encrypted DB) so the theme applies
  immediately on startup, before the DB is unlocked — the onboarding screen
  and unlock screen must also respect the theme
- A `data-theme` attribute is set on `<html>` by a small inline script in
  `app.html` (before the JS bundle loads) to prevent flash of wrong theme
- Theme toggle lives in the nav pane header, alongside the app name

```html
<!-- app.html — inline script to set data-theme before paint -->
<script>
  const t = localStorage.getItem('theme') ?? 'system';
  if (t === 'dark' || (t === 'system' && matchMedia('(prefers-color-scheme: dark)').matches)) {
    document.documentElement.setAttribute('data-theme', 'dark');
  } else {
    document.documentElement.setAttribute('data-theme', 'light');
  }
</script>
```

#### `src/app.css` — design tokens (no hardcoded px for layout)

```css
/* Dark palette (default; also applied by [data-theme="dark"]) */
:root,
[data-theme="dark"] {
  --color-bg:        hsl(220, 13%,  9%);
  --color-surface:   hsl(220, 13%, 13%);
  --color-border:    hsl(220, 13%, 20%);
  --color-text:      hsl(220, 14%, 90%);
  --color-muted:     hsl(220, 10%, 55%);
  --color-accent:    hsl(252, 80%, 68%);
  --color-positive:  hsl(150, 65%, 48%);
  --color-negative:  hsl(  0, 72%, 58%);
  --color-warning:   hsl( 38, 92%, 58%);
}

/* Light palette */
[data-theme="light"] {
  --color-bg:        hsl(220, 20%, 97%);
  --color-surface:   hsl(220, 20%, 100%);
  --color-border:    hsl(220, 13%, 82%);
  --color-text:      hsl(220, 14%, 12%);
  --color-muted:     hsl(220, 10%, 48%);
  --color-accent:    hsl(252, 70%, 55%);
  --color-positive:  hsl(150, 55%, 36%);
  --color-negative:  hsl(  0, 65%, 46%);
  --color-warning:   hsl( 38, 85%, 42%);
}

/* 3-pane grid — fluid fractions, no px */
:root {
  --col-nav:         3fr;    /* ~30% */
  --col-list:        4fr;    /* ~40% */
  --col-detail:      3fr;    /* ~30% */
  --col-collapsed:   2.5rem; /* icon-strip width when pane is collapsed */
}
```

#### `src/routes/+layout.svelte`

- Calls `is_onboarding_done()` on mount; redirects to `/onboarding` if false
- Reads `localStorage` for `{ navCollapsed, detailCollapsed }` and `theme` on mount
- CSS Grid with named areas: `"nav list detail"` — column widths derived from
  collapse state via `$derived` rune:

  ```
  navCollapsed=false, detailCollapsed=false  →  3fr 4fr 3fr
  navCollapsed=true,  detailCollapsed=false  →  2.5rem 4fr 3fr
  navCollapsed=false, detailCollapsed=true   →  3fr 4fr 2.5rem
  navCollapsed=true,  detailCollapsed=true   →  2.5rem 1fr 2.5rem
  ```

- **Wide (> 1024px):** grid with computed columns; both toggles visible
- **Medium (640–1024px):** nav + list only; detail as overlay on selection
- **Narrow (< 640px):** single column; bottom tab bar; collapse toggles hidden
- Theme toggle (System / Light / Dark icon cycle) in nav header; writes to `localStorage` and updates `data-theme` on `<html>`
- Navigation links: Dashboard / Accounts / Transactions / Import / Analytics / Settings
- Account tree with live balances in nav pane (below nav links)

---

### Phase 4 — Onboarding Flow

First-run flow shown before the main UI when `is_onboarding_done()` is false.
The wizard maintains step progress in local Svelte state only — not persisted
until the final step to preserve atomicity.

```
Screen 1: Welcome
  "Personal Finance should be private."
  [Get Started]

Screen 2: Set Master Password
  Password + Confirm fields, strength indicator
  [Next]  <-- only enabled when passwords match

Screen 3: Default Currency
  Searchable currency picker (INR default)
  [Next]

Screen 4: Seed Accounts (optional)
  "Add default accounts: Checking, Savings, Cash, Credit Card"
  [Finish Setup]  <-- calls setup_master_password(); writes onboarding_complete=true
```

`src/routes/onboarding/+page.svelte` — multi-step wizard component.
`+layout.svelte` redirects to `/onboarding` if `is_onboarding_done()` returns false.
All wizard state is local `$state`; the IPC call is made only on the final screen.

---

### Phase 5 — Core Pages

#### Dashboard (`src/routes/+page.svelte`)

- Net worth card (sum of all asset accounts)
- 30-day income vs expense summary bar
- Last 10 transactions list
- Quick-add transaction button

#### Accounts (`src/routes/accounts/+page.svelte`)

- Accounts grouped by type with running balances
- Add account modal (name, type, commodity)
- Delete with confirmation if account has associated postings

#### Transactions (`src/routes/transactions/+page.svelte`)

- Paginated ledger: date / payee / amount columns
- Expand row to see postings
- Manual entry form:
  - Client side: `$derived` balance sum; Submit disabled when != 0
  - Server side: Rust enforces SUM = 0 before writing; `Err` surfaced as toast

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
- Batch commit calls `commit_import_batch()` — Rust validates each set

Data flow is a pure pipeline:

```
File bytes
  -> parse_statement()     [Rust: calamine/csv + TOML template]
  -> Vec<ParsedRow>        [IPC boundary: serialised JSON]
  -> triage UI state       [Svelte: pure $derived view]
  -> user edits            [local state mutations only]
  -> commit_import_batch() [Rust: DB transaction]
```

---

### Phase 7 — Analytics

`src/routes/analytics/+page.svelte`

- Monthly income vs expense (SveltePlot bar chart)
- Running balance per account over time (SveltePlot line chart)
- Top spending categories (SveltePlot area / donut chart)

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

- Connection opened only after `unlock()` succeeds
- `PRAGMA rekey` executed on password change
- `PRAGMA journal_mode = WAL` for crash-safe writes

---

## Testing Strategy (mandatory)

Tests are written in this order: unit -> fuzz -> component -> E2E.
A CI run must pass all layers before merge.

---

### Layer 1 — Unit Tests (Rust)

Every module has `#[cfg(test)]` tests. Tests use `tempfile::NamedTempFile`
for isolated in-memory or on-disk DBs so they never share state.

| Module | What is tested |
|---|---|
| `template/` | TOML parse -> struct; regex extraction; multi-leg postings |
| `parser/excel.rs` | Valid row -> `ParsedRow::Valid`; corrupt cell -> `ParsedRow::Invalid`; never panics |
| `parser/csv_parser.rs` | BOM, varying delimiters, empty rows, malformed UTF-8 |
| `classifier.rs` | 0 / 1 / N training samples; prediction stability; vocabulary cap |
| `commands/transactions.rs` | SUM != 0 -> `Err`; SUM = 0 -> `Ok`; atomicity on partial failure |
| `commands/security.rs` | `is_onboarding_done()` returns false when key absent; true only after `setup_master_password()` completes |
| `db.rs` | Migration idempotency (run twice, no error); WAL mode confirmed |

```bash
cd src-tauri && cargo test
```

---

### Layer 2 — Fuzz Testing (cargo-fuzz)

> `cargo-fuzz` requires nightly Rust and runs on Linux / macOS.
> Use WSL2 locally or a Linux GitHub Actions runner for CI.
> Alternative for Tauri-specific IPC boundary testing:
> `tauri-fuzz` (CrabNebula) — <https://github.com/crabnebula-dev/tauri-fuzz>

All four fuzz targets exercise the zero-trust parsing surfaces.
Fuzz targets are pure functions — they do not touch the DB.

| Target | Input | Risk to guard against |
|---|---|---|
| `fuzz_parse_excel` | Arbitrary bytes as .xlsx | Panic / OOM in calamine |
| `fuzz_parse_csv` | Arbitrary bytes as .csv | Malformed UTF-8, delimiter confusion |
| `fuzz_apply_template` | Random TOML + row data | Regex catastrophic backtracking |
| `fuzz_commit_transaction` | Random `Vec<PostingInput>` | Balance invariant bypass |

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

---

### Layer 3 — Component Tests (Vitest)

```bash
pnpm add -D vitest @testing-library/svelte @vitest/coverage-v8
```

| Test | Asserts |
|---|---|
| Triage grid: `ParsedRow::Valid` row | Green class applied; no error text rendered |
| Triage grid: `ParsedRow::Invalid` row | Red class applied; `error_reason` text present |
| Manual entry form | Submit button disabled when `$derived` SUM != 0 |
| Manual entry form | Submit button enabled when SUM = 0 |
| Balance card | Correct INR formatting (paise to rupees, locale-aware) |
| Onboarding: step 2 | Next button disabled until passwords match |
| Onboarding: step 4 | Calls `setup_master_password` IPC on Finish (mocked) |
| Layout guard | Redirects to `/onboarding` when `is_onboarding_done()` mock returns false |

All IPC calls are mocked via `@tauri-apps/api/mocks`.

```bash
pnpm test
```

---

### Layer 4 — E2E Tests (Playwright + tauri-driver)

**Reference:** See `../job-autofill/apps/extension/` for the project conventions
(fixtures.ts pattern, `test:e2e:prepare` -> `test:e2e:run` script split,
`data-testid` selectors, `trace: "on-first-retry"` reporter config).

#### Why tauri-driver for E2E

Standard Playwright targets the Chrome DevTools Protocol, which is only
available via WebView2 (Windows). `tauri-driver` exposes a WebDriver-compatible
endpoint over the Tauri app's native webview on all platforms, letting
Playwright drive the real binary — not a mocked web page.

```
Playwright test runner
    |
    v
tauri-driver (WebDriver server) <---> Tauri .exe / .app
    |                                   |
    v                                   v
webview (WebView2 / WebKit)        Rust backend + DB
```

#### Setup

```bash
# Install tauri-driver (cross-platform WebDriver for Tauri)
cargo install tauri-driver

# Install Playwright
pnpm add -D @playwright/test
npx playwright install chromium   # only chromium needed for WebView2 on Windows
```

#### File layout

```
e2e/
├── playwright.config.ts
├── fixtures.ts            <- extends base test with app launch/teardown
├── helpers/
│   └── db.ts              <- reset DB between tests via temp dir
├── onboarding.spec.ts
├── accounts.spec.ts
├── transactions.spec.ts
├── import.spec.ts
└── analytics.spec.ts
```

#### `playwright.config.ts`

```typescript
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,        // Tauri app is a single process; tests are sequential
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    trace: "on-first-retry",
  },
  // tauri-driver launches the built app and exposes a WebDriver endpoint
  webServer: {
    command: "cargo tauri build --debug && tauri-driver",
    port: 4444,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
```

#### `fixtures.ts`

```typescript
import { test as base } from "@playwright/test";
import { spawn, type ChildProcess } from "child_process";

// Extend with app process handle for clean teardown
export const test = base.extend<{ appProcess: ChildProcess }>({
  appProcess: async ({}, use) => {
    const proc = spawn("tauri-driver", [], { stdio: "pipe" });
    await use(proc);
    proc.kill();
  },
});

export const expect = base.expect;
```

#### E2E test scenarios (by spec file)

**`onboarding.spec.ts`**

- Fresh install: main UI is blocked; `/onboarding` is shown
- Completing onboarding writes `onboarding_complete = true`; redirect to `/dashboard`
- Closing mid-onboarding (simulated by restarting the app): onboarding resumes, not skipped

**`accounts.spec.ts`**

- Create one account of each type; verify all 5 appear in the sidebar
- Delete an account with no postings; verify it disappears
- Attempt to delete an account with existing postings; verify confirmation dialog

**`transactions.spec.ts`**

- Enter a balanced transaction; verify it appears in the ledger
- Attempt an unbalanced transaction; verify error toast
- Delete a transaction; verify it is removed from ledger

**`import.spec.ts`**

- Drop a valid HDFC CSV; select the `hdfc_savings` template; verify all rows parse as Valid
- Commit; verify transactions appear in the ledger
- Drop a corrupt file; verify Invalid rows appear in red; correct one inline; commit

**`analytics.spec.ts`**

- After committing test transactions; verify chart elements are rendered (`data-testid` selectors)

> See the **Tooling & Code Style** section above for the full `package.json` scripts reference.

---

### Manual Verification Checklist

- [ ] Fresh install -> onboarding screens appear; main UI blocked until complete
- [ ] Mid-onboarding quit -> re-launch resumes onboarding (NOT skipped to dashboard)
- [ ] Wrong master password -> `unlock()` returns false, DB stays locked
- [ ] Create accounts of all 5 types; verify balances start at 0
- [ ] Balanced manual transaction -> saves; appears in ledger
- [ ] Unbalanced manual transaction -> Rust rejects with clear error toast
- [ ] Import happy path: valid HDFC CSV + template -> all rows green -> commit -> ledger updated
- [ ] Import unhappy path: corrupt file -> red rows -> edit inline -> commit
- [ ] `data.db` is unreadable as plaintext (hex editor shows cipher text, not SQL keywords)
- [ ] Window < 640px -> sidebar collapses to bottom tabs

---

## Complete File Map

```
personal/
├── Plan.md                              <- this file
├── blueprint.md
├── package.json                         [MODIFY] add all dev tooling + test deps
├── pnpm-workspace.yaml                  [MODIFY] add trustPolicy + sharedWorkspaceLockfile
├── .prettierrc                          [NEW]
├── .prettierignore                      [NEW]
├── eslint.config.js                     [NEW]
├── .editorconfig                        [NEW]
├── .gitattributes                       [NEW]
├── typedoc.json                         [NEW]
├── typedoc.tsconfig.json                [NEW]
├── .dependency-cruiser.js               [NEW]
├── .husky/
│   ├── pre-commit                       [NEW] format:check
│   └── pre-push                         [NEW] lint + cargo checks
├── .github/
│   └── workflows/
│       ├── ci.yml                       [NEW] lint + Rust check + unit + component + E2E
│       ├── release.yml                  [NEW] cross-platform build & GitHub Release
│       └── docs.yml                     [NEW] generate & deploy docs to GitHub Pages
├── svelte.config.js
├── vite.config.js
├── tsconfig.json                        [MODIFY] add noUncheckedIndexedAccess
│
├── e2e/                                 [NEW dir]
│   ├── playwright.config.ts
│   ├── fixtures.ts
│   ├── helpers/
│   │   └── db.ts
│   ├── onboarding.spec.ts
│   ├── accounts.spec.ts
│   ├── transactions.spec.ts
│   ├── import.spec.ts
│   └── analytics.spec.ts
│
├── src/
│   ├── app.css                          [NEW] design tokens
│   ├── app.html
│   └── routes/
│       ├── +layout.svelte               [MODIFY] sidebar shell + onboarding guard
│       ├── +layout.ts
│       ├── +page.svelte                 [MODIFY] dashboard
│       ├── onboarding/
│       │   └── +page.svelte             [NEW] multi-step wizard
│       ├── accounts/
│       │   └── +page.svelte             [NEW]
│       ├── transactions/
│       │   └── +page.svelte             [NEW]
│       ├── import/
│       │   └── +page.svelte             [NEW] triage grid
│       ├── analytics/
│       │   └── +page.svelte             [NEW]
│       └── settings/
│           └── +page.svelte             [NEW]
│
└── src-tauri/
    ├── Cargo.toml                       [MODIFY] add all crates
    ├── tauri.conf.json                  [MODIFY] rename, harden CSP
    ├── capabilities/
    │   └── default.json                 [MODIFY] restrict to dialog only
    ├── migrations/
    │   └── 0001_initial.sql             [NEW]
    ├── templates/                       [NEW dir]
    │   ├── form_26as_tds.toml
    │   ├── hdfc_savings.toml
    │   ├── axis_bank.toml
    │   ├── sbi_statement.toml
    │   └── generic_csv.toml
    ├── fuzz/                            [NEW dir]
    │   ├── Cargo.toml
    │   └── fuzz_targets/
    │       ├── fuzz_parse_excel.rs
    │       ├── fuzz_parse_csv.rs
    │       ├── fuzz_apply_template.rs
    │       └── fuzz_commit_transaction.rs
    └── src/
        ├── lib.rs                       [MODIFY] bootstrap DB + state
        ├── main.rs
        ├── error.rs                     [NEW]
        ├── models.rs                    [NEW]
        ├── db.rs                        [NEW]
        ├── classifier.rs                [NEW]
        ├── commands/
        │   ├── mod.rs                   [NEW]
        │   ├── accounts.rs              [NEW]
        │   ├── transactions.rs          [NEW]
        │   ├── import.rs                [NEW]
        │   └── security.rs              [NEW]
        ├── parser/
        │   ├── mod.rs                   [NEW]
        │   ├── excel.rs                 [NEW]
        │   └── csv_parser.rs            [NEW]
        └── template/
            ├── mod.rs                   [NEW]
            └── types.rs                 [NEW]
```

---

## CI / CD

Three workflows, adapted from the `job-autofill` reference.
Key differences from the reference:

- Rust toolchain + `cargo` steps required on every job that touches the backend
- `tauri-driver` for E2E instead of a headless browser; requires `xvfb-run` on Linux
- Cross-platform release matrix (`windows-latest`, `macos-latest`, `ubuntu-latest`)
- Fuzz job is Linux-only (cargo-fuzz / libFuzzer constraint)

### `ci.yml` — runs on every PR and push to `main`

Jobs run in this order (each depends on the previous passing):

```
lint  ──────────────────────────────────────────────────────┐
                                                            ├──► (all pass)
rust-check  ──────────────────────────────────────────────┐ │
  (cargo fmt --check + cargo clippy -D warnings)          │ │
                                                          │ │
test-unit ────────────────────────────────────────────────┤ │
  (cargo test  +  pnpm test)                              │ │
                                                          │ │
test-e2e  ────────────────────────────────────────────────┘ │
  (pnpm test:e2e on ubuntu-latest via xvfb-run)             │
                                                            │
             ◄──────────────────────────────────────────────┘
             merge allowed
```

```yaml
# .github/workflows/ci.yml (outline)
name: CI
on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: pnpm/action-setup@v5
        with: { version: 10 }
      - uses: actions/setup-node@v6
        with: { node-version: 24, cache: pnpm }
      - run: pnpm install
      - run: pnpm lint
      - run: pnpm format:check

  rust-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy, rustfmt }
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: src-tauri }
      - run: cargo fmt --all -- --check
        working-directory: src-tauri
      - run: cargo clippy --all-targets -- -D warnings
        working-directory: src-tauri

  test-unit:
    runs-on: ubuntu-latest
    needs: [lint, rust-check]
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: src-tauri }
      - uses: pnpm/action-setup@v5
        with: { version: 10 }
      - uses: actions/setup-node@v6
        with: { node-version: 24, cache: pnpm }
      - run: cargo test
        working-directory: src-tauri
      - run: pnpm install && pnpm test

  test-e2e:
    runs-on: ubuntu-latest
    needs: [test-unit]
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: src-tauri }
      - uses: pnpm/action-setup@v5
        with: { version: 10 }
      - uses: actions/setup-node@v6
        with: { node-version: 24, cache: pnpm }
      - run: pnpm install
      - name: Install system deps (WebKit, Tauri)
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
            librsvg2-dev patchelf xvfb
      - name: Install tauri-driver
        run: cargo install tauri-driver
      - name: Install Playwright
        run: npx playwright install chromium --with-deps
      - name: Build debug binary
        run: pnpm test:e2e:prepare
      - name: Run E2E tests
        run: xvfb-run pnpm test:e2e:run
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report
          path: playwright-report/
          retention-days: 1
```

> **Note — fuzz tests in CI:** `cargo-fuzz` requires nightly and only runs on
> Linux. Fuzz runs are not part of the regular PR gate (they are slow). Instead,
> run them on a schedule (e.g. weekly) or manually. A separate `fuzz.yml`
> workflow can be added later for scheduled fuzzing on a Linux runner.

---

### `release.yml` — triggered manually via `workflow_dispatch`

Pattern mirrors `job-autofill/release.yml`: check-tag → lint → test → build.
Key difference: Tauri requires a **platform matrix** to produce native binaries.

```
check-release-tag
       |
  ┌────┴────┐
lint  rust-check
  └────┬────┘
  test-unit + test-e2e
       |
  build-matrix (3 platforms in parallel)
  ┌────────────────────────────────┐
  │ windows-latest  (.msi / .exe)  │
  │ macos-latest    (.dmg / .app)  │
  │ ubuntu-latest   (.deb / .AppImage) │
  └────────────────────────────────┘
       |
  create-github-release
  (uploads all platform artifacts)
```

Version is read from `src-tauri/tauri.conf.json` `.version`.
A git tag `vX.Y.Z` is created automatically; if the tag already exists the
workflow exits early (idempotent — mirrors the reference pattern).

Secrets required:

- `TAURI_SIGNING_PRIVATE_KEY` — app signing key (Tauri updater)
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

---

### `docs.yml` — runs on push to `main`

Direct port of `job-autofill/static.yml`:

```yaml
name: Generate docs and deploy to Pages
on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  generate-and-deploy:
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/checkout@v6
      - uses: pnpm/action-setup@v5
        with: { version: 10 }
      - uses: actions/setup-node@v6
        with: { node-version: 24, cache: pnpm }
      - run: pnpm install
      - name: Install Graphviz
        run: sudo apt-get install -y graphviz
      - name: Build documentation
        run: pnpm docs:all
      - uses: actions/upload-pages-artifact@v3
        with: { path: docs/html }
      - uses: actions/deploy-pages@v4
        id: deployment
```

---

## Implementation Order

| # | Phase | Deliverable | Effort |
|---|---|---|---|
| 0 | Tooling bootstrap | Prettier, ESLint, EditorConfig, Husky, `.gitattributes`, TypeDoc, dependency-cruiser, CI workflows | Low |
| 1 | Dependencies & DB | Encrypted DB opens on launch | Medium |
| 2 | Rust IPC commands | All backend commands wired + unit tests | High |
| 3 | Design system & layout | Nav shell, tokens, dark mode | Medium |
| 4 | Onboarding | `is_onboarding_done()` guard + multi-step wizard | Low |
| 5 | Core pages | Accounts, Transactions, Dashboard | High |
| 6 | Import / Triage Grid | Full parse -> review -> commit pipeline | High |
| 7 | Analytics charts | SveltePlot visualisations | Medium |
| 8 | Import templates | 5 pre-bundled TOML templates | Low |
| 9 | Security hardening | CSP, capabilities, WAL | Medium |
| 10 | Fuzz tests | 4 targets with corpus seeds | Medium |
| 11 | Component tests | Vitest + @testing-library/svelte | Medium |
| 12 | E2E tests | Playwright + tauri-driver; all spec files | High |

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
| FrankenSQLite | Requires nightly Rust; experimental |
| `is_first_run()` | Unidirectional flag; cannot detect mid-onboarding abort |
| WebdriverIO for E2E | Official Tauri recommendation, but Playwright API is preferred per project conventions; tauri-driver exposes WebDriver so Playwright can drive it |
