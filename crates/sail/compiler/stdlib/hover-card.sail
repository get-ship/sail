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

<div popover use:anchor use:hoverCard sx="p-3 text-foreground bg-background border border-border rounded-md shadow-md z-(--z-popover)"><slot></slot></div>
