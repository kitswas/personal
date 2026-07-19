# ADR 0009 — Frontend Framework Evolution: Tauri to egui to iced

- **Date:** 2026-07-19
- **Status:** Accepted

## Context

The selection of a frontend framework is critical for a local-first desktop application designed for high information density (accounting ledgers, triage grids). The project has undergone a significant evolution in its frontend architecture, prioritizing a pure Rust native stack over web technologies.

### Phase 1: Tauri + Web Frameworks (Initial Consideration)

Initially, a web-based UI via Tauri (with SolidJS or React) was considered to leverage CSS and web charts.

- **Pros:** Familiar web ecosystem, extensive charting libraries (e.g., D3, Chart.js).
- **Cons:** Reliance on a WebView adds significant runtime overhead and binary bloat. Introducing Node.js/npm dependencies introduces TypeScript/JavaScript, a horrible language for correctness.
- **Outcome:** Rejected in favor of a pure Rust native GUI.

### Phase 2: egui (Immediate-Mode GUI)

The project pivoted to `egui` to achieve a lightweight, single-binary footprint with high performance.

- **Pros:** Extremely fast setup, zero external dependencies, robust `eframe` runtime, and immediate integration with Rust's ecosystem.
- **Cons:** Immediate-mode GUIs execute layout and rendering on every frame, which led to severe layout and sizing issues when attempting to build a complex, premium desktop interface. To mitigate this, a custom component library ([`egui-elegant`](https://github.com/kitswas/egui-elegant)) was explicitly written to try and get a good UI, but the underlying immediate-mode constraints remained a bottleneck for advanced flexbox-style layouts (like the 30-40-30 three-pane layout) and robust interactive data visualizations.
- **Outcome:** Initially accepted and heavily customized, but ultimately abandoned due to inherent layout and sizing limitations.

### Phase 3: iced (Retained-Mode GUI & Elm Architecture)

To resolve the limitations of immediate-mode layouts, the project migrated to `iced` (v0.14).

- **Pros:**
  - Uses **The Elm Architecture (TEA)**, which perfectly aligns with the strict Unidirectional Data Flow pattern of our core logic (`State` -> `view` -> `Message` -> `update`).
  - **Retained-mode layout engine:** Provides a robust flexbox layout model (`row!`, `column!`, `Length::FillPortion`), trivially solving responsive multi-pane layouts (ADR-0007).
  - **Native Canvas (`iced::widget::canvas`):** Provides a powerful, performant, TEA-compliant API for rendering interactive custom charts (like Sankey diagrams) without fighting an immediate-mode loop.
- **Cons:** Slightly steeper learning curve to manage `Cache` and `Action` lifecycles in the canvas.

## Decision

We formally standardize on **`iced`** (The Elm Architecture) as the frontend framework for the application. All previous `egui` and `eframe` implementations have been removed.

## Consequences

- **Good:** Complete alignment between the core Rust state machine and the UI framework.
- **Good:** Responsive layouts are drastically simpler to write and maintain without CSS.
- **Good:** High-performance, interactive custom data visualizations are now possible via `iced::widget::canvas`.
- **Constraint:** We must strictly adhere to TEA patterns, meaning all asynchronous side effects (e.g., SQLite I/O, parsing) must be wrapped in `iced::Task` (or channeled correctly) to prevent blocking the rendering thread.
