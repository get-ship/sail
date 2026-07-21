<!--
  <popover id="…" placement="bottom"> — L1 anchored overlay on the native
  popover API + use:anchor.

  Host paint: chrome on sx. Positioning + min/max-width residues + backdrop stay.
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<style>
  .sail-popover {
    position: fixed;
    inset: auto;
    margin: 0;
    left: var(--anchor-left);
    top: var(--anchor-top);
    min-width: 12rem; /* residue */
    max-width: 22rem; /* residue */
  }
  .sail-popover::backdrop { background: transparent; }
</style>

<div popover use:anchor sx="p-3 text-foreground bg-background border border-border rounded-md shadow-md z-(--z-popover)"><slot></slot></div>
