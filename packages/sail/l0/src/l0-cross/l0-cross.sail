<scope>
  props: {
    align: 'start' | 'center' | 'end' | 'stretch' | 'baseline' = 'start'
  }
  style rail = { size({ h: 16 }), bg(muted) }
</scope>
<row align={align()} gap="2" style(rail)>
  <slot></slot>
</row>
