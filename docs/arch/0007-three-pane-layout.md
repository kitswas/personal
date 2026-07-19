# ADR 0007 — Three-Pane Layout (30-40-30)

- **Date:** 2026-07-09
- **Status:** Accepted

## Context

A personal finance ledger has three distinct information densities that users
simultaneously need:

1. **Navigation context** — which account / view am I in? What are my current
   balances at a glance?
2. **Primary list** — the transaction ledger, import triage grid, or account
   list for the current view.
3. **Detail / action panel** — postings for the selected transaction, account
   summary, or quick-add form.

Two layout archetypes were considered:

**Two-pane (sidebar + content):**

```
┌────────┬──────────────────────────┐
│  Nav   │      Main content        │
│ (14%)  │        (86%)             │
└────────┴──────────────────────────┘
```

This collapses detail into the main column. Selecting a transaction opens a
modal or an inline expander, which hides part of the list. On a wide desktop
screen, ~70% of horizontal space goes unused or is filled with excessive
whitespace.

**Three-pane (nav + list + detail):**

```
┌──────┬──────────┬──────┐
│ Nav  │   List   │Detail│
│  30% │    40%   │  30% │
└──────┴──────────┴──────┘
```

All three information types are visible simultaneously on wide screens. This
matches the interaction model of desktop-native finance apps.

## Decision

Use a **three-pane layout** implemented with `iced`'s native flexbox model:

- **Left Nav:** `container(...).width(Length::FillPortion(3))`
- **Right Detail:** `container(...).width(Length::FillPortion(3))`
- **Center List:** `container(...).width(Length::FillPortion(4))` (automatically fills the remaining ~40% space).

### Responsive collapse strategy

The layout collapses progressively by checking `ui.available_width()` dynamically during the render loop.

- Wide `> 1024px`: Render all three panes.
- Medium `640–1024px`: Render Nav and Central, but render the Detail pane as an overlay modal or collapse the Nav pane.
- Narrow `< 640px`: Render a single Central pane with a top/bottom navigation bar.

### Empty state of the detail pane

The detail pane is **never blank**. When nothing is selected:

- On the Transactions route: shows the account's running balance chart
- On the Import route: shows template documentation for the selected template
- On the Accounts route: shows net-worth summary

## Consequences

- **Good:** Maximum information density on wide desktop screens.
- **Good:** Implemented entirely in Rust via pure `iced` flexbox layout logic, avoiding CSS and DOM completely.
- **Good:** The right pane acts as a contextual action surface, reducing popup dialogs.
- **Trade-off:** UI components must be nested correctly within `row!` and `column!` macros to allocate space.
