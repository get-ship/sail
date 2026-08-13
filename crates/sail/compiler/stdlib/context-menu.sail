<!--
  <context-menu for="…"> — L1 right-click menu. Native [popover] + use:contextMenu
  + use:menu. Paint from component.context-menu tokens + anchor geometry.
-->

<scope>
  props: {
    for = ''
    onSelect: (detail: { value: string }) => void
  }

  style host = {box: place({ position: fixed, inset: auto, left: bind(anchorLeft), top: bind(anchorTop), z: popover }).space({ m: 0, pad: 1 }), paint: ink({ color: context-menu.foreground }).fill({ color: context-menu }).border({ color: context-menu.border, width: 1 }).radius(md).elevation(md)}
</scope>

<div popover use:contextMenu use:menu style(host)><slot></slot></div>
