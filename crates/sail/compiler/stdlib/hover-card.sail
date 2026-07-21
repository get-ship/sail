<!--
  <hover-card id="…" placement="top"> — L1 hover-preview overlay. Native
  [popover] + use:anchor + use:hoverCard. Host paint: chrome on sx.
  Positioning + min/max-width residues + backdrop stay.
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<style>
  .sail-hover-card {
    position: fixed;
    inset: auto;
    margin: 0;
    left: var(--anchor-left);
    top: var(--anchor-top);
    min-width: 14rem;
    max-width: 20rem;
  }
  .sail-hover-card::backdrop { background: transparent; }
</style>

<div popover use:anchor use:hoverCard sx="p-3 text-foreground bg-background border border-border rounded-md shadow-md z-(--z-popover)"><slot></slot></div>
