<!--
  <hover-card> — L1 hover-preview overlay. Style value language only.
  Paint from popover / foreground / border; w(max) hugs content (no rail minW).
-->

<scope>
  props: {
    placement = ''
  }

  style panel = {position({ scheme: absolute, inset: auto, anchor: auto, area: bottom, z: popover }).pad(3).margin(0).size({ w: max, max-w: panel-max-w-sm }).fg(foreground).bg(overlay).border({ color: border, width: 1 }).radius(md).elevation(md).type({ family: body, size: sm }), match: { placement: { top: position({ area: top }), bottom: position({ area: bottom }), left: position({ area: left }), right: position({ area: right }) } }, animate: { enter: play(fade), exit: play(fade) }}
</scope>

<div popover use:hoverSurface style(panel)><slot></slot></div>
