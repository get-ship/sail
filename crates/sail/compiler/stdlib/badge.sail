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

<style>
  .sail-badge {
    /* Size: md base (exact tokens); sm/lg override below (residues kept literal where no exact token). */
    --badge-h: var(--control-h-2xs);
    --badge-px: var(--space-2);
    --badge-fs: 10.5px; /* residue literal — no text token */
    --badge-gap: var(--space-1);
    --badge-radius: var(--radius-sm);
    align-items: center;
    gap: var(--badge-gap);
    min-height: var(--badge-h);
    padding: 0 var(--badge-px);
    font-size: var(--badge-fs);
    border-radius: var(--badge-radius);
    -webkit-font-smoothing: antialiased;
  }
  .sail-badge[hidden] { display: none; }

  /* Tone: muted is the default. Scoped to no-tone/default (specificity 0,2,0)
     so the GLOBAL [tone='X'] channel (contracts.css, 0,1,0) fills every branded
     tone without a per-tone rule here and without an order-dependent clash. */
  .sail-badge:not([tone]),
  .sail-badge[tone='default'] {
    --tone: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --tone-contrast: var(--background);
    --tone-bg: var(--tone-muted-bg);
    --tone-border: var(--tone-muted-border);
  }

  .sail-badge[size='sm'] {
    --badge-h: 18px; /* residue */
    --badge-px: var(--space-1_5);
    --badge-fs: 9.5px; /* residue */
    --badge-gap: 3px; /* residue */
  }
  .sail-badge[size='lg'] {
    --badge-h: 26px; /* residue */
    --badge-px: var(--space-2_5);
    --badge-fs: 11.5px; /* residue */
    --badge-gap: var(--space-1_25);
  }

  /* Shape — square (radius-sm) default; pill opts into the full radius. */
  .sail-badge[pill] { --badge-radius: var(--shape-pill); }

  /* Variant matrix — soft base is host sx; solid/outline override. */
  .sail-badge[variant='solid'] {
    background: var(--tone);
    border-color: var(--tone);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 var(--hover-bg);
  }
  .sail-badge[variant='outline'] {
    background: transparent;
    border-color: var(--tone-border);
    color: var(--tone-fg);
  }

  @media (forced-colors: active) {
    .sail-badge { border-color: CanvasText; color: CanvasText; }
    .sail-badge[variant='solid'] { background: Highlight; color: HighlightText; border-color: Highlight; }
  }
</style>

<span sx="font-body font-semibold leading-none whitespace-nowrap align-middle tabular-nums bg-tone-bg text-tone-fg border border-tone-border">
  <slot></slot>
</span>
