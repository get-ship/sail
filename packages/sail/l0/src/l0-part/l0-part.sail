<scope>
  props: {
    kind: 'hug' | 'grow' | 'max' = 'hug'
  }
  style hugged = { size(hug), pad(3), bg(primary) }
  style leftover = { size(grow), pad(3), bg(accent) }
  style stretched = { size(max), pad(3), bg(muted) }
</scope>
<column style({ match: { kind: { hug: hugged, grow: leftover, max: stretched } } })>
  <slot></slot>
</column>
