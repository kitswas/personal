# ADR 0006 — Integration Testing: Rust and Accesskit

- **Date:** 2026-07-08
- **Status:** Accepted
- **Note:** Originally "Playwright via tauri-driver" before architecture pivot to pure Rust `egui`.

## Context

Prior to the pure Rust pivot, we considered using Playwright and `tauri-driver`.
However, an `egui` native desktop application does not run in a webview, meaning Web technologies (Playwright/WebDriver) are incompatible.

Options considered for native GUI testing:

| Option                                 | Drives real binary              | Cross-platform | Feedback            |
| -------------------------------------- | ------------------------------- | -------------- | ------------------- |
| `accesskit` validation                 | Yes — Integration               | Yes            | Validates UI tree   |
| Pure State Machine Unit Tests          | No — Rust backend only          | Yes            | Fast, deterministic |
| Screen-scraping / OCR tests            | Yes                             | No             | Flaky               |

## Decision

We adopt a two-layered testing approach:

1. **Pure State Machine Unit Tests:**
Because the application strictly follows Unidirectional Data Flow, the core logic is entirely independent of `egui`. We write unit tests that feed `Message` enumerations into the `apply_message` function and assert that the resulting `AppState` matches expectations.

2. **Accesskit Validation:**
For integration testing, we use `egui`'s built-in `accesskit` integration to programmatically query the semantic accessibility tree. This allows us to ensure UI elements are rendered properly without needing a flaky webdriver or OCR.

## Consequences

- **Good:** Fast, deterministic testing since state is completely decoupled from the view.
- **Good:** Eliminates the heavyweight Node.js / Playwright / Webdriver dependencies.
- **Good:** Validates the app's accessibility for screen readers as a free byproduct.
- **Trade-off:** We lose visual regression testing out of the box (requires setting up snapshot rendering tests if desired later).
