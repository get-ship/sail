<!--
  <page value="…"> — page control inside <pagination>. aria-current via
  use:pagination on the parent.

  Host paint: typeface/color/chrome/cursor on sx. Size channels (--pg-h/--pg-fs),
  hover/current/disabled/focus stay author CSS.
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<button type="button" sx="font-body font-medium text-foreground bg-transparent rounded-md cursor-pointer select-none"><slot></slot></button>
