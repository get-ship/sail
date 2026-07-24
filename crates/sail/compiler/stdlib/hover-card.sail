<!--
  <hover-card id="…" placement="top"> — L1 hover-preview overlay. Native
  [popover] + use:anchor + use:hoverSurface. Host paint: chrome + anchor
  geometry + panel min/max on sx. Backdrop transparent is global
  :where([popover])::backdrop.
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<div popover use:anchor use:hoverSurface sx="fixed inset: auto m: 0 left: (--anchor-left) top: (--anchor-top) min-w: (--panel-rail-w) max-w: panel-max-w-sm p: 3 color: foreground bg: background border border: border rounded: md shadow: md z: (--z-popover)"><slot></slot></div>
