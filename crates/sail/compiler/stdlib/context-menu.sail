<!--
  <context-menu for="…"> — L1 right-click menu. Native [popover] + use:contextMenu
  + use:menu. Host paint: chrome + anchor geometry on sx. Backdrop transparent is
  global :where([popover])::backdrop. min-width from role=menu contract.
-->

<scope>
  props: {
    for = ''
  }
</scope>

<div popover use:contextMenu use:menu sx="fixed inset: auto m: 0 left: (--anchor-left) top: (--anchor-top) p: 1 color: foreground bg: background border border: border rounded: md shadow: md z: (--z-popover)"><slot></slot></div>
