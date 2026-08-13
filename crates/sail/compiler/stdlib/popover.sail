<!--
  <popover> — L1 anchored overlay. Style value language only.
-->

<scope>
  props: {
    placement = ''
  }

  style panel = {box: place({ position: fixed, inset: auto, left: bind(anchorLeft), top: bind(anchorTop), z: popover }).space({ m: 0, pad: 3 }).size({ min-w: panel-menu-min-w }), paint: ink({ color: popover.foreground }).fill({ color: popover }).border({ color: popover.border, width: 1 }).radius(md).elevation(md)}
</scope>

<div popover use:anchor style(panel)>
  <slot></slot>
</div>
