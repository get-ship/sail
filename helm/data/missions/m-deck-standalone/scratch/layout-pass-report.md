# job/layout-pass — layout-element expressiveness pass

MISSION m-deck-standalone. Owner ask (2026-07-18): "we can add to row and grid,
I suspect those are not expressive enough, worth a dedicated pass for layout."

Targeted expressiveness expansion of the L1 layout keywords. Every axis added is
a first-class element attribute the compiler lowers to a class + injected CSS,
gated by the closed-enum lint — no sx/style escape. Source of truth:
`crates/sail/lint/src/rules/layout.rs` (the lint the `sail-layout` rule validates
against and the same tables the compiler's `preprocess::lower_layout` consults).

## The full axis audit (element × current axes × verdict)

Layout keywords: `<row>` `<column>` `<grid>` `<center>` `<stack>`, each lowering
to a `<div>` + atomic classes. `inline` is a boolean axis on every keyword.

| Element    | Axes BEFORE                                  | Gap found                                         | Verdict |
|------------|----------------------------------------------|---------------------------------------------------|---------|
| `<row>`    | gap, align, justify, direction, inline        | no way to express a **non-wrapping** row          | **ADD** `nowrap` |
| `<column>` | gap, align, direction, inline                 | **main (vertical) axis** not distributable        | **ADD** `justify` |
| `<grid>`   | gap, min, flow, cols, tracks, inline          | no explicit **row-count** axis                    | **ADD** `rows` |
| `<center>` | gap, inline                                   | intentionally opinionated; nothing missing        | keep |
| `<stack>`  | align, justify, inline                        | z-overlay complete                                | keep |
| justify enum (row/column/stack) | start, center, end, between, around | `space-evenly` missing                 | **ADD** `evenly` |
| align enum | start, center, end, stretch, baseline         | complete for `align-items`                        | keep |

## What was ADDED (each with a real use-case + red/green gate)

### 1. `<row nowrap>` — the single-line row  *(known gap #1)*
A `<row>` wraps by default (it absorbed `sail-cluster`); there was **no** way to
express a non-wrapping row. `nowrap` is a boolean axis (row-only, like `inline`):
presence layers `.sail-nowrap{flex-wrap:nowrap}` on top of the base's
`flex-wrap:wrap`, injected **after** the base rule so source order wins (no
specificity game). Composes with `inline` → `<row inline nowrap>`.
- **Real layouts**: toolbars, split panes, a full-height split that must rely on
  stretch (the tutorial workbench bug logged in LANG-GAPS), any horizontal strip
  that must not reflow onto a second line.
- **Gates** (`layout.rs`): `accepts_row_nowrap`, `lowers_row_nowrap_to_flex_wrap_override`
  (asserts the rule follows the base), `nowrap_composes_with_inline`,
  `plain_row_has_no_nowrap_class`, `rejects_nowrap_on_a_tag_that_doesnt_take_it`,
  `rejects_nowrap_with_a_value`, `rejects_dynamic_nowrap`.
- **Compiler emit** (`preprocess.rs`): `row_nowrap_emits_flex_wrap_override`,
  `row_inline_nowrap_composes`.

### 2. `<grid rows="N">` — explicit row count  *(known gap #2)*
`<grid>` accepted `cols` (fixed column count) but had no row-axis mirror. `rows`
is the same closed 1..=12 enum, lowering to
`.sail-gr-N{grid-template-rows:repeat(N,minmax(0,1fr))}` (the `minmax(0,1fr)`
overflow guard `cols` uses). It owns `grid-template-rows` — a **different** CSS
property than `cols`/`min`/`tracks`, so unlike `cols`↔`min` there is **no**
precedence to resolve; it composes freely into a fixed N×M matrix. Dynamic
`rows={n()}` lowers to a runtime class binding with all 12 rules pre-injected,
exactly like `cols`.
- **Real layouts**: dashboard/card grids with a fixed number of rows, calendar
  week-grids, any explicit-track matrix.
- **Gates** (`layout.rs`): `accepts_grid_rows_static_and_dynamic`,
  `lowers_grid_rows_to_repeat_minmax`, `rows_composes_with_cols`,
  `rejects_off_enum_rows`, `rejects_rows_on_a_tag_that_doesnt_take_it`.
- **Compiler emit** (`preprocess.rs`): `grid_rows_emits_repeat_minmax_track_list`,
  `grid_cols_and_rows_compose_into_a_fixed_matrix`,
  `grid_rows_dynamic_lowers_to_a_class_binding_with_all_rules`.

### 3. `justify` on `<column>` — the column's main (vertical) axis
`<row>` had `justify`; `<column>` did not — so a column's **main** axis (vertical,
since `.sail-col` is `flex-direction:column`) could not be distributed at all,
only its cross axis (`align`). This is the direct symmetric completion. It has
**no default** (unlike row's `justify="start"`): a plain `<column>` keeps
`justify-content`'s CSS initial and emits no class, so there is **zero output
churn** on existing columns — the axis only fires when the author asks.
- **Real layouts**: a full-height sidebar column pinning a logo to the top and
  nav to the bottom (`justify="between"`), card footers, any vertical
  space distribution.
- **Gates** (`layout.rs`): `accepts_justify_on_column`,
  `lowers_column_justify_and_has_no_default`.
  (`preprocess.rs`): `column_justify_and_evenly_emit`.

### 4. `justify="evenly"` — completes the distribution enum (row/column/stack)
`justify` had `between`/`around` but not `space-evenly` — the last standard
`justify-content` distribution. `between` puts extra space only *between* items;
`evenly` puts equal space *around and between*. Purely additive (new enum value,
no default change).
- **Real layouts**: evenly-spaced nav items / toolbar segments / tab strips.
- **Gates**: `accepts_and_lowers_justify_evenly` (`layout.rs`),
  `column_justify_and_evenly_emit` (`preprocess.rs`).

## Honest-refusals updated to teach the now-built forms

The diagnostics-audit left two honest refusals pointing at gaps. Both are now
closed and the refusals teach the built form:

- `crates/sail/lint/sx-vocabulary.json`:
  - `flex-nowrap` → `<row nowrap> — the single-line row`
    (was: "a non-wrapping row has no layout keyword yet")
  - `grid-rows` → `<grid rows="3"> — an explicit row count`
    (was: "there is no rows-count axis yet")
- `crates/sail/lint/src/rules/sx.rs` — `LayoutIntent` now **absorbs**
  `flex-nowrap` (→ `nowrap: true`) and `grid-rows-N` (→ `rows`), so a whole
  Tailwind breath like `flex-nowrap items-center gap-2` collapses into one
  buildable element `<row nowrap gap="2" align="center">` instead of falling
  through to a per-atom refusal. Gates: `flex_nowrap_redirects_to_a_buildable_row_nowrap`,
  `grid_rows_redirects_to_a_buildable_rows_axis`.
- The stale `// … have no layout keyword yet` comment block in `sx.rs` was
  rewritten to describe the absorption.

## FLAGGED for owner (design-ish — not guessed)

These are legitimate needs but each carries a design/naming decision the owner
should rule on, so they were **not** added:

1. **Per-axis gap (row-gap vs column-gap).** A grid or wrapping row sometimes
   wants tight rows + wide columns. CSS `gap` is `row-gap column-gap`. The
   *decision* is the surface shape — `gap-x`/`gap-y` twin axes, or a two-value
   `gap="4 2"`, or `row-gap`/`col-gap`. All are coherent; picking one is a naming
   call. **Recommendation**: two-value `gap="<row> <col>"` keeps one axis name and
   reads like the CSS shorthand, but flagging rather than guessing.
2. **`align-content` for multi-line cross-axis distribution.** When a `<row>`
   wraps onto several lines, `align-content` controls the spacing *between the
   lines* (distinct from `align`/`align-items`). Niche; a new axis name
   (`align-lines`? `content`?) is a design question. Flag.
3. **Child-level self-alignment (`align-self`) / per-item span control.**
   Grid child span is already handled via `data-col-span`/`data-row-span`
   selectors; `align-self` on an individual flex child has no element home. This
   is a *child* concern, not a container axis — likely wants a different
   mechanism (a wrapper element or a child attribute convention), so flag.

## The clean seam: what stays in sx (NOT element axes)

The spec's candidate list (min/max sizing, aspect ratio, sticky/position,
z-order, overflow scroll containers, responsive breakpoints) was audited against
the seam rule — *an axis that means the same on every element is sx; an axis that
means something specific to `<row>`/`<grid>` is an element attribute.* **All of
them already exist as sx atoms and correctly belong there** — adding them as
layout axes would duplicate the vocabulary and break the seam:

| Candidate (spec)          | Already expressible as sx            | Verdict |
|---------------------------|--------------------------------------|---------|
| min/max sizing            | `min-w`/`max-w`/`min-h`/`max-h`/`w`/`h`/`size`/`basis` | sx — keep |
| aspect ratio              | `aspect-square`/`aspect-video`/`aspect-auto` | sx — keep |
| sticky / position         | `sticky`/`absolute`/`fixed`/`relative` + `inset`/`top`/… | sx — keep |
| z-order / layering        | `z-*` (z-index scale); `<stack>` for overlay | sx + stack — keep |
| overflow scroll container | `overflow`/`overflow-x`/`overflow-y` (`auto`/`scroll`/…) | sx — keep |
| responsive / breakpoint   | sx condition prefixes (`mobile:`/`tablet:`/`desktop:`/…) | sx — keep |

The layout element only grows the axes that mean something **specific to the
container**: how it wraps (`nowrap`), how many tracks it has (`cols`/`rows`), and
how it distributes its own main/cross axis (`align`/`justify`). That is the whole
of what was added.

## Mapping to the real layouts audited

- **motion.dev hero (job/motion-gauntlet)**: a non-wrapping horizontal nav/CTA
  strip is now `<row nowrap>` rather than a hold-for-CSS. Radial glows / sticky
  headers were already sx (`bg-glow-*`, `sticky`).
- **Dashboard / card grids**: `<grid cols="4" rows="3">` expresses a fixed matrix
  first-class; auto-fill responsive grids stay on `min`.
- **Sticky headers / split panes**: sticky = `sx="sticky top-0"` (unchanged);
  the split's non-wrapping row = `<row nowrap>`; full-height column distribution
  = `<column justify="between">`.

## Gates summary

- `<row nowrap>` and `<grid rows="N">` compile + emit correct CSS — red/green in
  both `sail-lint` (lowering + lint) and `sail-compiler` (preprocess emit).
- Every added axis (`nowrap`, `rows`, column `justify`, `justify="evenly"`) has a
  red/green emit test.
- Honest-refusals for nowrap/rows updated to teach the built forms; no stale
  "no keyword yet" text remains (grepped clean).
- Generated layout reference regenerated:
  `apps/sail-site/src/language-surface.gen.ts` (`LAYOUT_KEYWORDS` now lists
  `nowrap`/`rows`/column `justify`; `SX_REFUSALS` teaches the built forms) and
  `.claude/skills/sail-authoring/SKILL.md` (via `sail-compiler skill`). The regen
  also folded in pre-existing generator drift (a `spin` motion preset, a
  `chromeOffset` action, an already-current `flex-wrap` refusal string) — those
  are not from this job but are the correct current output of the generators.
- `cargo test -p sail-lint -p sail-compiler -p sail-layout` — **green** (411 lint
  tests, full compiler suite).
- `cargo test --workspace` — the **deck stack does not build** (`deck-core`
  `build.rs` panics: SAIL bundle assets missing in this worktree — the #102 asset
  bootstrap blocker, unrelated to any file this job touched). All non-deck crates
  green.

## Files touched

- `crates/sail/lint/src/rules/layout.rs` — axes, enums, modifiers, boolean-axis
  handling, lowering, tests.
- `crates/sail/compiler/src/parse/preprocess.rs` — `nowrap` injection, consumed
  attrs, tests.
- `crates/sail/lint/src/rules/sx.rs` — `LayoutIntent` absorbs flex-nowrap /
  grid-rows, tests.
- `crates/sail/lint/sx-vocabulary.json` — refusal strings.
- `crates/sail/compiler/src/bin/sail-compiler.rs` — SKILL.md hard-rules prose.
- `apps/sail-site/src/language-surface.gen.ts`,
  `.claude/skills/sail-authoring/SKILL.md` — regenerated.
- `docs/LANG-GAPS.md` — nowrap gap moved to shipped; the three additions logged.
