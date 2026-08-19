<scope>
  props: {
    face: 'primary' | 'accent' | 'muted' = 'primary'
    kind: 'box' | 'bar' | 'short' | 'mid' | 'cell' | 'stretch' = 'box'
  }
  style paint = { bg(bind(face)) }
  style box = { size({ w: 4, h: 4 }) }
  style bar = { size({ w: 16, h: 8 }) }
  style short = { size({ w: 8, h: 4 }) }
  style mid = { size({ w: 8, h: 8 }) }
  style cell = { size({ h: 8 }) }
  style stretch = { size({ w: 8 }) }
</scope>
<column style(paint, { match: { kind: { box: box, bar: bar, short: short, mid: mid, cell: cell, stretch: stretch } } })>
  <slot></slot>
</column>
