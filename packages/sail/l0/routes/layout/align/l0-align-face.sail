<scope>
  style rail = { size({ w: 28, h: 16 }), bg(surface), border({ width: 1, style: solid, color: muted }) }
</scope>
<grid cols="1" rows="1" style(rail)>
  <slot></slot>
</grid>
