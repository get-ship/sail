<!--
  <toggle value="…"> — cell inside <toggle-group>. use:toggleGroup wires a11y.

  Paint: host sx (type/chrome/size channels/hover/pressed/disabled/focus).
  Size channels --tg-h/--tg-px set by parent toggle-group (contracts).
  Contracts: pressed/focus z-index so joined outline borders stack correctly.
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<button
  type="button"
  sx="font-body font-medium leading-none text-foreground bg-background cursor-pointer whitespace-nowrap select-none h-(--tg-h) px-(--tg-px) border border-border tap-highlight-none transition-colors duration-fast ease-out hover:bg-muted pressed:bg-foreground pressed:text-background pressed:border-foreground disabled:opacity-50 disabled:cursor-not-allowed focus:outline-solid focus:outline-2 focus:outline-ring"
>
  <slot></slot>
</button>
