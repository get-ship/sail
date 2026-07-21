<!--
  <dropdown id="…" placement="bottom-start"> — L1 anchored menu (element mode).
  Native [popover] + use:anchor + use:menu. Compose <menu-item>s.

  Host paint: chrome + anchor geometry (fixed/inset/anchor vars) on sx.
  Backdrop transparent is global :where([popover])::backdrop. min-width from
  role=menu contract (use:menu stamps role).
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<div popover use:anchor use:menu sx="fixed inset-auto m-0 left-(--anchor-left) top-(--anchor-top) p-1 text-foreground bg-background border border-border rounded-md shadow-md z-(--z-popover)"><slot></slot></div>
