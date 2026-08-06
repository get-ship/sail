<!--
  <popover> — L1 anchored overlay. Style value language only.
-->

<scope>
  props: {
    placement = ''
  }

  const panel = style({ box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).minW(panel-menu-min-w).pad(3), paint: fg(popover-foreground).bg(popover).borderWidth(1).border(border).radius(md).shadow(md).z(popover) })
</scope>

<div popover use:anchor>
  style(panel)
<slot></slot></div>
