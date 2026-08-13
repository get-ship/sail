<!--
  <hover-card> — L1 hover-preview overlay. Style value language only.
  Paint from popover / popover-foreground / border; w(max) hugs content (no rail minW).
-->

<scope>
  props: {
    placement = ''
  }

  style panel = {box: place({ position: fixed, inset: auto, left: bind(anchorLeft), top: bind(anchorTop), z: popover }).space({ m: 0, pad: 3 }).size({ w: max, max-w: panel-max-w-sm }), paint: ink({ color: popover-foreground }).fill({ color: popover }).border({ color: border, width: 1 }).radius(md).elevation(md), text: type({ family: body, size: sm })}
</scope>

<div popover use:anchor use:hoverSurface style(panel)><slot></slot></div>
