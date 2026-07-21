<!--
  <context-menu for="…"> — L1 right-click menu. Native [popover] + use:contextMenu
  + use:menu. Host paint: chrome on sx. Positioning + min-width + backdrop residue.
-->

<scope>
  props: {
    for = ''
  }
</scope>

<div popover use:contextMenu use:menu sx="p-1 text-foreground bg-background border border-border rounded-md shadow-md z-(--z-popover)"><slot></slot></div>
