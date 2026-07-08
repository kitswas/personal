# ADR 0005 — Single `Arc<Mutex<Connection>>` over a Connection Pool

- **Date:** 2026-07-08
- **Status:** Accepted

## Context

`rusqlite` is a synchronous API wrapping the C SQLite library. Tauri 2 runs
command handlers on Tokio's async thread pool. A connection management strategy
is required.

Options considered:

| Strategy | Concurrency | Complexity | Risk |
|---|---|---|---|
| `r2d2` / `deadpool` connection pool | Multiple concurrent readers | Medium | Pool exhaustion; WAL reader limits |
| `sqlx` async pool (not usable — see ADR 0001) | High concurrency | Low API surface | Incompatible with SQLCipher |
| Single `Arc<Mutex<Connection>>` + `spawn_blocking` | Serialised | Low | None |

**Why a pool is unnecessary here:**

SQLite in WAL mode supports multiple concurrent *readers* but only one
concurrent *writer*. For a single-user desktop application, all meaningful
operations involve writes (inserting transactions, updating settings). A
connection pool would provide no real throughput benefit — writes would still
serialise at the SQLite level.

More importantly, SQLCipher's `PRAGMA key` is per-connection. A pool of N
connections would require N separate unlock calls and N separate key validations,
complicating the unlock lifecycle and creating a window where some pooled
connections are unlocked and others are not.

**Why `spawn_blocking`:**

`rusqlite` calls block the calling thread. Calling them directly in an `async`
function would stall the Tokio executor. `tokio::task::spawn_blocking` offloads
the blocking call to a dedicated thread pool, keeping the async runtime
responsive.

## Decision

Maintain a single `Arc<Mutex<Connection>>` stored in `tauri::State`.

```rust
pub type DbConn = Arc<Mutex<rusqlite::Connection>>;
```

Every Tauri command that accesses the DB:

1. Clones the `Arc` (cheap reference count increment)
2. Calls `tokio::task::spawn_blocking(move || { let conn = arc.lock()?; ... })`
3. Holds the `Mutex` guard only for the duration of the SQL operation
4. Releases the guard before returning

The `Mutex` is a `std::sync::Mutex` (not `tokio::sync::Mutex`) because
`rusqlite::Connection` is `!Send` and the lock is always acquired and released
on the same `spawn_blocking` thread.

## Consequences

- **Good:** Simple lifecycle. One connection, one unlock call, one PRAGMA key.
- **Good:** No pool exhaustion, no pool configuration tuning.
- **Good:** All DB access is serialised — no possibility of concurrent write
  corruption.
- **Good:** Consistent with WAL mode's write-serialisation guarantees.
- **Trade-off:** Read queries also serialise behind the Mutex. For a
  single-user desktop app this is imperceptible (SQLite read latency is
  microseconds for typical ledger query sizes).
- **Future:** If profiling reveals read contention (e.g., analytics queries
  blocking import commits), a dedicated read-only connection can be added as
  a second `tauri::State` entry without changing the command API.
