# ADR 0006 — E2E Testing: Playwright via `tauri-driver`

- **Date:** 2026-07-08
- **Status:** Accepted

## Context

Tauri 2 does not expose Chrome DevTools Protocol (CDP) on all platforms.
On Windows it uses WebView2 (which does expose CDP); on macOS/Linux it uses
WebKit (which does not). This complicates E2E automation.

Options considered:

| Option                                 | Drives real binary              | Cross-platform | API preference            |
| -------------------------------------- | ------------------------------- | -------------- | ------------------------- |
| WebdriverIO + `@wdio/tauri-service`    | Yes — official Tauri support    | Yes            | Unfamiliar                |
| Playwright + `tauri-driver`            | Yes — WebDriver bridge          | Yes            | **Preferred** (see below) |
| Playwright against a mocked dev server | No — Rust backend not exercised | Yes            | Preferred                 |
| `tauri-fuzz` (CrabNebula)              | Partial — IPC boundary only     | Linux/macOS    | For fuzz layer, not E2E   |

**Why not WebdriverIO:**
WebdriverIO is the officially recommended tool for Tauri E2E testing. However,
this project's conventions (see `../job-autofill`) use Playwright for
integration tests: `playwright.config.ts`, `fixtures.ts`, `test.extend<>`,
`data-testid` selectors, `trace: "on-first-retry"`. Switching to WebdriverIO
would create inconsistency across the developer's projects and require learning
a new API surface.

**Why not Playwright against a mocked dev server:**
Mocking the IPC layer excludes the Rust backend from test coverage. The highest-
risk logic (balance invariant, onboarding atomicity, SQLCipher unlock) lives in
Rust and would be untested by mocked E2E tests.

**`tauri-driver`:**
`tauri-driver` is an official Tauri project that implements a WebDriver-
compatible server. It launches the compiled Tauri binary and proxies WebDriver
commands to the native webview, regardless of the underlying engine. Playwright
connects to it as if it were a standard WebDriver endpoint.

```
Playwright test runner
    └─ WebDriver protocol ─► tauri-driver ─► Tauri .exe
                                                 └─ Rust backend + encrypted DB
```

This gives true end-to-end coverage: the test drives the UI of the real binary,
which calls real Rust IPC commands, which read and write the real encrypted DB.

## Decision

Use `@playwright/test` with `tauri-driver` as the WebDriver backend.

- Tests live in `e2e/` following the same conventions as `../job-autofill`:
  `fixtures.ts` for app lifecycle, `data-testid` selectors, split
  `test:e2e:prepare` / `test:e2e:run` scripts.
- `workers: 1` because a single Tauri process is launched per test run.
- Each spec file resets relevant DB state at the start of its tests to ensure
  isolation (via a dedicated Tauri command `reset_test_db` gated behind
  `#[cfg(test)]` or a `--test-mode` flag).
- The E2E layer is the last to run in CI: unit → fuzz → component → E2E.

## Consequences

- **Good:** Playwright API consistency with the rest of the developer's projects.
- **Good:** Tests exercise the full stack — UI, IPC, Rust backend, encrypted DB.
- **Good:** `tauri-driver` is cross-platform; tests can run on Windows CI
  (GitHub-hosted `windows-latest` runner).
- **Trade-off:** E2E tests require a compiled binary (`cargo tauri build --debug`),
  making the prepare step slow (~minutes). This is expected and acceptable for
  the final CI gate.
- **Trade-off:** `tauri-driver` is not as battle-tested as WebdriverIO's Tauri
  service. If it proves unreliable in CI, the fallback is WebdriverIO (same
  test scenarios, different runner API).
- **Constraint:** Each E2E test suite must be deterministic. Tests must not
  depend on the order of other tests. DB state must be explicitly seeded and
  torn down per spec.
