<!--
  <hover-card> — L1 hover-preview overlay. Style value language only.
  Paint from popover / foreground / border; w(max) hugs content (no rail minW).
-->

<scope>
  props: {
    placement = ''
  }

  style panel = {position({ scheme: fixed, inset: auto, l: bind(anchorLeft), t: bind(anchorTop), z: popover }).pad(3).margin(0).size({ w: max, max-w: panel-max-w-sm }).fg(foreground).bg(overlay).border({ color: border, width: 1 }).radius(md).elevation(md).type({ family: body, size: sm })}
</scope>

<div popover use:anchor use:hoverSurface style(panel)><slot></slot></div>
