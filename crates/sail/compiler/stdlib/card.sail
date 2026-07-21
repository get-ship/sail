<!--
  <card> — L1 language element (element mode). Generic surface: header/body/footer
  composition, tone-tinted border, accent bar, status rail, interactive lift.

  Paint (one path):
    host sx=         — base surface, radius, shadow, overflow, transition
    CVA-in-props     — variant + padding maps via sx atoms
    compound         — interactive / fill / status-rail / accent-bar flags
    global [tone]    — channel reads for border/fill
    contracts        — named unexpressibles only: backdrop-filter, accent-bar
                       ::before stripe, header/footer slot chrome
  No residual card matrix in _residues.css.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: {
      default: 'bg-surface-1',
      outline: 'bg-transparent',
    }
    padding: {
      none: 'p-0',
      sm: 'py-2 px-3',
      md: 'py-4 px-5',
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
    { when: { 'status-rail': true }, sx: 'border-l-2 border-l-tone-fg' },
  ]
</scope>

<div sx="relative overflow-clip rounded-lg border border-border text-foreground shadow-xs transition-colors duration-fast ease-out">
  <slot></slot>
</div>
