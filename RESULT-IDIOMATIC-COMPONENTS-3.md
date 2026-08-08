# RESULT — idiomatic fine-comb components-3

**Date:** 2026-08-08  
**Branch:** `job/idiomatic-components-3`  
**Rundoc:** `RUNDOC-IDIOMATIC-CORPUS-FINECOMB.md`  
**Job:** `IDIOMATIC-BATCH-COMPONENTS-3`  
**Push:** none  

## Files (12 — last charts + diff/md/timeline + chat core)

| # | File | LOC after | Notes |
|---|------|----------:|-------|
| 1 | `packages/components/sail/charts/src/sail-line-chart/sail-line-chart.sail` | ~141 | header scrub; multi-line template/Style |
| 2 | `packages/components/sail/charts/src/sail-bar-chart/sail-bar-chart.sail` | ~125 | header scrub; multi-line template/Style |
| 3 | `packages/components/sail/charts/src/sail-gauge/sail-gauge.sail` | ~120 | header scrub; multi-line center Style |
| 4 | `packages/components/sail/charts/src/sail-sparkline/sail-sparkline.sail` | ~104 | header scrub; multi-line SVG Style |
| 5 | `packages/components/sail/charts/src/sail-donut/sail-donut.sail` | ~119 | header scrub; multi-line center Style |
| 6 | `packages/components/sail/primitives/src/sail-diff/sail-diff.sail` | ~142 | **diff-lines.ts extract**; named Style; controller thin |
| 7 | `packages/components/sail/primitives/src/sail-markdown/sail-markdown.sail` | ~34 | **markdown-render.ts extract**; migration essay scrub |
| 8 | `packages/components/sail/primitives/src/sail-timeline/sail-timeline-item.sail` | ~126 | parity/LANG-GAP essay scrub; multi-line compound Style |
| 9 | `packages/components/sail/primitives/src/sail-timeline/sail-timeline.sail` | ~16 | Lit parity table / NEW SURFACE essay scrub |
| 10 | `packages/components/sail/chat-elements/src/sail-composer/sail-composer.sail` | ~136 | named Style (frame/field); multi-line |
| 11 | `packages/components/sail/chat-elements/src/sail-tool-call/sail-tool-call.sail` | ~83 | named Style already tip; comment/format polish |
| 12 | `packages/components/sail/chat-elements/src/sail-message/sail-message.sail` | ~68 | named Style bubble when map formatted; comment scrub |

### Compositions / modules added

| Path | Why |
|------|-----|
| `packages/components/sail/primitives/src/sail-diff/diff-lines.ts` | X9 — pure LCS + unified parsers out of controller |
| `packages/components/sail/primitives/src/sail-markdown/markdown-render.ts` | X9 — pure markdown→HTML out of controller |

### Test import retarget (supporting)

| Path | Change |
|------|--------|
| `packages/components/sail/primitives/test/sail-markdown.test.ts` | `renderMarkdown` from `./markdown-render` (not controller emit) |

## Smell table (all 12 — required)

| File | Smells | Action taken | Needs human? |
|------|--------|--------------|--------------|
| `sail-line-chart` | MIGRATED essay; identity `error` via mount; SVG stroke raw `var(--border)`; aria table DRY cross-file | Comment scrub; multi-line template | low — shared chart aria-table partial/component optional |
| `sail-bar-chart` | same family as line-chart | Comment scrub; multi-line | low — same aria table |
| `sail-gauge` | identity `error`; diameter not `size` (framework channel collision); SVG muted stroke | Comment scrub; multi-line | no |
| `sail-sparkline` | lightweight empty/loading exception (spinner + empty SVG); identity error | Comment scrub; multi-line | no (exception intentional) |
| `sail-donut` | identity `error`; diameter naming | Comment scrub; multi-line | no |
| `sail-diff` | fat pure LCS in controller; duplicated match arms; **data-line-kind** part stamps; `<pre>` for whitespace (no Style pre setter) | **diff-lines extract**; named Style per kind; data-* kept (pack tests) | low — partial DRY blocked without Style-on-invoke or kind→style map; whitespace Style gap |
| `sail-markdown` | migration essay; **bindHtml from `/internal`**; pure parser in controller; raw HTML prose | **markdown-render extract**; essay scrub; re-export kept | **yes** — public `bindHtml` (or sanctioned markdown paint) without `/internal` |
| `sail-timeline-item` | first/last mount-time static; interactive role flip; forced-colors residual | Comment scrub; multi-line Style | note — dynamic list first/last would need mutation observer / composition |
| `sail-timeline` | Lit parity memorial table | Scrubbed to brief host doc | no |
| `sail-composer` | identity `attachments` via defineProperty; host `onSubmit`; data-state disabled; dual field if | Named Style; comment scrub | **yes** — pack CE identity props (`attachments`) same family as media-image/cmdk |
| `sail-tool-call` | data-state status group paint (pack-internal) | Format polish | no |
| `sail-message` | data-state multi-token group paint (pack-internal) | Format polish | no |

### Needs-human roll-up

1. **`bindHtml` only on `@ship/sail-runtime/internal`** — sail-markdown paints parser HTML via internal API. Prefer public export or a first-class markdown/prose paint primitive so pack controllers stay off `/internal`.
2. **Pack CE identity props** — composer `attachments` (defineProperty), chart `error` (mount host read): not reflectable Scope props when type is `unknown` / array identity. Same open SMELLS family as site-1 / components-1.
3. **Whitespace Style for code/diff lines** — diff keeps `<pre>` because white-space control is incomplete for the pre/pre-wrap + break-word combination wanted; report only.
4. **Shared chart aria-description table** — copy-pasted across line/bar/donut; cross-file → real component, not same-file partial. Optional follow-up pack helper, not invented here.
5. **diff match-arm DRY** — four near-identical rows; partial without Style-on-invoke still needs match inside for paint. Named styles help; full single-arm bind would need kind→style map language support.

## Checklist hits

| | |
|--|--|
| **S** | Controllers already signal/derived; chart engines retained; no new stores; identity gaps reported not invented |
| **Y1** | Multi-line object Style on charts/composer/timeline-item; named Style on diff/composer/tool-call/message |
| **Y2** | Named Style preferred where multi-use (diff kinds, composer field variants, tool-call/message) |
| **Y3 bind** | Existing tone bind on timeline-item dot retained; no new enumeration matrices |
| **P** | Same-file partial for diff rows skipped (Style-on-invoke ban + paint-by-kind); gap reported |
| **C** | Charts keep skeleton/empty-state/error-state/spinner/accordion/button; no invented widgets |
| **X3** | Comment scrub all 12 (MIGRATED / NEW SURFACE / Lit parity / migration essays) |
| **X9** | diff-lines.ts + markdown-render.ts pure extracts |
| **X1** | Relative imports under pack `src/` legal; markdown still needs `/internal` for bindHtml — reported |

## Gaps reported (language / missing primitive)

| Gap | Notes |
|-----|--------|
| Public bindHtml / markdown paint | markdown uses `@ship/sail-runtime/internal` |
| Pack CE identity reflection | error identity, attachments array |
| white-space Style completeness | diff `<pre>` residual |
| Shared chart aria table component | cross-file DRY not v1 partial |
| No new L1/L2 invented | batch reported only |

## Commands run

```text
PATH=release-integrate/target/debug:$PATH
sail-compiler build --component-name <tag> <path>   # ×12 — all compiled 1 file(s)
```

Policies (from pack `sail.config.json`):

- charts: mixed (sx, raw_containers)
- primitives: full (sx, variants)
- chat-elements: mixed (sx, raw_containers)

Unit tests: workspace `node_modules` not installed in this worktree; gate is **compile green ×12** (same as components-1). `sail-markdown.test.ts` import path updated for extract.

## Visual regression (takumi)

**Skipped honest gate:** pack CEs are interactive / data-driven (SVG from props, markdown bindHtml, composer field, accordion). Static `sail render` does not run full controllers / identity props — pixel before/after would be empty or misleading. Gate for this batch is **compile green ×12**.

## Non-goals respected

- No codemods  
- No inventing language / identity APIs  
- No feat/next / no push  
- Only the 12 listed `.sail` files (+ supporting `diff-lines.ts`, `markdown-render.ts`, test import retarget)  
- components-3b (citation, thinking-dots, annotation-list, annotation-marker) **not** in this batch  

## Commit

Local on `job/idiomatic-components-3` — **DO NOT push.**

**Tip SHA:** `PENDING` — local only, **not pushed**

**Full tip SHA:** `PENDING`

**Feature commit (fine-comb):** `77c291cd`
