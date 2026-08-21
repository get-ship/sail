<scope>
  props: {
    kind: 'col' | 'sm' | 'lg' = 'col'
  }
  style col = { size({ w: 38ch, max-w: 100% }), pad(4), bg(primary), fg(surface) }
  style sm = { size({ w: 20ch, max-w: 100% }), pad(4), bg(accent), fg(surface) }
  style lg = { size({ w: 60ch, max-w: 100% }), pad(4), bg(primary), fg(surface) }
</scope>
<column style({ match: { kind: { col: col, sm: sm, lg: lg } } })>
  <slot></slot>
</column>
