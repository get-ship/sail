<!--
  <popover id="…" placement="bottom"> — L1 anchored overlay on the native
  popover API + use:anchor.

  Host paint: chrome + anchor geometry on sx. Backdrop transparent is global
  :where([popover])::backdrop. Global popover max-width still applies.
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<div popover use:anchor sx="fixed inset: auto m: 0 left: (--anchor-left) top: (--anchor-top) min-w: (--menu-min-w) p: 3 color: foreground bg: background border border: border rounded: md shadow: md z: (--z-popover)"><slot></slot></div>
