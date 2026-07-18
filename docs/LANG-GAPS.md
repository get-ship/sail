# Language gaps — the migration ledger

Every place the L2 migration hit something the language cannot say. A gap is
an opportunity to grow the vocabulary, not a reason for CSS. Components that
hit a gap HOLD (keep their `<style>`) until the gap is ruled on — no port
ships degraded.

Ruling key: OPEN (needs a decision) · SHIPPED (atom exists) · REJECTED.

## Shipped this cycle

- `font:` — typography roles (`mono`/`body`/`heading` → `var(--*-font)`).
- `fill` size value — viewport remainder: `calc(100dvh - var(--chrome-offset))`;
  chrome publishes its own offset (`style:--chrome-offset` on the shell).
- `<row nowrap>` — the single-line row (was "Nowrap row / split element"
  below). Boolean axis; layers `flex-wrap:nowrap` on the base, composes with
  `inline`. (job/layout-pass)
- `<grid rows="N">` — explicit grid-template-rows count (1..12), the row-axis
  mirror of `cols`; composes with `cols` into a fixed N×M matrix.
  (job/layout-pass)
- `<column justify>` + `justify="evenly"` — the column's main (vertical) axis
  distribution, plus `space-evenly` completing the justify enum on
  row/column/stack. (job/layout-pass)

## Open — systemic (block multiple components)

- **Density channel**: `--density-row-py/px/gap/text/icon` are a core design
  axis (rows/tiles resize with `data-density`) and sx cannot reference them.
  Blocks: sail-prop-matrix (cells), sail-component-viewer (prop rows), every
  row-shaped surface to come. Candidate: density values join the space/text
  vocabularies (`p:density-row-py`, `text:density`), or a `row-shaped` idiom.
- **`case:` atom** (`upper`/`lower`/`none`): the label idiom (xs + wide
  tracking + case) is everywhere — tutorial eyebrow, source-panel labels,
  prop-matrix cell labels, component-viewer meta labels. Static text can be
  written cased; dynamic labels (e.g. `lang()`) cannot.
- **Grid track shapes**: `<grid>` speaks equal `cols` and a `min` axis; the
  migrations needed `repeat(auto-fill, minmax(180px, 1fr))` (with a real
  width token) and proportional tracks (`1fr 1.4fr auto`). Candidate: a
  closed `tracks` preset vocabulary on `<grid>`.
- **`wrap:` atom** (`pre`/`pre-wrap`/`anywhere`/`nowrap`): sail-diff needs
  pre-wrap + break-word for code lines; currently a plain `<pre>` that
  overflows. Also the reading-column wrap rule in base.css wants to be
  language, not chrome CSS.

- **Prose typography surface**: rendered-content components (sail-markdown)
  and reading chrome (the site's .doc in base.css) need an entire
  typography-of-prose sheet for native child elements — heading rhythm,
  list markers, blockquote, code/pre, table cells. Not sx-shaped (children
  are runtime-rendered). Candidate: a framework prose sheet next to the
  contracts, or a <prose> element. Holds sail-markdown.

## Open — single-site (so far)

- **Stage width tokens**: sail-render's viewport framing (desktop 1280 /
  tablet 768 / mobile 390). No device-width tokens exist; `w`/`max-w` only
  speak tokens. Blocks sail-render.
- **Column clamp**: exact `minmax(0, 30rem)` pane (tutorial lesson column);
  nearest today is `grow:1 + max-w:panel-max-w-md`.
- **Hairline insets**: sail-diff's deliberate 1px line padding; smallest space
  token is coarser. Candidate: `--space-hairline` token.
- **Transform presets**: hover scaleY (tutorial stepper), press translateY
  (buttons). Motion presets cover enter/exit; hover/press micro-transforms
  have no home.
- **Gradient/tint tokens**: radial stage tints (tutorial preview, base.css
  `.split .live`).
- **Mix-strength shades**: `color-mix` hover/current derivations; the
  strength tokens exist (`--hover-strength`) but sx cannot apply them.
- **Leading steps**: 1.6/1.7 line-heights snapped to 1.5/1.55 tokens
  (code-editor sizes).
- **Sibling-position dividers**: `:last-child` border removal on generated
  rows (component-viewer). Language answer is probably a `divided` prop on a
  list container, not a `last:` prefix.
- **forced-colors values**: the condition prefix exists but system colors
  (`CanvasText`) are not expressible values.

## Held components (keep `<style>` pending rulings)

- `sail-render` — stage width tokens.
- `sail-prop-matrix` — density channel, case:, grid tracks, hover transition.
- `sail-component-viewer` — density channel, case:, grid tracks, dividers.

## Parked decisions (rule against live usage, not references)

- **Icon framing** (tone-accented box around a glyph: bare/tile/solid,
  radius/size matrix — the capability behind the retired `set="app"`):
  parked until the ship-app repo migration produces real consumers.
  Candidates then: an existing element absorbs it (avatar/chip),
  composition, or a dedicated element designed against the actual usage.

## Minted this pass — rename freely, the spelling is yours

- **`sx="visibility:assistive"`** — hides the box, keeps the element in the
  accessibility tree. A third value on the existing `visibility:` axis
  (`visible` / `hidden` / `assistive`): who can see this — everyone, nobody,
  or assistive tech only.

  It exists because every graphic owes its reader a text alternative, and the
  25 unmigrated components are full of them: each chart carries a data table,
  the canvas an annotation list. There is no single CSS declaration for it —
  `display:none` and `visibility:hidden` both drop the element from the a11y
  tree too — so it lowers to the nine-declaration block, the one atom whose
  axis is not its CSS property.

  Admitted under the ratified sx rule (2), theme-neutral mechanics: it is
  physics with no design opinion. The NAME is the open question; the capability
  is not. Alternatives: `visibility:sr-only` (Tailwind's), `visibility:a11y`,
  or an L1 `<visually-hidden>` wrapper. One-line change in
  `crates/sail/lint/src/rules/sx.rs`.

## From the five-pack migration (charts · canvas · chat-elements · band · media)

Each of these is a design the language could not say. Nothing was approximated;
the declaration was left out and written down here.

### Costs a user can see today

- **band `angle`** — a ±1deg tilt bleeding one step past each container edge.
  `rotate:` is 0/45/90/180/270 and there is no negative margin. The prop
  reflects and styles nothing.
- **media `fit`** — `object-fit: contain|cover|none`. No `object:` atom, AND a
  variant can never reach a descendant (variants lower to `:scope[attr]
  .sail-root`), so even with the atom the `<img>` inside is unreachable. The prop
  reflects and styles nothing.
- **composer double focus ring** — the frame owns `:focus-within`, so the inner
  control must not draw its own. No `outline:` axis (`ring:` is box-shadow), and
  the framework's `:where(input):focus-visible{outline:…}` still fires.
- **thinking-dots no longer reads as motion** — the bounce was translateY(-25%)
  + opacity. `pulse` bakes scale 1→1.04: sub-pixel on a 7px dot.
- **canvas `transform-origin: 0 0`** — the world scales about its top-left,
  which is what `screenToWorld`/`worldToScreen` assume. With the CSS default
  (50% 50%) the pointer↔world mapping desyncs at any zoom ≠ 1.
- **canvas `touch-action: none`** — without it a touch pan scrolls the page.
- **thumbnail / waveform sizes (48px)** — space tops out at 40px, `control-h-xl`
  is 44px. Approximation is forbidden, so they are unsized.

### The shape of the holes

- **No transition on a state change.** `motion:` is trigger-driven; there is no
  "animate this property whenever it changes" and no `transition:` atom. Hover
  shadows, chevron rotations, and zoom steps all snap now. The framework's own
  button contract transitions these properties — the capability exists, the
  authoring surface does not.
- **Border width.** `border*:` always lowers to `1px solid var(--token)`. A 3px
  category stripe reads as a category stripe; 1px reads as a border.
- **Continuous animation.** `motion:loop` hardcodes `direction:'alternate'` (an
  alternating marquee scrolls backwards), presets bake scalar from/to with a
  duration TOKEN (60–420ms), and there is no percentage-of-own-width translate.
  A marquee is none of those things.
- **Component knobs.** sx cannot emit `var(--knob, var(--token))`, so a
  consumer-overridable per-instance channel has no home. SVG survived only
  because presentation attributes take the fallback form for free.
- **Per-item variants.** `<variants>` keys on HOST attributes, so a per-node
  `kind` or per-row status cannot drive one.
- **`:empty` collapse.** A legend-less chart carries dead space; there is no
  `empty:` state prefix.
- **Fine steps.** Layout `gap` takes integer 0–8 while the space scale has
  `half`/`1_25`/`1_5`. The two vocabularies disagree.
- **Missing atoms named by real designs**: `object-fit`, `outline`, `touch-action`,
  `transform-origin`, `will-change`, `cursor:crosshair`, `opacity:85`,
  `font-variant-numeric`, `case:capitalize`, `overflow-wrap:anywhere`,
  `display:inline-grid`.

### Compiler bugs found (not language gaps)

- **`sail-root` is clobbered.** `dom_emit.rs` emits `classList.add("sail-root")`
  and then `setAttribute("class", …)`. Any root that is a layout keyword or has
  a dynamic axis loses the class — so `variants`/`compound`, which lower to
  `:scope[attr] .sail-root`, SILENTLY NEVER MATCH. Verified in jsdom.
- **A layout root defeats the derived host display.** `.sail-root{display:inline-flex}`
  emits before `.sail-row{display:flex}` at equal specificity, so a `<row>`-rooted
  component is block-level whatever it asked for.
- **A prop named `size` is hijacked** by the framework size channel and retyped
  to xs|sm|md|lg. `<sail-donut size="120">` clamped to "md" and the chart painted
  nothing. Any numeric prop called `size` is a trap.
