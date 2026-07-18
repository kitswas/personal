# ADR 0008 — Theme System: System / Light / Dark

- **Date:** 2026-07-09
- **Status:** Accepted

## Context

The application needs a colour theme that respects user preference. Options:

1. **Dark-only** — simpler, but forces all users into one mode.
2. **System default only** — respects OS preference but gives no manual override.
3. **Three-mode: System / Light / Dark** — user can follow the OS or override.

Because this is a personal finance app used for focused daily work, user
comfort matters. A three-mode toggle is the standard expectation in desktop software.

### Where to store the preference

| Storage                                  | Accessible before DB unlock? | Verdict                      |
| ---------------------------------------- | ---------------------------- | ---------------------------- |
| Encrypted DB `settings` table            | No — requires `unlock()`     | Rejected                     |
| Unencrypted SQLite / local JSON config   | Yes                          | **Chosen**                   |

The theme must apply on the **first paint** to prevent a flash of wrong colour. Since `eframe` configures context visuals immediately on startup, we must read the preference before creating the native window.

## Decision

### Storage

Theme preference is stored in a lightweight unencrypted local settings file (e.g., `config.toml` or `AppConfig` via `directories` crate).

### Egui Implementation

Themes are applied via `egui::Context::set_visuals()`.

```rust
if user_prefers_dark {
    ctx.set_visuals(egui::Visuals::dark());
} else {
    ctx.set_visuals(egui::Visuals::light());
}
```

In "System" mode, the application relies on `eframe`'s built-in capability to detect OS theme changes and updates the `Visuals` dynamically in the render loop.

### UI Toggle

A single icon button cycles through the three modes:
`system (monitor icon) → light (sun) → dark (moon) → system …`

## Consequences

- **Good:** First paint has the correct theme — no flash of white.
- **Good:** Works before DB unlock (onboarding screen, password screen both benefit from the theme immediately).
- **Good:** Native `egui` visuals completely eliminate CSS and class toggling complexity.
- **Trade-off:** Theme preference is not synced with the encrypted DB, but this is acceptable for a cosmetic config.
