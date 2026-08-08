<!--
  <context-menu for="…"> — L1 right-click menu. Native [popover] + use:contextMenu
  + use:menu. Paint from component.context-menu tokens + anchor geometry.
-->

<scope>
  props: {
    for = ''
    onSelect: (detail: { value: string }) => void
  }

  style host = { box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).pad(1), paint: fg(context-menu.foreground).bg(context-menu).borderWidth(1).border(context-menu.border).radius(md).shadow(md).z(popover) }
</scope>

<div popover use:contextMenu use:menu style(host)><slot></slot></div>
