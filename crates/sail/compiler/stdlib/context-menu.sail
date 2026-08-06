<!--
  <context-menu for="…"> — L1 right-click menu. Native [popover] + use:contextMenu
  + use:menu. Paint (Style product): bg/fg from component.context-menu tokens
  (cssNames context-menu, context-menu-foreground) + anchor geometry. Backdrop
  transparent is global :where([popover])::backdrop. min-width from role=menu
  contract.
-->

<scope>
  props: {
    for = ''
  
    onSelect: (detail: { value: string }) => void
}

  const host = style({ box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).pad(1), paint: fg(context-menu-foreground).bg(context-menu).borderWidth(1).border(border).radius(md).shadow(md).z(popover) })
</scope>

<div popover use:contextMenu use:menu>
  style(host)
<slot></slot></div>
