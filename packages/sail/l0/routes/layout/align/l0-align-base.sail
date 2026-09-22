<scope>
  props: {
    face: 'primary' | 'accent' | 'muted' = 'primary'
    kind: 'even' | 'high' | 'low' = 'even'
  }
  style paint = { size({ w: 8 }), bg(bind(face)) }
  style even = { pad(2) }
  style high = { pad({ t: 5, x: 2, b: 1 }) }
  style low = { pad(1) }
</scope>
<column style(paint, { match: { kind: { even: even, high: high, low: low } } })>
  <slot></slot>
</column>
