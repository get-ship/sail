# RESULT — idiomatic fine-comb IDIOMATIC-BATCH-SITE-6

**Date:** 2026-08-08  
**Branch:** `job/idiomatic-site-6`  
**Rundoc:** `RUNDOC-IDIOMATIC-CORPUS-FINECOMB.md`  
**Skill:** `sail-authoring` (tip Style product; not retired `sx`)  
**Push:** none  

## Files (7)

| File | LOC after | Hits |
|------|----------:|------|
| `apps/sail-site/src/store-total/store-total.sail` | 22 | Y2/X3 |
| `apps/sail-site/src/sail-components-rail/sail-components-rail.sail` | 14 | Y2/X3 + bare-attr scrub |
| `apps/sail-site/routes/_layout.sail` | 5 | X3 |
| `apps/sail-site/routes/components/_layout.sail` | 4 | X3 |
| `apps/sail-site/routes/ai/_layout.sail` | 4 | X3 |
| `apps/sail-site/routes/deck/_layout.sail` | 4 | X3 |
| `apps/sail-site/routes/sail/_layout.sail` | 4 | X3 |

**Paired compositions:** none (controllers already thin; layouts are mount-only).

## Checklist hits (summary)

| Key | What changed |
|-----|----------------|
| **S** | store-total already `useStore(tally)` + thin return; no state rewrite. Layouts have no controllers. |
| **Y1/Y2** | store-total: inline `style({…})` → named `style line` / `style total`. components-rail: inline geometry → named `style shell` / `style scrollMain`. |
| **Y3 bind** | N/A — no value-enumeration matrices. |
| **P1** | N/A — no same-file repeated structure. |
| **C** | No invented widgets; segment layouts still mount existing rails / site-header. |
| **X1** | Imports already legal (`@ship/sail-runtime`, relative compositions). |
| **X2** | Removed residual bare attrs `components-shell` / `components-scroll` on components-rail (selector-style leftovers; not used elsewhere). |
| **X3** | Comment scrub on all 7: drop deck-mount/process essays; keep short purpose (and one non-obvious geometry constraint on components-rail). |
| **X9** | Controllers stay wire-up; layouts remain single-tag mounts. |

## Structural / Style notes

| File | Before | After |
|------|--------|--------|
| `store-total.sail` | Inline object Style on `<p>` / `<strong>` | Named scope styles; teaching text unchanged |
| `sail-components-rail.sail` | Inline Style + unused `components-shell` / `components-scroll` attrs | Named `shell` / `scrollMain`; attrs removed; geometry preserved |
| `*_layout.sail` (5) | Long mount/architecture comments | Short purpose comments; markup unchanged |

## Smell table (required — all 7)

| File | Smell | Severity | Notes | Human? |
|------|-------|----------|-------|--------|
| `store-total.sail` | Demo widget; store dynamics partial in takumi | low | Same as store-counter; shared tally not fully exercised in static render | no |
| `store-total.sail` | Teaching ellipsis lead-in (“…and this line”) | n/a | Intentional voice for State chapter | no |
| `sail-components-rail.sail` | No sidebar (unlike docs/deck/ai rails) | n/a | Catalog is full-bleed by design | no |
| `sail-components-rail.sail` | Static render is empty shell (slot only) | low | Needs host + child route for real paint | no |
| `routes/_layout.sail` | Mount-only chrome | none | Fine-comb = comment scrub | no |
| `components/_layout.sail` | Mount-only | none | Fine-comb = comment scrub | no |
| `ai/_layout.sail` | Mount-only | none | Fine-comb = comment scrub | no |
| `deck/_layout.sail` | Mount-only | none | Fine-comb = comment scrub | no |
| `sail/_layout.sail` | Mount-only | none | Fine-comb = comment scrub | no |

## Gaps reported (language / missing primitive / docs)

1. **Open from prior batches (unchanged):** grid `cols` not breakpoint-aware; issue #32 residue on deck data; `animate:reveal` runtime; sail-media-image identity; gen `SX_EXTRA_FORMS` prefix-era residual.  
2. **No new language gaps** from this batch — files are tip store/style/layout mounts.

## Commands run

```text
export PATH="…/release-integrate/target/debug:$PATH"
sail-compiler skill   # sail-authoring present
sail-compiler build --component-name store-total|sail-components-rail <paths>
sail-compiler build apps/sail-site/routes/{,_layout,components,ai,deck,sail}/_layout.sail
# all EXIT 0

sail render …/{store-total,components-rail,root-layout}-{before,after}-dark.png
```

## Visual

| Surface | Path | Notes |
|---------|------|-------|
| store-total before/after | `scratch/web-qa/idiomatic-site-6/screenshots/store-total-*-dark.png` | Named styles; expect near-identical paint |
| components-rail before/after | `…/components-rail-*-dark.png` | Named geometry; empty shell (slot) |
| root layout before/after | `…/root-layout-*-dark.png` | Comment-only; static custom-element shell |

Static `sail render` does not fully expand stores, custom chrome, or projected route slots.

## SHA

`482be569732cf5fa519341182956a040a6dea7ec` on `job/idiomatic-site-6` (local only; no push).

## Acceptance

| Criterion | Status |
|-----------|--------|
| Fine-comb 7 site-6 files only | pass |
| No codemods | pass |
| Tip Style product (not retired sx) | pass |
| Full smell table | pass (above) |
| Compile green | pass (per-file + component builds exit 0) |
| Comment scrub | pass on touch |
| Teaching content preserved | pass (store-total prose unchanged) |
| Commit local; **no push** | pass |
