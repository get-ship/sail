<scope>
  style specimen = { size({ w: 16, h: 8 }), pad({ x: 4, y: 16 }), bg(surface), fg(muted), type(mono), radius(sm) }
</scope>
<column data-inspect-specimen style(specimen)>
  <slot></slot>
</column>
