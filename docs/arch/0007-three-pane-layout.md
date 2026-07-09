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
matches the interaction model of desktop-native finance apps (GnuCash, Ledger
Live, Banktivity) and email clients (Outlook, Thunderbird) where list +
detail side-by-side is the established convention for record-oriented data.

## Decision

Use a **three-pane layout** with proportions approximately **30 / 40 / 30**,
implemented with CSS Grid using fluid fractions (`fr`) — no hardcoded pixel
widths anywhere.

```css
/* Wide: all 3 panes */
.shell {
  display: grid;
  grid-template-columns: var(--col-nav) var(--col-list) var(--col-detail);
  grid-template-areas: "nav list detail";
  height: 100vh;
}
```

### Responsive collapse strategy

The layout collapses progressively via CSS media queries and a Svelte
`$derived` `layoutMode` rune — no JavaScript resize observers needed.

| Viewport | Columns | Detail pane |
|---|---|---|
| Wide `> 1024px` | nav (3fr) + list (4fr) + detail (3fr) | Always visible |
| Medium `640–1024px` | nav (30%) + list (70%) | Hidden by default; slides in as an overlay on item selection (CSS transform + opacity) |
| Narrow `< 640px` | Single column (100vw) | Becomes a pushed view via SvelteKit page transition; bottom tab bar replaces nav pane |

### Pane responsibilities

| Pane | Content |
|---|---|
| **Nav (left)** | App name + tagline header; nav links with active route indicator; account tree with live balance per account |
| **List (center)** | Transaction ledger / import triage / account rows — the primary scrollable list for the current route |
| **Detail (right)** | Context panel driven by center selection: transaction postings, account summary card, quick-add transaction form; shows a neutral hint state when nothing is selected |

### Empty state of the detail pane

The detail pane is **never blank**. When nothing is selected:
- On the Transactions route: shows the account's running balance chart
- On the Import route: shows template documentation for the selected template
- On the Accounts route: shows net-worth summary

This prevents the jarring "grey void" common in 3-pane apps when first loading.

## Consequences

- **Good:** Maximum information density on wide desktop screens — matches the
  target use case (desktop-first Tauri app).
- **Good:** Implemented entirely in CSS Grid with `fr` units. No JS for layout
  — the layout is intrinsic and cannot get out of sync with state.
- **Good:** Progressive collapse to 2-pane and then 1-pane satisfies the
  blueprint's mobile-first fluid requirement without a separate mobile codebase.
- **Good:** The detail pane doubles as a quick-action surface (quick-add form),
  reducing modal usage.
- **Trade-off:** The right detail pane adds a `$derived` context store
  (`selectedItem`) that must be kept in sync with navigation. This is the
  one piece of shared cross-pane state; it lives in `src/lib/stores/selection.ts`
  and is the single source of truth (per ADR 0001's no-duplication rule).
- **Trade-off:** On medium viewports, the detail pane is an overlay. The slide
  animation must be implemented with CSS `transform`/`opacity` transitions only
  (no JS animation libraries) to keep bundle size minimal.
- **Constraint:** Hardcoded pixel dimensions for the grid columns are forbidden.
  All layout sizing uses `fr`, `%`, `vw`, or `vh`. This is enforced by the
  linter (a custom ESLint rule or `stylelint` rule can flag `px` in
  `grid-template-columns`).
