<!--
  <slider> — compiles-away alias for native range input.
  Paint: RANGE_SKIN_CSS in control contracts (no host Style). use:rangeFill
  syncs value→--rng-pct for filled track. Tone/size ordinary props for skin.
-->

<scope>
  props: {
    value = 0
    min = 0
    max = 100
    step = 1
    disabled = false
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'sm' | 'md' | 'lg' = 'md'
    name = ''
  }
</scope>

<input type="range" use:rangeFill />
