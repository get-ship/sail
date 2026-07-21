<!--
  <menu-item value="…"> — item inside <dropdown> / <context-menu>. role=menuitem
  wired by use:menu on the parent.

  Host paint: typeface/color/chrome/cursor on sx. Flex row geometry + hover/
  focus/disabled states stay author CSS (interactive + layout-on-button).
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<button type="button" sx="font-body text-sm leading-tight text-foreground bg-transparent rounded-sm cursor-pointer whitespace-nowrap"><slot></slot></button>
