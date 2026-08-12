# RESULT — L1 surface scaffold (residual first-class peel)

**Date:** 2026-08-11  
**Branch:** `job/release-integrate`  
**SPEC:** SPEC-L1-SURFACE-SCAFFOLD (residual after B6)  
**Base tip:** `31ead1a6` (after B6)

## Landed

### Residual — accordion · combobox · theme-picker peel
First-class packages under contract + peeled from `_legacy`:

| Tag | Axes (prop-registry only) | Faces |
|-----|---------------------------|-------|
| **accordion** | open bool; disabled bool; name string; multiple bool; onToggle event | component.accordion.bg, fg, border, hover-bg, chevron |
| **combobox** | placeholder, value strings; onSelect event | component.combobox.bg, fg (listbox; field chrome = free globals) |
| **theme-picker** | value string; inline bool; modeOnly bool; onThemeChange event | component.theme-picker.bg, fg, trigger-fg, option-fg, option-hover-bg |

Each host has: `contract.ts`, `doc-<slug>.sail` (specimen left / knobs right ~320px), meta `host: doc-<slug>` + `stageOwnsKnobs: true`, shell `DOC_CE_SLUGS` + mount, visual.cross expanded.

**No invent:** checkbox (alias), dropzone (action), layout (keywords). No card ladder / product invent.

### Visual verification
| Tag | `bun run visual` |
|-----|------------------|
| accordion | ✓ green |
| combobox | ✓ green · behavior ok (onSelect callback; open via pointerdown) |
| theme-picker | ✓ green · render anatomy only (behavior deferred) |

Checker: `bun apps/sail-site/scripts/check-l1-surface-contract.mjs` → **ok (47 contracts)**.

### Checker notes
- KNOB_LABELS extended: open, inline, mode only → modeOnly
- Events marked `event: true` — no docs knobs
- combobox list paint in visual uses `var(--popover)` (cascade honesty vs inherited `--combobox` alias)
- theme-picker behavior not in harness (themeSelect localStorage on opaque origin + runtime catalog)

### DETAILS updates
- accordion / combobox / theme-picker props aligned to prop-registry (onToggle, onSelect, inline, modeOnly, onThemeChange)

## Matrix
All residual hosts → `scaffold_ready` / operator pending. Pairing queue: [PAIRING-QUEUE.md](./PAIRING-QUEUE.md).

## Residuals / blockers for operator
1. Multi-theme taste approval (operator gate)
2. theme-picker live catalog: effective ids, mode filter, localStorage apply — product depth
3. combobox `--combobox` token cascade under themed sections (aliases resolve on :root; list paint via popover in harness)
4. Playwright knobs→paint structural tests (later)
5. Remaining legacy: dropzone, layout keywords, pack CEs, checkbox alias

## No invent
- Axes from prop-registry only
- Faces only existing token-registry leaves
- theme-picker knobs = registry props only (no catalog invent)

## No push
Local commits only on `job/release-integrate`.
