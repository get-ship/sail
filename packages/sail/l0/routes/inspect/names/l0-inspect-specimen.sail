<scope>
  style specimen = { pad({ x: 4, y: 16 }), bg(surface), fg(muted), type(mono), radius(sm), border({ width: 1, style: solid, color: muted }) }
</scope>
<column data-inspect-specimen style(specimen)>
  <slot></slot>
</column>
