# ADR 0002 — Onboarding Guard: `is_onboarding_done()` over `is_first_run()`

- **Date:** 2026-07-08
- **Status:** Accepted

## Context

The application must show a setup wizard on first launch and then never show
it again once setup is complete. Two strategies were considered:

**`is_first_run()`** — a boolean that flips to `false` the moment the app
first launches. The flag is typically written at startup (or derived from the
absence of a DB file).

**`is_onboarding_done()`** — a boolean that becomes `true` only when the user
explicitly completes the final step of the wizard.

The failure mode of `is_first_run()` is:

```
Launch -> is_first_run() = true -> show wizard
User completes Screen 2 of 4 -> force-quits
Launch -> is_first_run() = false -> SKIP wizard -> broken state
```

The master password would be set but default accounts would not exist and
`default_commodity` would be unwritten. The app would load a half-initialised
DB and silently produce incorrect behaviour.

## Decision

Use `is_onboarding_done()`. The implementation reads the `settings` table for
the key `onboarding_complete`.

```rust
// Returns Ok(false) if the key is absent OR value != "true"
pub fn is_onboarding_done(conn: &Connection) -> Result<bool, AppError> {
    let result: rusqlite::Result<String> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'onboarding_complete'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(v) => Ok(v == "true"),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
        Err(e) => Err(AppError::Db(e)),
    }
}
```

The key `onboarding_complete = true` is written **only** in the final step of
the wizard, inside the **same SQLite transaction** that writes:

- `default_commodity`
- Seed account rows (if selected)

If the app is force-quit between any earlier step and the final commit, the
key is absent, `is_onboarding_done()` returns `false`, and onboarding restarts
from the beginning. No partial state is silently accepted.

## Consequences

- **Good:** Crash-resilient by construction. Partial onboarding is always
  resumed, never silently skipped.
- **Good:** The guard is re-checkable at any time (e.g., after a factory
  reset or manual DB deletion).
- **Good:** The condition is stored in the encrypted DB, so the key itself is
  protected at rest.
- **Trade-off:** The user must redo all wizard steps if the app crashes during
  onboarding. This is acceptable — the wizard takes under 60 seconds.
- **Constraint:** The `settings` table must exist before `is_onboarding_done()`
  is called. `db.rs` creates it unconditionally as part of migration 0001.
