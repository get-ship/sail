<!--
  <card> — L1 language element (element mode). The flagship surface: a
  generic container with a header/body/footer structure, tone-tinted
  border/rail/fill, an accent bar, and an interactive hover lift. Bare tag:
  it lowers away at compile time; the skin routes to the framework layer
  keyed on .sail-card; the contract feeds the element manifest.

  Faithful to the packages/components/sail/primitives/src/sail-card/
  baseline for tone/variant/padding/accent-bar/status-rail/fill. tone rides
  the global [tone] channel (contracts.css) — no per-value CSS here, same
  as the baseline's "NEW SURFACE" note.

  STRUCTURAL DIVERGENCE — header/footer slots: element mode's use-site
  splicing (parse/preprocess.rs) only projects a single DEFAULT slot today;
  named-slot projection ("<slot name=x>" picking out only the matching
  use-site children) is unimplemented compiler machinery, and extending it
  is out of scope for a stdlib-source-only promotion. Since element mode is
  light DOM (the tag is replaced by its structure snippet, no real shadow
  root), a use site's children splice in with their ORIGINAL attributes
  intact — including `slot="header"`/`slot="footer"`. So instead of real
  named slots, this skin selects on those attributes directly against the
  root's direct children: `:host > [slot='header']` / `:host > [slot=
  'footer']`. Every existing `<sail-card>` use site already writes
  slot="header"/"footer" markup (it was authoring for real named slots
  before) — that markup needs ZERO changes; only the tag rename applies.
  Presence-selection replaces the baseline's `reflect-empty` (a runtime
  slotchange listener, itself component-only / unavailable in element
  mode): a header/footer border only ever renders when a matching child
  exists, which the attribute selector already expresses for free.

  LANG-GAP — `interactive`'s a11y wiring: the baseline's controller sets
  role="button"/tabindex="0" on mount and an Enter/Space→click keydown
  listener (packages/components/.../sail-card.controller.ts). That is
  runtime JS — element mode is static structure, so it cannot ship here.
  The prop keeps its full CSS treatment (cursor, hover lift, active
  press, focus-visible ring — all static :host([interactive]) rules), but
  a use site that needs real keyboard-activatable semantics must add
  role="button" tabindex="0" and its own keydown handling itself (or slot
  in a real <button>/<a>). No current usage sets `interactive`, so this
  migrates clean today.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: 'default' | 'outline' = 'default'
    padding: 'md' | 'none' | 'sm' | 'lg' = 'md'
    interactive = false
    'accent-bar': 'off' | 'hover' | 'static' = 'off'
    'status-rail' = false
    fill = false
  }
</scope>

<div>
  <slot></slot>
</div>
