<!--
  <badge> — L1 language element (element mode). A small, static, tone-colored
  status label. No behavior, no control flow — pure surface. Bare tag: lowers
  to <span class="sail-badge">.

  Paint (one path):
    host sx=         — soft base (attr-absent) + typeface/layout-ish text
    CVA-in-props     — variant (solid/outline) + size maps via sx atoms
    compound         — pill → rounded-full
    global [tone]    — channel reads; PE muted default lives in contracts
  No residual badge matrix.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    // soft is host sx (attr-absent base). solid/outline override via CVA.
    variant: {
      soft: 'bg-tone-bg text-tone-fg border-tone-border',
      solid: 'bg-tone text-tone-contrast border-tone shadow-xs',
      outline: 'bg-transparent text-tone-fg border-tone-border',
    }
    size: {
      sm: 'min-h-4 px-1_5 text-xs',
      md: 'min-h-control-h-2xs px-2 text-xs',
      lg: 'min-h-control-h-xs px-2_5 text-xs',
    }
    pill = false
  }
  compound: [
    { when: { pill: true }, sx: 'rounded-full' },
  ]
</scope>

<span sx="font-body font-semibold leading-none whitespace-nowrap align-middle tabular-nums antialiased border rounded-sm bg-tone-bg text-tone-fg border-tone-border">
  <slot></slot>
</span>
