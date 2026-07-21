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

<style>
  .sail-page {
    align-items: center;
    justify-content: center;
    min-width: var(--pg-h, var(--control-h-md));
    height: var(--pg-h, var(--control-h-md));
    padding: 0 var(--space-2);
    border: 1px solid transparent;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .sail-page:hover:not([disabled]):not([aria-current='page']) { background: var(--muted); }
  .sail-page[aria-current='page'] {
    color: var(--tone-fg, var(--primary-foreground));
    background: var(--tone, var(--primary));
    border-color: var(--tone, var(--primary));
  }
  .sail-page[disabled] { opacity: 0.5; cursor: not-allowed; }
  .sail-page:focus-visible { outline: var(--ring-width) solid var(--ring); outline-offset: 1px; /* residue */ }
</style>

<button type="button" sx="font-body font-medium text-foreground bg-transparent rounded-md cursor-pointer select-none"><slot></slot></button>
