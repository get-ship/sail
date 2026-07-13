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

## Open — systemic (block multiple components)

- **Density channel**: `--density-row-py/px/gap/text/icon` are a core design
  axis (rows/tiles resize with `data-density`) and sx cannot reference them.
  Blocks: sui-prop-matrix (cells), sui-component-viewer (prop rows), every
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
- **`wrap:` atom** (`pre`/`pre-wrap`/`anywhere`/`nowrap`): sui-diff needs
  pre-wrap + break-word for code lines; currently a plain `<pre>` that
  overflows. Also the reading-column wrap rule in base.css wants to be
  language, not chrome CSS.

- **Prose typography surface**: rendered-content components (sui-markdown)
  and reading chrome (the site's .doc in base.css) need an entire
  typography-of-prose sheet for native child elements — heading rhythm,
  list markers, blockquote, code/pre, table cells. Not sx-shaped (children
  are runtime-rendered). Candidate: a framework prose sheet next to the
  contracts, or a <prose> element. Holds sui-markdown.

## Open — single-site (so far)

- **Stage width tokens**: sui-render's viewport framing (desktop 1280 /
  tablet 768 / mobile 390). No device-width tokens exist; `w`/`max-w` only
  speak tokens. Blocks sui-render.
- **Nowrap row / split element**: `<row>` wraps by default, so a full-height
  split cannot rely on stretch (the tutorial workbench bug). Either a nowrap
  word on `<row>` or a dedicated panes element.
- **Column clamp**: exact `minmax(0, 30rem)` pane (tutorial lesson column);
  nearest today is `grow:1 + max-w:panel-max-w-md`.
- **Hairline insets**: sui-diff's deliberate 1px line padding; smallest space
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

- `sui-render` — stage width tokens.
- `sui-prop-matrix` — density channel, case:, grid tracks, hover transition.
- `sui-component-viewer` — density channel, case:, grid tracks, dividers.

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
  `crates/sui/lint/src/rules/sx.rs`.
