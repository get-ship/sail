<!--
  <toggle value="…"> — cell inside <toggle-group>. use:toggleGroup wires a11y.

  Host paint: typeface/color/chrome/cursor on sx. Size channels, hover/pressed/
  disabled/focus stay author CSS.
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<button type="button" sx="font-body font-medium leading-none text-foreground bg-background cursor-pointer whitespace-nowrap select-none"><slot></slot></button>
