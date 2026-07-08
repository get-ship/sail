# SUI QA harness — component render-correctness gate

Deterministic, programmatic verification that every catalogued component
actually renders in the component viewer. No vision, no eyeballing: the probe
loads each `/components/<tag>` detail page (which mounts the **real** component
and drives it into its interesting state — overlays open, disclosure expands),
then asserts on the box model + composited colours. Agents fix what it flags.

## Run it

```bash
# from apps/sui-site, with the deck dev server on :7878
node qa/probe.mjs <tag> [--theme=a,b]     # one component, verbose JSON
node qa/probe.mjs --sweep [--theme=a,b]   # all components, clustered + report.json
```

Default themes: `ship-light,glass,ship-dark`. The sweep writes
`qa/report.json` (per-component host size + defects) and prints a cluster
summary ("few-shared vs many-unique").

## Signals (all deterministic)

| flag | meaning |
|------|---------|
| `NOT-RENDERED` | the real `<tag>` element is absent (unregistered pack / crash) |
| `EMPTY-RENDER` | element exists but paints nothing (host + all descendants zero-area) |
| `OUTSIDE-STAGE` | a painted box escapes the `.detail-stage` bounds (rect math — flags even when `overflow:hidden` clips it) |
| `OVERFLOW-X` | a box scrolls horizontally with `overflow-x:visible` |
| `LOW-CONTRAST` | visible text below ~2.5:1 against its effective background |
| `ERR:…` | uncaught page error (compiler/runtime break) |

Colours are rasterised through a 1×1 canvas → **true sRGB**, so oklch tokens
are measured correctly (a naive regex parse garbles oklch as RGB and invents
false low-contrast on normal body text — do not reintroduce it).

`EXPECTED_EMPTY` (in probe.mjs) suppresses `EMPTY-RENDER` for components that
render nothing at the host BY CONTRACT (sui-portal teleports, sui-toast-host is
an empty stack, sui-render / sui-prop-knobs need a JS descriptor).

## How the viewer stays honest

- The `/components/[tag]` route imports `../../../src/register-all` (every pack)
  — **register-all is the single source of truth** for what the viewer can
  mount. A pack missing there = every one of its components shows NOT-RENDERED.
- Seeds live in `src/sui-catalog/preview-seeds.ts`. A seed's `drive` directive
  (`attrs` / `methods` / `click` / `hover`) opens overlays for real; `staticHTML`
  is a **gallery-card poster only** and is ignored by the detail viewer.
- Full-bleed surfaces (canvas/board/code/mermaid) get `.detail-fill`; downward
  dropdowns (color-picker/select/…) get `.detail-top` — see the FILL_TAGS /
  TOP_ALIGN_TAGS sets in the detail controller.

## Blind spots (probe passes, still visually wrong)

The box-model gate CANNOT catch "rendered but unstyled/structurally wrong"
(e.g. an accordion whose items didn't register renders as run-on text but has
paint, fits, and contrasts). Backstop those with screenshots
(`playwright` element `.screenshot()` on `.detail-stage`) during review.

## What this gate has caught (feat/next → spike/sui-sx)

- Detail route registered ZERO components (missing `sui-primitives` import).
- Site never imported 6 packs → 24 components unregistered.
- Primitives index dropped 4 registrations (accordion-item, timeline, alert,
  navigation-item) → e.g. accordion rendered as mashed text.
- oklch contrast false-positives (20 components) — a probe bug, now fixed.
- sui-board crash (seed node shape), heatmap SVG missing width/height,
  json-view/mermaid missing seeds, canvas/table collapse in centered stage.
