<scope>
  props: {
    kind: 'col' | 'sm' | 'lg' = 'col'
  }
  style col = { size({ w: 38ch, max-w: 100% }), margin({ x: auto }), pad(4), bg(muted) }
  style sm = { size({ w: 20ch, max-w: 100% }), margin({ x: auto }), pad(4), bg(primary) }
  style lg = { size({ w: 60ch, max-w: 100% }), margin({ x: auto }), pad(4), bg(muted) }
</scope>
<column style({ match: { kind: { col: col, sm: sm, lg: lg } } })>
  <slot></slot>
</column>
