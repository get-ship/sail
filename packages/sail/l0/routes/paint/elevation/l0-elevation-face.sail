<scope>
  props: {
    kind: 'rail' | 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl' = 'rail'
  }
  style rail = { pad(2) }
  style noneStep = { size({ w: 6, h: 6 }), radius(sm), bg(muted), elevation({ shadow: none }) }
  style xs = { size({ w: 6, h: 6 }), radius(sm), bg(muted), elevation({ shadow: xs }) }
  style sm = { size({ w: 6, h: 6 }), radius(sm), bg(muted), elevation({ shadow: sm }) }
  style md = { size({ w: 6, h: 6 }), radius(sm), bg(muted), elevation({ shadow: md }) }
  style lg = { size({ w: 6, h: 6 }), radius(sm), bg(muted), elevation({ shadow: lg }) }
  style xl = { size({ w: 6, h: 6 }), radius(sm), bg(muted), elevation({ shadow: xl }) }
</scope>
<column style({ match: { kind: { rail: rail, none: noneStep, xs: xs, sm: sm, md: md, lg: lg, xl: xl } } })>
  <slot></slot>
</column>
