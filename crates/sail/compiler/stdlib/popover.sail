<!--
  <popover id="…" placement="bottom"> — L1 anchored overlay on the native
  popover API + use:anchor.

  Host paint: chrome on sx. Anchor positioning recipe in contracts.
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<div popover use:anchor sx="p-3 text-foreground bg-background border border-border rounded-md shadow-md z-(--z-popover)"><slot></slot></div>
