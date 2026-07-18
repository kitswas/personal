# ADR 0005 — Dedicated Database Thread (Actor Pattern)

- **Date:** 2026-07-08
- **Status:** Accepted
- **Note:** Updated from `Arc<Mutex>` to the Actor Pattern on 2026-07-18.

## Context

`rusqlite` is a synchronous API wrapping the C SQLite library. Since our application relies heavily on Tokio for asynchronous background tasks, we must manage how these concurrent tasks interact with the synchronous database connection safely.

Options considered:

| Strategy                                           | Concurrency                 | Complexity      | Risk                               |
| -------------------------------------------------- | --------------------------- | --------------- | ---------------------------------- |
| `r2d2` / `deadpool` connection pool                | Multiple concurrent readers | Medium          | Pool exhaustion; key management    |
| Single `Arc<Mutex<Connection>>` + `spawn_blocking` | Serialized by Mutex         | Low             | Deadlocks; thread contention       |
| **Dedicated DB Thread (Actor Pattern)**            | Serialized by Channel       | Medium          | None                               |

**Why a pool is unnecessary here:**

SQLite in WAL mode supports multiple concurrent _readers_ but only one concurrent _writer_. For a single-user desktop application, all meaningful operations involve writes (inserting transactions, updating settings). A connection pool would provide no real throughput benefit because writes serialize at the SQLite file level anyway.
Furthermore, SQLCipher's `PRAGMA key` is per-connection. A pool of N connections would require N separate unlock calls, creating a window where some pooled connections are unlocked and others are not, significantly complicating the cryptographic lifecycle.

**Why `Arc<Mutex<Connection>>` was rejected:**

Originally, we proposed sharing the connection via an `Arc<Mutex>`. However, this approach introduces lock contention, risks deadlocking (if a guard is accidentally held across an `await` point), and litters the codebase with repetitive `tokio::task::spawn_blocking` boilerplate in every command.

## Decision

We will use the **Actor Pattern** by dedicating a single OS thread to own the database connection exclusively.

1. **The DB Actor**: At startup, we spawn a single dedicated thread. This thread opens the database, decrypts it via `PRAGMA key`, and enters a continuous loop listening to a `crossbeam_channel::Receiver<DbRequest>` (or `std::sync::mpsc::Receiver`).
2. **The Messages**: We define a `DbRequest` enum representing all possible database operations. If the caller expects a return value, the enum variant contains a `tokio::sync::oneshot::Sender` to transmit the result back.
3. **The Workflow**: Background tasks never touch `rusqlite` directly. They construct a `DbRequest` and send it to the actor thread, awaiting the response.

```rust
pub enum DbRequest {
    GetRunningBalances {
        reply_to: oneshot::Sender<Result<f64, AppError>>,
    },
    CommitTransaction {
        transaction: TransactionData,
        reply_to: oneshot::Sender<Result<(), AppError>>,
    },
}
```

## Consequences

- **Good:** Zero locking overhead. There is no `Mutex`, eliminating lock contention and deadlocks entirely.
- **Good:** Perfect fit for SQLite. A single dedicated thread perfectly models SQLite's strict write-serialization constraint.
- **Good:** Clean encryption lifecycle. Because only one thread ever opens the connection, we handle the `PRAGMA key` decryption exactly once, in a highly isolated scope.
- **Good:** Enforces clean architectural boundaries. SQL logic cannot leak into UI commands; it remains strictly inside the DB actor.
- **Trade-off:** Requires slightly more boilerplate to define the request and response channels (`DbRequest`), but this pays off in long-term stability and predictability.
