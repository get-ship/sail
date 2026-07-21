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

<style>
  .sail-toggle {
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    height: var(--tg-h, var(--control-h-md));
    padding: 0 var(--tg-px, var(--space-3));
    border: 1px solid var(--border);
    -webkit-tap-highlight-color: transparent;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }
  .sail-toggle:hover:not([disabled]):not([aria-pressed='true']) { background: var(--muted); }
  .sail-toggle[aria-pressed='true'] {
    color: var(--background);
    background: var(--foreground);
    border-color: var(--foreground);
    position: relative;
    z-index: 1;
  }
  .sail-toggle[disabled] { opacity: 0.5; cursor: not-allowed; }
  .sail-toggle:focus-visible {
    outline: var(--ring-width) solid var(--ring);
    outline-offset: 1px;
    position: relative;
    z-index: 2;
  }
</style>

<button type="button" sx="font-body font-medium leading-none text-foreground bg-background cursor-pointer whitespace-nowrap select-none"><slot></slot></button>
