<!--
  <tab name="…"> — tab button inside <tab-list>. use:tabs on parent wires a11y.

  Host paint: typeface/color/chrome/cursor on sx. Geometry channels, underline
  active, hover/disabled/focus stay author CSS.
-->

<scope>
  props: {
    name = ''
    disabled = false
  }
</scope>

<button type="button" sx="font-body font-medium text-muted-foreground bg-transparent cursor-pointer whitespace-nowrap select-none"><slot></slot></button>
