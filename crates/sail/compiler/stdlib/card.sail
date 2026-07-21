<!--
  <card> — L1 language element (element mode). Generic surface: header/body/footer
  composition, tone-tinted border, accent bar, status rail, interactive lift.

  Paint (one path):
    host sx=         — base surface + PE padding md (attr-absent; defaults not stamped)
    CVA-in-props     — variant + padding maps (first key = default = PE)
    compound         — interactive / fill flags only
    global [tone]    — channel reads for border/fill
    contracts        — named unexpressibles: backdrop-filter, accent-bar
                       ::before stripe, status-rail 3px, header/footer slot chrome
  No residual card matrix in _residues.css.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    // first key = default = PE surface fill
    variant: {
      default: 'bg-surface-1',
      outline: 'bg-transparent',
    }
    // first key = default = PE md
    padding: {
      md: 'py-4 px-5',
      none: 'p-0',
      sm: 'py-2 px-3',
      lg: 'py-5 px-6',
    }
    interactive = false
    'accent-bar': 'off' | 'hover' | 'static' = 'off'
    'status-rail' = false
    fill = false
  }
  compound: [
    { when: { interactive: true }, sx: 'cursor-pointer tap-highlight-none hover:border-border-strong hover:shadow-hover active:shadow-xs focus:outline-solid focus:outline-2 focus:outline-ring focus:outline-offset-2' },
    { when: { fill: true }, sx: 'bg-tone-bg border-tone-border' },
  ]
</scope>

<div sx="relative overflow-clip rounded-lg border border-border text-foreground shadow-xs transition-colors duration-fast ease-out bg-surface-1 py-4 px-5">
  <slot></slot>
</div>
