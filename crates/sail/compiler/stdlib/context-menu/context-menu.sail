<!--
  <context-menu for="…"> — L1 right-click menu. Native [popover] + use:contextMenu
  + use:menu. Paint from popover / foreground / border + anchor geometry.
-->

<scope>
  props: {
    for = ''
    onSelect: (detail: { value: string }) => void
  }

  style host = {position({ scheme: absolute, inset: auto, anchor: auto, area: bottom, z: popover }).pad(1).margin(0).fg(foreground).bg(overlay).border({ color: border, width: 1 }).radius(md).elevation(md)}
</scope>

<div popover use:contextMenu use:menu style(host)><slot></slot></div>
