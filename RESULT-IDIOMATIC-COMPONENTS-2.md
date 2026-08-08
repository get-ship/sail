# RESULT — idiomatic fine-comb components-2

**Date:** 2026-08-08  
**Branch:** `job/idiomatic-components-2`  
**Rundoc:** `RUNDOC-IDIOMATIC-CORPUS-FINECOMB.md`  
**Job:** `IDIOMATIC-BATCH-COMPONENTS-2`  
**Push:** none  

## Files (12 L2 packs)

| # | File | LOC after | Notes |
|---|------|----------:|-------|
| 1 | `packages/components/sail/primitives/src/sail-tree/sail-tree.sail` | ~220 | header/authority scrub; multi-line row Style |
| 2 | `packages/components/sail/primitives/src/sail-file-upload/sail-file-upload.sail` | ~240 | Lit parity table scrub; **file-math.ts**; multi-line Style |
| 3 | `packages/components/sail/media/src/sail-media-av/sail-media-av.sail` | ~142 | **media-src.ts**; scope `style mediaChrome =`; identity src kept |
| 4 | `packages/components/sail/media/src/sail-media-csv/sail-media-csv.sail` | ~144 | essay scrub; multi-line error Style |
| 5 | `packages/components/sail/mermaid/src/sail-mermaid/sail-mermaid.sail` | ~141 | async-render essay scrub; multi-line error panel Style |
| 6 | `packages/components/sail/charts/src/sail-heatmap/sail-heatmap.sail` | ~103 | migration essay scrub; **heatmap-grid.ts** |
| 7 | `packages/components/sail/chat-elements/src/sail-message-list/sail-message-list.sail` | ~143 | header scrub; multi-line Style |
| 8 | `packages/components/sail/primitives/src/sail-json-view/sail-json-view.sail` | ~116 | **json-rows.ts**; scope `style guidePad =` |
| 9 | `packages/components/sail/viewer/src/sail-component-viewer/sail-component-viewer.sail` | ~144 | scope `style sectionLabel =`; comment scrub |
| 10 | `packages/components/sail/canvas/src/sail-canvas/sail-canvas.sail` | ~122 | header scrub; multi-line viewport Style |
| 11 | `packages/components/sail/primitives/src/sail-error-boundary/sail-error-boundary.sail` | ~107 | NEW SURFACE essay scrub; multi-line fallback Style |
| 12 | `packages/components/sail/charts/src/sail-area-chart/sail-area-chart.sail` | ~111 | migration essay scrub; multi-line Style |

### Compositions / modules added

| Path | Why |
|------|-----|
| `packages/components/sail/primitives/src/sail-file-upload/file-math.ts` | X9 — fmtSize + matchesAccept pure helpers |
| `packages/components/sail/media/src/sail-media-av/media-src.ts` | X9 — mediaUrlOf / contentTypeOf / bytesOf |
| `packages/components/sail/charts/src/sail-heatmap/heatmap-grid.ts` | X9 — parseAxis + toGrid reshape |
| `packages/components/sail/primitives/src/sail-json-view/json-rows.ts` | X9 — buildJsonRows flatten walk |

## Smell table (all 12 — required)

| File | Smells | Action taken | Needs human? |
|------|--------|--------------|--------------|
| `sail-tree` | identity `tree` via Object.defineProperty; data-node-id / data-expanded / data-selected stamps; data-action clear-filters; INDENT_STEP raw px; fat row anatomy | Comment scrub; multi-line Style; engine/view/interactions already extracted | **yes** — pack CE identity props (same family as media src / cmdk items); data-* pack anatomy vs app ban |
| `sail-file-upload` | data-idx remove buttons + closest delegation; data-state over; hand SVG icons; no tone prop surface (header claimed tone, scope has none) | file-math extract; Lit parity table scrub; multi-line Style | low — remove via on:click index bind if partial/for ergonomics allow; icon L1 already exists |
| `sail-media-av` | **src identity only** (Object.defineProperty); emit from `@ship/sail-runtime/internal`; native video/audio attribute poke in action | media-src extract; named mediaChrome Style | **yes** — identity reflection (SMELLS site-1 / components-1 family) |
| `sail-media-csv` | src identity; data-row-id stamp; createTable from `@ship/sail-primitives` | Essay scrub | note — identity src |
| `sail-mermaid` | bindHtml from `/internal`; MutationObserver theme; document-level theme event; seq guard async | Essay scrub; multi-line Style | note — /internal bindHtml is pack pattern (markdown/graph-canvas too); not app-facing |
| `sail-heatmap` | cell-size/gap raw number props (px geometry); error identity-only host patch; values alias of data | heatmap-grid extract; migration scrub | low — continuous cell-size vs tokens (chart geometry OK) |
| `sail-message-list` | **items identity**; BOTTOM_THRESHOLD / ESTIMATED_MESSAGE_HEIGHT raw px; virtualizer spacer binds | Header scrub | **yes** — items identity (message list family) |
| `sail-json-view` | walk was fat in controller; guide pads repeated | json-rows extract; guidePad named Style | no |
| `sail-component-viewer` | querySelectorAll `[slot=stage]` to write variant attr; data-variant chips; data-stage stamp | sectionLabel named Style; comment scrub | **yes** — stage targeting without querySelector (slot/ref composition); data-variant pack stamps |
| `sail-canvas` | canvas identity inject; host.emit; world transform CSS string derived | Header scrub; multi-line Style | no (pack-internal identity substrate) |
| `sail-error-boundary` | createElement for template→slot clone; data-error-message; querySelector fallback slot | NEW SURFACE essay scrub | note — createElement is template projection seam, not app DOM invent |
| `sail-area-chart` | series identity via normalizeSeries; error host patch; raw SVG stroke attrs | Migration scrub | no |

### Needs-human roll-up

1. **Pack CE identity props** (`tree`, `src` on media-av/csv, `items` on message-list, `canvas` inject, chart series/error): not reflectable attrs; consumers assign host properties. Same open gap as components-1 / site-1 SMELLS — need language/runtime first-class identity, no invent this pass.
2. **component-viewer stage drive via querySelectorAll(`[slot=stage]`)**: variant chips poke slotted specimen attributes. Prefer action/ref composition so chips do not walk light DOM.
3. **Pack-internal data-\* stamps** (tree row state, file-upload data-idx, message/component viewer): allowed in L2 packs today vs app `sail-no-data-attr`; long-term part stamps / ARIA-only interaction paths.
4. **`@ship/sail-runtime/internal`** (bindHtml, emit) still used by mermaid/media-av — pack-sanctioned seam, not app author path; document vs public surface.

## Checklist hits

| | |
|--|--|
| **S** | Signals/derived retained; pure helpers out of controllers (S/X9); identity props reported not invented; no new stores |
| **Y1** | Multi-line object Style on tree row, file-upload, mermaid error, canvas, error-boundary, charts |
| **Y2** | Named Style in scope: `mediaChrome`, `guidePad`, `sectionLabel` |
| **Y3 bind** | Existing binds retained (cellPs, rowsH, rowY, worldTx) |
| **P** | N/A for multi-file; same-file guide pads DRY via named Style (not partial — pads are one-span) |
| **C** | No invent widgets; engines (createTable, createChartData, createCanvas, virtualizer) preferred |
| **X3** | Comment scrub all 12 (NEW SURFACE, MIGRATED, Lit parity tables, authority essays) |
| **X9** | Four pure-helper modules extracted; remaining fat is engine glue (tree/message-list) |
| **X1** | Legal relative imports under pack `src/`; /internal imports pre-existing pack pattern (reported) |

## Gaps reported (language / missing primitive)

| Gap | Notes |
|-----|--------|
| Pack CE identity reflection | tree / src / items / canvas / series — SMELLS open |
| Stage/ref for slotted specimen | component-viewer querySelectorAll stage |
| data-* pack anatomy | tree stamps, data-idx remove, data-variant chips |
| Public bindHtml / emit | still under `/internal` for packs |
| No new L1/L2 invented | batch reported only |

## Commands run

```text
PATH=release-integrate/target/debug:$PATH
sail-compiler build --component-name <tag> <path>   # ×12 — all compiled 1 file(s)
```

Policies (from pack `sail.config.json`):

- primitives: full (sx, variants, raw_containers)
- media / mermaid / charts / chat-elements / viewer: mixed (sx)
- canvas: mixed (sx, network self)

## Visual regression (takumi)

**Skipped honest gate:** interactive pack CEs (tree engine, virtualizer, media identity, canvas pan/zoom, mermaid async). Static `sail render` does not run full controllers / identity props / shadow engines — pixel before/after would be empty or misleading. Gate for this batch is **compile green ×12**.

## Non-goals respected

- No codemods  
- No inventing language / identity APIs  
- No feat/next / no push  
- Only the 12 listed `.sail` files (+ supporting pure helper modules)  
- Foreign dirt / Cargo.lock left alone  

## Commit

Local on `job/idiomatic-components-2` — **DO NOT push.**

**Tip SHA:** `c25b49c5` — local only, **not pushed**

**Full tip SHA:** `c25b49c56d58f26301de6264b1dff76b1f3b5d64`

**Feature commit (fine-comb):** `8b3db9e5`
