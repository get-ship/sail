# RESULT — L1 surface scaffold (B5 layout/data peel)

**Date:** 2026-08-11  
**Branch:** `job/release-integrate`  
**SPEC:** SPEC-L1-SURFACE-SCAFFOLD (B5 peel)  
**Base tip:** `016c36b7` (after B4)

## Landed

### B5 — layout / data / segmented peel batch
First-class packages under contract + peeled from `_legacy`:

| Tag | Axes (prop-registry only) | Faces |
|-----|---------------------------|-------|
| **scroll-area** | orientation vertical\|horizontal\|both default=vertical | component.scroll-area.thumb, thumb-hover |
| **ticker** | duration fast\|normal\|slow default=normal; reverse bool | *(none — no component.ticker.\*)* |
| **table** | dense, striped, hoverable, bordered bools default false | component.table.fg, muted-fg, sticky-bg, stripe |
| **pagination** | value string; onPageChange event | *(none — no component.pagination.\*)* |
| **toggle-group** | type single\|multiple default=single; value string; variant outline\|ghost\|segmented default=outline; orientation horizontal\|vertical default=horizontal; onToggleChange event | component.toggle-group.bg, border (toggle cell leaves stay on `<toggle>`) |

Each host has: `contract.ts`, `doc-<slug>.sail` (specimen left / knobs right ~320px), meta `host: doc-<slug>` + `stageOwnsKnobs: true`, shell `DOC_CE_SLUGS` + mount, visual expanded for `visual.cross`.

**Segmented control:** stays on **toggle-group** host — no top-level `toggle/` package invent.

### Visual verification
| Tag | `bun run visual` |
|-----|------------------|
| scroll-area | ✓ green (orientation vertical · horizontal · both) |
| ticker | ✓ green (**new** `specs/ticker.mjs`) |
| table | ✓ green (dense · striped · hoverable · bordered + rest) |
| pagination | ✓ green (value matrix + behavior ok) |
| toggle-group | ✓ green (type · variant · orientation · value + behavior ok) |

Checker: `bun apps/sail-site/scripts/check-l1-surface-contract.mjs` → **ok (32 contracts)**.

### Checker notes
- KNOB_LABELS extended: `dense`, `striped`, `hoverable`, `bordered`, `reverse`, `duration`, `segmented` (type/orientation already present).
- Events (`onPageChange`, `onToggleChange`) marked `event: true` — no docs knobs.
- Empty faces OK for ticker + pagination (registry has no component leaves).

### Browser smoke (`http://127.0.0.1:7892`)
- Run when site is up; expect each B5 detail to mount `doc-*` with knobs/specimen.

## Matrix
All B5 hosts → `scaffold_ready` / operator pending. Pairing queue: [PAIRING-QUEUE.md](./PAIRING-QUEUE.md).

## Residuals / blockers for operator
1. Multi-theme taste approval (operator gate) — especially toggle-group segmented rail, table density
2. Playwright knobs→paint structural tests (later)
3. No PE invent: component.ticker.*, component.pagination.*, toggle as hub tile
4. Table component token alias freeze: `--table-*` computed on `:root` from globals — themes rebind sources; th paint may not re-alias until product rebinds component.table.* per theme
5. scroll-area thumb-hover leaf remains parked unread (SPEC-D1)
6. B6+ peels still legacy residual

## No invent
- Axes from prop-registry only
- Faces only existing token-registry leaves
- Segmented host = toggle-group (not a new package)

## No push
Local commits only on `job/release-integrate`.
