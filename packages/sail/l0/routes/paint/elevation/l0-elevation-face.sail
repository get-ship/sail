<scope>
  props: {
    kind: 'rail' | 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl' = 'rail'
  }
  style rail = { size({ w: 100%, h: 8 }) }
  style noneStep = { size(grow), radius(sm), bg(muted), elevation({ shadow: none }) }
  style xs = { size(grow), radius(sm), bg(muted), elevation({ shadow: xs }) }
  style sm = { size(grow), radius(sm), bg(muted), elevation({ shadow: sm }) }
  style md = { size(grow), radius(sm), bg(muted), elevation({ shadow: md }) }
  style lg = { size(grow), radius(sm), bg(muted), elevation({ shadow: lg }) }
  style xl = { size(grow), radius(sm), bg(muted), elevation({ shadow: xl }) }
</scope>
<column style({ match: { kind: { rail: rail, none: noneStep, xs: xs, sm: sm, md: md, lg: lg, xl: xl } } })>
  <slot></slot>
</column>
