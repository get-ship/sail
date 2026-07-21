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

<style>
  .sail-tab {
    align-items: center;
    gap: var(--space-2);
    padding: var(--tab-py, var(--space-2)) var(--tab-px, var(--space-3));
    line-height: 1;
    border: 0;
    border-bottom: 2px solid transparent;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    -webkit-tap-highlight-color: transparent;
    transition:
      color var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }
  .sail-tab:hover:not([disabled]) { color: var(--foreground); }
  .sail-tab[aria-selected='true'] {
    color: var(--foreground);
    font-weight: var(--font-weight-semibold);
    border-bottom-color: var(--tone, var(--primary));
  }
  .sail-tab[disabled] { opacity: 0.5; cursor: not-allowed; }
  .sail-tab:focus-visible {
    outline: var(--ring-width) solid var(--tone-ring, var(--ring));
    outline-offset: calc(-1 * var(--ring-width));
  }
</style>

<button type="button" sx="font-body font-medium text-muted-foreground bg-transparent cursor-pointer whitespace-nowrap select-none"><slot></slot></button>
