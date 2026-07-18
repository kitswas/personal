# AGENTS.md

Cross-tool agent instructions for any AI coding assistant working on this repository. Read this entirely before writing any code.

## 1. Single Source of Truth (Absolute Rule)

No piece of state lives in two places. State is strictly held in the `state.rs` module. The UI projection in `app.rs` is strictly read-only and may only emit `Message` intents. Do not duplicate state across different components or views.

## 2. Explicit Decisions & Zero Ambiguity

Think deeply before you write code. All architectural and logic decisions must be explicitly reasoned out. If a requirement is ambiguous, state your assumption clearly or ask for clarification before proceeding. Do not guess.

## 3. Every Line of Code is a Liability

Code requires understanding, testing, maintaining, and debugging.

- **No Speculative Engineering:** Do not build abstractions, features, or configuration keys "just in case." Solve only the immediate problem using the simplest, most direct approach. Keep it simple.
- **Delete Whenever Possible:** Strive for minimalism. Dead, unused, or replaced code must be completely removed from the codebase, never bypassed or commented out.

## 4. Strict Architectural Boundaries

Respect the Unidirectional Data Flow pattern.

- **State (`state.rs`):** Pure state management. Holds `AppState`, `Message`, and `Command` enums. Transitions state via a pure `apply_message` function.
- **UI (`app.rs`):** Strictly for immediate-mode GUI projection using `egui`. It takes an immutable reference to `AppState` and a message transmitter. It MUST NEVER mutate state directly.
- **Async Runtime:** Side effects (like `LoadData`, DB queries) are emitted as `Command`s. They are spawned in Tokio background tasks and their results are piped back to the main thread as `Message`s via channels.

## 5. Total Correctness & Functional Patterns

Design the codebase to be low-maintenance and highly predictable.

- **Functional Paradigm:** Use functional programming patterns throughout the codebase (pure functions, immutable state, map/filter/reduce pipelines).
- **Total Correctness:** Prove total program correctness by ensuring two conditions are met: **termination** (no infinite loops or deadlocks) and **partial correctness** (producing the right output for given inputs).
- **Deterministic Convergence:** Ensure all conditional branches and state machines converge correctly and predictably.

## 6. Crash-Resilience & Atomic Operations

Design for failure. Assume the user will force-quit the application or the machine will lose power in the middle of a critical operation.

- Use **atomic operations** wherever possible (e.g., write to a temporary file and atomically rename it, or use strict database transactions).
- Never leave local storage, configuration files, or database rows in a corrupted, half-written state.

## 7. Zero-Defect Safety

- **No Unhandled Exceptions:** Exhaustively handle all errors via `Result` and exhaustively match `Option`. Do not leave `unwrap()` or `expect()` in production paths.
- **Exhaustive Domain Errors:** Do not use `anyhow` or generic strings for errors returned to the UI. Define specific Rust `enum`s for all failure modes.
- **Exhaustive State Handling:** Rust enums must be exhaustively matched in `apply_message`.
- **No Undefined Behavior or Data Races:** Rely on Rust's borrow checker and type system. All library crates must carry `#![deny(unsafe_code)]`. Do not use `unsafe` blocks in application code under any circumstances.

## 8. Asynchronous UI State Visibility

For every asynchronous operation, the UI must explicitly and accurately reflect the current execution state to the user. Do not silently fail or trap the user in a loading state.

- **State Machines over Flags:** Never use independent boolean flags (`is_loading`, `is_error`). Wrap async commands in state enums using discriminated variants (`Idle`, `Loading`, `Success(T)`, `Error(E)`).
- **Processing:** Always show a clear active state. Use definite progress indicators if the operation length is quantifiable or has checkpoints, or indefinite indicators (spinners) if unknown.
- **Success:** Provide unambiguous visual confirmation when an operation completes successfully.
- **Failure:** Catch all errors and surface them gracefully to the UI. The user must be provided with a clear, actionable, and human-readable reason for the failure.

## 9. Git Workflow & Holistic Atomic Commits

- **Atomic Commits:** Every commit must be a single, self-contained logical unit (one feature, one bug fix, or one refactor).
- **Independently Revertible:** Every single commit in the history must be independently revertible without breaking the application state or the CI pipeline.
- **Complete Context:** A code change commit must simultaneously include all relevant **test updates** and **documentation updates**. Do not defer tests or docs to follow-up commits.
- **Test Per Commit:** Run the full relevant test suite for _every_ commit. The commit history must remain entirely green and compilable at every step.

## 10. Tooling and Environment Baseline

- **Standard Cargo:** Use standard Cargo tooling. Treat `cargo clippy` warnings as errors.
- **Validation:** Before finishing a task, ensure the crate compiles and validates successfully via `cargo check` and `cargo test`.

## 11. Dependency Management & Ecosystem Security

- **Minimal Dependencies:** Keep dependencies to an absolute bare minimum. Write utility functions yourself if the alternative is importing a micro-library.
- **48-Hour Rule:** Any new package or version bump must have a minimum release age of **48 hours** before integration.

## 12. Cross-Platform Developer Experience (DX)

The developer experience must be completely frictionless. Ensure that build scripts, testing, and lifecycle commands execute seamlessly across Windows, macOS, and Linux. Do not introduce hardcoded bash-isms or OS-specific dependencies in local dev scripts.

## 13. Directory Map & Restricted Zones

- `src/` — Pure Rust application code (egui Frontend + Backend logic).
- `docs/arch/` — Architecture Decision Records (ADRs).
- **Restricted:** Do not modify lockfiles (`Cargo.lock`), third-party vendored code, or this `AGENTS.md` file unless explicitly instructed.

## 14. Documentation Standards (Why, not What)

Code documentation (Rustdoc) must exclusively explain **WHY** a block of code exists, the rationale behind a decision, or the context of a workaround. It must **NEVER** explain **WHAT** the code is doing (the syntax itself is the "what").

- We already maintain detailed manual architecture documentation in the `docs/arch/` ADRs. Refer to them for system-wide context.
- Use `cargo-modules` for generating codebase architecture diagrams automatically rather than manually maintaining them.

## 15. Common Commands

Use these to validate your work before submitting changes:

- **Build / Run:** `cargo run`
- **Backend validation:** `cargo fmt --all -- --check` && `cargo clippy --all-targets -- -D warnings`
- **Run unit tests:** `cargo test`
- **Run fuzz tests:** `cargo +nightly fuzz run <target>`
