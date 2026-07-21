<!--
  <context-menu for="…"> — L1 right-click menu. Native [popover] + use:contextMenu
  + use:menu. Host paint: chrome on sx. Positioning + min-width + backdrop residue.
-->

<scope>
  props: {
    for = ''
  }
</scope>

<style>
  .sail-context-menu {
    position: fixed;
    inset: auto;
    margin: 0;
    left: var(--anchor-left);
    top: var(--anchor-top);
    min-width: 12rem;
  }
  .sail-context-menu::backdrop { background: transparent; }
</style>

<div popover use:contextMenu use:menu sx="p-1 text-foreground bg-background border border-border rounded-md shadow-md z-(--z-popover)"><slot></slot></div>
