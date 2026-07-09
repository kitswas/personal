# ADR 0008 — Theme System: System / Light / Dark

- **Date:** 2026-07-09
- **Status:** Accepted

## Context

The application needs a colour theme that respects user preference. Options:

1. **Dark-only** — simpler, but forces all users into one mode.
2. **System default only** — respects `prefers-color-scheme` but gives no manual override.
3. **Three-mode: System / Light / Dark** — user can follow the OS or override.

Because this is a personal finance app used for focused daily work, user
comfort matters. Some users work in bright environments and find dark UIs
harder to read; others find light UIs harsh. A three-mode toggle is the
standard expectation in 2026 desktop software.

### Where to store the preference

| Storage | Accessible before DB unlock? | Sensitive? | Verdict |
|---|---|---|---|
| Encrypted DB `settings` table | No — requires `unlock()` | No | Rejected |
| `localStorage` | Yes | No | **Chosen** |
| OS-level config file (Tauri `AppConfig`) | Yes | No | Overkill for a UI preference |

The theme must apply on the **first paint** — before the Svelte bundle
executes — to prevent a flash of wrong colour. This requires a small inline
`<script>` in `app.html` that reads `localStorage` and sets `data-theme` on
`<html>` synchronously. The encrypted DB is not available at that point.

## Decision

### Storage

Theme preference is stored in `localStorage` under the key `theme` with values
`'system'`, `'light'`, or `'dark'`. Default (key absent) is treated as
`'system'`.

### Flash prevention

An inline blocking script in `src/app.html` runs before any CSS or JS:

```html
<script>
  (function () {
    const t = localStorage.getItem("theme") ?? "system";
    const prefersDark = matchMedia("(prefers-color-scheme: dark)").matches;
    const isDark = t === "dark" || (t === "system" && prefersDark);
    document.documentElement.setAttribute("data-theme", isDark ? "dark" : "light");
  })();
</script>
```

This is the **only** acceptable use of an inline script. It is intentionally
kept to a single IIFE with no external dependencies.

### CSS implementation

Themes are applied via a `data-theme` attribute on `<html>`, not via a class.
This separates theme intent from component state.

```css
/* Dark palette — also the default when data-theme is absent */
:root,
[data-theme="dark"] { /* ... dark tokens ... */ }

/* Light palette */
[data-theme="light"] { /* ... light tokens ... */ }
```

System mode does **not** use `@media (prefers-color-scheme)` in CSS. Instead,
the inline script resolves the OS preference at runtime and writes a concrete
`data-theme` value. This avoids specificity conflicts between media queries and
explicit `data-theme` values.

### Svelte theme store

```typescript
// src/lib/stores/theme.ts
// Pure functions — no mutation outside the store setter

type Theme = "system" | "light" | "dark";

const STORAGE_KEY = "theme";

const resolveActual = (t: Theme): "light" | "dark" =>
  t === "system"
    ? matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
    : t;

const apply = (t: Theme): void => {
  document.documentElement.setAttribute("data-theme", resolveActual(t));
  localStorage.setItem(STORAGE_KEY, t);
};

export const theme = (() => {
  let value = $state<Theme>(
    (localStorage.getItem(STORAGE_KEY) as Theme | null) ?? "system"
  );

  return {
    get current() { return value; },
    set: (t: Theme) => { value = t; apply(t); },
  };
})();
```

The store also listens to `window.matchMedia("(prefers-color-scheme: dark)")
.addEventListener("change", ...)` so that when the user is in System mode and
their OS switches themes (e.g. at sunset), the app updates immediately without
requiring a restart.

### UI — theme toggle in nav pane header

A single icon button cycles through the three modes:
`system (monitor icon) → light (sun) → dark (moon) → system …`

The toggle is accessible: `aria-label="Theme: {current}"`,
`data-testid="theme-toggle"`.

## Consequences

- **Good:** First paint has the correct theme — no flash.
- **Good:** Works before DB unlock (onboarding screen, password screen both
  benefit from the theme immediately).
- **Good:** OS preference is respected in System mode and updates live.
- **Good:** Pure CSS implementation via `data-theme` — no runtime style
  injection, no `!important` battles.
- **Trade-off:** Theme preference is not encrypted or synced with the DB. If the
  user reinstalls or clears `localStorage`, the preference resets to System.
  This is acceptable — it is a cosmetic preference, not financial data.
- **Constraint:** The inline script in `app.html` must remain a plain IIFE with
  no imports. It must not reference any application module.
- **Constraint:** Tauri's CSP must allow `script-src 'self' 'unsafe-inline'`
  for inline scripts in `app.html`. This is the minimum necessary for flash
  prevention and is a known Tauri pattern. The inline script is audited and
  pinned; no other inline scripts are permitted.
