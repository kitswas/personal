# AGENTS.md

Cross-tool agent instructions for any AI coding assistant working on this repository. Read this entirely before writing any code.

## 1. Single Source of Truth (Absolute Rule)

No piece of state lives in two places. If a constant, configuration, or state field exists in the Rust backend, do not duplicate it in the TypeScript frontend (or vice versa). Expose it via Tauri IPC, generate bindings, or pass it dynamically. Adding a duplicate state field breeds drift bugs and will be automatically rejected.

## 2. Explicit Decisions & Zero Ambiguity

Think deeply before you write code. All architectural and logic decisions must be explicitly reasoned out. If a requirement is ambiguous, state your assumption clearly or ask for clarification before proceeding. Do not guess.

## 3. Strict Architectural Boundaries

Respect the Tauri paradigm.

* **Frontend (`src/`):** Strictly for UI, state management, and user interaction. Written in TypeScript. Must not contain OS-level logic or hardcoded system paths.
* **Backend (`src-tauri/`):** Strictly for system/OS access, file system operations, and heavy computation. Written in Rust.
* **Communication:** Cross-boundary communication happens exclusively through Tauri IPC commands and events. Shared DTOs/types must be synchronized (preferably via generated bindings).

## 4. Total Correctness & Functional Patterns

Design the codebase to be low-maintenance and highly predictable.

* **Functional Paradigm:** Use functional programming patterns throughout the codebase (pure functions, immutable state, map/filter/reduce pipelines).
* **Total Correctness:** Prove total program correctness by ensuring two conditions are met: **termination** (no infinite loops or deadlocks) and **partial correctness** (producing the right output for given inputs).
* **Deterministic Convergence:** Ensure all conditional branches and state machines converge correctly and predictably.

## 5. Crash-Resilience & Atomic Operations

Design for failure. Assume the user will force-quit the application or the machine will lose power in the middle of a critical operation.

* Use **atomic operations** wherever possible (e.g., write to a temporary file and atomically rename it, or use strict database transactions).
* Never leave local storage, configuration files, or database rows in a corrupted, half-written state.

## 6. Zero-Defect Safety

* **No Unhandled Exceptions:** Exhaustively handle all errors. In Rust, propagate errors via `Result` and exhaustively match `Option`. Do not leave `unwrap()` or `expect()` in production paths.
* **No Undefined Behavior or Data Races:** Rely on Rust's borrow checker and type system. Do not use `unsafe` code unless absolutely mathematically necessary.

## 7. Git Workflow & Atomic Commits

* **Atomic Commits:** Every commit must be a single, self-contained logical unit (one feature, one bug fix, or one refactor).
* **Independently Revertible:** Do not mix unrelated changes. Every single commit in the history must be independently revertible without breaking the application state.
* **Test Per Commit:** Run the full relevant test suite for *every* commit. The commit history must remain entirely green and compilable at every step.

## 8. Tooling and Environment Baseline

* **Frontend:** Use `pnpm` exclusively (never `npm` or `yarn`). Run strictly typed TypeScript.
* **Backend:** Use standard Cargo tooling. Treat `cargo clippy` warnings as errors.
* **Validation:** Before finishing a task, ensure both environments compile and validate successfully.

## 9. Read Before Write

Inspect existing module structures, trait implementations, and adjacent tests before modifying anything. Do not invent speculative abstractions, and do not add configuration keys or dependencies without a concrete, immediate use case.

## 10. Directory Map & Restricted Zones

* `src/` — Frontend application code (React/Vue/Svelte, etc.).
* `src-tauri/src/` — Rust backend code and IPC command handlers.
* `docs/arch/` — Architecture Decision Records (ADRs).
* **Restricted:** Do not modify auto-generated IPC bindings, lockfiles (`pnpm-lock.yaml`, `Cargo.lock`), third-party vendored code, or this `AGENTS.md` file unless explicitly instructed.

## 11. Common Commands

Use these to validate your work before submitting changes:

* **Install dependencies:** `pnpm install`
* **Start dev server (Frontend + Rust):** `pnpm tauri dev`
* **Frontend validation:** `pnpm typecheck` && `pnpm lint`
* **Backend validation:** `cargo fmt --all -- --check` && `cargo clippy --all-targets -- -D warnings`
* **Run tests:** `cargo test` (Backend) / `pnpm test` (Frontend)
