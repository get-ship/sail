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

<style>
  .sail-menu-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-2_5);
    border: 0;
    text-align: start;
  }
  .sail-menu-item:hover:not([disabled]) { background: var(--muted); }
  .sail-menu-item:focus-visible { background: var(--muted); outline: var(--ring-width) solid var(--ring); outline-offset: -1px; }
  .sail-menu-item[disabled] { opacity: 0.5; cursor: not-allowed; }
</style>

<button type="button" sx="font-body text-sm leading-tight text-foreground bg-transparent rounded-sm cursor-pointer whitespace-nowrap"><slot></slot></button>
