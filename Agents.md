# AGENTS.md

Cross-tool agent instructions for any AI coding assistant working on this repository. Read this entirely before writing any code.

## 1. Single Source of Truth (Absolute Rule)

No piece of state lives in two places. If a constant, configuration, or state field exists in the Rust backend, do not duplicate it in the TypeScript frontend (or vice versa). Expose it via Tauri IPC, generate bindings, or pass it dynamically. Adding a duplicate state field breeds drift bugs and will be automatically rejected.

## 2. Explicit Decisions & Zero Ambiguity

Think deeply before you write code. All architectural and logic decisions must be explicitly reasoned out. If a requirement is ambiguous, state your assumption clearly or ask for clarification before proceeding. Do not guess.

## 3. Strict Architectural Boundaries

Respect the Tauri paradigm.

- **Frontend (`src/`):** Strictly for UI, state management, and user interaction. Written in TypeScript. Must not contain OS-level logic or hardcoded system paths.
- **Backend (`src-tauri/`):** Strictly for system/OS access, file system operations, and heavy computation. Written in Rust.
- **Communication:** Cross-boundary communication happens exclusively through Tauri IPC commands and events. Shared DTOs/types must be synchronized (preferably via generated bindings).

## 4. Total Correctness & Functional Patterns

Design the codebase to be low-maintenance and highly predictable.

- **Functional Paradigm:** Use functional programming patterns throughout the codebase (pure functions, immutable state, map/filter/reduce pipelines).
- **Total Correctness:** Prove total program correctness by ensuring two conditions are met: **termination** (no infinite loops or deadlocks) and **partial correctness** (producing the right output for given inputs).
- **Deterministic Convergence:** Ensure all conditional branches and state machines converge correctly and predictably.

## 5. Crash-Resilience & Atomic Operations

Design for failure. Assume the user will force-quit the application or the machine will lose power in the middle of a critical operation.

- Use **atomic operations** wherever possible (e.g., write to a temporary file and atomically rename it, or use strict database transactions).
- Never leave local storage, configuration files, or database rows in a corrupted, half-written state.

## 6. Zero-Defect Safety

- **No Unhandled Exceptions:** Exhaustively handle all errors. In Rust, propagate errors via `Result` and exhaustively match `Option`. Do not leave `unwrap()` or `expect()` in production paths.
- **No Undefined Behavior or Data Races:** Rely on Rust's borrow checker and type system. Do not use `unsafe` code unless absolutely mathematically necessary.

## 7. Git Workflow & Holistic Atomic Commits

- **Atomic Commits:** Every commit must be a single, self-contained logical unit (one feature, one bug fix, or one refactor).
- **Independently Revertible:** Every single commit in the history must be independently revertible without breaking the application state or the CI pipeline.
- **Complete Context:** A code change commit must simultaneously include all relevant **test updates** and **documentation updates**. Do not defer tests or docs to follow-up commits.
- **Test Per Commit:** Run the full relevant test suite for *every* commit. The commit history must remain entirely green and compilable at every step.

## 8. Tooling and Environment Baseline

- **Frontend:** Use `pnpm` exclusively (never `npm` or `yarn`). Run strictly typed TypeScript.
- **Backend:** Use standard Cargo tooling. Treat `cargo clippy` warnings as errors.
- **Validation:** Before finishing a task, ensure both environments compile and validate successfully.

## 9. Dependency Management & Ecosystem Security

- **Minimal JS Dependencies:** The JavaScript ecosystem carries supply chain risks. Keep frontend dependencies to an absolute bare minimum. Write utility functions yourself if the alternative is importing a micro-library.
- **48-Hour Rule:** Any new npm package or version bump must have a minimum release age of **48 hours** before integration.

## 10. Cross-Platform Developer Experience (DX)

The developer experience must be completely frictionless. Ensure that build scripts, testing, and lifecycle commands execute seamlessly across Windows, macOS, and Linux. Do not introduce hardcoded bash-isms or OS-specific dependencies in local dev scripts.

## 11. Directory Map & Restricted Zones

- `src/` — Frontend application code (React/Vue/Svelte, etc.).
- `src-tauri/src/` — Rust backend code and IPC command handlers.
- `docs/arch/` — Architecture Decision Records (ADRs).
- **Restricted:** Do not modify auto-generated IPC bindings, lockfiles (`pnpm-lock.yaml`, `Cargo.lock`), third-party vendored code, or this `AGENTS.md` file unless explicitly instructed.

## 12. Auto-Generated Architecture Documentation

Do not manually draw static system diagrams that will fall out of date. Generate architecture diagrams and dependency graphs directly from the codebase.

- Rely on tools like `dependency-cruiser`, `typedoc`, and `mermaid` to programmatically map system boundaries.

## 13. Common Commands

Use these to validate your work before submitting changes:

- **Install dependencies:** `pnpm install`
- **Start dev server (Frontend + Rust):** `pnpm tauri dev`
- **Frontend validation:** `pnpm check` && `pnpm lint`
- **Backend validation:** `cargo fmt --all -- --check` && `cargo clippy --all-targets -- -D warnings`
- **Run tests:** `cargo test` (Backend) / `pnpm test` (Frontend)
- **Generate Docs/Graphs:** `pnpm docs:all` (Generates TypeDoc definitions and codebase dependency graphs)
