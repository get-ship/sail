# use-constrain report (Part A)

**Branch:** `job/use-constrain`  
**Worktree:** `/home/madvib/dev/Ship/worktrees/use-constrain`  
**Date:** 2026-07-20

## Goal

Make `use:` honest: author mix-in API + Action-body floor (no class / no style assignment except `--*` custom properties).

## Delivered

### Docs
- `docs/ARCHITECTURE.md` — new section **`use:` — author mix-in API (not the L1 brain)** with normative shape, is/is-not, registry role, no-JS honesty, Action floor table, `tests/perf/**` exemption.
- Enforcement map row for `sail-action-no-style`.
- Softened `icon.sail` “behavior rides use:icon” wording toward Part B controllers.

### Lint (`sail-lint`)
- New rule `sail-action-no-style` (`crates/sail/lint/src/rules/action_no_style.rs`).
- Dispatch: `is_action_source_path` → `packages/sail/runtime/src/actions/**/*.ts`.
- Exempt: paths containing `/tests/perf/` (bench interop class=).
- Allow: `style.setProperty/removeProperty` when name is `--*` literal **or** non-literal (helpers like `KEYS[i]` / `name`).
- Refuse: `.style` access otherwise; plain CSS names in `setProperty('height',…)`; `.className`; `.classList` mutation; `set/removeAttribute('class'|'style')`.
- Controllers unchanged: still refuse all `.style` including custom props (`sail-no-imperative-style`).

### Registry action fixes
| Action | Change |
| --- | --- |
| `anchor` | writes `--anchor-left` / `--anchor-top` only |
| `contextMenu` | same vars for pointer placement |
| `autosize` | `data-autosize` + `--autosize-h`; skin applies height |
| `icon` | dropped `setAttribute('class', …)`; skin is `.sail-icon svg` |
| `spotlight` | `data-sail='spotlight-glow'` instead of className |
| `targetCursor` | `data-sail='tc-*'` instead of className |

Floating skins (`popover`, `dropdown`, `hover-card`, `context-menu`, `combobox` listbox, `theme-picker` panel) consume `left/top: var(--anchor-*)`.  
Runtime textarea skin: `textarea[data-autosize] { height: var(--autosize-h, auto); … }`.  
Convention tokens: `anchor-left`, `anchor-top`, `autosize-h`.

### Intentionally not touched (Part B / peer)
- `chip.sail`, `actions/chip.ts`, element controllers — owned by `L1ControllerChip`.

## Gates

```
cargo test -p sail-lint --lib     → 400 passed
cargo test -p sail-compiler --lib → 441 passed
cargo run -p sail-lint --example check -- packages/sail/runtime/src/actions
  → findings: 0
```

## Notes
- Companion overlay sheets still inject `<style id=…>` (not in Part A refuse list). Anatomy is data-attr keyed.
- Author-local Actions outside `runtime/src/actions/` are not path-detectable yet; registry gate ships first.
- Bench path exemption is fail-open by prefix only — no new config key.
