# RESULT — idiomatic fine-comb components-1

**Date:** 2026-08-08  
**Branch:** `job/idiomatic-components-1`  
**Rundoc:** `RUNDOC-IDIOMATIC-CORPUS-FINECOMB.md`  
**Job:** `IDIOMATIC-BATCH-COMPONENTS-1`  
**Push:** none  

## Files (12 fat L2 packs)

| # | File | LOC after | Notes |
|---|------|----------:|-------|
| 1 | `packages/components/sail/code-editor/src/sail-code-editor/sail-code-editor.sail` | ~692 | header scrub; CVA size/chrome residue kept |
| 2 | `packages/components/sail/primitives/src/sail-color-picker/sail-color-picker.sail` | ~627 | multi-line Style; panel DRY blocked by partial purity |
| 3 | `packages/components/sail/primitives/src/sail-calendar/sail-calendar.sail` | ~400 | engine already thin seam; comment scrub |
| 4 | `packages/components/sail/primitives/src/sail-cmdk/sail-cmdk.sail` | ~340 | items identity + dialog bridge; comment scrub |
| 5 | `packages/components/sail/primitives/src/sail-date-picker/sail-date-picker.sail` | ~369 | **date-math.ts extract**; multi-line Style; migration essay scrub |
| 6 | `packages/components/sail/canvas/src/sail-graph-canvas/sail-graph-canvas.sail` | ~321 | header scrub; template-slot querySelector kept |
| 7 | `packages/components/sail/canvas/src/sail-annotation/sail-annotation-layer.sail` | ~323 | header scrub |
| 8 | `packages/components/sail/viewer/src/sail-render/sail-render.sail` | ~280 | comment scrub; engine seam intentional |
| 9 | `packages/components/sail/canvas/src/sail-draw/sail-draw.sail` | ~265 | header scrub |
| 10 | `packages/components/sail/media/src/sail-media-image/sail-media-image.sail` | ~233 | identity `src` documented; **no invent** |
| 11 | `packages/components/sail/primitives/src/sail-tree-grid/sail-tree-grid.sail` | ~211 | anatomy essay scrub |
| 12 | `packages/components/sail/canvas/src/sail-board/sail-board.sail` | ~231 | header scrub |

### Compositions / modules added

| Path | Why |
|------|-----|
| `packages/components/sail/primitives/src/sail-date-picker/date-math.ts` | X9 — pure ISO helpers out of controller |

## Smell table (all 12 — required)

| File | Smells | Action taken | Needs human? |
|------|--------|--------------|--------------|
| `sail-code-editor` | fat controller (~650 setup LOC CM6 glue); CVA utility-string maps for size/chrome; `document.createElement` in CM hover tooltip | Comment scrub only; CVA left (pack policy variants:true); createElement is CM Tooltip API not app DOM | **yes** — thin: extract CM mount/diag/completion to composition module; tip Style match for size/chrome |
| `sail-color-picker` | fat controller; **querySelector** for panel/trigger/hex (popover show/hide + attach); duplicated panel markup (swatch vs default); data-* part stamps | Comment scrub; multi-line Style; partial DRY **attempted** — `sail-partial-leaked-ident` blocks controller signal reads without ~15-arg bag | **yes** — popover/ref composition so panel show is action-owned; partial signal ergonomics for panel DRY |
| `sail-calendar` | glue controller still long; querySelector for focus cell; data-iso/data-nav part stamps; virtualized agenda **GAP** | Comment scrub; engine+interactions already extracted | note — agenda virtualization product gap (pre-existing) |
| `sail-cmdk` | **items identity** via Object.defineProperty (not prop); querySelectorAll for scrollIntoView; fat filter/nav controller | Comment scrub | **yes** — first-class identity prop reflection (same family as media-image src / cmdk items) |
| `sail-date-picker` | tone/size props present; size/tone not fully on Style match (host CVA residual?); document pointer-down dismiss; data-act/data-iso stamps | date-math extract; multi-line Style; migration/NEW SURFACE essay scrubbed | low — tone/size paint ownership audit |
| `sail-graph-canvas` | querySelectorAll `template[slot=node]`; edge stroke CSS vars; data-id anatomy | Header scrub | no (template discovery is pack-internal) |
| `sail-annotation-layer` | network POST (pack policy allows self); mode/staging surface OK | Header scrub | no |
| `sail-render` | createElement in **engine/** (legitimate render-engine seam); fat lifecycle controller | Comment scrub; engine purity documented | note — still fat lifecycle; further split optional |
| `sail-draw` | fat scene glue; identity `value`/scene | Header scrub | no |
| `sail-media-image` | **`src` identity only** (Object.defineProperty) — apps use querySelector to assign; not reflectable attr | Documented in header; **no invent API** | **yes** — already on SMELLS (site-1); pack CE identity props |
| `sail-tree-grid` | data-action / data-col-id / data-node-id stamps; fixed INDENT_STEP/ROW_HEIGHT px constants | Anatomy essay scrub | low — px constants vs density tokens |
| `sail-board` | closest `[data-node-id]` / `[data-port-id]` for hit testing; identity board model | Header scrub | no (pack-internal hit testing) |

### Needs-human roll-up

1. **Pack CE identity props** (`sail-media-image.src`, `sail-cmdk.items`, board/draw/calendar identity): not reflectable; consumers poke host via `querySelector` after mount (site-1 SMELLS). Need language/runtime first-class identity without inventing ad-hoc per pack.
2. **color-picker popover + querySelector**: panel showPopover races if-render; attach path uses data-* querySelector. Prefer action-owned popover + capture refs (captureInput already partial).
3. **Partial purity vs controller signals**: cannot DRY color-picker panel without leaking signals or a large param bag (`sail-partial-leaked-ident`). Language ergonomics for “panel fragment over local UI signals.”
4. **code-editor fat controller + CVA size/chrome**: tip Style match for size/chrome; extract CM lifecycle to composition module (ops/lang/theme already split).
5. **calendar virtualized agenda**: explicit product GAP (pre-existing) — not invented this pass.

## Checklist hits

| | |
|--|--|
| **S** | Signals/derived already in graph; date-math extract (S pure helpers); identity props reported not invented; no new stores |
| **Y1** | Multi-line object Style on date-picker / color-picker triggers+panels / media-image gallery |
| **Y2** | Named Style not forced where single-use; partial path blocked for color panel |
| **Y3 bind** | Existing binds retained (swatchColor, zoomTx, evt* geometry, rowY) |
| **P** | color-panel partial attempted → purity fail; reported gap; no fake cross-file partial |
| **C** | No invent widgets; engines/compositions already preferred where present |
| **X3** | Comment scrub all 12 (architecture essays, feat/next migration narrative, Slice authority) |
| **X9** | date-picker pure helpers → `date-math.ts`; remaining fat controllers reported |
| **X1** | Legal relative imports under pack `src/` (engine, color-math, date-math) |

## Gaps reported (language / missing primitive)

| Gap | Notes |
|-----|--------|
| Pack CE identity reflection | media-image `src`, cmdk `items`, canvas value/* — SMELLS site-1 already open |
| Partial + controller signals | panel DRY needs either allowed free reads or cheaper param packing |
| Popover show race / action | color-picker querySelector + rAF ensurePanelShown |
| code-editor CVA utilities | size/chrome still utility-string maps under variants:true pack policy |
| calendar agenda virtualization | product GAP |
| No new L1/L2 invented | batch reported only |

## Commands run

```text
PATH=release-integrate/target/debug:$PATH
sail-compiler build --component-name <tag> <path>   # ×12 — all compiled 1 file(s)
```

Policies (from pack `sail.config.json`):

- code-editor: full (sx, variants, unconfined)
- primitives: full (sx, variants)
- canvas: mixed (sx, network self)
- media/viewer: mixed (sx)

## Visual regression (takumi)

**Skipped honest gate:** these are interactive pack CEs (CM6, dialog, canvas, zoom/pan). Static `sail render` does not run full controllers / identity props / shadow engines — pixel before/after would be empty or misleading. Gate for this batch is **compile green ×12**.

## Non-goals respected

- No codemods  
- No inventing language / identity APIs  
- No feat/next / no push  
- Only the 12 listed `.sail` files (+ supporting `date-math.ts`)  
- Foreign dirt / Cargo.lock left alone  

## Commit

Local on `job/idiomatic-components-1` — **DO NOT push.**

**Tip SHA:** `43e74daf` — local only, **not pushed**

**Full tip SHA:** `43e74daf4e1d0857277cd079ea9d2461d95899ce`

**Feature commit (fine-comb):** `632f83b4`
