<!--
  <hover-card id="…" placement="top"> — L1 hover-preview overlay. Native
  [popover] + use:anchor + use:hoverSurface. Host paint: chrome on sx.
  Anchor positioning recipe in contracts.
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<div popover use:anchor use:hoverSurface sx="p-3 text-foreground bg-background border border-border rounded-md shadow-md z-(--z-popover)"><slot></slot></div>
