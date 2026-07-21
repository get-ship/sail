<!--
  <badge> — L1 language element (element mode). A small, static, tone-colored
  status label. No behavior, no control flow — pure surface. Bare tag: lowers
  to <span class="sail-badge">.

  Paint (one path):
    host sx=         — soft base + PE size md (attr-absent; defaults not stamped)
    CVA-in-props     — variant + size maps (first key = default = PE)
    compound         — pill → rounded-full
    global [tone]    — channel reads; PE muted default lives in contracts
  No residual badge matrix.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    // first key = soft (PE soft host base); solid/outline override
    variant: {
      soft: 'bg-tone-bg text-tone-fg border-tone-border',
      solid: 'bg-tone text-tone-contrast border-tone shadow-xs',
      outline: 'bg-transparent text-tone-fg border-tone-border',
    }
    // first key = default = PE md
    size: {
      md: 'min-h-control-h-2xs px-2 text-xs',
      sm: 'min-h-4 px-1_5 text-xs',
      lg: 'min-h-control-h-xs px-2_5 text-xs',
    }
    pill = false
  }
  compound: [
    { when: { pill: true }, sx: 'rounded-full' },
  ]
</scope>

<span sx="font-body font-semibold leading-none whitespace-nowrap align-middle tabular-nums antialiased border rounded-sm bg-tone-bg text-tone-fg border-tone-border min-h-control-h-2xs px-2 text-xs">
  <slot></slot>
</span>
