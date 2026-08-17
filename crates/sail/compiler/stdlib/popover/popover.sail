<!--
  <popover> — L1 anchored overlay. Style value language only.
-->

<scope>
  props: {
    placement = ''
  }

  style panel = {position({ scheme: fixed, inset: auto, l: bind(anchorLeft), t: bind(anchorTop), z: popover }).pad(3).margin(0).size({ min-w: 12rem }).fg(foreground).bg(popover).border({ color: border, width: 1 }).radius(md).elevation(md)}
</scope>

<div popover use:anchor style(panel)>
  <slot></slot>
</div>
