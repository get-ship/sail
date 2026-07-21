<!--
  <badge> — L1 language element (element mode). A small, static, tone-colored
  status label. No behavior, no control flow — pure surface. Bare tag: lowers
  to <span class="sail-badge">.

  Paint (one path):
    host sx=         — soft base + PE size md (attr-absent; defaults not stamped)
    CVA-in-props     — variant + size maps (first key = default = PE)
    compound         — pill → rounded-full
    channels:        — soft-status muted --tone-* when tone absent/default
    global [tone]    — branded channel reads (primary/… override soft baseline)
  Zero contracts PE.
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
  channels: {
    default: {
      tone: 'muted-foreground',
      'tone-fg': 'muted-foreground',
      'tone-contrast': 'background',
      'tone-bg': 'tone-muted-bg',
      'tone-border': 'tone-muted-border',
    }
    tone: {
      default: {
        tone: 'muted-foreground',
        'tone-fg': 'muted-foreground',
        'tone-contrast': 'background',
        'tone-bg': 'tone-muted-bg',
        'tone-border': 'tone-muted-border',
      }
    }
  }
</scope>

<span sx="font-body font-semibold leading-none whitespace-nowrap align-middle tabular-nums antialiased border rounded-sm bg-tone-bg text-tone-fg border-tone-border min-h-control-h-2xs px-2 text-xs">
  <slot></slot>
</span>
