# RESULT — L1 surface scaffold (B6 overlays + product hosts peel)

**Date:** 2026-08-11  
**Branch:** `job/release-integrate`  
**SPEC:** SPEC-L1-SURFACE-SCAFFOLD (B6 peel)  
**Commit:** `30bea8d5`  
**Base tip:** `79e8703a` (after B5)

## Landed

### B6 — overlays + product / layout hosts peel batch
First-class packages under contract + peeled from `_legacy`:

| Tag | Axes (prop-registry only) | Faces |
|-----|---------------------------|-------|
| **tabs** | active string; orientation horizontal\|vertical default=horizontal; onTabChange event | FACE_MERGE: component.tab.* + component.tab-list.* (no component.tabs.*) |
| **navbar** | placement top\|bottom\|rail default=top; sticky/bordered/autohide bools; offset number | component.navbar.bg, border, fg |
| **card** | tone optional; variant default\|outline; padding md\|none\|sm\|lg; interactive; accent-bar off\|hover\|static; status-rail; fill | component.card.bg, fg, texture*, backdrop-filter |
| **avatar** | src, icon-slug (docsSkip); name, initials; tone optional; size xs–xl default md; shape round\|square | component.avatar.bg, fg |
| **breadcrumb** | *(no PE props)* | component.breadcrumb.fg, current |
| **dialog** | size sm\|md\|lg\|full; backdrop default\|blur\|solid; side center\|start\|end\|top\|bottom; dismissible; onClose/onDismiss events | component.dialog.* |
| **popover** | placement string | component.popover.bg, fg, border |
| **tooltip** | placement top\|bottom\|left\|right default top; content string | component.tooltip.bg, fg |
| **dropdown** | placement string; onSelect event | component.dropdown.bg, fg, border |
| **hover-card** | placement string | component.hover-card.bg, fg, border |
| **context-menu** | for string (docsSkip); onSelect event | component.context-menu.bg, fg, border |
| **toast** | tone optional; duration string; onDismiss event | component.toast.bg, fg, border, stripe |

Each host has: `contract.ts`, `doc-<slug>.sail` (specimen left / knobs right ~320px), meta `host: doc-<slug>` + `stageOwnsKnobs: true`, shell `DOC_CE_SLUGS` + mount, visual expanded for `visual.cross`.

**No invent:** card soft/elevated/ghost, avatar product variants beyond registry, overlay behavior depth beyond registry props, top-level legend / toggle package.

### Visual verification
| Tag | `bun run visual` |
|-----|------------------|
| tabs | ✓ green · behavior ok |
| navbar | ✓ green |
| card | ✓ green |
| avatar | ✓ green |
| breadcrumb | ✓ green |
| dialog | ✓ green · behavior ok |
| popover | ✓ green · behavior ok |
| tooltip | ✓ green |
| dropdown | ✓ green · behavior ok |
| hover-card | ✓ green · behavior ok |
| context-menu | ✓ green · behavior ok |
| toast | ✓ green · behavior ok |

Checker: `bun apps/sail-site/scripts/check-l1-surface-contract.mjs` → **ok (44 contracts)**.

### Checker notes
- KNOB_LABELS extended: placement, sticky, autohide, offset, padding, interactive, accent bar, status rail, fill, backdrop, side, content, active, initials
- Events marked `event: true` — no docs knobs
- `for` (context-menu), `src` / `icon-slug` (avatar) docsSkip
- Breadcrumb zero-axis host OK (missing from prop-registry gen)

### Browser smoke (`http://127.0.0.1:7892`)
- Site was down — not exercised.

## Matrix
All B6 hosts → `scaffold_ready` / operator pending. Pairing queue: [PAIRING-QUEUE.md](./PAIRING-QUEUE.md).

## Residuals / blockers for operator
1. Multi-theme taste approval (operator gate) — especially card texture/backdrop, tab strip, overlay elevation
2. Overlay pointer/placement geometry depth stays operator later (scaffold asserts open/close / anatomy)
3. Toast/menu CustomEvents removed from runtime (callbacks only) — visual behavior updated accordingly
4. component.dialog.* often aliases globals; some themes leave --dialog unbound per theme
5. Playwright knobs→paint structural tests (later)
6. Remaining legacy residual: accordion, combobox, dropzone, theme-picker, layout packs, checkbox alias

## No invent
- Axes from prop-registry only
- Faces only existing token-registry leaves (FACE_MERGE for tabs)
- Card variant ladder stays default|outline

## No push
Local commits only on `job/release-integrate`.
