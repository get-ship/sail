# SUI QA harness — component render-correctness gate

Deterministic, programmatic verification that every catalogued component
actually renders in the component viewer. No vision, no eyeballing: the probe
loads each `/components/<tag>` detail page (which mounts the **real** component
and drives it into its interesting state — overlays open, disclosure expands),
then asserts on the box model + composited colours. Agents fix what it flags.

## Run it

```bash
# from apps/sail-site, with the deck dev server on :7878
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
render nothing at the host BY CONTRACT (sail-portal teleports, sail-toast-host is
an empty stack, sail-render / sail-prop-knobs need a JS descriptor).

## How the viewer stays honest

- The `/components/[tag]` route imports `../../../src/register-all` (every pack)
  — **register-all is the single source of truth** for what the viewer can
  mount. A pack missing there = every one of its components shows NOT-RENDERED.
- Seeds live in `src/sail-catalog/preview-seeds.ts`. A seed's `drive` directive
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

## What this gate has caught (feat/next → spike/sail-sx)

- Detail route registered ZERO components (missing `sail-primitives` import).
- Site never imported 6 packs → 24 components unregistered.
- Primitives index dropped 4 registrations (accordion-item, timeline, alert,
  navigation-item) → e.g. accordion rendered as mashed text.
- oklch contrast false-positives (20 components) — a probe bug, now fixed.
- sail-board crash (seed node shape), heatmap SVG missing width/height,
  json-view/mermaid missing seeds, canvas/table collapse in centered stage.

## Behavior + surface-quality harness

Where `probe.mjs` proves the component *renders*, `behavior.mjs` proves the
component *does the right thing when its advertised props/behavior are used*.
It is the second axis of the harness — correctness, not rendering.

```bash
# from apps/sail-site, with the deck dev server on :7878
node qa/behavior.mjs              # full sweep → behavior-report.json + 12 screenshots
node qa/behavior.mjs sail-dialog   # one component, verbose
node qa/behavior.mjs --overlay    # only the overlay-family surface-consistency pass
```

### What it checks

| check | what it asserts |
|---|---|
| `tone` (enums) | at least 2 of {bg, border, color} change across documented values |
| `size` (sm/md/lg/…) | host or inner height differs measurably across values |
| `disabled` | the composite control's interactive descendant is non-functional |
| `open` (boolean) | setting `open` actually expands the panel (>1 painted box); falls back to `showPopover()` for popover="auto" |
| overlay drive | open the overlay; check it's painted, sized sanely; capture screenshot |

Every targeted check returns PASS / FAIL / REVIEW. **REVIEW = the harness
couldn't confirm either way** — typically because the prop's effect lives on
a small element (icon part, ring, etc.) that the "largest descendant"
heuristic doesn't sample. The accompanying `qa/qa-shots/<tag>.png` screenshot
is the operator's visual confirmation handle.

### Surface-consistency (overlay family)

`sail-context-menu`, `sail-select`, `sail-dropdown-menu`, `sail-popover`,
`sail-dialog`, `sail-alert-dialog`, `sail-tooltip`, `sail-hover-card`,
`sail-combobox`, `sail-cmdk`, `sail-drawer`, `sail-sheet` — these must share a
common surface (background / border / radius / elevation). The harness samples
each overlay's panel after opening, clusters by fingerprint, and flags any
that diverge from the dominant cluster.

### Re-run command

```bash
# from repo root
(cd apps/sail-site && node qa/behavior.mjs)
```

Exit code is non-zero on any FAIL — gates CI and the operator gets a
fix-vs-cut triage list in `behavior-report.json` + per-overlay PNGs in
`qa/qa-shots/`.
