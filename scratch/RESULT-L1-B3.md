# RESULT — L1 surface scaffold (B3 input surface peel)

**Date:** 2026-08-11  
**Branch:** `job/release-integrate`  
**SPEC:** SPEC-L1-SURFACE-SCAFFOLD (B3 peel)  
**Base tip:** `ec8ec82a` (after B2)

## Landed

### B3 — input surface peel batch
First-class packages under contract + peeled from `_legacy`:

| Tag | Axes (prop-registry only) | Faces |
|-----|---------------------------|-------|
| **input** | type (19 values) · size xs\|sm\|md\|lg · invalid · disabled · placeholder | component.field.bg/border/border-hover/error/fg/hint |
| **textarea** | size xs\|sm\|md\|lg · invalid · disabled · placeholder · rows (default 3) | component.field.* (shared) |
| **select** | size xs\|sm\|md\|lg · invalid · disabled · multiple | component.field.* (shared) |

Each host has: `contract.ts`, `doc-<slug>.sail` (specimen left / knobs right ~320px), meta `host: doc-<slug>` + `stageOwnsKnobs: true`, shell `DOC_CE_SLUGS` + mount, visual expanded for `visual.cross`.

### Alias (no CE invent)
| Tag | Kind | Notes |
|-----|------|-------|
| **checkbox** | alias | Hub meta only; blurb points to `/components/input`. Legacy residual demos `<input type="checkbox">`. **Not** in CONTRACTS. No fantasy checkbox PE. |

### Visual verification
| Tag | `bun run visual` |
|-----|------------------|
| input | ✓ green (freeform field stamps + type matrix; probe skips hidden) |
| textarea | ✓ green (freeform stamps + size ladder + autosize behavior) |
| select | ✓ green (xs ladder + multiple) |

Checker: `bun apps/sail-site/scripts/check-l1-surface-contract.mjs` → **ok (20 contracts)**.

### Browser smoke (`http://127.0.0.1:7892`)
- Hub 200
- `/components/input` → `doc-input` + knobs Type/Size/Placeholder/Invalid/Disabled
- `/components/textarea` → `doc-textarea` + Size/Rows/Placeholder/Invalid/Disabled
- `/components/select` → `doc-select` + Size/Invalid/Disabled/Multiple
- `/components/checkbox` → legacy alias residual (input type=checkbox demo)
- `/components/legend` empty state (“Nothing named”)
- No pageerrors on smoke set

## Matrix
All B3 hosts → `scaffold_ready` / operator pending. Pairing queue: [PAIRING-QUEUE.md](./PAIRING-QUEUE.md).

## Residuals / blockers for operator
1. Multi-theme taste approval (operator gate)
2. Playwright knobs→paint structural tests (later)
3. No PE invent: filled/unstyled, component.input.* leaves, checkbox host tag
4. Void `<input>` parse: boolean attrs must not precede `style(...)` (put style first)
5. B4+ peels still legacy residual

## No invent
- Axes from prop-registry only
- Shared field faces only (`component.field.*`)
- checkbox remains alias — real control is input type=checkbox

## No push
Local commits only on `job/release-integrate`.
