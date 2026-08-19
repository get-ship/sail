<!--
  <popover> — L1 anchored overlay. Style value language only.
-->

<scope>
  props: {
    placement = ''
  }

  style panel = {position({ scheme: absolute, inset: auto, anchor: auto, area: bottom, z: popover }).pad(3).margin(0).size({ min-w: 12rem }).fg(foreground).bg(overlay).border({ color: border, width: 1 }).radius(md).elevation(md), match: { placement: { top: position({ area: top }), bottom: position({ area: bottom }), left: position({ area: left }), right: position({ area: right }), 'top-end': position({ area: top-end }), 'bottom-end': position({ area: bottom-end }), 'left-end': position({ area: left-end }), 'right-end': position({ area: right-end }) } }}
</scope>

<div popover style(panel)>
  <slot></slot>
</div>
