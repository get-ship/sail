<!--
  <badge> — L1 language element (element mode). A small, static, tone-colored
  status label. No behavior, no control flow — pure surface. Bare tag: lowers
  to <span class="sail-badge">; the skin routes to the framework @layer keyed on
  the element + attributes; the props feed the manifest + bare shorthand.

  Host paint: soft look is the attr-absent base on host `sx=` (bg/text/border +
  typeface). solid/outline stay attribute CSS (and solid inset shadow). Geometry
  stays channel CSS — size residues have no exact tokens; gap/align-items are
  layout atoms refused on non-layout hosts. Default muted tone fill stays author
  CSS (specificity vs global [tone]).

  Faithful to the feat/next sail-badge baseline for tone/variant/size/pill.
  Two baseline shadow-isms dissolve at L1:
  - host/.badge split → collapsed onto the single root span (same as <label>
    folded its host/inner-label split), since light DOM has one node.
  - the named `leading` slot (an icon wrapper) → dropped; a use site puts a
    leading icon inline in the default slot (light-DOM composition), which
    element mode's single-default-slot projection expresses directly.

  tone rides the GLOBAL [tone] channel (contracts.css) — [tone='X'] fills
  --tone/-fg/-bg/-border/-contrast, so there are no per-tone rules here; the
  base :host is the muted default (a soft, low-key label).
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: 'solid' | 'soft' | 'outline' = 'solid'
    size: 'sm' | 'md' | 'lg' = 'md'
    pill = false
  }
</scope>

<span sx="font-body font-semibold leading-none whitespace-nowrap align-middle tabular-nums bg-tone-bg text-tone-fg border border-tone-border">
  <slot></slot>
</span>
