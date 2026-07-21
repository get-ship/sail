<!--
  <dropdown id="…" placement="bottom-start"> — L1 anchored menu (element mode).
  Native [popover] + use:anchor + use:menu. Compose <menu-item>s.

  Host paint: chrome (color/bg/border/radius/shadow/pad/z) on sx. Positioning
  (fixed/anchor insets) + backdrop + min-width residue stay author CSS.
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<div popover use:anchor use:menu sx="p-1 text-foreground bg-background border border-border rounded-md shadow-md z-(--z-popover)"><slot></slot></div>
