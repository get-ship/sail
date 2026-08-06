<!--
  <hover-card> — L1 hover-preview overlay. Style value language only.
-->

<scope>
  props: {
    placement = ''
  }

  const panel = style({ box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).minW(panel-rail-w).maxW(panel-max-w-sm).pad(3), paint: fg(hover-card-foreground).bg(hover-card).borderWidth(1).border(border).radius(md).shadow(md).z(popover) })
</scope>

<div popover use:anchor use:hoverSurface>
  style(panel)
<slot></slot></div>
