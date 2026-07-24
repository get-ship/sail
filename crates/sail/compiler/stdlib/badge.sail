<!--
  <badge> — L1 language element (element mode). A small, static, tone-colored
  status label. No behavior, no control flow — pure surface. Bare tag: lowers
  to <span class="sail-badge">.

  Paint (one path):
    host sx=         — soft base + PE size md (attr-absent; defaults not stamped)
    CVA-in-props     — variant + size maps (first key = default = PE)
    compound         — pill → rounded-full; soft+default tone → muted soft PE
  Variance = CVA-in-props axes + optional compound only. tone/size/variant are
  ordinary props (axes or unions), not language channels.
  Zero contracts PE.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    // first key = soft (PE soft host base); solid/outline override
    variant: {
      soft: 'bg: tone-bg color: tone-fg border: tone-border',
      solid: 'bg: tone color: tone-contrast border: tone shadow: xs',
      outline: 'bg: transparent color: tone-fg border: tone-border',
    }
    // first key = default = PE md
    size: {
      md: 'min-h: control-h-2xs px: 2 font-size: xs',
      sm: 'min-h: 4 px: 1_5 font-size: xs',
      lg: 'min-h: control-h-xs px: 2_5 font-size: xs',
    }
    pill = false
  }
  compound: [
    { when: { pill: true }, sx: 'rounded: full' },
    { when: { variant: 'soft', tone: 'default' }, sx: 'bg: tone-muted-bg color: muted-foreground border: tone-muted-border' },
  ]
</scope>

<span sx="font-family: body font-weight: semibold leading: none whitespace: nowrap align: middle tabular-nums antialiased border rounded: sm bg: tone-bg color: tone-fg border: tone-border min-h: control-h-2xs px: 2 font-size: xs">
  <slot></slot>
</span>
